use crate::{
    phonology::{
        decode_vowel,
        rule::{self, check_nucleus_validity, NucleusStatus},
        BaseVowel, Case, Coda, Onset, Shape, Tone,
    },
    rule_engine::RuleEngine,
    RootVowel,
};

use crate::composition::{Cased, Syllable};

/// Whether `c` is an ASCII consonant character.
///
/// Consonants are the ASCII letters minus the vowels `a, e, i, o, u` (and minus
/// `y`, which the Vietnamese vowel codec owns). `đ` / `Đ` are also onsets.
#[inline(always)]
const fn is_ascii_consonant(c: char) -> bool {
    matches!(
        c,
        'b'..='d' | 'f'..='h' | 'j'..='n' | 'p'..='t' | 'v'..='x' | 'z' |
        'B'..='D' | 'F'..='H' | 'J'..='N' | 'P'..='T' | 'V'..='X' | 'Z' |
        'đ' | 'Đ'
    )
}

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
pub enum ParsePhase {
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
pub enum DeadReason {
    /// Unknown / unreachable failure.
    #[default]
    Unknown,
    /// The consonant cluster is not a valid Vietnamese onset.
    InvalidOnset,
    /// The vowel nucleus violates the Vietnamese vowel-rule table.
    InvalidVowelSequence,
    /// The final consonant cluster is not a valid Vietnamese coda.
    InvalidCoda,
    /// A character that is neither a consonant nor a vowel (`?`, `,`, `/`, …).
    InvalidCharacter,
}

/// Overall result of a parse.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStatus {
    /// The syllable is still being built and may become valid.
    #[default]
    Incomplete,
    /// The syllable is a complete, valid Vietnamese syllable.
    Valid,
    /// The syllable can never become valid.
    Dead(DeadReason),
}

/// Incremental syllable parser driven by a [`RuleEngine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parser<KM: RuleEngine> {
    rule_engine: KM,

    syllable: Syllable,
    phase: ParsePhase,
    status: ParseStatus,
}

impl<KM: RuleEngine> Parser<KM> {
    /// Creates a parser backed by `mapping`, starting in the `Onset` phase.
    #[inline(always)]
    pub fn new(rule_engine: KM) -> Self {
        Self {
            syllable: Syllable::default(),
            phase: ParsePhase::Onset,
            status: ParseStatus::Incomplete,
            rule_engine: rule_engine,
        }
    }

