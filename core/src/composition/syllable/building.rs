use crate::{
    keymap::Keymap,
    phonology::{
        decode_vowel,
        rules::{NucleusState, TonePlacement},
        BaseVowel, CasedBaseVowel, Coda, Onset, RootVowel, Shape, Tone,
    },
};
use arrayvec::ArrayVec;
use std::cell::Cell;

// Helper function
#[inline(always)]
const fn is_q(ch: char) -> bool {
    (ch as u32 | 0x20) == ('q' as u32)
}

#[inline(always)]
const fn is_i(ch: char) -> bool {
    (ch as u32 | 0x20) == ('i' as u32)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEffect {
    /// Mutated an existing character (e.g. assigned a tone, added a hat/hook:
    /// `a` + `w` -> `ă`).
    Transformed,
    /// Changed the buffer structure (inserted a new character or removed one).
    StructurallyChanged,
}

/// Effect of applying a transform key (shape/tone mark) to the syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformEffect {
    /// Applied a new mark/transform to the syllable (e.g. `a` + `w` -> `ă`).
    Applied,
    /// Removed/undid an existing mark, stripping back to base (e.g. `ă` + `w` -> `a`).
    Reverted,
    /// The key cannot transform the current state; pass through as a literal char.
    Ignored,
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
pub struct BuildingSyllableBuilder {
    onset_kind: Onset,
    onset: ArrayVec<char, { Onset::MAX_LEN }>,

    vowels: ArrayVec<CasedBaseVowel, 3>,

    coda_kind: Coda,
    coda: ArrayVec<char, { Coda::MAX_LEN }>,

    tone: Tone,
}

impl BuildingSyllableBuilder {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.onset.len() + self.vowels.len() + self.coda.len()
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
    pub fn vowels(&self) -> &[CasedBaseVowel] {
        &self.vowels
    }

    #[inline(always)]
    pub const fn tone(&self) -> Tone {
        self.tone
    }

    #[inline]
    pub fn reset(&mut self) {
        self.onset_kind = Onset::None;
        self.onset.clear();
        self.vowels.clear();
        self.coda_kind = Coda::None;
        self.coda.clear();
        self.tone = Tone::Flat;
    }

    #[inline(always)]
    pub fn tone_index(&self, tone_placement: TonePlacement) -> Option<usize> {
        tone_placement.vowel_index(&self.vowels, self.coda.is_empty())
    }

    #[inline(always)]
    pub fn to_chars(&self, tone_placement: TonePlacement) -> Vec<char> {
        let mut output = Vec::with_capacity(self.len());

        // 1. Onset
        output.extend(self.onset.iter().copied());

        // 2. Vowels
        if self.tone.is_some() {
            let tone_pos = self.tone_index(tone_placement);
            output.extend(self.vowels.iter().enumerate().map(|(idx, vowel)| {
                let active_tone = if Some(idx) == tone_pos {
                    self.tone
                } else {
                    Tone::Flat
                };
                vowel.to_char_tone(active_tone)
            }));
        } else {
            output.extend(self.vowels.iter().map(|vowel| vowel.to_char()));
        }

        // 3. Coda
        output.extend(self.coda.iter().copied());

        output
    }
}

impl BuildingSyllableBuilder {
    #[inline(always)]
    pub fn push<KM: Keymap>(
        &mut self,
        keymap: &KM,
        key: char,
    ) -> Result<InputEffect, SyllableError> {
        // ─────────────────────────── Onset ───────────────────────────
        //
        // No vowel and no coda means we are still parsing the onset.
        //
        // A stroke key can modify an existing D/Đ onset before being
        // interpreted as a literal character.
        if self.coda.is_empty() && self.vowels.is_empty() {
            if self.try_toggle_d_stroke(keymap, key) == TransformEffect::Applied {
                return Ok(InputEffect::Transformed);
            }
            // Try to consume the input as part of the onset.
            if self.push_onset(key) {
                return Ok(InputEffect::StructurallyChanged);
            }
            // Alone Q is a transitional onset and can only be followed by U.
            if self.onset.len() == 1 && is_q(self.onset[0]) {
                return Err(SyllableError::InvalidOnset);
            }
            // The input may start the vowel nucleus.
            if let Some(decoded) = decode_vowel(key) {
                if self.push_vowel(decoded) {
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableError::InvalidNucleus);
            }
            return Err(SyllableError::InvalidOnset);
        }

        // ─────────────────────────── Nucleus ───────────────────────────
        //
        // A vowel nucleus and/or a coda exists: a transform key (tone /
        // shape / stroke) is offered to the syllable once, then the input is
        // parsed as a literal vowel or coda character.
        if self.try_transform(keymap, key, None) == TransformEffect::Applied {
            return Ok(InputEffect::Transformed);
        }

        if self.coda.is_empty() {
            // Vowel literal.
            if let Some(decoded) = decode_vowel(key) {
                if self.push_vowel(decoded) {
                    self.normalize_uo_horn();
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableError::InvalidNucleus);
            }
            // Coda fallback: push vowel failed then consider char as coda.
            if self.push_coda(key) {
                // fold a leftover `uơ` / `ưo` prefix into `ươ` once a coda lands
                self.normalize_uo_horn();
                return Ok(InputEffect::StructurallyChanged);
            }
            return Err(SyllableError::InvalidCoda);
        }

        // Coda literal.
        if self.push_coda(key) {
            return Ok(InputEffect::StructurallyChanged);
        }
        Err(SyllableError::InvalidCoda)
    }

    /// Handles a literal character while building the onset.
    ///
    /// Returns:
    /// - `true` if the input was consumed by the onset.
    /// - `false` if the input must be handled as a vowel.
    ///
    /// `q` is kept as a transitional prefix waiting for `u` (to form `qu`);
    /// `i` is deliberately left for the vowel parser so that `gi` can remain
    /// ambiguous until another vowel follows.
    fn push_onset(&mut self, input: char) -> bool {
        // `q` is a transitional onset prefix; wait for `u`.
        if self.onset.is_empty() && is_q(input) {
            self.onset.push(input);
            return true;
        }
        // Do not accept `i` immediately as onset. Keep it as a vowel so
        // `gi` can be resolved later if another vowel follows.
        // A toned `i` (í, ì, ỉ, ị, ĩ) already fails `try_update_onset`, so no
        // special care is needed here.
        if is_i(input) {
            return false;
        }

        self.try_update_onset(
            |onset| onset.push(input),
            |onset| {
                onset.pop();
            },
        )
    }

    // ─────────────────────────── Vowel ───────────────────────────

    /// Handles a literal character while in the `Vowel` phase.
    ///
    /// A vowel joins the nucleus (or falls through to kill); a possible coda
    /// starter moves the phase into `Coda` and appends the character; anything
    /// else (invalid coda, non-letter) kills the parse.
    fn push_vowel(&mut self, (cased_base, tone): (CasedBaseVowel, Tone)) -> bool {
        let vowels_len = self.vowels.len();
        if vowels_len == 0 {
            // First vowel; add it to the nucleus.
            self.vowels.push(cased_base);
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
            // For example `i` can be continued typing with a pretoned vowel `ế`, forming `iế`.
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
        if vowels_len == 1 && self.vowels[0].value == BaseVowel::I && self.onset_kind == Onset::G {
            let i = self.vowels.pop().expect("vowels must contain i ");
            self.onset.push(if i.is_upper { 'I' } else { 'i' });
            self.onset_kind = Onset::Gi;
        }

        self.vowels.push(cased_base);

        if NucleusState::Dead == self.validate_vowels() {
            // rollback state
            self.vowels.pop();
            self.tone = old_tone;
            return false;
        }

        return true;
    }

    /// Handles a literal character while building the coda.
    ///
    /// Returns `true` if the input was consumed as part of the coda;
    /// otherwise returns `false`.
    #[inline(always)]
    fn push_coda(&mut self, input: char) -> bool {
        self.try_update_coda(
            |coda| coda.push(input),
            |coda| {
                coda.pop();
            },
        )
    }
}

impl BuildingSyllableBuilder {
    #[inline(always)]
    pub fn insert<KM: Keymap>(
        &mut self,
        keymap: &KM,
        index: usize,
        key: char,
    ) -> Result<InputEffect, SyllableError> {
        let onset_len = self.onset.len();
        let vowels_len = self.vowels.len();
        let vowel_boundary = onset_len + vowels_len;
        let total_len = vowel_boundary + self.coda.len();

        // Insert at the end == push.
        if index >= total_len {
            return self.push(keymap, key);
        }

        debug_assert!(index < total_len);

        // ─────────────────────────── Onset ───────────────────────────

        if index <= onset_len {
            // The d/đ modifier is handled before literal insertion.
            if self.try_toggle_d_stroke(keymap, key) == TransformEffect::Applied {
                return Ok(InputEffect::Transformed);
            }

            // The onset consumes the input.
            if self.insert_onset(index, key) {
                self.normalize_i_placement();
                return Ok(InputEffect::StructurallyChanged);
            }

            // Only the last onset boundary is allowed to fall through to vowel.
            if index != onset_len {
                return Err(SyllableError::InvalidOnset);
            }

            if let Some(decoded) = decode_vowel(key) {
                if self.insert_vowel(0, decoded) {
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableError::InvalidNucleus);
            }

            return Err(SyllableError::InvalidOnset);
        }

        // ─────────────────────────── Vowel ───────────────────────────

        if index <= vowel_boundary {
            let vowel_index = index - onset_len;

            // The modifier is handled before literal insertion.
            if self.try_transform(keymap, key, Some(vowel_index)) == TransformEffect::Applied {
                return Ok(InputEffect::Transformed);
            }

            if let Some(decoded) = decode_vowel(key) {
                if self.insert_vowel(vowel_index, decoded) {
                    self.normalize_uo_horn();
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableError::InvalidNucleus);
            }

            if vowel_index == vowels_len {
                if self.insert_coda(0, key) {
                    self.normalize_uo_horn();
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableError::InvalidCoda);
            }

            return Err(SyllableError::InvalidNucleus);
        }

        // ─────────────────────────── Coda ───────────────────────────

        if self.try_transform(keymap, key, None) == TransformEffect::Applied {
            return Ok(InputEffect::Transformed);
        }

        let coda_index = index - vowel_boundary;

        if self.insert_coda(coda_index, key) {
            return Ok(InputEffect::StructurallyChanged);
        }

        Err(SyllableError::InvalidCoda)
    }

    // ─────────────────────────── Insert Onset ───────────────────────────

    /// Tries to insert a literal into the onset.
    ///
    /// `true`  -> the onset consumed the input.
    /// `false` -> the onset does not accept the input.
    ///
    /// This function does NOT fall back to a vowel.
    #[inline(always)]
    fn insert_onset(&mut self, index: usize, input: char) -> bool {
        self.try_update_onset(
            |onset| onset.insert(index, input),
            |onset| {
                onset.remove(index);
            },
        )
    }

    // ─────────────────────────── Insert Vowel ───────────────────────────

    /// Inserts a literal vowel into the nucleus.
    ///
    /// This function does not handle transforms.
    /// The transform has already been handled by `insert()`.
    ///
    /// If the input is not a vowel and we are at the end of the nucleus,
    /// the input is tried as a coda.
    #[inline]
    fn insert_vowel(
        &mut self,
        vowel_index: usize,
        (cased_base, tone): (CasedBaseVowel, Tone),
    ) -> bool {
        debug_assert!(vowel_index <= self.vowels.len());
        if self.vowels.len() >= 3 {
            return false;
        }

        let old_tone = self.tone;

        // A pretoned vowel carrying a tone must not conflict with the tone
        // already on the syllable. e.g. "á" is already typed and a raw "ắ" is
        // pushed straight into the buffer - the two tones would collide (áắ)
        if self.tone == Tone::Flat {
            // If the tone is currently flat, a pretoned vowel can be used.
            // For example `i` can be continued typing with a pretoned vowel `ế`, forming `iế`.
            self.tone = tone;
        } else if tone != Tone::Flat {
            // Tone conflict
            return false;
        }

        self.vowels.insert(vowel_index, cased_base);

        if self.validate_vowels() == NucleusState::Dead {
            self.vowels.remove(vowel_index);
            self.tone = old_tone;

            return false;
        }

        true
    }

    // ─────────────────────────── Insert Coda ───────────────────────────

    /// Tries to insert a literal into the coda.
    ///
    /// `true`  -> the coda consumed the input.
    /// `false` -> the coda does not accept the input.
    #[inline(always)]
    fn insert_coda(&mut self, coda_index: usize, input: char) -> bool {
        self.try_update_coda(
            |coda| coda.insert(coda_index, input),
            |coda| {
                coda.remove(coda_index);
            },
        )
    }
}

// NOTE: UNCHECKED
impl BuildingSyllableBuilder {
    #[inline]
    pub fn remove(
        &mut self,
        index: usize,
        tone_placement: TonePlacement,
    ) -> Result<InputEffect, SyllableError> {
        let onset_len = self.onset.len();
        let vowels_len = self.vowels.len();
        let total_len = onset_len + vowels_len + self.coda.len();

        debug_assert!(index < total_len);

        // This is not the hot path so a transaction is fine here; may be optimized later
        self.transaction(|this| {
            // ─────────────────────────── Onset ───────────────────────────

            if index < onset_len {
                if !this.remove_onset(index) {
                    return Err(SyllableError::InvalidOnset);
                }

                // Removing onset may expose `I` and move it into the nucleus.
                this.normalize_i_placement();

                if this.validate_vowels() == NucleusState::Dead {
                    return Err(SyllableError::InvalidNucleus);
                }

                return Ok(InputEffect::StructurallyChanged);
            }

            // ─────────────────────────── Vowel ───────────────────────────

            let vowel_index = index - onset_len;

            if vowel_index < vowels_len {
                if !this.remove_vowel(vowel_index, tone_placement) {
                    return Err(SyllableError::InvalidNucleus);
                }

                // Removing a vowel may expose `I` from the onset.
                this.normalize_i_placement();

                if this.validate_vowels() == NucleusState::Dead {
                    return Err(SyllableError::InvalidNucleus);
                }

                return Ok(InputEffect::StructurallyChanged);
            }

            // ─────────────────────────── Coda ───────────────────────────

            let coda_index = vowel_index - vowels_len;

            if !this.remove_coda(coda_index) {
                return Err(SyllableError::InvalidCoda);
            }

            Ok(InputEffect::StructurallyChanged)
        })
    }

    // ─────────────────────────── Remove Onset ───────────────────────────

    /// Removes one literal character from the onset and revalidates it.
    ///
    /// The operation is locally transactional: if the resulting onset is invalid,
    /// the removed character is restored.
    #[inline(always)]
    fn remove_onset(&mut self, index: usize) -> bool {
        debug_assert!(index < self.onset.len());

        let removed = Cell::new(None);

        self.try_update_onset(
            |onset| {
                removed.set(Some(onset.remove(index)));
            },
            |onset| {
                onset.insert(index, removed.get().unwrap());
            },
        )
    }

    // ─────────────────────────── Remove Vowel ───────────────────────────

    /// Removes one vowel from the nucleus.
    ///
    /// Returns `false` only when `index` is outside the vowel range.
    #[inline(always)]
    fn remove_vowel(&mut self, index: usize, tone_placement: TonePlacement) -> bool {
        debug_assert!(index < self.vowels.len());

        // Removing a vowel may change the tone position according to the
        // selected tone scheme. Recalculate it here when that logic is added.
        let tone_pos = tone_placement.vowel_index(&self.vowels, self.coda.is_empty());

        if tone_pos == Some(index) {
            self.tone = Tone::Flat;
        }

        // Do not recalculate UO normalization after deletion.
        //
        // For example:
        //
        //     ư ơ o
        //
        // Removing `ơ` leaves:
        //
        //     ư o
        //
        // It is ambiguous whether `o` should become `ơ`, so deletion preserves
        // the remaining literal vowels.

        if index >= self.vowels.len() {
            return false;
        }

        self.vowels.remove(index);

        // A tone without a vowel has no semantic target.
        if self.vowels.is_empty() {
            self.tone = Tone::Flat;
        }

        true
    }

    // ─────────────────────────── Remove Coda ───────────────────────────

    /// Removes one literal character from the coda and revalidates it.
    ///
    /// The operation is locally transactional: if the resulting coda is invalid,
    /// the removed character is restored.
    #[inline(always)]
    fn remove_coda(&mut self, index: usize) -> bool {
        debug_assert!(index < self.coda.len());

        let removed = Cell::new(None);

        self.try_update_coda(
            |coda| {
                removed.set(Some(coda.remove(index)));
            },
            |coda| {
                coda.insert(index, removed.get().unwrap());
            },
        )
    }
}

impl BuildingSyllableBuilder {
    #[inline]
    fn transaction<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, SyllableError>,
    ) -> Result<T, SyllableError> {
        let snapshot = self.clone();

        match f(self) {
            Ok(value) => Ok(value),
            Err(err) => {
                *self = snapshot;
                Err(err)
            }
        }
    }
    /// Safely operates on `coda` by passing in a `mutate` and a `revert` closure.
    ///
    /// If `Coda::from_chars` validation succeeds, `coda_kind` is updated.
    /// If it fails, the `revert` closure is called to undo the `coda` change.
    #[inline(always)]
    pub fn try_update_coda<F, R>(&mut self, update: F, revert: R) -> bool
    where
        F: FnOnce(&mut ArrayVec<char, { Coda::MAX_LEN }>),
        R: FnOnce(&mut ArrayVec<char, { Coda::MAX_LEN }>),
    {
        if self.coda.len() >= Coda::MAX_LEN {
            return false;
        }
        // 1. Apply the edit to the coda array.
        update(&mut self.coda);

        // 2. Validate the coda phonology.
        match Coda::from_chars(&self.coda) {
            Ok(kind) => {
                self.coda_kind = kind;
                true
            }
            Err(_) => {
                // 3. Revert if the coda is invalid.
                revert(&mut self.coda);
                false
            }
        }
    }

    #[inline(always)]
    pub fn try_update_onset<F, R>(&mut self, update: F, revert: R) -> bool
    where
        F: FnOnce(&mut ArrayVec<char, { Onset::MAX_LEN }>),
        R: FnOnce(&mut ArrayVec<char, { Onset::MAX_LEN }>),
    {
        if self.onset.len() >= Onset::MAX_LEN {
            return false;
        }

        // 1. Apply the edit to the onset array.
        update(&mut self.onset);

        // 2. Validate the onset phonology.
        match Onset::from_chars(&self.onset) {
            Ok(kind) => {
                self.onset_kind = kind;
                true
            }
            Err(_) => {
                // 3. Revert if the onset is invalid.
                revert(&mut self.onset);
                false
            }
        }
    }

    /// Normalizes an unmarked `u o` prefix that arrived without a shape key.
    ///
    /// `uơ → ươ` and `ưo → ươ` are folded once at least two vowels are present.
    #[inline]
    fn normalize_uo_horn(&mut self) {
        // A UO-based normalization requires at least two vowels.
        // Without a coda, wait until the third vowel is present.
        if self.vowels.len() < 2 || (self.coda.is_empty() && self.vowels.len() < 3) {
            return;
        }

        match (self.vowels[0].value, self.vowels[1].value) {
            (BaseVowel::U, BaseVowel::OHorn) => {
                self.vowels[0].value = BaseVowel::UHorn;
            }
            (BaseVowel::UHorn, BaseVowel::O) => {
                self.vowels[1].value = BaseVowel::OHorn;
            }
            _ => {}
        }
    }

    #[inline]
    fn normalize_i_placement(&mut self) {
        let vowels_len = self.vowels.len();
        // G + I + V -> Gi + V
        if self.onset_kind == Onset::G && vowels_len >= 2 && self.vowels[0].value == BaseVowel::I {
            let i = self.vowels.remove(0);

            self.onset.push(if i.is_upper { 'I' } else { 'i' });
            self.onset_kind = Onset::Gi;
            return;
        }

        // Gi without a vowel -> G + I
        if self.onset_kind == Onset::Gi && vowels_len == 0 {
            let i = self.onset.pop().expect("onset must contain i");
            self.onset_kind = Onset::G;
            self.vowels
                .push(CasedBaseVowel::new(BaseVowel::I, i == 'I'));

            return;
        }

        // I is left alone in the onset after G was removed.
        //
        // I + V -> nucleus I + V
        if self.onset.len() == 1 && is_i(self.onset[0]) {
            let i = self.onset.pop().unwrap();
            self.onset_kind = Onset::None;
            self.vowels
                .insert(0, CasedBaseVowel::new(BaseVowel::I, i == 'I'));
        }
    }

    /// Validates the vowel nucleus against the rule table, keeping only the
    /// first `N` vowels.
    #[inline(always)]
    fn validate_vowels(&self) -> NucleusState {
        let mut buf = [BaseVowel::A; 3];
        let len = self.vowels.len().min(3);

        for i in 0..len {
            buf[i] = self.vowels[i].value;
        }

        NucleusState::from_vowels(&buf[..len])
    }

    /// Applies or toggles a tone on the syllable.
    ///
    /// Tapping the same tone again toggles the syllable back to `Flat`;
    /// a different tone replaces the current one.
    #[inline]
    fn apply_tone(&mut self, tone: Tone) -> TransformEffect {
        if self.vowels.is_empty() {
            return TransformEffect::Ignored;
        }
        // Same tone -> toggle back to Flat.
        else if self.tone == tone {
            self.tone = Tone::Flat;
            return TransformEffect::Reverted;
        }

        // Different tone -> replace the current tone.
        self.tone = tone;
        TransformEffect::Applied
    }

    /// Toggles the D-stroke on the onset cluster (`d` ↔ `đ`, `D` ↔ `Đ`).
    ///
    /// Only applies when the onset is a lone `D`/`Đ`; otherwise the stroke key
    /// cannot act here.
    #[inline]
    fn toggle_d_stroke(&mut self) -> TransformEffect {
        debug_assert!(!self.onset.is_empty());
        let onset_chars = &mut self.onset;

        match self.onset_kind {
            Onset::D => {
                onset_chars[0] = if onset_chars[0] == 'd' { 'đ' } else { 'Đ' };
                self.onset_kind = Onset::DStroke;
                TransformEffect::Applied
            }
            Onset::DStroke => {
                onset_chars[0] = if onset_chars[0] == 'đ' { 'd' } else { 'D' };
                self.onset_kind = Onset::D;
                TransformEffect::Reverted
            }
            _ => TransformEffect::Ignored,
        }
    }

    /// Whether the nucleus starts with an unmarked `u o` pair.
    #[inline(always)]
    fn vowels_starts_with_uo(&self) -> bool {
        self.vowels.len() > 1
            && self.vowels[0].value.root() == RootVowel::U
            && self.vowels[1].value.root() == RootVowel::O
    }

    /// Applies `shape` to the vowel at `index`, re-validating the nucleus.
    ///
    /// Applying the shape it already has reverts it; an invalid result rolls
    /// the vowel back.
    fn apply_vowel_shape(&mut self, vowel_index: usize, shape: Shape) -> TransformEffect {
        debug_assert!(vowel_index < self.vowels.len());

        let old = self.vowels[vowel_index].value;

        // If the vowel already has this shape -> Toggle/Undo back to the base form
        if old.has_shape(shape) && shape.is_some() {
            self.vowels[vowel_index].value = old.remove_shape();
            return TransformEffect::Reverted;
        }

        // Try applying the new shape
        let Ok(new) = old.replace_shape(shape) else {
            return TransformEffect::Ignored;
        };

        self.vowels[vowel_index].value = new;

        // A single vowel is always valid
        if self.vowels.len() < 2 {
            return TransformEffect::Applied;
        }
        // 4. Check the validity of the vowel combination (Nucleus)
        match self.validate_vowels() {
            NucleusState::Valid => TransformEffect::Applied,
            NucleusState::InComplete => TransformEffect::Applied,
            NucleusState::Dead => {
                self.vowels[vowel_index].value = old;
                TransformEffect::Ignored
            }
        }
    }

    /// Applies a Horn shape to the `u o` prefix (requires at least 2 vowels).
    fn apply_uo_horn(&mut self) -> TransformEffect {
        if self.vowels.len() < 2 {
            return TransformEffect::Ignored;
        }

        match (self.vowels[0].value, self.vowels[1].value) {
            // ươ -> uo (Revert)
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                self.vowels[0].value = BaseVowel::U;
                self.vowels[1].value = BaseVowel::O;
                TransformEffect::Reverted
            }

            // ưô, ưo, uo, uô -> transform element at index 1 into Horn
            (BaseVowel::UHorn, BaseVowel::OCircumflex | BaseVowel::O)
            | (BaseVowel::U, BaseVowel::O | BaseVowel::OCircumflex) => {
                self.apply_vowel_shape(1, Shape::Horn)
            }

            // uơ -> transform element at index 0 into Horn (becomes ươ)
            (BaseVowel::U, BaseVowel::OHorn) => self.apply_vowel_shape(0, Shape::Horn),

            _ => TransformEffect::Ignored,
        }
    }

    fn apply_uo_circumflex(&mut self) -> TransformEffect {
        if self.vowels.len() < 2 {
            return TransformEffect::Ignored;
        }

        match (self.vowels[0].value, self.vowels[1].value) {
            // uo, uơ -> uô
            (BaseVowel::U, BaseVowel::O | BaseVowel::OHorn) => {
                self.apply_vowel_shape(1, Shape::Circumflex)
            }

            // uô -> uo (Undo/Revert)
            (BaseVowel::U, BaseVowel::OCircumflex) => {
                self.vowels[1].value = BaseVowel::O;
                TransformEffect::Reverted
            }

            // ươ, ưo -> uô (switch v0 from UHorn to U, then apply Circumflex to v1)
            (BaseVowel::UHorn, BaseVowel::OHorn | BaseVowel::O) => {
                let prev_u = self.vowels[0].value;
                self.vowels[0].value = BaseVowel::U;

                match self.apply_vowel_shape(1, Shape::Circumflex) {
                    TransformEffect::Ignored => {
                        self.vowels[0].value = prev_u; // Rollback if it could not be applied
                        TransformEffect::Ignored
                    }
                    effect => effect,
                }
            }

            _ => TransformEffect::Ignored,
        }
    }

    /// Tries to apply `key` as a shape transform on the vowel sequence,
    /// scanning from the last vowel backwards.
    fn try_transform_shape<KM: Keymap>(
        &mut self,
        keymap: &KM,
        key: char,
        vowel_upper_bound_idx: Option<usize>,
    ) -> TransformEffect {
        // 1. Determine the cursor upper bound.
        let max_len = match vowel_upper_bound_idx {
            Some(idx) => idx.min(self.vowels.len()),
            None => self.vowels.len(),
        };

        // If the cursor is at position 0 there are no vowels to its left -> skip immediately!
        if max_len == 0 {
            return TransformEffect::Ignored;
        }
        // 2. Handle the special "uo" case (uow -> ươ, uoo -> uô).
        // NEW CONDITION: only check "uo" when the cursor is AFTER the 'u' (i.e. max_len >= 1)
        else if self.vowels_starts_with_uo() {
            if let Some(shape) = keymap
                .decode_shape(key, RootVowel::O)
                .or_else(|| keymap.decode_shape(key, RootVowel::U))
            {
                match shape {
                    Shape::Horn => return self.apply_uo_horn(),
                    Shape::Circumflex => return self.apply_uo_circumflex(),
                    _ => {}
                }
            }
        }

        // 3. Scan backwards from (max_len - 1) to 0 (only vowels BEFORE the cursor)
        for index in (0..max_len).rev() {
            let base = self.vowels[index].value;

            if let Some(shape) = keymap.decode_shape(key, base.root()) {
                let effect = self.apply_vowel_shape(index, shape);
                if effect != TransformEffect::Ignored {
                    return effect;
                }
            }
        }

        TransformEffect::Ignored
    }

    #[inline]
    fn try_toggle_d_stroke<KM: Keymap>(&mut self, keymap: &KM, key: char) -> TransformEffect {
        if !self.onset.is_empty() && keymap.is_stroke_key(key) {
            return self.toggle_d_stroke();
        }
        TransformEffect::Ignored
    }

    #[inline]
    fn try_transform<KM: Keymap>(
        &mut self,
        keymap: &KM,
        key: char,
        vowel_upper_bound_idx: Option<usize>,
    ) -> TransformEffect {
        if !self.vowels.is_empty() {
            // 1. Tone
            if let Some(tone) = keymap.decode_tone(key) {
                return self.apply_tone(tone);
            }

            // 2. Vowel diacritic (Shape: hat, hook, crescent)
            if keymap.is_shape_key(key) {
                return self.try_transform_shape(keymap, key, vowel_upper_bound_idx);
            }
        }

        // 3. D-stroke
        self.try_toggle_d_stroke(keymap, key)
    }
}
