//! The building-phase syllable: an incremental, validated Vietnamese syllable
//! under construction. Editing paths live in sibling modules:
//!
//! * [`push`]: appending at the end,
//! * [`insert`]: explicit-cursor insertion,
//! * [`remove`]: deletion.
use crate::{
    keymap::Keymap,
    phonology::{
        BaseVowel, Coda, ExtendedBaseVowel, NucleusState, Onset, PhonotacticValidator, RootVowel,
        Shape, Tone, TonePlacement, ValidationError, NUCLEUS_MAX_LEN,
    },
    util::InlineVec,
};

mod insert;
mod push;
mod remove;

#[inline(always)]
const fn is_q_ignore_case(ch: char) -> bool {
    matches!(ch, 'q' | 'Q')
}

#[inline(always)]
const fn is_i_ignore_case(ch: char) -> bool {
    matches!(ch, 'i' | 'I')
}

/// Effect of applying a transform key (shape/tone mark) to the syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransformResult {
    /// Applied a new mark to the syllable (e.g. `a` + `w` -> `ă`).
    Applied,
    /// Undid an existing mark back to base (e.g. `ă` + `w` -> `a`).
    Reverted,
    /// The key cannot transform the current state; pass through as a literal char.
    NotApplicable,
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

type Nucleus = InlineVec<ExtendedBaseVowel, NUCLEUS_MAX_LEN>;

impl Nucleus {
    /// Copies the vowels into `dst`, returning the number copied.
    #[inline(always)]
    fn bases(&self, dst: &mut [BaseVowel]) -> usize {
        let count = self.len().min(dst.len());
        for (src, dst) in self.iter().take(count).zip(&mut dst[..count]) {
            *dst = src.get();
        }
        count
    }
}

type OnsetChars = InlineVec<char, { Onset::MAX_LEN }>;
type CodaChars = InlineVec<char, { Coda::MAX_LEN }>;

/// A single Vietnamese syllable under construction.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BuildingSyllable {
    onset_kind: Onset,
    onset: OnsetChars,

    nucleus: Nucleus,
    nucleus_state: NucleusState, // cached from check_nucleus(); never `Dead`

    coda_kind: Coda,
    coda: CodaChars,

    tone: Tone,
}

impl BuildingSyllable {
    const MAX_LEN: usize = Coda::MAX_LEN + NUCLEUS_MAX_LEN + Onset::MAX_LEN;

    // ─────────────────────────── Accessors ───────────────────────────

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
    pub fn vowels(&self) -> &[ExtendedBaseVowel] {
        &self.nucleus
    }

    #[inline(always)]
    pub const fn tone(&self) -> Tone {
        self.tone
    }

    #[inline(always)]
    pub fn tone_vowel_index(&self, tone_placement: TonePlacement) -> Option<usize> {
        tone_placement.vowel_index(&self.nucleus[..], self.coda.is_empty())
    }

    // ─────────────────────────── Lifecycle ───────────────────────────

    #[inline]
    pub fn reset(&mut self) {
        self.onset_kind = Onset::default();
        self.onset.clear();

        self.nucleus.clear();
        self.nucleus_state = NucleusState::default();

        self.coda_kind = Coda::default();
        self.coda.clear();

        self.tone = Tone::default();
    }

    // ─────────────────────────── Rendering ───────────────────────────

    #[inline(always)]
    pub fn to_chars(&self, tone_placement: TonePlacement) -> InlineVec<char, { Self::MAX_LEN }> {
        let mut output = InlineVec::default();

        // 1. Onset (contiguous chars -> one memcpy)
        output.extend_from_slice(&self.onset);

        // 2. Vowels
        if self.tone.is_some() {
            let tone_pos = self.tone_vowel_index(tone_placement);
            for (idx, vowel) in self.nucleus.iter().enumerate() {
                let active_tone = if Some(idx) == tone_pos {
                    self.tone
                } else {
                    Tone::Flat
                };
                output.push(vowel.to_char_tone(active_tone));
            }
        } else {
            output.extend(self.nucleus.iter().map(|vowel| vowel.to_char()));
        }

        // 3. Coda (contiguous chars -> one memcpy)
        output.extend_from_slice(&self.coda);

        output
    }

    pub fn validate<V>(&self, validator: V) -> Result<(), ValidationError>
    where
        V: PhonotacticValidator,
    {
        let mut buf = [BaseVowel::Y; NUCLEUS_MAX_LEN];
        let count = self.nucleus.bases(&mut buf);
        validator.validate(self.onset_kind, &buf[..count], self.coda_kind, self.tone)
    }

    // ─────────────────────────── Validation ───────────────────────────

    /// Mutates the onset; updates `onset_kind` on success, otherwise undoes
    /// the change with the undo data.
    #[inline(always)]
    pub fn try_update_onset<F, R, T>(&mut self, update: F, rollback: R) -> bool
    where
        F: FnOnce(&mut OnsetChars) -> T,
        R: FnOnce(&mut OnsetChars, T),
    {
        let undo_data = update(&mut self.onset);

        // Validate the onset.
        match Onset::from_chars(&self.onset) {
            Ok(kind) => {
                self.onset_kind = kind;
                true
            }
            Err(_) => {
                // Undo the mutation.
                rollback(&mut self.onset, undo_data);
                false
            }
        }
    }

