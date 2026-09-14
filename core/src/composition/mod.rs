pub mod buffer;
pub mod syllable;

pub use buffer::Buffer;
pub use syllable::{Cased, Syllable};

use crate::{
    encode_vowel,
    phonology::{
        decode_vowel,
        rule::{self, check_nucleus_validity, NucleusStatus},
        BaseVowel, Case, Coda, Onset, Shape, Tone,
    },
    rule_engine::RuleEngine,
    RootVowel,
};

/// Outcome of applying a transform key to the current syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransformEffect {
    /// The transform was applied to the syllable.
    Applied,
    /// The transform undid an existing mark; the key falls through as a literal.
    Reverted,
    /// The key could not act as a transform here.
    NotApplicable,
}

/// The parsing phase the syllable is currently in.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyllableParsePhase {
    /// Reading the initial consonant(s).
    #[default]
    Onset,
    /// Reading the vowel nucleus.
    Vowel,
    /// Reading the final consonant(s).
    Coda,
}

/// Why a syllable was rejected.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyllableParseIssue {
    /// Unknown / unreachable failure.
    #[default]
    Unknown,
    /// The consonant cluster is not a valid Vietnamese onset.
    InvalidOnset,
    /// The vowel nucleus violates the Vietnamese vowel-rule table.
    InvalidNucleus,
    /// The final consonant cluster is not a valid Vietnamese coda.
    InvalidCoda,
}

/// Whether a recorded character belongs to the accepted syllable or to the
/// rejected input that ended the parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharStatus {
    /// The character was part of the last valid syllable.
    Accepted(char),
    /// The character could not be parsed and killed the composition.
    Rejected(char),
}

/// Snapshot taken once the composition becomes invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferrerState {
    /// Why the syllable could not be completed.
    pub issue: SyllableParseIssue,
    /// The toneless text actually built up to the failure: every accepted
    /// syllable character, followed by the rejected input (the killer and any
    /// characters appended afterwards).
    pub toneless: Buffer<CharStatus>,
}

/// Incremental syllable parser driven by a [`RuleEngine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition<KM: RuleEngine> {
    rule_engine: KM,

    input: Buffer<char>,

    // The syllable currently being built. Once the composition becomes
    // invalid, this still holds the last valid partial syllable.
    syllable: Syllable,
    syllable_parse_phase: SyllableParsePhase,

    // Deferred state after the syllable can no longer be parsed; `None` while
    // the composition is still valid.
    deferred: Option<DeferrerState>,
}

impl<KM: RuleEngine> Composition<KM> {
    #[inline(always)]
    pub fn syllable(&self) -> &Syllable {
        &self.syllable
    }

    #[inline(always)]
    pub fn input(&self) -> &Buffer<char> {
        &self.input
    }

    #[inline(always)]
    pub fn syllable_parse_phase(&self) -> SyllableParsePhase {
        self.syllable_parse_phase
    }

    #[inline(always)]
    pub fn defered(&self) -> &Option<DeferrerState> {
        &self.deferred
    }

