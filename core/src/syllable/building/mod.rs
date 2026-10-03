//! The building-phase syllable: an incremental Vietnamese syllable under
//! construction; editing paths live in the sibling `push`, `insert`, `remove`,
//! `error`, and `types` modules. Every mutator validates as it goes, so a
//! rejected edit is rolled back and the syllable keeps its contents.
use crate::{
    keymap::Keymap,
    phonology::{
        validate_phonotactics, BaseVowel, Coda, NucleusState, NucleusStateResolver, Onset,
        RootVowel, Shape, Tone, TonePlacement, Vowel, NUCLEUS_MAX_LEN,
    },
};

use super::SyllableChars;

mod error;
mod insert;
mod push;
mod remove;
mod types;

use types::*;

pub use error::SyllableBuildError;
pub use types::{TransformEffect, TransformTarget};

/// A single Vietnamese syllable under construction.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct BuildingSyllable {
    onset_kind: Onset,
    onset: OnsetChars,

    nucleus: TonelessNucleus, // All toneless vowels; the tone is held in `tone`
    nucleus_state: NucleusState,

    coda_kind: Coda,
    coda: CodaChars,

    tone: Tone,
}

#[inline(always)]
const fn is_q_ignore_case(ch: char) -> bool {
    matches!(ch, 'q' | 'Q')
}

#[inline(always)]
const fn is_i_ignore_case(ch: char) -> bool {
    matches!(ch, 'i' | 'I')
}

impl BuildingSyllable {
    /// The longest word this builder can produce: onset, nucleus, coda.
    pub(crate) const MAX_LEN: usize = Coda::MAX_LEN + NUCLEUS_MAX_LEN + Onset::MAX_LEN;

    // ─────────────────────────── Accessors ───────────────────────────