    /// Clears the syllable and returns the parser to the `Onset` phase.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.syllable = Syllable::default();
        self.phase = ParsePhase::Onset;
        self.status = ParseStatus::Incomplete;
    }

    /// Returns the syllable built so far.
    #[inline(always)]
    pub const fn syllable(&self) -> &Syllable {
        &self.syllable
    }

    /// Returns the current parse status.
    #[inline(always)]
    pub const fn status(&self) -> ParseStatus {
        self.status
    }

    /// Returns the phase the parser is currently in.
    #[inline(always)]
    pub const fn phase(&self) -> ParsePhase {
        self.phase
    }

    /// Marks the syllable as dead for `reason`.
    #[inline(always)]
    const fn kill(&mut self, reason: DeadReason) -> ParseStatus {
        self.status = ParseStatus::Dead(reason);
        self.status
    }

    /// Resets the parser to the initial state.
    #[inline(always)]
    pub const fn revive(&mut self) -> ParseStatus {
        self.status = ParseStatus::Incomplete;
        self.status
    }

    /// Advances the parser to the given phase.
    #[inline(always)]
    const fn set_phase(&mut self, phase: ParsePhase) {
        self.phase = phase;
    }

    /// Pushes one input buffer character and returns the new status.
    ///
    /// No character is consumed once the syllable is dead.
    pub fn push(&mut self, input: char) -> ParseStatus {
        if let ParseStatus::Dead(_) = self.status {
            return self.status;
        }

        match self.phase {
            ParsePhase::Onset => self.push_onset(input),
            ParsePhase::Vowel => self.push_vowel(input),
            ParsePhase::Coda => self.push_coda(input),
        }
    }

    /// Pushes a sequence of input characters.
    ///
    /// Returns the resulting status and, when the parse is dead, the index of
    /// the character at which it first died. A parser that was already dead
    /// before the call yields `None`.
    pub fn parse(&mut self, chars: &[char]) -> (ParseStatus, Option<usize>) {
        if let ParseStatus::Dead(_) = self.status {
            return (self.status, None);
        }

        for (index, ch) in chars.iter().enumerate() {
            if let status @ ParseStatus::Dead(_) = self.push(*ch) {
                return (status, Some(index));
            }
        }

        (self.status, None)
    }

    // ─────────────────────────── Onset ───────────────────────────

    #[inline(always)]
    fn push_onset(&mut self, input: char) -> ParseStatus {
        if self.rule_engine.stroke(input) {
            return self.push_onset_stroke_transform(input);
        }

        self.push_onset_literal(input)
    }

    #[inline(always)]
    fn push_onset_literal(&mut self, input: char) -> ParseStatus {
        if let Some((base, tone, case)) = decode_vowel(input) {
            let onset_chars = &self.syllable.onset_chars;
            let onset_len = onset_chars.len();

            // qu
            // A `u` following a lone `q` is kept as the onset of `qu`.
            // Only the plain `u` counts here: a precomposed `ư` is an actual vowel.
            if onset_len == 1 && base == BaseVowel::U && onset_chars[0].to_ascii_lowercase() == 'q'
            {
                self.syllable.onset_chars.push(input);
                return self.status;
            }

            // A zero onset is also valid: "a", "ă", "â", ...
            if onset_len > 0 {
                match Onset::from_chars(&onset_chars) {
                    Ok(kind) => {
                        self.syllable.onset = Some(kind);
                    }
                    Err(_) => return self.kill(DeadReason::InvalidOnset),
                }
            }

            self.set_phase(ParsePhase::Vowel);
            return self.push_decoded_vowel(base, tone, case);
        }

        // Do not check len here because i want it be satilize
        // For example when i type 'đk' is a abbreviation
        self.syllable.onset_chars.push(input);
        return self.status;
    }

    /// Handles a transform key while in the `Onset` phase.
    ///
    /// Only the D-stroke is meaningful here; every other key falls back to the
    /// literal handlers.
    #[inline]
    fn push_onset_stroke_transform(&mut self, input: char) -> ParseStatus {
        match self.try_d_stroke() {
            TransformEffect::Applied => self.status,
            TransformEffect::Reverted | TransformEffect::NotApplicable => {
                // After reverting, the stroke key is treated as a literal.

                // Any key that is not a stroke key is treated as an ordinary letter.
                self.push_onset_literal(input)
            }
        }
    }

    // ─────────────────────────── Vowel ───────────────────────────

    #[inline(always)]
    fn push_vowel(&mut self, input: char) -> ParseStatus {
        // Telex/VNI doubling keys (`aa`, `oo`, `ee`, ...) and tone keys
        // (`s`, `f`, `r`, `x`, `j`, digits) are themselves vowels or ASCII
        // consonants, so the transform check must come first here.
        if self.rule_engine.is_rule_key(input) {
            return self.push_vowel_transform(input);
        }

        self.push_vowel_literal(input)
    }

    /// With "gi" followed by another vowel, the 'i' leaves the vowel
    /// sequence and joins the onset so "gi" becomes the onset `Onset::Gi`.
    #[inline(always)]
    fn resolve_gi_onset(&mut self) -> bool {
        let syllable = &mut self.syllable;

        let Some(i) = syllable.vowels.pop() else {
            return false;
        };

        syllable
            .onset_chars
            .push(if i.case == Case::Lower { 'i' } else { 'I' });
        syllable.onset = Some(Onset::Gi);

        true
    }

    /// Handles a vowel in the `Vowel` phase — adds it to the nucleus.
    #[inline]
    fn push_decoded_vowel(&mut self, base: BaseVowel, tone: Tone, case: Case) -> ParseStatus {
        let vowels_len = self.syllable.vowels.len();

        if vowels_len == 0 {
            // First vowel; add it to the nucleus.
            self.syllable.vowels.push(Cased { value: base, case });
            self.syllable.tone = tone;
            return self.status;
        }
        // Vietnamese vowel sequence supports at most 3 vowels.
        else if vowels_len >= 3 {
            return self.kill(DeadReason::InvalidVowelSequence);
        }
        // ---------------------------------------------------------
        // Special case: gi
        // A syllable prefix of "gi" is ambiguous: if another vowel
        // follows the 'i', "gi" becomes the onset; otherwise 'g'
        // stays the onset and the 'i' is the nucleus.
        // ---------------------------------------------------------
        else if vowels_len == 1
            && self.syllable.vowels[0].value == BaseVowel::I
            && self.syllable.onset == Some(Onset::G)
        {
            if !self.resolve_gi_onset() {
                // Unreachable in practice
                return self.kill(DeadReason::Unknown);
            }
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
            return self.kill(DeadReason::InvalidVowelSequence);
        }

        self.syllable.vowels.push(Cased { value: base, case });

        // `normalize_uo` validates the length itself and only acts once
        // there are at least two vowels - which is exactly when a third
        // vowel is being pushed here.
        self.normalize_uo_prefix();

        self.status
    }

    #[inline(always)]
    fn push_vowel_literal(&mut self, input: char) -> ParseStatus {
        if let Some((base, tone, case)) = decode_vowel(input) {
            return self.push_decoded_vowel(base, tone, case);
        } else if is_ascii_consonant(input) {
            // An already-dead vowel sequence cannot be rescued once the coda has
            // started, since the sequence has fused with the coda.
            // e.g. 'chơa' + 'c' -> 'chơac' (invalid coda).
            if NucleusStatus::Dead == self.validate_nucleus::<3>() {
                return self.kill(DeadReason::InvalidVowelSequence);
            }

            // `normalize_uo` validates the length itself and only acts once there
            // are at least two vowels - which is exactly when a coda character is
            // being pushed here.
            self.normalize_uo_prefix();

            self.set_phase(ParsePhase::Coda);
            return self.push_coda_consonant(input);
        }

        self.kill(DeadReason::InvalidCharacter)
    }

    /// Handles a transform key while in the `Vowel` phase.
    ///
    /// Tries tones, then the D-stroke, then vowel shapes.
    #[inline]
    fn push_vowel_transform(&mut self, input: char) -> ParseStatus {
        // `Some` means the key resolved into a tone or a stroke.
        if let Some(effect) = self.try_tone_or_stroke_transform(input) {
            return match effect {
                TransformEffect::Applied => self.status,
                TransformEffect::Reverted | TransformEffect::NotApplicable => {
                    self.push_vowel_literal(input)
                }
            };
        }

        // No tone or stroke effect: try a shape instead.
        match self.try_shape_transform(input) {
            TransformEffect::Applied => self.status,
            TransformEffect::Reverted | TransformEffect::NotApplicable => {
                self.push_vowel_literal(input)
            }
        }
    }

    // ─────────────────────────── Coda ───────────────────────────

    #[inline(always)]
    fn push_coda(&mut self, input: char) -> ParseStatus {
        // Telex tone keys (`s`, `f`, `r`, `x`, `j`) and the `d` stroke are
        // ASCII consonants, so the transform check must come first here.
        if self.rule_engine.is_rule_key(input) {
            return self.push_coda_transform(input);
        }

        self.push_coda_literal(input)
    }

    #[inline(always)]
    fn push_coda_literal(&mut self, input: char) -> ParseStatus {
        if is_ascii_consonant(input) {
            return self.push_coda_consonant(input);
        }

        self.kill(DeadReason::InvalidCharacter)
    }

    /// Handles a consonant while in the `Coda` phase.
    #[inline]
    fn push_coda_consonant(&mut self, input: char) -> ParseStatus {
        if self.syllable.coda_chars.len() >= Coda::MAX_CODA_LEN {
            return self.kill(DeadReason::InvalidCoda);
        }

        self.syllable.coda_chars.push(input);

        // Re-validate right away to catch a dead coda.
        match Coda::from_chars(&self.syllable.coda_chars) {
            Ok(kind) => {
                self.syllable.coda = Some(kind);
                self.status
            }
            Err(_) => self.kill(DeadReason::InvalidCoda),
        }
    }

    /// Handles a transform key while in the `Coda` phase.
    ///
    /// Tone, D-stroke and shape transforms are forwarded to the vowel
    /// handlers; anything else falls back to the literal handlers.
    #[inline]
    fn push_coda_transform(&mut self, input: char) -> ParseStatus {
        if let Some(effect) = self.try_tone_or_stroke_transform(input) {
            return match effect {
                TransformEffect::Applied => self.status,
                TransformEffect::Reverted | TransformEffect::NotApplicable => {
                    self.push_coda_literal(input)
                }
            };
        }

        match self.try_shape_transform(input) {
            TransformEffect::Applied => self.status,
            TransformEffect::Reverted | TransformEffect::NotApplicable => {
                self.push_coda_literal(input)
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
            return Some(self.try_d_stroke());
        }

        None
    }

    /// Whether the nucleus starts with an unmarked `u o` pair.
    #[inline(always)]
    fn is_uo_first(&self) -> bool {
        return self.syllable.vowels.len() > 1
            && self.syllable.vowels[0].value.root() == RootVowel::U
            && self.syllable.vowels[1].value.root() == RootVowel::O;
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

    /// Toggles the D-stroke on the last onset character.
    #[inline]
    fn try_d_stroke(&mut self) -> TransformEffect {
        for ch in self.syllable.onset_chars.iter_mut().rev() {
            match *ch {
                'd' => {
                    *ch = 'đ';
                    return TransformEffect::Applied;
                }
                'D' => {
                    *ch = 'Đ';
                    return TransformEffect::Applied;
                }
                'đ' => {
                    *ch = 'd';
                    return TransformEffect::Reverted;
                }
                'Đ' => {
                    *ch = 'D';
                    return TransformEffect::Reverted;
                }
                _ => {}
            }
        }

        TransformEffect::NotApplicable
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

            if shape == Shape::Horn && self.is_uo_first() {
                return self.try_uo_horn();
            } else if shape == Shape::Circumflex && self.is_uo_first() {
                return self.try_uo_circumflex();
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
    fn normalize_uo_prefix(&mut self) {
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