    /// Creates a parser backed by `mapping`, starting in the `Onset` phase.
    #[inline(always)]
    pub fn new(rule_engine: KM) -> Self {
        Self {
            rule_engine: rule_engine,

            input: Buffer::default(),

            syllable: Syllable::default(),
            syllable_parse_phase: SyllableParsePhase::Onset,

            deferred: None,
        }
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    /// Marks the syllable as invalid: freezes the currently accepted toneless
    /// text together with the `killer` character that caused the failure, and
    /// records why the parse stopped.
    #[inline]
    fn kill_by(&mut self, killer: char, issue: SyllableParseIssue) {
        let mut toneless = Buffer::<CharStatus>::with_capacity(self.syllable.len() + 1);

        toneless.extend(
            self.syllable
                .onset
                .chars()
                .iter()
                .copied()
                .map(CharStatus::Accepted),
        );

        for vowel in &self.syllable.vowels {
            let ch = encode_vowel(vowel.value, Tone::Flat, vowel.case);
            toneless.push(CharStatus::Accepted(ch));
        }

        toneless.extend(
            self.syllable
                .coda
                .chars()
                .iter()
                .copied()
                .map(CharStatus::Accepted),
        );

        toneless.push(CharStatus::Rejected(killer));
        self.deferred = Some(DeferrerState { issue, toneless });
    }

    /// Appends one input character to the composition.
    ///
    /// While the composition is still valid the character is dispatched to the
    /// current phase; once invalid, it is only added to the deferred rejected
    /// buffer and otherwise ignored.
    pub fn append(&mut self, input: char) {
        self.input.push(input);

        if let Some(invalid) = self.deferred.as_mut() {
            invalid.toneless.push(CharStatus::Rejected(input));
            return;
        }

        match self.syllable_parse_phase {
            SyllableParsePhase::Onset => self.append_onset(input),
            SyllableParsePhase::Vowel => self.append_vowel(input),
            SyllableParsePhase::Coda => self.append_coda(input),
        }

        // let syllable = &self.syllable;

        // if let ParseStatus::Dead(_) = self.status {
        //     return self.status;
        // }
    }

    /// Inserts `input` into the syllable at `index`.
    ///
    /// Not implemented yet: anything at or past the end of the syllable simply
    /// falls through to [`Self::append`].
    pub fn insert_at(&mut self, index: usize, input: char) {
        let (onset_len, vowel_len, coda_len, len) = self.syllable.len_parts();

        // If the index is beyond the end of the syllable, push the input as-is.
        if index > len {
            return self.append(input);
        }

        // Example: "trường"
        //
        // chars: ['t', 'r', 'ư', 'ờ', 'n', 'g']
        //        [0]  [1]  [2]  [3]  [4]  [5]
        //
        // onset: ['t', 'r']
        // vowels: ['ư', 'ờ']
        // coda: ['n', 'g']
        //
        // Vowel range is between onset end and coda start:
        //
        // "tr|ườ|ng"
        //     ^   ^
        //     2   4
        //
        // onset_end = 2
        // coda_start = 4

        // Onset insertion positions: 0..=1 in example
        if index < onset_len {
            // Only one effect can happens that is stroke d
        }
        // Vowel insertion positions: 2..=4 in example
        else if index < onset_len + vowel_len + 1 {
            // 1. Stroke modifiers:
            //    Any position in the vowel sequence may update/revert the stroke.
            //
            // 2. Tone modifiers:
            //    Any position in the vowel sequence may update/revert the tone.
            //    Tone is stored at syllable level; the renderer recalculates
            //    the tone position.
            //
            // 3. Shape modifiers:
            //    Apply/revert only when the modifier is immediately after
            //    a compatible vowel.
        }
        // Coda insertion positions: 5.. in example
        else {
            // 1. Any stroke modifier key typed in any position in coda sequence will update or revert the stroke
            // 2. Any tone modifier key typed in any position in coda sequence will update or revert the tone
            // 3. Any shape modifier key typed in any position in coda sequence will update or revert the shape
        }
    }

    /// Removes the character at `index` from the syllable.
    ///
    /// Not implemented yet; placeholder for in-place editing of the current
    /// syllable.
    pub fn remove_at(&mut self, index: usize) {
        let (onset_len, vowel_len, coda_len, len) = self.syllable.len_parts();

        // Example: "trường"
        //
        // chars: ['t', 'r', 'ư', 'ờ', 'n', 'g']
        //        [0]  [1]  [2]  [3]  [4]  [5]
        //
        // onset: ['t', 'r']
        // vowels: ['ư', 'ờ']
        // coda: ['n', 'g']
        //
        // Vowel range is between onset end and coda start:
        //
        // "tr|ườ|ng"
        //     ^   ^
        //     2   4
        //
        // onset_end = 2
        // coda_start = 4

        // Onset removal positions: 0..=1 in example
        if index < onset_len {
            // Just allow remove of onset characters
            // Need to revalidate the onset after removal
            // May be need to revalidate all syllable
        }
        // Vowel removal positions: 2..=4 in example
        else if index < onset_len + vowel_len + 1 {
            // If the syllable had tone. And the removal position is not the tone position after caculated.
            // the tone needs to be recalculated position after removal (May be do in another method)

            // May need to revalidate the nucleus after removal
            // May be need to revalidate all syllable

            // NOTE: Do not recaculate `uo` because it is hard to predict
            // For example `ư ơ o` and we remove the `ơ` at position 2
            // then should we change the o at position 3 to `ơ`?
            // Because current nucleus is `ưo`?
        }
        // Coda removal positions: 5.. in example
        else {
            // Just allow remove of coda characters
            // Need to revalidate the coda after removal
            // May be need to revalidate all syllable
        }
    }

    // ─────────────────────────── Onset ───────────────────────────

    /// Dispatches one input character to the onset-phase handlers, trying the
    /// D-stroke transform before treating it as a literal onset character.
    #[inline(always)]
    fn append_onset(&mut self, input: char) {
        if self.rule_engine.stroke(input) {
            return self.append_onset_stroke_transform(input);
        }

        self.append_onset_literal(input)
    }

    /// Handles a literal (non-transform) character while in the `Onset` phase.
    ///
    /// `q` is kept as a transitional prefix waiting for `u` (to form `qu`);
    /// a vowel ends the onset and starts the nucleus; a consonant is appended
    /// if it still forms a valid cluster — otherwise the parse is killed.
    fn append_onset_literal(&mut self, input: char) {
        let onset_len = self.syllable.onset.len();

        // `q` is a transitional onset prefix; wait for `u`.
        if onset_len == 0 && (input as u32 | 0x20) == ('q' as u32) {
            self.syllable.onset.push_as(input, Onset::None);
            return;
        }

        if let Some((base, tone, case)) = decode_vowel(input) {
            // qu
            // A `u` following a lone `q` is kept as the onset of `qu`.
            // Only the plain `u` counts here: a precomposed `ư` is an actual vowel.
            if onset_len == 1
                && self.syllable.onset.kind() == Onset::None
                && (self.syllable.onset[0] as u32 | 0x20) == ('q' as u32)
            {
                if base == BaseVowel::U {
                    self.syllable.onset.push(input);
                    return;
                }

                // `q` cannot start a vowel nucleus by itself.
                self.kill_by(input, SyllableParseIssue::InvalidOnset);
                return;
            }

            self.syllable_parse_phase = SyllableParsePhase::Vowel;
            if self.append_decoded_vowel(base, tone, case) {
                return;
            }
            return self.kill_by(input, SyllableParseIssue::InvalidNucleus);
        }

        // Only consonant insertion is limited by onset length.

        if onset_len >= Onset::MAX_ONSET_LEN || !self.syllable.onset.push(input) {
            self.kill_by(input, SyllableParseIssue::InvalidOnset);
        }
    }

    /// Handles a transform key while in the `Onset` phase.
    ///
    /// Only the D-stroke is meaningful here; every other key falls back to the
    /// literal handlers.
    #[inline]
    fn append_onset_stroke_transform(&mut self, input: char) {
        match self.try_stroke() {
            // Stroke applied in place; the stroke key itself is not stored literally.
            TransformEffect::Applied => return,

            TransformEffect::Reverted | TransformEffect::NotApplicable => {
                // - After reverting, the stroke key is treated as a literal.
                // Example: 'đ' and typed `d` revert `đ` to `dd` so we remove the stroke in old đ and add new d

                // - Any key that is not a stroke key is treated as an ordinary letter.
                self.append_onset_literal(input)
            }
        }
    }

    // ─────────────────────────── Vowel ───────────────────────────

    /// Dispatches one input character to the vowel-phase handlers, trying
    /// transform keys (tones, D-stroke, shapes) before treating it as a
    /// literal character.
    #[inline(always)]
    fn append_vowel(&mut self, input: char) {
        // Telex/VNI doubling keys (`aa`, `oo`, `ee`, ...) and tone keys
        // (`s`, `f`, `r`, `x`, `j`, digits) are themselves vowels or ASCII
        // consonants, so the transform check must come first here.
        if self.rule_engine.is_rule_key(input) {
            return self.append_vowel_transform(input);
        }

        self.append_vowel_literal(input)
    }

    /// Appends an already-decoded vowel to the nucleus.
    ///
    /// Returns `true` when the vowel joined the nucleus (possibly resolving
    /// `gi` as a phoneme boundary). Returns `false` when the nucleus cannot
    /// accept it (too many vowels, tone conflict, or an invalid sequence);
    /// the caller is responsible for calling [`Self::kill_by`].
    fn append_decoded_vowel(&mut self, base: BaseVowel, tone: Tone, case: Case) -> bool {
        let vowels_len = self.syllable.vowels.len();

        if vowels_len == 0 {
            // First vowel; add it to the nucleus.
            self.syllable.vowels.push(Cased { value: base, case });
            self.syllable.tone = tone;
            return true;
        }
        // Vietnamese vowel sequence supports at most 3 vowels.
        else if vowels_len >= 3 {
            return false;
        }

        // Append the new vowel to the sequence.

        // A pretoned vowel carrying a tone must not conflict with the tone
        // already on the syllable. e.g. "á" is already typed and a raw "ắ" is
        // pushed straight into the buffer - the two tones would collide (áắ)
        if self.syllable.tone == Tone::Flat {
            // If the tone is currently flat, a pretoned vowel can be used.
            // For example `i` can be contiued typed with an pretoned vowel `ế`. to `iế`
            self.syllable.tone = tone;
        } else if tone != Tone::Flat {
            // Tone conflict
            return false;
        }

        // ---------------------------------------------------------
        // Special case: gi
        // A syllable prefix of "gi" is ambiguous: if another vowel
        // follows the 'i', "gi" becomes the onset; otherwise 'g'
        // stays the onset and the 'i' is the nucleus.
        // ---------------------------------------------------------
        if vowels_len == 1
            && self.syllable.vowels[0].value == BaseVowel::I
            && self.syllable.onset.kind() == Onset::G
        {
            let i = self.syllable.vowels.pop().unwrap();
            self.syllable
                .onset
                .push_as(if i.case == Case::Lower { 'i' } else { 'I' }, Onset::Gi);
        }

        self.syllable.vowels.push(Cased { value: base, case });

        if NucleusStatus::Dead == self.validate_nucleus::<3>() {
            self.syllable.vowels.pop(); // pop back
            return false;
        }

        true
    }

    /// Handles a literal character while in the `Vowel` phase.
    ///
    /// A vowel joins the nucleus (or falls through to kill); a possible coda
    /// starter moves the phase into `Coda` and appends the character; anything
    /// else (invalid coda, non-letter) kills the parse.
    #[inline]
    fn append_vowel_literal(&mut self, input: char) {
        if let Some((base, tone, case)) = decode_vowel(input) {
            if self.syllable.vowels.len() == 2 {
                self.finalize_uo_prefix();
            }

            if self.append_decoded_vowel(base, tone, case) {
                return;
            }

            return self.kill_by(input, SyllableParseIssue::InvalidNucleus);
        } else if self.append_coda_element(input) {
            // `normalize_uo` validates the length itself and only acts once there
            // are at least two vowels - which is exactly when a coda character is
            // being pushed here.
            self.finalize_uo_prefix();
            self.syllable_parse_phase = SyllableParsePhase::Coda;
            return;
        }

        // Not a valid coda character
        self.kill_by(input, SyllableParseIssue::InvalidCoda)
    }

    /// Handles a transform key while in the `Vowel` phase.
    ///
    /// Tries tones, then the D-stroke, then vowel shapes.
    #[inline]
    fn append_vowel_transform(&mut self, input: char) {
        // `Some` means the key resolved into a tone or a stroke.
        if let Some(effect) = self.try_tone_or_stroke_transform(input) {
            return match effect {
                TransformEffect::Applied => return,
                TransformEffect::Reverted | TransformEffect::NotApplicable => {
                    self.append_vowel_literal(input)
                }
            };
        }

        // No tone or stroke effect: try a shape instead.
        match self.try_shape_transform(input) {
            TransformEffect::Applied => return,
            TransformEffect::Reverted | TransformEffect::NotApplicable => {
                self.append_vowel_literal(input)
            }
        }
    }

    // ─────────────────────────── Coda ───────────────────────────

    /// Dispatches one input character to the coda-phase handlers, trying
    /// transform keys before literals.
    #[inline(always)]
    fn append_coda(&mut self, input: char) {
        // Telex tone keys (`s`, `f`, `r`, `x`, `j`) and the `d` stroke are
        // ASCII consonants, so the transform check must come first here.
        if self.rule_engine.is_rule_key(input) {
            return self.append_coda_transform(input);
        }

        self.append_coda_literal(input)
    }

    /// Handles a literal character while in the `Coda` phase; kills the parse
    /// unless the consonant extends the coda to a valid cluster.
    #[inline(always)]
    fn append_coda_literal(&mut self, input: char) {
        if self.append_coda_element(input) {
            return;
        }

        self.kill_by(input, SyllableParseIssue::InvalidCoda);
    }

    /// Appends a consonant to the coda, returning `false` when the extended coda
    /// is full or no longer a valid Vietnamese coda (the character is rolled
    /// back in that case).
    #[inline]
    fn append_coda_element(&mut self, input: char) -> bool {
        if self.syllable.coda.len() >= Coda::MAX_CODA_LEN {
            return false;
        }

        self.syllable.coda.push(input)
    }

    /// Handles a transform key while in the `Coda` phase.
    ///
    /// Tone, D-stroke and shape transforms are forwarded to the vowel
    /// handlers; anything else falls back to the literal handlers.
    #[inline]
    fn append_coda_transform(&mut self, input: char) {
        if let Some(effect) = self.try_tone_or_stroke_transform(input) {
            return match effect {
                TransformEffect::Applied => return,
                TransformEffect::Reverted | TransformEffect::NotApplicable => {
                    self.append_coda_literal(input)
                }
            };
        }

        match self.try_shape_transform(input) {
            TransformEffect::Applied => return,
            TransformEffect::Reverted | TransformEffect::NotApplicable => {
                self.append_coda_literal(input)
            }
        }
    }

    // ─────────────────────────── Shared ───────────────────────────

    /// Resolves a tone or D-stroke transform key.
    #[inline(always)]
    fn try_tone_or_stroke_transform(&mut self, key: char) -> Option<TransformEffect> {
        // ---------------------------------------------------------
        // 1. Tone
        // ---------------------------------------------------------
        if let Some(tone) = self.rule_engine.tone(key) {
            return Some(self.try_tone(tone));
        }

        // ---------------------------------------------------------
        // 2. D-stroke
        // ---------------------------------------------------------
        if self.rule_engine.stroke(key) {
            return Some(self.try_stroke());
        }

        None
    }

    /// Applies or toggles a tone on the syllable.
    ///
    /// Tapping the same tone again toggles the syllable back to `Flat`;
    /// a different tone replaces the current one.
    #[inline(always)]
    fn try_tone(&mut self, tone: Tone) -> TransformEffect {
        let syllable = &mut self.syllable;

        if syllable.vowels.is_empty() {
            return TransformEffect::NotApplicable;
        }

        // Same tone -> toggle back to Flat.
        if syllable.tone == tone {
            syllable.tone = Tone::Flat;
            return TransformEffect::Reverted;
        }

        // Different tone -> replace the current tone.
        syllable.tone = tone;
        TransformEffect::Applied
    }

    /// Toggles the D-stroke on the onset cluster (`d` ↔ `đ`, `D` ↔ `Đ`).
    ///
    /// Only applies when the onset is a lone `D`/`Đ`; otherwise the stroke key
    /// cannot act here.
    #[inline]
    fn try_stroke(&mut self) -> TransformEffect {
        let onset = &mut self.syllable.onset;

        match onset.kind() {
            Onset::D => {
                let new_char = if onset[0] == 'd' { 'đ' } else { 'Đ' };
                onset.replace_at(0, new_char, Onset::Đ);
                return TransformEffect::Applied;
            }
            Onset::Đ => {
                let new_char = if onset[0] == 'đ' { 'd' } else { 'D' };
                onset.replace_at(0, new_char, Onset::D);
                return TransformEffect::Reverted;
            }
            _ => return TransformEffect::NotApplicable,
        }
    }

    /// Whether the nucleus starts with an unmarked `u o` pair.
    #[inline(always)]
    fn is_uo_prefix(&self) -> bool {
        let vowels = &self.syllable.vowels;
        return vowels.len() > 1
            && vowels[0].value.root() == RootVowel::U
            && vowels[1].value.root() == RootVowel::O;
    }

    /// Applies a Horn shape to the `u o` prefix (requires at least 2 vowels).
    #[inline]
    fn try_uo_horn(&mut self) -> TransformEffect {
        let vowels = &self.syllable.vowels;

        return match (vowels[0].value, vowels[1].value) {
            // ươ -> uow
            //
            // This always reverts, even with a third vowel.
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                self.syllable.vowels[0].value = BaseVowel::U;
                self.syllable.vowels[1].value = BaseVowel::O;
                TransformEffect::Reverted
            }

            // ưô -> not applicable
            (BaseVowel::UHorn, BaseVowel::OCircumflex) => TransformEffect::NotApplicable,

            // ưo -> ươ
            (BaseVowel::UHorn, BaseVowel::O) => self.try_vowel_shape(1, Shape::Horn),

            // uo -> uơ
            (BaseVowel::U, BaseVowel::O) => self.try_vowel_shape(1, Shape::Horn),

            // uơ -> ươ
            (BaseVowel::U, BaseVowel::OHorn) => self.try_vowel_shape(0, Shape::Horn),

            // uô -> uơ
            (BaseVowel::U, BaseVowel::OCircumflex) => self.try_vowel_shape(1, Shape::Horn),

            _ => TransformEffect::NotApplicable,
        };
    }

