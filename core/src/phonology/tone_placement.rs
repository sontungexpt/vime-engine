use super::{BaseVowel, ExtendedBaseVowel};

/// A read-only view over a vowel nucleus.
pub trait NucleusView {
    fn len(&self) -> usize;
    /// Returns the vowel at `index`.
    ///
    /// # Safety
    ///
    /// `index` must be `< len()`. All call sites are private helpers of
    /// [`TonePlacement::vowel_index`], which dispatches only after matching on
    /// the length (`2`, `3`, or a ≥4 fallback), so they always pass an
    /// in-bounds index. `get_unchecked` also keeps a `debug_assert!`, catching
    /// any future misuse in debug/test builds.
    fn at(&self, index: usize) -> BaseVowel;
}

impl NucleusView for [BaseVowel] {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn at(&self, index: usize) -> BaseVowel {
        // SAFETY: call sites are the `vowel_index` length-matched helpers
        // (2 / 3 / ≥4), so `index` is always `< self.len()`. `BaseVowel` is
        // `Copy`, so a by-value read is sound.
        unsafe { *self.get_unchecked(index) }
    }
}

impl NucleusView for [ExtendedBaseVowel] {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn at(&self, index: usize) -> BaseVowel {
        // SAFETY: callers only reach this via the `len`-matched helpers, so
        // `index < self.len()`. `get()` then masks the case bit back to a valid
        // `BaseVowel` discriminant.
        unsafe { self.get_unchecked(index).get() }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TonePlacement {
    /// Modern standard orthography ("học sinh", "hóa", "thúy").
    #[default]
    Modern,
    /// Pre-1975 classic orthography ("hoá", "thúy").
    Old,
}

impl TonePlacement {
    /// Determines the 0-based relative index of the vowel within the provided
    /// `vowels` array/slice that should receive the tone mark.
    ///
    /// The returned index is strictly local to `vowels` (i.e. `0..vowels.len()`),
    /// pointing directly to the target vowel element rather than an absolute character
    /// position in the full syllable or a global vowel identifier.
    ///
    /// Returns `None` if the nucleus is empty (`vowels.len() == 0`).
    #[inline]
    pub fn vowel_index<V>(self, vowels: &V, coda_is_empty: bool) -> Option<usize>
    where
        V: NucleusView + ?Sized,
    {
        let len = vowels.len();
        match len {
            0 => None,
            1 => Some(0),
            2 => Some(match self {
                Self::Modern => Self::tone_index_2_modern(vowels),
                Self::Old => Self::tone_index_2_old(vowels, coda_is_empty),
            }),
            3 => Some(Self::tone_index_3(vowels)),
            _ => Self::tone_index_fallback(vowels),
        }
    }

    /// Tone placement for 2-vowel nucleus under Modern standard.
    #[inline(always)]
    fn tone_index_2_modern<V>(vowels: &V) -> usize
    where
        V: NucleusView + ?Sized,
    {
        let v0 = vowels.at(0);
        let v1 = vowels.at(1);

        // Rule 1: Diacritic/shaped vowel always takes the tone (e.g., "thuế" -> ê, "cuối" -> ô).
        if v1.is_shaped() {
            return 1;
        }
        if v0.is_shaped() {
            return 0;
        }

        // Rule 2: Open diphthongs "oa", "oe", "uy" place tone on the second vowel ("hóa", "hoe", "thúy").
        match (v0, v1) {
            (BaseVowel::O, BaseVowel::A | BaseVowel::E) | (BaseVowel::U, BaseVowel::Y) => 1,
            // Default: First vowel takes tone ("mía", "ai", "ao").
            _ => 0,
        }
    }

    /// Tone placement for 2-vowel nucleus under Old/Classic standard.
    #[inline(always)]
    fn tone_index_2_old<V>(vowels: &V, coda_is_empty: bool) -> usize
    where
        V: NucleusView + ?Sized,
    {
        let v0 = vowels.at(0);
        let v1 = vowels.at(1);

        // Rule 1: Diacritic/shaped vowel always takes the tone ("thuế" -> ê, "cuối" -> ô).
        if v1.is_shaped() {
            return 1;
        }
        if v0.is_shaped() {
            return 0;
        }

        // Rule 2: Open syllable -> first vowel ("hoá", "thúy"). Closed syllable -> second vowel ("hoán", "thuýth").
        if coda_is_empty {
            0
        } else {
            1
        }
    }

    /// Tone placement for 3-vowel nucleus (e.g., "oai", "uôi", "uyu").
    #[inline(always)]
    fn tone_index_3<V>(vowels: &V) -> usize
    where
        V: NucleusView + ?Sized,
    {
        // Rightmost shaped vowel wins (e.g., "uôi" -> index 1 'ô')
        if vowels.at(2).is_shaped() {
            2
        } else if vowels.at(1).is_shaped() {
            1
        } else if vowels.at(0).is_shaped() {
            0
        } else {
            // Unshaped triphthong (e.g., "oai", "uye") -> center vowel
            1
        }
    }

    /// Fallback for >3 vowels: the one with the highest [`BaseVowel::id`]. Never
    /// reached by the composition model, which caps nuclei at three.
    #[inline]
    fn tone_index_fallback<V>(vowels: &V) -> Option<usize>
    where
        V: NucleusView + ?Sized,
    {
        let mut best = vowels.at(0);
        let mut at = 0;

        for index in 1..vowels.len() {
            let v = vowels.at(index);

            if v > best {
                best = v;
                at = index;
            }
        }

        Some(at)
    }
}
