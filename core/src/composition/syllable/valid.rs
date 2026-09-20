use crate::{
    keymap::Keymap,
    phonology::{
        check_nucleus_validity, decode_vowel, BaseVowel, CasedBaseVowel, Coda, NucleusStatus,
        Onset, RootVowel, Shape, Tone, ToneScheme,
    },
};
use arrayvec::ArrayVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputResult {
    Transformed,
    Inserted,
}

/// Outcome of applying a transform key to the current syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformResult {
    /// The transform was applied to the syllable.
    Applied,
    /// The transform undid an existing mark; the key falls through as a literal.
    Reverted,
    /// The key could not act as a transform here.
    NotApplicable,
}

/// The parsing phase the syllable is currently in.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushPhase {
    /// Reading the initial consonant(s).
    #[default]
    Onset,
    /// Reading the vowel nucleus.
    Vowel,
    /// Reading the final consonant(s).
    Coda,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyllableError {
    /// The consonant cluster is not a valid Vietnamese onset.
    InvalidOnset,
    /// The vowel nucleus violates the Vietnamese vowel-rule table.
    InvalidNucleus,
    /// The final consonant cluster is not a valid Vietnamese coda.
    InvalidCoda,
}

/// A single Vietnamese syllable under construction.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ValidSyllableBuilder {
    onset_kind: Onset,
    onset: ArrayVec<char, { Onset::MAX_ONSET_LEN }>,

    nucleus: ArrayVec<CasedBaseVowel, 3>,

    coda_kind: Coda,
    coda: ArrayVec<char, { Coda::MAX_CODA_LEN }>,

    tone: Tone,

    push_phase: PushPhase,
}

impl ValidSyllableBuilder {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.onset.len() + self.nucleus.len() + self.coda.len()
    }

    #[inline(always)]
    pub fn onset(&self) -> &[char] {
        &self.onset
    }

    #[inline(always)]
    pub fn onset_kind(&self) -> Onset {
        self.onset_kind
    }

    #[inline(always)]
    pub fn coda(&self) -> &[char] {
        &self.coda
    }

    #[inline(always)]
    pub fn coda_kind(&self) -> Coda {
        self.coda_kind
    }

    #[inline(always)]
    pub fn nucleus(&self) -> &[CasedBaseVowel] {
        &self.nucleus
    }

    #[inline(always)]
    pub const fn tone(&self) -> Tone {
        self.tone
    }

    #[inline]
    pub fn reset(&mut self) {
        self.onset_kind = Onset::None;
        self.onset.clear();
        self.nucleus.clear();
        self.coda_kind = Coda::None;
        self.coda.clear();
        self.tone = Tone::Flat;
        self.push_phase = PushPhase::Onset;
    }
}

impl ValidSyllableBuilder {
    /// Applies or toggles a tone on the syllable.
    ///
    /// Tapping the same tone again toggles the syllable back to `Flat`;
    /// a different tone replaces the current one.
    #[inline]
    fn apply_tone(&mut self, tone: Tone) -> TransformResult {
        if self.nucleus.is_empty() {
            return TransformResult::NotApplicable;
        }
        // Same tone -> toggle back to Flat.
        else if self.tone == tone {
            self.tone = Tone::Flat;
            return TransformResult::Reverted;
        }

        // Different tone -> replace the current tone.
        self.tone = tone;
        TransformResult::Applied
    }

    /// Toggles the D-stroke on the onset cluster (`d` ↔ `đ`, `D` ↔ `Đ`).
    ///
    /// Only applies when the onset is a lone `D`/`Đ`; otherwise the stroke key
    /// cannot act here.
    #[inline]
    fn toggle_d_stroke(&mut self) -> TransformResult {
        let onset_chars = &mut self.onset;

        match self.onset_kind {
            Onset::D => {
                onset_chars[0] = if onset_chars[0] == 'd' { 'đ' } else { 'Đ' };
                self.onset_kind = Onset::DStroke;
                TransformResult::Applied
            }
            Onset::DStroke => {
                onset_chars[0] = if onset_chars[0] == 'đ' { 'd' } else { 'D' };
                self.onset_kind = Onset::D;
                TransformResult::Reverted
            }
            _ => TransformResult::NotApplicable,
        }
    }