    /// Applies a Circumflex shape to the `u o` prefix (requires at least 2 vowels).
    #[inline]
    fn try_uo_circumflex(&mut self) -> TransformEffect {
        let vowels = &self.syllable.vowels;
        let first = vowels[0].value;

        return match (first, vowels[1].value) {
            // ươ -> uô
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                self.syllable.vowels[0].value = BaseVowel::U;

                match self.try_vowel_shape(1, Shape::Circumflex) {
                    effect @ (TransformEffect::Applied | TransformEffect::Reverted) => effect,
                    TransformEffect::NotApplicable => {
                        self.syllable.vowels[0].value = first;
                        TransformEffect::NotApplicable
                    }
                }
            }

            // ưô -> not applicable
            (BaseVowel::UHorn, BaseVowel::OCircumflex) => TransformEffect::NotApplicable,

            // ưo -> uô
            (BaseVowel::UHorn, BaseVowel::O) => {
                self.syllable.vowels[0].value = BaseVowel::U;

                match self.try_vowel_shape(1, Shape::Circumflex) {
                    effect @ (TransformEffect::Applied | TransformEffect::Reverted) => effect,
                    TransformEffect::NotApplicable => {
                        self.syllable.vowels[0].value = first;
                        TransformEffect::NotApplicable
                    }
                }
            }

            // uo -> uô
            (BaseVowel::U, BaseVowel::O) => self.try_vowel_shape(1, Shape::Circumflex),

            // uơ -> uô
            (BaseVowel::U, BaseVowel::OHorn) => self.try_vowel_shape(1, Shape::Circumflex),

            // uô -> undo (revert to "uo")
            (BaseVowel::U, BaseVowel::OCircumflex) => {
                self.syllable.vowels[1].value = BaseVowel::O;
                TransformEffect::Reverted
            }
            _ => TransformEffect::NotApplicable,
        };
    }

    /// Tries to apply `key` as a shape transform on the vowel sequence,
    /// scanning from the last vowel backwards.
    fn try_shape_transform(&mut self, key: char) -> TransformEffect {
        let vseq_len = self.syllable.vowels.len();

        for index in (0..vseq_len).rev() {
            let base = self.syllable.vowels[index].value;

            let Some(shape) = self.rule_engine.shape(key, base.root()) else {
                continue;
            };

            // Handle the special u / o cases.

            if self.is_uo_prefix() {
                match shape {
                    Shape::Horn => return self.try_uo_horn(),
                    Shape::Circumflex => return self.try_uo_circumflex(),
                    _ => {}
                }
            }

            let effect = self.try_vowel_shape(index, shape);
            // If that failed, keep trying the earlier vowels.
            if effect != TransformEffect::NotApplicable {
                return effect;
            }
        }

        TransformEffect::NotApplicable
    }

    /// Validates the vowel nucleus against the rule table, keeping only the
    /// first `N` vowels.
    #[inline(always)]
    fn validate_nucleus<const N: usize>(&self) -> NucleusStatus {
        let mut buf = [BaseVowel::A; N];
        let len = self.syllable.vowels.len().min(N);

        for (i, v) in self.syllable.vowels.iter().take(len).enumerate() {
            buf[i] = v.value;
        }

        check_nucleus_validity(&buf[..len])
    }

    /// Applies `shape` to the vowel at `index`, re-validating the nucleus.
    ///
    /// Applying the shape it already has reverts it; an invalid result rolls
    /// the vowel back.
    #[inline]
    fn try_vowel_shape(&mut self, index: usize, shape: Shape) -> TransformEffect {
        let old = self.syllable.vowels[index].value;

        if old.shape() == shape && shape != Shape::None {
            self.syllable.vowels[index].value = old.remove_shape();
            return TransformEffect::Reverted;
        }

        let Ok(new) = old.replace_shape(shape) else {
            return TransformEffect::NotApplicable;
        };

        self.syllable.vowels[index].value = new;

        if self.syllable.vowels.len() < 2 {
            return TransformEffect::Applied;
        }

        match self.validate_nucleus::<3>() {
            rule::NucleusStatus::Valid => TransformEffect::Applied,
            rule::NucleusStatus::InComplete => TransformEffect::Applied,
            rule::NucleusStatus::Dead => {
                self.syllable.vowels[index].value = old;
                TransformEffect::NotApplicable
            }
        }
    }

    /// Normalizes an unmarked `u o` prefix that arrived without a shape key.
    ///
    /// `uơ → ươ` and `ưo → ươ` are folded once at least two vowels are present.
    #[inline]
    fn finalize_uo_prefix(&mut self) {
        let vowels = &mut self.syllable.vowels;
        if vowels.len() < 2 {
            return;
        }

        match (vowels[0].value, vowels[1].value) {
            (BaseVowel::U, BaseVowel::OHorn) => {
                vowels[0].value = BaseVowel::UHorn;
            }
            (BaseVowel::UHorn, BaseVowel::O) => {
                vowels[1].value = BaseVowel::OHorn;
            }
            _ => {}
        }
    }
}