    /// Total rendered length: onset + vowels + coda.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.onset.len() + self.nucleus.len() + self.coda.len()
    }

    /// Total UTF-8 length: onset + vowels + coda.
    #[inline(always)]
    pub fn len_utf8(&self) -> usize {
        let mut sum = (self.onset.len() + (self.onset_kind == Onset::DStroke) as usize)
            + self.coda.len()
            + (self.tone.is_some() as usize);

        // Temp vars
        let len = self.nucleus.len();

        let mut i = 0;
        while i < len {
            sum += if self.nucleus[i].base().is_shaped() {
                2
            } else {
                1
            };
            i += 1;
        }

        sum
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

    /// The nucleus vowels; all are stored toneless — the syllable's tone
    /// lives in [`Self::tone`] and is applied when rendering.
    #[inline(always)]
    pub fn nucleus(&self) -> &[Vowel] {
        self.nucleus.vowels()
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn nucleus_state(&self) -> NucleusState {
        self.nucleus_state
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub const fn tone(&self) -> Tone {
        self.tone
    }

    #[inline(always)]
    pub fn tone_vowel_index(&self, tone_placement: TonePlacement) -> Option<usize> {
        tone_placement.vowel_index(&self.nucleus, self.coda.is_empty())
    }

    // ─────────────────────────── Lifecycle ───────────────────────────

    /// Returns the syllable to its empty state, dropping every part.
    #[allow(dead_code)]
    #[inline]
    pub fn reset(&mut self) {
        self.onset_kind = Onset::None;
        self.onset.clear();

        self.nucleus.clear();
        self.nucleus_state = NucleusState::InComplete;

        self.coda_kind = Coda::None;
        self.coda.clear();

        self.tone = Tone::Flat;
    }

    // ─────────────────────────── Rendering ───────────────────────────

    /// Appends the rendered syllable to `output`: onset, then the tone-marked
    /// vowels, then the coda. Writes straight to `output`, unlike
    /// [`Self::to_chars`], which builds an intermediate buffer.
    pub fn write_to(&self, tone_placement: TonePlacement, output: &mut String) {
        // UTF-8 estimate: onset/coda are ASCII except one possible `đ`, while
        // each nucleus character can take up to 3 bytes.
        let estimated_bytes = self.onset.len() + 1 + self.coda.len() + self.nucleus.len() * 3;
        output.reserve(estimated_bytes);

        for &c in self.onset.iter() {
            output.push(c);
        }

        // Nucleus vowels are stored flat; apply the tone only to the vowel
        // chosen by the placement rules.
        if self.tone == Tone::Flat {
            for &vowel in self.nucleus.iter() {
                output.push(vowel.to_char());
            }
        } else {
            let tone_pos = self.tone_vowel_index(tone_placement);
            for (idx, vowel) in self.nucleus.iter().enumerate() {
                let tone = if Some(idx) == tone_pos {
                    self.tone
                } else {
                    Tone::Flat
                };

                output.push(vowel.with_tone(tone).to_char());
            }
        }

        for &c in self.coda.iter() {
            output.push(c);
        }
    }

    /// Yields the rendered characters in the same order as [`Self::to_chars`].
    #[inline(always)]
    pub fn iter_chars(&self, tone_placement: TonePlacement) -> impl Iterator<Item = char> + '_ {
        self.to_chars(tone_placement).into_iter()
    }

    /// Renders into a [`SyllableChars`]: onset, then the tone-marked vowels,
    /// then the coda. A word this builder can produce is at most
    /// [`Self::MAX_LEN`] characters, so it always fits the inline capacity.
    pub fn to_chars(&self, tone_placement: TonePlacement) -> SyllableChars {
        let mut output = SyllableChars::new();

        output.extend_from_slice(&self.onset);

        // Nucleus vowels are stored flat; apply the tone only to the vowel
        // chosen by the placement rules.
        if self.tone == Tone::Flat {
            for &vowel in self.nucleus.iter() {
                output.push(vowel.to_char());
            }
        } else {
            let tone_pos = self.tone_vowel_index(tone_placement);
            for (idx, vowel) in self.nucleus.iter().enumerate() {
                let tone = if Some(idx) == tone_pos {
                    self.tone
                } else {
                    Tone::Flat
                };

                output.push(vowel.with_tone(tone).to_char());
            }
        }

        output.extend_from_slice(&self.coda);

        output
    }

    // ─────────────────────────── Validation ───────────────────────────

    // The `try_update_*` methods below share one shape: mutate, re-derive the
    // cached state, and undo with the rollback data on failure; each returns
    // whether the syllable is still valid.

    /// Whether the syllable passes the phonotactic rules; an incomplete
    /// nucleus fails first, so a half-typed syllable never reaches them.
    #[allow(dead_code)]
    pub fn is_phonotactically_valid(&self) -> bool {
        if self.nucleus_state.is_incomplete() {
            return false;
        }

        validate_phonotactics(self.onset_kind, &self.nucleus, self.tone, self.coda_kind).is_ok()
    }

    /// Mutates the onset; updates `onset_kind` on success, otherwise undoes
    /// the change with the undo data.
    #[inline(always)]
    fn try_update_onset<F, R, T>(&mut self, update: F, rollback: R) -> bool
    where
        F: FnOnce(&mut OnsetChars) -> T,
        R: FnOnce(&mut OnsetChars, T),
    {
        let undo_data = update(&mut self.onset);

        match Onset::from_chars(&self.onset) {
            Ok(kind) => {
                self.onset_kind = kind;
                true
            }
            Err(_) => {
                rollback(&mut self.onset, undo_data);
                false
            }
        }
    }

    /// Mutates the nucleus; caches `nucleus_state` on success, else rolls
    /// back. The caller owns the tone invariant: `update` must insert only
    /// toneless vowels, with the syllable's tone kept in `self.tone`.
    #[inline(always)]
    fn try_update_nucleus<F, R, T>(&mut self, update: F, rollback: R) -> bool
    where
        F: FnOnce(&mut TonelessNucleus) -> T,
        R: FnOnce(&mut TonelessNucleus, T),
    {
        let undo_data = update(&mut self.nucleus);

        // A single vowel is always a nucleus; there is nothing to check.
        if self.nucleus.len() == 1 {
            self.nucleus_state = NucleusState::Valid;
            return true;
        }

        let state = self.nucleus.resolve_state();

        if state.is_dead() {
            rollback(&mut self.nucleus, undo_data);
            return false;
        }

        self.nucleus_state = state;

        true
    }

    /// Mutates the coda; updates `coda_kind` on success, otherwise undoes the
    /// change with the undo data.
    #[inline(always)]
    fn try_update_coda<F, R, T>(&mut self, update: F, rollback: R) -> bool
    where
        F: FnOnce(&mut CodaChars) -> T,
        R: FnOnce(&mut CodaChars, T),
    {
        let undo_data = update(&mut self.coda);

        match Coda::from_chars(&self.coda) {
            Ok(kind) => {
                self.coda_kind = kind;
                true
            }
            Err(_) => {
                rollback(&mut self.coda, undo_data);
                false
            }
        }
    }

    // ─────────────────────────── Normalization ───────────────────────────

    /// Folds an unmarked `u o` prefix that arrived without a shape key:
    /// `uơ → ươ` and `ưo → ươ` once at least two vowels are present.
    #[inline]
    fn normalize_uo_horn(&mut self) {
        // Only once the pair is unambiguous: two vowels need a coda, three need none.
        if self.nucleus.len() < 2 || (self.nucleus.len() < 3 && self.coda.is_empty()) {
            return;
        }

        use BaseVowel::*;
        match (self.nucleus[0].base(), self.nucleus[1].base()) {
            (U, OHorn) => {
                self.nucleus[0].set_base(UHorn);
            }
            (UHorn, O) => {
                self.nucleus[1].set_base(OHorn);
            }
            _ => {}
        }
    }

    // NOTE:
    // Kept commented out intentionally as a reference for the previous
    // `G + I + V <-> Gi + V` normalization strategy.
    // Do not remove unless this legacy logic is confirmed to be no longer
    // useful for reference or future restoration.
    //
    // #[inline]
    // fn normalize_i_placement(&mut self) {
    //     let vowels_len = self.nucleus.len();
    //     // G + I + V -> Gi + V
    //     if self.onset_kind == Onset::G && vowels_len >= 2 && self.nucleus[0].base() == BaseVowel::I
    //     {
    //         let i = self.nucleus.remove(0);

    //         self.onset.push(if i.is_upper() { 'I' } else { 'i' });
    //         self.onset_kind = Onset::Gi;
    //         return;
    //     }

    //     // Gi without a vowel -> G + I
    //     if self.onset_kind == Onset::Gi && vowels_len == 0 {
    //         let i = self.onset.pop().expect("onset must contain i");
    //         self.onset_kind = Onset::G;
    //         self.nucleus
    //             .push(Vowel::new(BaseVowel::I, Tone::Flat, i == 'I'));

    //         return;
    //     }

    //     // A lone `i` left in the onset drops back into the nucleus (I + V).
    //     if self.onset.len() == 1 && is_i_ignore_case(self.onset[0]) {
    //         let i = self.onset.pop().unwrap();
    //         self.onset_kind = Onset::None;
    //         self.nucleus
    //             .insert(0, Vowel::new(BaseVowel::I, Tone::Flat, i == 'I'));
    //     }
    // }

    // ─────────────────────────── Transforms ───────────────────────────

    /// Routes `key` to the transform it names: tone, vowel shape, or the
    /// D-stroke, in that order.
    #[inline]
    fn try_transform<KM: Keymap>(
        &mut self,
        keymap: &KM,
        key: char,
        vowel_upper_bound_idx: Option<usize>,
    ) -> TransformEffect {
        // 1. Tone — an undecodable tone key falls through to shape and stroke.
        if keymap.is_tone_key(key) {
            if let Some(tone) = keymap.decode_tone(key) {
                return self.apply_tone(tone);
            }
        }

        // 2. Vowel diacritic (shape: hat, hook, crescent).
        if keymap.is_shape_key(key) {
            return self.try_transform_shape(keymap, key, vowel_upper_bound_idx);
        }

        // 3. D-stroke.
        self.try_toggle_d_stroke(keymap, key)
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
            Some(idx) => idx.min(self.nucleus.len()),
            None => self.nucleus.len(),
        };

        // No vowel before the cursor -> nothing to transform.
        if max_len == 0 {
            return TransformEffect::None;
        }

        // Special "uo" case (uow -> ươ, uoo -> uô) when the cursor is after the `u`.
        if self.nucleus_starts_with_uo() {
            let shape = match keymap.decode_shape(key, RootVowel::O) {
                Some(s) => s,
                None => match keymap.decode_shape(key, RootVowel::U) {
                    // Only Horn may target `u`; any other U-shape is not applicable.
                    Some(Shape::Horn) => Shape::Horn,
                    _ => return TransformEffect::None,
                },
            };
            return self.apply_uo_shape(shape);
        }

        for index in (0..max_len).rev() {
            let base = self.nucleus[index].base();

            if let Some(shape) = keymap.decode_shape(key, base.root()) {
                match self.apply_vowel_shape(index, shape) {
                    TransformEffect::None => continue,
                    effect => return effect,
                }
            }
        }

        TransformEffect::None
    }

    /// Applies the D-stroke when `key` is the stroke key, else reports
    /// [`TransformResult::NotApplicable`].
    #[inline(always)]
    fn try_toggle_d_stroke<KM: Keymap>(&mut self, keymap: &KM, key: char) -> TransformEffect {
        if keymap.is_stroke_key(key) {
            return self.toggle_d_stroke();
        }
        TransformEffect::None
    }

    /// Toggles the D-stroke on the onset (`d` ↔ `đ`, `D` ↔ `Đ`); only a lone
    /// `D`/`Đ` onset can take it, otherwise `NotApplicable`.
    #[inline(always)]
    fn toggle_d_stroke(&mut self) -> TransformEffect {
        match self.onset_kind {
            Onset::D => {
                debug_assert!(
                    matches!(self.onset[0], 'd' | 'D'),
                    "Onset state desync: onset_kind is D, but onset[0] is {:?}",
                    self.onset[0]
                );
                self.onset[0] = if self.onset[0] == 'd' { 'đ' } else { 'Đ' };
                self.onset_kind = Onset::DStroke;
                TransformEffect::Applied(TransformTarget::DStroke)
            }
            Onset::DStroke => {
                debug_assert!(
                    matches!(self.onset[0], 'đ' | 'Đ'),
                    "Onset state desync: onset_kind is DStroke, but onset[0] is {:?}",
                    self.onset[0]
                );

                self.onset[0] = if self.onset[0] == 'đ' { 'd' } else { 'D' };
                self.onset_kind = Onset::D;
                TransformEffect::Reverted(TransformTarget::DStroke)
            }
            _ => TransformEffect::None,
        }
    }

    /// Applies or toggles a tone: the same tone again reverts to `Flat`, a
    /// different one replaces it; with no nucleus it is `NotApplicable`.
    #[inline]
    fn apply_tone(&mut self, tone: Tone) -> TransformEffect {
        if self.nucleus.is_empty() {
            return TransformEffect::None;
        }
        if self.tone == tone {
            self.tone = Tone::Flat;
            return TransformEffect::Reverted(TransformTarget::Tone);
        }

        self.tone = tone;
        TransformEffect::Applied(TransformTarget::Tone)
    }

    /// Applies `shape` to the vowel at `vowel_index`, re-validating the
    /// nucleus: re-applying the current shape reverts it, an invalid result
    /// rolls the vowel back.
    fn apply_vowel_shape(&mut self, vowel_index: usize, shape: Shape) -> TransformEffect {
        debug_assert!(
            vowel_index < self.nucleus.len(),
            "vowel_index ({vowel_index}) out of bounds for nucleus of length {}",
            self.nucleus.len()
        );

        let old = self.nucleus[vowel_index].base();

        if old.is_shape(shape) && shape.is_some() {
            self.nucleus[vowel_index].set_base(old.remove_shape());
            return TransformEffect::Reverted(TransformTarget::Nucleus(vowel_index));
        }

        let Some(new) = old.replace_shape(shape) else {
            return TransformEffect::None;
        };

        if self.try_update_nucleus(
            |nucleus| {
                nucleus[vowel_index].set_base(new);
            },
            |nucleus, _| {
                nucleus[vowel_index].set_base(old);
            },
        ) {
            return TransformEffect::Applied(TransformTarget::Nucleus(vowel_index));
        }

        TransformEffect::None
    }

    /// Applies a shape to a `u o`-prefix nucleus (needs at least 2 vowels).
    fn apply_uo_shape(&mut self, shape: Shape) -> TransformEffect {
        if self.nucleus.len() < 2 {
            return TransformEffect::None;
        }

        use BaseVowel::*;
        match shape {
            Shape::Horn => match (self.nucleus[0].base(), self.nucleus[1].base()) {
                // ươ -> uo (revert).
                (UHorn, OHorn) => {
                    self.nucleus[0].set_base(U);
                    self.nucleus[1].set_base(O);
                    TransformEffect::Reverted(TransformTarget::UoPair)
                }

                // uơ -> Horn index 0 (becomes ươ).
                (U, OHorn) => self.apply_vowel_shape(0, Shape::Horn),

                // ưô, ưo, uo, uô -> Horn the vowel at index 1.
                (UHorn | U, O | OCircumflex) => self.apply_vowel_shape(1, Shape::Horn),

                _ => return TransformEffect::None,
            },

            Shape::Circumflex => match (self.nucleus[0].base(), self.nucleus[1].base()) {
                // uô -> uo (revert).
                (U, OCircumflex) => {
                    self.nucleus[1].set_base(BaseVowel::O);
                    TransformEffect::Reverted(TransformTarget::Nucleus(1))
                }

                // uo, uơ -> uô (Circumflex on index 1).
                (U, O | OHorn) => self.apply_vowel_shape(1, Shape::Circumflex),

                // ươ, ưo -> uô: drop the Horn on `ư`, then Circumflex the `o`.
                (UHorn, OHorn | O) => {
                    let old_o = self.nucleus[1].base();

                    if self.try_update_nucleus(
                        |nucleus| {
                            nucleus[0].set_base(U);
                            nucleus[1].set_base(OCircumflex);
                        },
                        |nucleus, _| {
                            nucleus[0].set_base(UHorn);
                            nucleus[1].set_base(old_o);
                        },
                    ) {
                        return TransformEffect::Applied(TransformTarget::UoPair);
                    }

                    return TransformEffect::None;
                }

                _ => TransformEffect::None,
            },

            _ => TransformEffect::None,
        }
    }

    /// Whether the nucleus starts with an unmarked `u o` pair.
    #[inline(always)]
    fn nucleus_starts_with_uo(&self) -> bool {
        self.nucleus.len() > 1
            && self.nucleus[0].root() == RootVowel::U
            && self.nucleus[1].root() == RootVowel::O
    }
}
