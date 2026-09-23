use crate::{
    composition::syllable::InputEffect,
    keymap::Keymap,
    phonology::{
        decode_vowel,
        rules::{NucleusState, TonePlacement},
        BaseVowel, CasedBaseVowel, Coda, Onset, RootVowel, Shape, Tone,
    },
};
use arrayvec::ArrayVec;

#[inline(always)]
const fn is_q(ch: char) -> bool {
    matches!(ch, 'q' | 'Q')
}

#[inline(always)]
const fn is_i(ch: char) -> bool {
    matches!(ch, 'i' | 'I')
}

/// Effect of applying a transform key (shape/tone mark) to the syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformEffect {
    /// Applied a new mark to the syllable (e.g. `a` + `w` -> `ă`).
    Applied,
    /// Undid an existing mark back to base (e.g. `ă` + `w` -> `a`).
    Reverted,
    /// The key cannot transform the current state; pass through as a literal char.
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyllableBuildError {
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
    ) -> Result<InputEffect, SyllableBuildError> {
        // ─────────────────────────── Onset ───────────────────────────
        //
        // No vowel and no coda: still parsing the onset. A stroke key can
        // modify an existing D/Đ before being read as a literal character.
        if self.coda.is_empty() && self.vowels.is_empty() {
            if self.try_toggle_d_stroke(keymap, key) == TransformEffect::Applied {
                return Ok(InputEffect::Transformed);
            }

            if self.push_onset(key) {
                return Ok(InputEffect::StructurallyChanged);
            }

            // A lone Q must be followed by U to be valid.
            if self.onset.len() == 1 && is_q(self.onset[0]) {
                return Err(SyllableBuildError::InvalidOnset);
            }

            // The input may start the vowel nucleus.
            let Some(decoded) = decode_vowel(key) else {
                return Err(SyllableBuildError::InvalidOnset);
            };
            if !self.push_vowel(decoded) {
                return Err(SyllableBuildError::InvalidNucleus);
            }
            return Ok(InputEffect::StructurallyChanged);
        }

        // ─────────────────────────── Nucleus ───────────────────────────
        //
        // Vowels and/or a coda exist: offer the transform keys (tone / shape /
        // stroke) once, then parse the input as a literal vowel or coda.
        if self.try_transform(keymap, key, None) == TransformEffect::Applied {
            return Ok(InputEffect::Transformed);
        }

        if self.coda.is_empty() {
            // Vowel literal...
            let Some(decoded) = decode_vowel(key) else {
                // ...otherwise fall back to a coda.
                if self.push_coda(key) {
                    // Fold a leftover `uơ` / `ưo` prefix into `ươ` once a coda lands.
                    self.normalize_uo_horn();
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableBuildError::InvalidCoda);
            };
            // The nucleus may still reject the decoded vowel.
            if !self.push_vowel(decoded) {
                return Err(SyllableBuildError::InvalidNucleus);
            }
            self.normalize_uo_horn();
            return Ok(InputEffect::StructurallyChanged);
        }

        // Coda literal.
        if self.push_coda(key) {
            return Ok(InputEffect::StructurallyChanged);
        }
        Err(SyllableBuildError::InvalidCoda)
    }

    /// Consumes a literal character into the onset; returns `false` to fall
    /// back to the vowel parser. `q` is a transitional prefix waiting for `u`;
    /// `i` is left for the nucleus so `gi` stays ambiguous until another vowel.
    #[inline]
    fn push_onset(&mut self, input: char) -> bool {
        // Keep `i` in the nucleus so `g + i + V` can later become `gi`.
        if is_i(input) {
            return false;
        }

        if self.onset.is_empty() && is_q(input) {
            self.onset.push(input);
            return true;
        }

        self.try_update_onset(
            |onset| {
                onset.push(input);
            },
            |onset, _| {
                onset.pop();
            },
        )
    }

    // ─────────────────────────── Vowel ───────────────────────────

    /// Adds a decoded vowel to the nucleus (max 3 vowels); returns `false`
    /// when the tone conflicts or the resulting nucleus is invalid.
    fn push_vowel(&mut self, (vowel, tone): (CasedBaseVowel, Tone)) -> bool {
        let vowels_len = self.vowels.len();
        if vowels_len == 0 {
            // First vowel; adopt it and its tone.
            self.vowels.push(vowel);
            self.tone = tone;
            return true;
        } else if vowels_len >= 3 {
            return false;
        }

        let old_tone = self.tone;

        // A pretoned vowel's tone must not conflict with the tone already on
        // the syllable (e.g. `á` + pretoned `ắ` would collide as `áắ`).
        if self.tone == Tone::Flat {
            // A flat tone adopts the incoming vowel's tone (e.g. `iế`).
            self.tone = tone;
        } else if tone != Tone::Flat {
            return false;
        }

        // A second vowel after `g i` promotes `i` into the onset, forming `gi`;
        if vowels_len == 1 && *self.vowels[0].value() == BaseVowel::I && self.onset_kind == Onset::G
        {
            let i = self.vowels.pop().expect("vowels must contain i ");
            self.onset.push(if i.is_upper() { 'I' } else { 'i' });
            self.onset_kind = Onset::Gi;
        }

        self.vowels.push(vowel);

        if NucleusState::Dead == self.validate_vowels() {
            // Roll back the append and the adopted tone.
            self.vowels.pop();
            self.tone = old_tone;
            return false;
        }

        return true;
    }

    /// Consumes a literal char into the coda; returns `false` if the coda does
    /// not accept the input.
    #[inline]
    fn push_coda(&mut self, input: char) -> bool {
        self.try_update_coda(
            |coda| {
                coda.push(input);
            },
            |coda, _| {
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
    ) -> Result<InputEffect, SyllableBuildError> {
        let onset_len = self.onset.len();
        let vowels_len = self.vowels.len();
        let vowel_boundary = onset_len + vowels_len;
        let total_len = vowel_boundary + self.coda.len();

        // Append at the end == push.
        if index >= total_len {
            return self.push(keymap, key);
        }

        debug_assert!(index < total_len);

        // ─────────────────────────── Onset ───────────────────────────

        if index <= onset_len {
            // The d/đ stroke is toggled before literal insertion.
            if self.try_toggle_d_stroke(keymap, key) == TransformEffect::Applied {
                return Ok(InputEffect::Transformed);
            }

            // The onset consumes the input.
            if self.insert_onset(index, key) {
                self.normalize_i_placement();
                return Ok(InputEffect::StructurallyChanged);
            }

            // Only the last onset boundary may fall through to a vowel.
            if index != onset_len {
                return Err(SyllableBuildError::InvalidOnset);
            }

            if let Some(decoded) = decode_vowel(key) {
                if self.insert_vowel(0, decoded) {
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableBuildError::InvalidNucleus);
            }

            return Err(SyllableBuildError::InvalidOnset);
        }

        // ─────────────────────────── Vowel ───────────────────────────

        if index <= vowel_boundary {
            let vowel_index = index - onset_len;

            // Transforms are handled before literal insertion.
            if self.try_transform(keymap, key, Some(vowel_index)) == TransformEffect::Applied {
                return Ok(InputEffect::Transformed);
            }

            if let Some(decoded) = decode_vowel(key) {
                if self.insert_vowel(vowel_index, decoded) {
                    self.normalize_uo_horn();
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableBuildError::InvalidNucleus);
            }

            if vowel_index == vowels_len {
                if self.insert_coda(0, key) {
                    self.normalize_uo_horn();
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableBuildError::InvalidCoda);
            }

            return Err(SyllableBuildError::InvalidNucleus);
        }

        // ─────────────────────────── Coda ───────────────────────────

        if self.try_transform(keymap, key, None) == TransformEffect::Applied {
            return Ok(InputEffect::Transformed);
        }

        let coda_index = index - vowel_boundary;

        if self.insert_coda(coda_index, key) {
            return Ok(InputEffect::StructurallyChanged);
        }

        Err(SyllableBuildError::InvalidCoda)
    }

    // ─────────────────────────── Insert Onset ───────────────────────────

    /// Inserts a literal char into the onset (no vowel fallback).
    #[inline(always)]
    fn insert_onset(&mut self, index: usize, input: char) -> bool {
        self.try_update_onset(
            |onset| {
                onset.insert(index, input);
            },
            |onset, _| {
                onset.remove(index);
            },
        )
    }

    // ─────────────────────────── Insert Vowel ───────────────────────────

    /// Inserts a literal vowel into the nucleus; transforms are already handled
    /// by `insert()`.
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

        // A pretoned vowel's tone must not conflict with the tone already on
        // the syllable (e.g. `á` + pretoned `ắ` would collide as `áắ`).
        if self.tone == Tone::Flat {
            // A flat tone adopts the incoming vowel's tone (e.g. `iế`).
            self.tone = tone;
        } else if tone != Tone::Flat {
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

    /// Inserts a literal char into the coda.
    #[inline(always)]
    fn insert_coda(&mut self, coda_index: usize, input: char) -> bool {
        self.try_update_coda(
            |coda| {
                coda.insert(coda_index, input);
                Some(())
            },
            |coda, _| {
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
    ) -> Result<InputEffect, SyllableBuildError> {
        let onset_len = self.onset.len();
        let vowels_len = self.vowels.len();
        let total_len = onset_len + vowels_len + self.coda.len();

        debug_assert!(index < total_len);

        // Not a hot path; a whole-call transaction is fine.
        self.transaction(|this| {
            // ─────────────────────────── Onset ───────────────────────────

            if index < onset_len {
                if !this.remove_onset(index) {
                    return Err(SyllableBuildError::InvalidOnset);
                }

                // Removing onset may expose `I` and move it into the nucleus.
                this.normalize_i_placement();

                if this.validate_vowels() == NucleusState::Dead {
                    return Err(SyllableBuildError::InvalidNucleus);
                }

                return Ok(InputEffect::StructurallyChanged);
            }

            // ─────────────────────────── Vowel ───────────────────────────

            let vowel_index = index - onset_len;

            if vowel_index < vowels_len {
                if !this.remove_vowel(vowel_index, tone_placement) {
                    return Err(SyllableBuildError::InvalidNucleus);
                }

                // Removing a vowel may expose `I` from the onset.
                this.normalize_i_placement();

                if this.validate_vowels() == NucleusState::Dead {
                    return Err(SyllableBuildError::InvalidNucleus);
                }

                return Ok(InputEffect::StructurallyChanged);
            }

            // ─────────────────────────── Coda ───────────────────────────

            let coda_index = vowel_index - vowels_len;

            if !this.remove_coda(coda_index) {
                return Err(SyllableBuildError::InvalidCoda);
            }

            Ok(InputEffect::StructurallyChanged)
        })
    }

    // ─────────────────────────── Remove Onset ───────────────────────────

    /// Removes one character from the onset, restoring it if the result is invalid.
    #[inline(always)]
    fn remove_onset(&mut self, index: usize) -> bool {
        debug_assert!(index < self.onset.len());

        self.try_update_onset(
            |onset| onset.remove(index),
            |onset, removed| {
                onset.insert(index, removed);
            },
        )
    }

    // ─────────────────────────── Remove Vowel ───────────────────────────

    /// Removes the vowel at `index`, clearing the tone when it targeted that vowel
    /// or the nucleus becomes empty.
    #[inline(always)]
    fn remove_vowel(&mut self, index: usize, tone_placement: TonePlacement) -> bool {
        debug_assert!(index < self.vowels.len());

        // Recalculate the tone position after a removal shifts the vowels.
        let tone_pos = tone_placement.vowel_index(&self.vowels, self.coda.is_empty());

        if tone_pos == Some(index) {
            self.tone = Tone::Flat;
        }

        // Do not recompute UO normalization on deletion: `ươo` minus `ơ` leaves an
        // ambiguous `ưo`, so the remaining literal vowels are preserved.

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

    /// Removes one char from the coda, restoring it if the result is invalid.
    #[inline(always)]
    fn remove_coda(&mut self, index: usize) -> bool {
        debug_assert!(index < self.coda.len());

        self.try_update_coda(
            |coda| coda.remove(index),
            |coda, removed| {
                coda.insert(index, removed);
            },
        )
    }
}

impl BuildingSyllableBuilder {
    #[inline]
    fn transaction<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, SyllableBuildError>,
    ) -> Result<T, SyllableBuildError> {
        let snapshot = self.clone();

        match f(self) {
            Ok(value) => Ok(value),
            Err(err) => {
                *self = snapshot;
                Err(err)
            }
        }
    }
    /// Safely mutates the coda, validating the result: on success `coda_kind`
    /// is updated; on failure `revert` undoes the change with the undo data
    /// returned by `update`.
    #[inline(always)]
    pub fn try_update_coda<F, R, T>(&mut self, update: F, revert: R) -> bool
    where
        F: FnOnce(&mut ArrayVec<char, { Coda::MAX_LEN }>) -> T,
        R: FnOnce(&mut ArrayVec<char, { Coda::MAX_LEN }>, T),
    {
        if self.coda.len() >= Coda::MAX_LEN {
            return false;
        }

        let undo_data = update(&mut self.coda);

        match Coda::from_chars(&self.coda) {
            Ok(kind) => {
                self.coda_kind = kind;
                true
            }
            Err(_) => {
                // Only revert if `update` returned undo data.
                revert(&mut self.coda, undo_data);
                false
            }
        }
    }

    /// Safely mutates the onset, validating the result: on success `onset_kind`
    /// is updated; on failure `revert` undoes the change with the undo data
    /// returned by `update`.
    #[inline(always)]
    pub fn try_update_onset<F, R, T>(&mut self, update: F, revert: R) -> bool
    where
        F: FnOnce(&mut ArrayVec<char, { Onset::MAX_LEN }>) -> T,
        R: FnOnce(&mut ArrayVec<char, { Onset::MAX_LEN }>, T),
    {
        if self.onset.len() >= Onset::MAX_LEN {
            return false;
        }

        let undo_data = update(&mut self.onset);

        // Validate the onset.
        match Onset::from_chars(&self.onset) {
            Ok(kind) => {
                self.onset_kind = kind;
                true
            }
            Err(_) => {
                // Only revert if `update` returned undo data.
                revert(&mut self.onset, undo_data);
                false
            }
        }
    }

    /// Normalizes an unmarked `u o` prefix that arrived without a shape key.
    ///
    /// `uơ → ươ` and `ưo → ươ` are folded once at least two vowels are present.
    #[inline]
    fn normalize_uo_horn(&mut self) {
        // Needs at least two vowels (three while still coda-less).
        if self.vowels.len() < 2 || (self.coda.is_empty() && self.vowels.len() < 3) {
            return;
        }

        match (*self.vowels[0].value(), *self.vowels[1].value()) {
            (BaseVowel::U, BaseVowel::OHorn) => {
                self.vowels[0].set_value(BaseVowel::UHorn);
            }
            (BaseVowel::UHorn, BaseVowel::O) => {
                self.vowels[1].set_value(BaseVowel::OHorn);
            }
            _ => {}
        }
    }

    #[inline]
    fn normalize_i_placement(&mut self) {
        let vowels_len = self.vowels.len();
        // G + I + V -> Gi + V
        if self.onset_kind == Onset::G && vowels_len >= 2 && *self.vowels[0].value() == BaseVowel::I
        {
            let i = self.vowels.remove(0);

            self.onset.push(if i.is_upper() { 'I' } else { 'i' });
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

        // A lone `i` left in the onset drops back into the nucleus (I + V).
        if self.onset.len() == 1 && is_i(self.onset[0]) {
            let i = self.onset.pop().unwrap();
            self.onset_kind = Onset::None;
            self.vowels
                .insert(0, CasedBaseVowel::new(BaseVowel::I, i == 'I'));
        }
    }

    /// Validates the vowel nucleus against the rule table.
    #[inline(always)]
    fn validate_vowels(&self) -> NucleusState {
        let mut buf = [BaseVowel::A; 3];
        let len = self.vowels.len().min(3);

        for i in 0..len {
            buf[i] = *self.vowels[i].value();
        }

        NucleusState::check(&buf[..len])
    }

    /// Applies or toggles a tone on the syllable.
    ///
    /// Tapping the same tone again reverts to `Flat`; a different tone replaces
    /// the current one.
    #[inline]
    fn apply_tone(&mut self, tone: Tone) -> TransformEffect {
        if self.vowels.is_empty() {
            return TransformEffect::Ignored;
        }
        // Same tone toggles back to flat.
        if self.tone == tone {
            self.tone = Tone::Flat;
            return TransformEffect::Reverted;
        }

        // Replace the current tone.
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
            && self.vowels[0].value().root() == RootVowel::U
            && self.vowels[1].value().root() == RootVowel::O
    }

    /// Applies `shape` to the vowel at `index`, re-validating the nucleus.
    ///
    /// Applying the shape it already has reverts it; an invalid result rolls
    /// the vowel back.
    fn apply_vowel_shape(&mut self, vowel_index: usize, shape: Shape) -> TransformEffect {
        debug_assert!(vowel_index < self.vowels.len());

        let old = *self.vowels[vowel_index].value();

        // Shape already present -> revert to base.
        if old.has_shape(shape) && shape.is_some() {
            self.vowels[vowel_index].set_value(old.remove_shape());
            return TransformEffect::Reverted;
        }

        // Try applying the new shape.
        let Ok(new) = old.replace_shape(shape) else {
            return TransformEffect::Ignored;
        };

        self.vowels[vowel_index].set_value(new);

        // A lone vowel is always valid.
        if self.vowels.len() < 2 {
            return TransformEffect::Applied;
        }
        // Re-validate the vowel combination.
        match self.validate_vowels() {
            NucleusState::Valid => TransformEffect::Applied,
            NucleusState::InComplete => TransformEffect::Applied,
            NucleusState::Dead => {
                self.vowels[vowel_index].set_value(old);
                TransformEffect::Ignored
            }
        }
    }

    /// Applies a Horn shape to the `u o` prefix (needs at least 2 vowels).
    fn apply_uo_horn(&mut self) -> TransformEffect {
        if self.vowels.len() < 2 {
            return TransformEffect::Ignored;
        }

        match (*self.vowels[0].value(), *self.vowels[1].value()) {
            // ươ -> uo (revert).
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                self.vowels[0].set_value(BaseVowel::U);
                self.vowels[1].set_value(BaseVowel::O);
                TransformEffect::Reverted
            }

            // ưô, ưo, uo, uô -> Horn the vowel at index 1.
            (BaseVowel::UHorn, BaseVowel::OCircumflex | BaseVowel::O)
            | (BaseVowel::U, BaseVowel::O | BaseVowel::OCircumflex) => {
                self.apply_vowel_shape(1, Shape::Horn)
            }

            // uơ -> Horn index 0 (becomes ươ).
            (BaseVowel::U, BaseVowel::OHorn) => self.apply_vowel_shape(0, Shape::Horn),

            _ => TransformEffect::Ignored,
        }
    }

    fn apply_uo_circumflex(&mut self) -> TransformEffect {
        if self.vowels.len() < 2 {
            return TransformEffect::Ignored;
        }

        match (*self.vowels[0].value(), *self.vowels[1].value()) {
            // uo, uơ -> uô (Circumflex on index 1).
            (BaseVowel::U, BaseVowel::O | BaseVowel::OHorn) => {
                self.apply_vowel_shape(1, Shape::Circumflex)
            }

            // uô -> uo (revert).
            (BaseVowel::U, BaseVowel::OCircumflex) => {
                self.vowels[1].set_value(BaseVowel::O);
                TransformEffect::Reverted
            }

            // ươ, ưo -> uô: drop the Horn on `ư`, then Circumflex the `o`.
            (BaseVowel::UHorn, BaseVowel::OHorn | BaseVowel::O) => {
                let prev_u = *self.vowels[0].value();
                self.vowels[0].set_value(BaseVowel::U);

                match self.apply_vowel_shape(1, Shape::Circumflex) {
                    TransformEffect::Ignored => {
                        self.vowels[0].set_value(prev_u);
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
        // Scan vowels up to the cursor.
        let max_len = match vowel_upper_bound_idx {
            Some(idx) => idx.min(self.vowels.len()),
            None => self.vowels.len(),
        };

        // No vowel before the cursor -> nothing to transform.
        if max_len == 0 {
            return TransformEffect::Ignored;
        }
        // Special "uo" case (uow -> ươ, uoo -> uô) when the cursor is after the `u`.
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

        // Scan backwards through the vowels before the cursor.
        for index in (0..max_len).rev() {
            let base = *self.vowels[index].value();

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
            // 1. Tone.
            if let Some(tone) = keymap.decode_tone(key) {
                return self.apply_tone(tone);
            }

            // 2. Vowel diacritic (shape: hat, hook, crescent).
            if keymap.is_shape_key(key) {
                return self.try_transform_shape(keymap, key, vowel_upper_bound_idx);
            }
        }

        // 3. D-stroke.
        self.try_toggle_d_stroke(keymap, key)
    }
}