    /// Whether the nucleus starts with an unmarked `u o` pair.
    #[inline(always)]
    fn nucleus_starts_with_uo(&self) -> bool {
        let vowels = &self.nucleus;
        vowels.len() > 1
            && vowels[0].value.root() == RootVowel::U
            && vowels[1].value.root() == RootVowel::O
    }

    /// Normalizes an unmarked `u o` prefix that arrived without a shape key.
    ///
    /// `uơ → ươ` and `ưo → ươ` are folded once at least two vowels are present.
    #[inline]
    fn finalize_uo_prefix(&mut self) {
        let vowels = &mut self.nucleus;
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

    /// Validates the vowel nucleus against the rule table, keeping only the
    /// first `N` vowels.
    #[inline(always)]
    fn validate_nucleus<const N: usize>(&self) -> NucleusStatus {
        let mut buf = [BaseVowel::A; N];
        let len = self.nucleus.len().min(N);

        for (i, v) in self.nucleus.iter().take(len).enumerate() {
            buf[i] = v.value;
        }

        check_nucleus_validity(&buf[..len])
    }

    #[inline(always)]
    pub fn tone_index(&self, tone_scheme: ToneScheme) -> Option<usize> {
        tone_scheme.tone_index(&self.nucleus, self.coda.is_empty())
    }
}

impl ValidSyllableBuilder {
    #[inline(always)]
    pub fn push<KM: Keymap>(
        &mut self,
        keymap: &KM,
        input: char,
    ) -> Result<InputResult, SyllableError> {
        match self.push_phase {
            PushPhase::Onset => self.push_onset(keymap, input),
            PushPhase::Vowel => self.push_vowel(keymap, input),
            PushPhase::Coda => self.push_coda(keymap, input),
        }
    }

    // ─────────────────────────── Onset ───────────────────────────
    #[inline(always)]
    fn push_onset<KM: Keymap>(
        &mut self,
        keymap: &KM,
        input: char,
    ) -> Result<InputResult, SyllableError> {
        if keymap.is_stroke_key(input) {
            return self.push_onset_transform(input);
        }

        self.push_onset_literal(input);
    }

    #[inline]
    fn push_onset_char(&mut self, input: char) -> bool {
        if self.onset.len() >= Onset::MAX_ONSET_LEN {
            return false;
        }

        self.onset.push(input);
        match Onset::from_chars(&self.onset) {
            Ok(kind) => {
                self.onset_kind = kind;
                true
            }
            Err(_) => {
                self.onset.pop();
                false
            }
        }
    }

    /// Handles a literal (non-transform) character while in the `Onset` phase.
    ///
    /// `q` is kept as a transitional prefix waiting for `u` (to form `qu`);
    /// a vowel ends the onset and starts the nucleus; a consonant is appended
    /// if it still forms a valid cluster — otherwise the parse is killed.
    fn push_onset_literal(&mut self, input: char) -> Result<InputResult, SyllableError> {
        let onset_len = self.onset.len();

        // `q` is a transitional onset prefix; wait for `u`.
        if onset_len == 0 && (input as u32 | 0x20) == ('q' as u32) {
            self.onset.push(input);
            return Ok(InputResult::Inserted);
        } else if let Some((base_cased, tone)) = decode_vowel(input) {
            // qu
            // A `u` following a lone `q` is kept as the onset of `qu`.
            // Only the plain `u` counts here: a precomposed `ư` is an actual vowel.
            if onset_len == 1 && (self.onset[0] as u32 | 0x20) == ('q' as u32) {
                if base_cased.value == BaseVowel::U {
                    self.onset.push(input);
                    self.onset_kind = Onset::Qu;
                    return Ok(InputResult::Inserted);
                }

                // `q` cannot start a vowel nucleus by itself.
                return Err(SyllableError::InvalidOnset);
            }

            if self.push_decoded_vowel(base_cased, tone) {
                self.push_phase = PushPhase::Vowel;
                return Ok(InputResult::Inserted);
            }

            return Err(SyllableError::InvalidNucleus);
        } else if self.push_onset_char(input) {
            return Ok(InputResult::Inserted);
        }

        Err(SyllableError::InvalidOnset)
    }

