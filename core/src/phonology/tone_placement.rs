use super::{BaseVowel, BaseVowelSlice};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum TonePlacement {
    /// Modern standard orthography ("học sinh", "hóa", "thúy").
    #[default]
    Modern = 0,
    /// Pre-1975 classic orthography ("hoá", "thúy").
    Old = 1,
}

impl TonePlacement {
    /// Returns the index within `vowels` of the vowel that takes the tone mark,
    /// or `None` when the nucleus is empty. The index is local to `vowels`
    /// (`0..vowels.len()`), not an absolute position in the syllable.
    ///
    /// The unchecked reads below stay in bounds because each helper is reached
    /// only at its own length (`tone_index_2_*` on `len == 2`, `tone_index_3` on
    /// `len == 3`, the fallback over `1..len`), so every index is below `len` —
    /// [`BaseVowelSlice::at_unchecked`]'s contract. `len` is not re-read in
    /// between and the helpers take `&V`, so nothing can change it, and they are
    /// `#[inline(always)]` so the length check stays adjacent to the reads.
    #[inline]
    pub fn vowel_index<V>(self, vowels: &V, coda_is_empty: bool) -> Option<usize>
    where
        V: BaseVowelSlice + ?Sized,
    {
        match vowels.len() {
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

    /// Tone placement for a 2-vowel nucleus under the modern standard.
    #[inline(always)]
    fn tone_index_2_modern<V>(vowels: &V) -> usize
    where
        V: BaseVowelSlice + ?Sized,
    {
        let v1 = unsafe { vowels.at_unchecked(1) };

        // Rule 1: Diacritic/shaped vowel always takes the tone (e.g., "thuế" -> ê, "cuối" -> ô).
        if v1.is_shaped() {
            return 1;
        }

        let v0 = unsafe { vowels.at_unchecked(0) };

        if v0.is_shaped() {
            return 0;
        }

        // Rule 2: Open diphthongs "oa", "oe", "uy" place tone on the second vowel ("hoá", "hoé", "thuý").
        match (v0, v1) {
            (BaseVowel::O, BaseVowel::A | BaseVowel::E) | (BaseVowel::U, BaseVowel::Y) => 1,
            // Default: First vowel takes tone ("mía", "ai", "ao").
            _ => 0,
        }
    }

    /// Tone placement for a 2-vowel nucleus under the classic standard.
    #[inline(always)]
    fn tone_index_2_old<V>(vowels: &V, coda_is_empty: bool) -> usize
    where
        V: BaseVowelSlice + ?Sized,
    {
        // Rule 1: Diacritic/shaped vowel always takes the tone ("thuế" -> ê, "cuối" -> ô).
        let v1 = unsafe { vowels.at_unchecked(1) };
        if v1.is_shaped() {
            return 1;
        }

        let v0 = unsafe { vowels.at_unchecked(0) };
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

    /// Tone placement for a 3-vowel nucleus (e.g. "oai", "uôi", "uyu").
    #[inline(always)]
    fn tone_index_3<V>(vowels: &V) -> usize
    where
        V: BaseVowelSlice + ?Sized,
    {
        // Rightmost shaped vowel wins (e.g., "uôi" -> index 1 'ô')
        if unsafe { vowels.at_unchecked(2).is_shaped() } {
            2
        } else if unsafe { vowels.at_unchecked(1).is_shaped() } {
            1
        } else if unsafe { vowels.at_unchecked(0).is_shaped() } {
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
        V: BaseVowelSlice + ?Sized,
    {
        let mut best = unsafe { vowels.at_unchecked(0) };
        let mut at = 0;

        for index in 1..vowels.len() {
            let v = unsafe { vowels.at_unchecked(index) };

            if v > best {
                best = v;
                at = index;
            }
        }

        Some(at)
    }
}
