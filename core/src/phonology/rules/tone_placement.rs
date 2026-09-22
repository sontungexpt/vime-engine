use super::super::{vowel_sequence::VowelSequence, BaseVowel};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TonePlacement {
    /// Modern standard orthography ("học sinh" placement).
    #[default]
    Modern,
    /// Pre-1975 "old style" placement.
    Old,
}

impl TonePlacement {
    /// Index of the tone-bearing vowel, or `None` for an empty nucleus.
    #[inline]
    pub fn tone_index<V>(self, vowels: &V, coda_is_empty: bool) -> Option<usize>
    where
        V: VowelSequence + ?Sized,
    {
        match self {
            Self::Modern => tone_index_modern(vowels),
            Self::Old => tone_index_old(vowels, coda_is_empty),
        }
    }
}

/// Tone-bearing vowel index under the modern standard, or `None` for an empty
/// nucleus.
#[inline]
pub fn tone_index_modern<V>(vowels: &V) -> Option<usize>
where
    V: VowelSequence + ?Sized,
{
    let len = vowels.len();

    match len {
        0 => None,
        1 => Some(0),
        2 => {
            let v0 = vowels.at(0);
            let v1 = vowels.at(1);

            // Marked vowel always takes the tone (thuế -> ê, cuối -> ô).
            if v1.is_shaped() {
                return Some(1);
            } else if v0.is_shaped() {
                return Some(0);
            }

            // oa / oe / uy -> tone on the second (hoá, khoèo, huý).
            match (v0, v1) {
                (BaseVowel::O, BaseVowel::A | BaseVowel::E) | (BaseVowel::U, BaseVowel::Y) => {
                    Some(1)
                }

                // Everything else -> tone on the first (ia -> i, ai -> a).
                _ => Some(0),
            }
        }
        3 => Some(tone_index_3(vowels)),
        _ => fallback_tone_index(vowels),
    }
}

/// Tone-bearing vowel index under the pre-1975 style, or `None` for an empty
/// nucleus.
///
/// Marked vowel always wins; otherwise open syllable -> first, closed ->
/// second (tòa -> o, toán -> a).
#[inline]
pub fn tone_index_old<V>(vowels: &V, coda_is_empty: bool) -> Option<usize>
where
    V: VowelSequence + ?Sized,
{
    let len = vowels.len();

    match len {
        0 => None,
        1 => Some(0),
        2 => {
            let v0 = vowels.at(0);
            let v1 = vowels.at(1);

            // Marked vowel always takes the tone (thuế -> ê, cuối -> ô).
            if v1.is_shaped() {
                return Some(1);
            } else if v0.is_shaped() {
                return Some(0);
            }
            // Open syllable -> first, closed syllable -> second.
            else if coda_is_empty {
                return Some(0);
            }

            Some(1)
        }
        3 => Some(tone_index_3(vowels)),
        _ => fallback_tone_index(vowels),
    }
}

/// Tone-bearing vowel of a 3-vowel nucleus: rightmost marked vowel wins
/// (uôi -> ô), else the middle (oai -> a).
#[inline(always)]
fn tone_index_3<V>(vowels: &V) -> usize
where
    V: VowelSequence + ?Sized,
{
    if vowels.at(2).is_shaped() {
        2
    } else if vowels.at(1).is_shaped() {
        1
    } else if vowels.at(0).is_shaped() {
        0
    } else {
        1
    }
}

/// Fallback for >3 vowels: the one with the highest [`BaseVowel::id`]. Never
/// reached by the composition model, which caps nuclei at three.
#[inline]
fn fallback_tone_index<V>(vowels: &V) -> Option<usize>
where
    V: VowelSequence + ?Sized,
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