    /// Handles a transform key while in the `Onset` phase.
    ///
    /// Only the D-stroke is meaningful here; every other key falls back to the
    /// literal handlers.
    #[inline]
    fn push_onset_transform(&mut self, input: char) -> Result<InputResult, SyllableError> {
        match self.toggle_d_stroke() {
            // Stroke applied in place; the stroke key itself is not stored literally.
            TransformResult::Applied => Ok(InputResult::Transformed),

            TransformResult::Reverted | TransformResult::NotApplicable => {
                // - After reverting, the stroke key is treated as a literal.
                // Example: 'đ' and typed `d` revert `đ` to `dd` so we remove the stroke in old đ and add new d

                // - Any key that is not a stroke key is treated as an ordinary letter.
                self.push_onset_literal(input)
            }
        }
    }

    // ─────────────────────────── Vowel ───────────────────────────

    /// Dispatches one input character to the vowel-phase handlers, trying
    /// transform keys (tones, D-stroke, shapes) before treating it as a
    /// literal character.
    #[inline(always)]
    fn push_vowel<KM: Keymap>(
        &mut self,
        keymap: &KM,
        input: char,
    ) -> Result<InputResult, SyllableError> {
        // Telex/VNI doubling keys (`aa`, `oo`, `ee`, ...) and tone keys
        // (`s`, `f`, `r`, `x`, `j`, digits) are themselves vowels or ASCII
        // consonants, so the transform check must come first here.
        if keymap.is_transform_key(input) {
            return self.push_vowel_transform(keymap, input);
        }

        self.push_vowel_literal(input)
    }