    /// Mutates the nucleus; caches `nucleus_state` on success, otherwise rolls
    /// the change back.
    #[inline(always)]
    pub fn try_update_nucleus<F, R, T>(&mut self, update: F, rollback: R) -> bool
    where
        F: FnOnce(&mut Nucleus) -> T,
        R: FnOnce(&mut Nucleus, T),
    {
        let undo_data = update(&mut self.nucleus);

        if self.nucleus.len() < 2 {
            self.nucleus_state = NucleusState::Valid;
            return true;
        }

        let state = self.check_nucleus();

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
    pub fn try_update_coda<F, R, T>(&mut self, update: F, rollback: R) -> bool
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
                // Undo the mutation.
                rollback(&mut self.coda, undo_data);
                false
            }
        }
    }

    /// Validates the vowel nucleus against the rule table.
    #[inline(always)]
    fn check_nucleus(&self) -> NucleusState {
        let mut buf = [BaseVowel::A; NUCLEUS_MAX_LEN];
        let count = self.nucleus.bases(&mut buf);
        NucleusState::check(&buf[..count])
    }

    // ─────────────────────────── Normalization ───────────────────────────

    #[inline]
    fn normalize_i_placement(&mut self) {
        let vowels_len = self.nucleus.len();
        // G + I + V -> Gi + V
        if self.onset_kind == Onset::G && vowels_len >= 2 && self.nucleus[0].get() == BaseVowel::I {
            let i = self.nucleus.remove(0);

            self.onset.push(if i.is_upper() { 'I' } else { 'i' });
            self.onset_kind = Onset::Gi;
            return;
        }

        // Gi without a vowel -> G + I
        if self.onset_kind == Onset::Gi && vowels_len == 0 {
            let i = self.onset.pop().expect("onset must contain i");
            self.onset_kind = Onset::G;
            self.nucleus
                .push(ExtendedBaseVowel::with_case(BaseVowel::I, i == 'I'));

            return;
        }

        // A lone `i` left in the onset drops back into the nucleus (I + V).
        if self.onset.len() == 1 && is_i_ignore_case(self.onset[0]) {
            let i = self.onset.pop().unwrap();
            self.onset_kind = Onset::None;
            self.nucleus
                .insert(0, ExtendedBaseVowel::with_case(BaseVowel::I, i == 'I'));
        }
    }

    /// Normalizes an unmarked `u o` prefix that arrived without a shape key.
    ///
    /// `uơ → ươ` and `ưo → ươ` are folded once at least two vowels are present.
    #[inline]
    fn normalize_uo_horn(&mut self) {
        // Needs 2+ vowels with a coda, or 3+ vowels on their own.
        if self.nucleus.len() < 2 || (self.nucleus.len() < 3 && self.coda.is_empty()) {
            return;
        }

        use BaseVowel::*;
        match (self.nucleus[0].get(), self.nucleus[1].get()) {
            (U, OHorn) => {
                self.nucleus[0].set(UHorn);
            }
            (UHorn, O) => {
                self.nucleus[1].set(OHorn);
            }
            _ => {}
        }
    }

    // ─────────────────────────── Transforms ───────────────────────────

    #[inline]
    fn try_transform<KM: Keymap>(
        &mut self,
        keymap: &KM,
        key: char,
        vowel_upper_bound_idx: Option<usize>,
    ) -> TransformResult {
        // 1. Tone.
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
    ) -> TransformResult {
        // Scan vowels up to the cursor.
        let max_len = match vowel_upper_bound_idx {
            Some(idx) => idx.min(self.nucleus.len()),
            None => self.nucleus.len(),
        };

        // No vowel before the cursor -> nothing to transform.
        if max_len == 0 {
            return TransformResult::NotApplicable;
        }

        // Special "uo" case (uow -> ươ, uoo -> uô) when the cursor is after the `u`.
        if self.nucleus_starts_with_uo() {
            let shape = match keymap.decode_shape(key, RootVowel::O) {
                Some(s) => s,
                None => match keymap.decode_shape(key, RootVowel::U) {
                    // Only Horn may target `u`; any other U-shape is not applicable.
                    Some(Shape::Horn) => Shape::Horn,
                    _ => return TransformResult::NotApplicable,
                },
            };
            return self.apply_uo_shape(shape);
        }

        // Scan backwards through the vowels before the cursor.
        for index in (0..max_len).rev() {
            let base = self.nucleus[index].get();

            if let Some(shape) = keymap.decode_shape(key, base.root()) {
                match self.apply_vowel_shape(index, shape) {
                    TransformResult::NotApplicable => continue,
                    effect => return effect,
                }
            }
        }

        TransformResult::NotApplicable
    }

    #[inline]
    fn try_toggle_d_stroke<KM: Keymap>(&mut self, keymap: &KM, key: char) -> TransformResult {
        if keymap.is_stroke_key(key) {
            return self.toggle_d_stroke();
        }
        TransformResult::NotApplicable
    }

    /// Toggles the D-stroke on the onset cluster (`d` ↔ `đ`, `D` ↔ `Đ`).
    ///
    /// Only applies when the onset is a lone `D`/`Đ`; otherwise the stroke key
    /// cannot act here.
    #[inline]
    fn toggle_d_stroke(&mut self) -> TransformResult {
        match self.onset_kind {
            Onset::D => {
                debug_assert!(
                    matches!(self.onset[0], 'd' | 'D'),
                    "Onset state desync: onset_kind is D, but onset[0] is {:?}",
                    self.onset[0]
                );
                self.onset[0] = if self.onset[0] == 'd' { 'đ' } else { 'Đ' };
                self.onset_kind = Onset::DStroke;
                TransformResult::Applied
            }
            Onset::DStroke => {
                debug_assert!(
                    matches!(self.onset[0], 'đ' | 'Đ'),
                    "Onset state desync: onset_kind is D, but onset[0] is {:?}",
                    self.onset[0]
                );

                self.onset[0] = if self.onset[0] == 'đ' { 'd' } else { 'D' };
                self.onset_kind = Onset::D;
                TransformResult::Reverted
            }
            _ => TransformResult::NotApplicable,
        }
    }

    /// Applies or toggles a tone on the syllable.
    ///
    /// Tapping the same tone again reverts to `Flat`; a different tone replaces
    /// the current one.
    #[inline]
    fn apply_tone(&mut self, tone: Tone) -> TransformResult {
        if self.nucleus.is_empty() {
            return TransformResult::NotApplicable;
        }
        if self.tone == tone {
            self.tone = Tone::Flat;
            return TransformResult::Reverted;
        }

        self.tone = tone;
        TransformResult::Applied
    }

    /// Applies `shape` to the vowel at `index`, re-validating the nucleus.
    ///
    /// Applying the shape it already has reverts it; an invalid result rolls
    /// the vowel back.
    fn apply_vowel_shape(&mut self, vowel_index: usize, shape: Shape) -> TransformResult {
        debug_assert!(vowel_index < self.nucleus.len());

        let old = self.nucleus[vowel_index].get();

        // Shape already present -> revert to base.
        if old.has_shape(shape) && shape.is_some() {
            self.nucleus[vowel_index].set(old.remove_shape());
            return TransformResult::Reverted;
        }

        // Try applying the new shape.
        let Ok(new) = old.replace_shape(shape) else {
            return TransformResult::NotApplicable;
        };

        if self.try_update_nucleus(
            |nucleus| {
                nucleus[vowel_index].set(new);
            },
            |nucleus, _| {
                nucleus[vowel_index].set(old);
            },
        ) {
            return TransformResult::Applied;
        }

        TransformResult::NotApplicable
    }

    /// Applies a shape to a `u o`-prefix nucleus (needs at least 2 vowels).
    fn apply_uo_shape(&mut self, shape: Shape) -> TransformResult {
        if self.nucleus.len() < 2 {
            return TransformResult::NotApplicable;
        }

        use BaseVowel::*;
        match shape {
            Shape::Horn => match (self.nucleus[0].get(), self.nucleus[1].get()) {
                // ươ -> uo (revert).
                (UHorn, OHorn) => {
                    self.nucleus[0].set(U);
                    self.nucleus[1].set(O);
                    TransformResult::Reverted
                }

                // uơ -> Horn index 0 (becomes ươ).
                (U, OHorn) => self.apply_vowel_shape(0, Shape::Horn),

                // ưô, ưo, uo, uô -> Horn the vowel at index 1.
                (UHorn | U, O | OCircumflex) => self.apply_vowel_shape(1, Shape::Horn),

                _ => return TransformResult::NotApplicable,
            },

            Shape::Circumflex => match (self.nucleus[0].get(), self.nucleus[1].get()) {
                // uô -> uo (revert).
                (U, OCircumflex) => {
                    self.nucleus[1].set(BaseVowel::O);
                    TransformResult::Reverted
                }

                // uo, uơ -> uô (Circumflex on index 1).
                (U, O | OHorn) => self.apply_vowel_shape(1, Shape::Circumflex),

                // ươ, ưo -> uô: drop the Horn on `ư`, then Circumflex the `o`.
                (UHorn, OHorn | O) => {
                    let oldo = self.nucleus[1].get();

                    if self.try_update_nucleus(
                        |nucleus| {
                            nucleus[0].set(U);
                            nucleus[1].set(OCircumflex);
                        },
                        |nucleus, _| {
                            nucleus[0].set(UHorn);
                            nucleus[1].set(oldo);
                        },
                    ) {
                        return TransformResult::Applied;
                    }

                    return TransformResult::NotApplicable;
                }

                _ => TransformResult::NotApplicable,
            },

            _ => TransformResult::NotApplicable,
        }
    }

    /// Whether the nucleus starts with an unmarked `u o` pair.
    #[inline(always)]
    fn nucleus_starts_with_uo(&self) -> bool {
        self.nucleus.len() > 1
            && self.nucleus[0].get().root() == RootVowel::U
            && self.nucleus[1].get().root() == RootVowel::O
    }
}