    /// Appends an already-decoded vowel to the nucleus.
    ///
    /// Returns `true` when the vowel joined the nucleus (possibly resolving
    /// `gi` as a phoneme boundary). Returns `false` when the nucleus cannot
    /// accept it (too many vowels, tone conflict, or an invalid sequence);
    /// the caller is responsible for reporting the failure.
    fn push_decoded_vowel(&mut self, cased_base: CasedBaseVowel, tone: Tone) -> bool {
        let vowels_len = self.nucleus.len();

        if vowels_len == 0 {
            // First vowel; add it to the nucleus.
            self.nucleus.push(cased_base);
            self.tone = tone;
            return true;
        }
        // Vietnamese vowel sequence supports at most 3 vowels.
        else if vowels_len >= 3 {
            return false;
        }

        // Append the new vowel to the sequence.

        let old_tone = self.tone;

        // A pretoned vowel carrying a tone must not conflict with the tone
        // already on the syllable. e.g. "á" is already typed and a raw "ắ" is
        // pushed straight into the buffer - the two tones would collide (áắ)
        if self.tone == Tone::Flat {
            // If the tone is currently flat, a pretoned vowel can be used.
            // For example `i` can be contiued typed with an pretoned vowel `ế`. to `iế`
            self.tone = tone;
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
        if vowels_len == 1 && self.nucleus[0].value == BaseVowel::I && self.onset_kind == Onset::G {
            let i = self.nucleus.pop().unwrap();
            self.onset.push(if i.uppercase { 'I' } else { 'i' });
            self.onset_kind = Onset::Gi;
        }

        self.nucleus.push(cased_base);

        if NucleusStatus::Dead == self.validate_nucleus::<3>() {
            // rollback state
            self.nucleus.pop();
            self.tone = old_tone;
            return false;
        }

        true
    }

    /// Handles a literal character while in the `Vowel` phase.
    ///
    /// A vowel joins the nucleus (or falls through to kill); a possible coda
    /// starter moves the phase into `Coda` and appends the character; anything
    /// else (invalid coda, non-letter) kills the parse.
    fn push_vowel_literal(&mut self, input: char) -> Result<InputResult, SyllableError> {
        if let Some((base_cased, tone)) = decode_vowel(input) {
            if !self.push_decoded_vowel(base_cased, tone) {
                return Err(SyllableError::InvalidNucleus);
            }

            if self.nucleus.len() == 3 {
                self.finalize_uo_prefix();
            }

            return Ok(InputResult::Inserted);
        } else if self.push_coda_char(input) {
            // `normalize_uo` validates the length itself and only acts once there
            // are at least two vowels - which is exactly when a coda character is
            // being pushed here.
            self.finalize_uo_prefix();
            self.push_phase = PushPhase::Coda;
            return Ok(InputResult::Inserted);
        }

        Err(SyllableError::InvalidCoda)
    }

    /// Handles a transform key while in the `Vowel` phase.
    ///
    /// Tries tones, then the D-stroke, then vowel shapes.
    #[inline]
    fn push_vowel_transform<KM: Keymap>(
        &mut self,
        keymap: &KM,
        input: char,
    ) -> Result<InputResult, SyllableError> {
        // `Some` means the key resolved into a tone or a stroke.
        if let Some(effect) = self.handle_tone_or_d_stroke(keymap, input) {
            return match effect {
                TransformResult::Applied => Ok(InputResult::Transformed),
                TransformResult::Reverted | TransformResult::NotApplicable => {
                    self.push_vowel_literal(input)
                }
            };
        }

        // No tone or stroke effect: try a shape instead.
        match self.try_shape_transform(keymap, input) {
            TransformResult::Applied => Ok(InputResult::Transformed),
            TransformResult::Reverted | TransformResult::NotApplicable => {
                self.push_vowel_literal(input)
            }
        }
    }

    // ─────────────────────────── Coda ───────────────────────────

    /// Dispatches one input character to the coda-phase handlers, trying
    /// transform keys before literals.
    #[inline(always)]
    fn push_coda<KM: Keymap>(
        &mut self,
        keymap: &KM,
        input: char,
    ) -> Result<InputResult, SyllableError> {
        // Telex tone keys (`s`, `f`, `r`, `x`, `j`) and the `d` stroke are
        // ASCII consonants, so the transform check must come first here.
        if keymap.is_transform_key(input) {
            return self.push_coda_transform(keymap, input);
        }

        self.push_coda_literal(input)
    }

    /// Handles a literal character while in the `Coda` phase; kills the parse
    /// unless the consonant extends the coda to a valid cluster.
    #[inline(always)]
    fn push_coda_literal(&mut self, input: char) -> Result<InputResult, SyllableError> {
        if self.push_coda_char(input) {
            return Ok(InputResult::Inserted);
        }

        Err(SyllableError::InvalidCoda)
    }

    /// Appends a consonant to the coda, returning `false` when the extended coda
    /// is full or no longer a valid Vietnamese coda (the character is rolled
    /// back in that case).
    #[inline]
    fn push_coda_char(&mut self, input: char) -> bool {
        if self.coda.len() >= Coda::MAX_CODA_LEN {
            return false;
        }

        self.coda.push(input);
        match Coda::from_chars(&self.coda) {
            Ok(kind) => {
                self.coda_kind = kind;
                true
            }
            Err(_) => {
                self.coda.pop();
                false
            }
        }
    }

    /// Handles a transform key while in the `Coda` phase.
    ///
    /// Tone, D-stroke and shape transforms are forwarded to the vowel
    /// handlers; anything else falls back to the literal handlers.
    #[inline]
    fn push_coda_transform<KM: Keymap>(
        &mut self,
        keymap: &KM,
        input: char,
    ) -> Result<InputResult, SyllableError> {
        if let Some(effect) = self.handle_tone_or_d_stroke(keymap, input) {
            return match effect {
                TransformResult::Applied => Ok(InputResult::Transformed),
                TransformResult::Reverted | TransformResult::NotApplicable => {
                    self.push_coda_literal(input)
                }
            };
        }

        match self.try_shape_transform(keymap, input) {
            TransformResult::Applied => Ok(InputResult::Transformed),
            TransformResult::Reverted | TransformResult::NotApplicable => {
                self.push_coda_literal(input)
            }
        }
    }

    // ─────────────────────────── Shared ───────────────────────────

    /// Resolves a tone or D-stroke transform key.
    #[inline]
    fn handle_tone_or_d_stroke<KM: Keymap>(
        &mut self,
        keymap: &KM,
        key: char,
    ) -> Option<TransformResult> {
        // ---------------------------------------------------------
        // 1. Tone
        // ---------------------------------------------------------
        if let Some(tone) = keymap.decode_tone(key) {
            return Some(self.apply_tone(tone));
        }

        // ---------------------------------------------------------
        // 2. D-stroke
        // ---------------------------------------------------------
        if keymap.is_stroke_key(key) {
            return Some(self.toggle_d_stroke());
        }

        None
    }

    /// Applies a Horn shape to the `u o` prefix (requires at least 2 vowels).
    fn try_uo_horn(&mut self) -> TransformResult {
        let vowels = &self.nucleus;

        match (vowels[0].value, vowels[1].value) {
            // ươ -> uow
            //
            // This always reverts, even with a third vowel.
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                self.nucleus[0].value = BaseVowel::U;
                self.nucleus[1].value = BaseVowel::O;
                TransformResult::Reverted
            }

            // ưô -> not applicable
            (BaseVowel::UHorn, BaseVowel::OCircumflex) => TransformResult::NotApplicable,

            // ưo -> ươ
            (BaseVowel::UHorn, BaseVowel::O) => self.try_vowel_shape(1, Shape::Horn),

            // uo -> uơ
            (BaseVowel::U, BaseVowel::O) => self.try_vowel_shape(1, Shape::Horn),

            // uơ -> ươ
            (BaseVowel::U, BaseVowel::OHorn) => self.try_vowel_shape(0, Shape::Horn),

            // uô -> uơ
            (BaseVowel::U, BaseVowel::OCircumflex) => self.try_vowel_shape(1, Shape::Horn),

            _ => TransformResult::NotApplicable,
        }
    }

    /// Applies a Circumflex shape to the `u o` prefix (requires at least 2 vowels).
    fn try_uo_circumflex(&mut self) -> TransformResult {
        let vowels = &self.nucleus;
        let first = vowels[0].value;

        match (first, vowels[1].value) {
            // ươ -> uô
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                self.nucleus[0].value = BaseVowel::U;

                match self.try_vowel_shape(1, Shape::Circumflex) {
                    effect @ (TransformResult::Applied | TransformResult::Reverted) => effect,
                    TransformResult::NotApplicable => {
                        self.nucleus[0].value = first;
                        TransformResult::NotApplicable
                    }
                }
            }

            // ưô -> not applicable
            (BaseVowel::UHorn, BaseVowel::OCircumflex) => TransformResult::NotApplicable,

            // ưo -> uô
            (BaseVowel::UHorn, BaseVowel::O) => {
                self.nucleus[0].value = BaseVowel::U;

                match self.try_vowel_shape(1, Shape::Circumflex) {
                    effect @ (TransformResult::Applied | TransformResult::Reverted) => effect,
                    TransformResult::NotApplicable => {
                        self.nucleus[0].value = first;
                        TransformResult::NotApplicable
                    }
                }
            }

            // uo -> uô
            (BaseVowel::U, BaseVowel::O) => self.try_vowel_shape(1, Shape::Circumflex),

            // uơ -> uô
            (BaseVowel::U, BaseVowel::OHorn) => self.try_vowel_shape(1, Shape::Circumflex),

            // uô -> undo (revert to "uo")
            (BaseVowel::U, BaseVowel::OCircumflex) => {
                self.nucleus[1].value = BaseVowel::O;
                TransformResult::Reverted
            }
            _ => TransformResult::NotApplicable,
        }
    }

    /// Tries to apply `key` as a shape transform on the vowel sequence,
    /// scanning from the last vowel backwards.
    fn try_shape_transform<KM: Keymap>(&mut self, keymap: &KM, key: char) -> TransformResult {
        let vseq_len = self.nucleus.len();

        for index in (0..vseq_len).rev() {
            let base = self.nucleus[index].value;

            let Some(shape) = keymap.decode_shape(key, base.root()) else {
                continue;
            };

            // Handle the special u / o cases.

            if self.nucleus_starts_with_uo() {
                match shape {
                    Shape::Horn => return self.try_uo_horn(),
                    Shape::Circumflex => return self.try_uo_circumflex(),
                    _ => {}
                }
            }

            let effect = self.try_vowel_shape(index, shape);
            // If that failed, keep trying the earlier vowels.
            if effect != TransformResult::NotApplicable {
                return effect;
            }
        }

        TransformResult::NotApplicable
    }

    /// Applies `shape` to the vowel at `index`, re-validating the nucleus.
    ///
    /// Applying the shape it already has reverts it; an invalid result rolls
    /// the vowel back.
    fn try_vowel_shape(&mut self, index: usize, shape: Shape) -> TransformResult {
        let old = self.nucleus[index].value;

        if old.shape() == shape && shape != Shape::None {
            self.nucleus[index].value = old.remove_shape();
            return TransformResult::Reverted;
        }

        let Ok(new) = old.replace_shape(shape) else {
            return TransformResult::NotApplicable;
        };

        self.nucleus[index].value = new;

        if self.nucleus.len() < 2 {
            return TransformResult::Applied;
        }

        match self.validate_nucleus::<3>() {
            NucleusStatus::Valid => TransformResult::Applied,
            NucleusStatus::InComplete => TransformResult::Applied,
            NucleusStatus::Dead => {
                self.nucleus[index].value = old;
                TransformResult::NotApplicable
            }
        }
    }
}
