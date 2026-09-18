//! Vietnamese orthography rules.
//!
//! The tone-bearing vowel of a syllable is chosen by an orthography — the
//! modern standard ("học sinh" placement) or the pre-1975 "old style". The
//! placement logic is independent of any renderer or syllable representation,
//! so callers can ask "where would the tone mark go" without building output
//! text.
//!
//! [`analyze_modern`] and [`analyze_old`] answer that question for a
//! [`VowelSequence`].

use crate::phonology::{BaseVowel, RootVowel, Shape};

/// A read-only view over a vowel nucleus, independent of the owning structure.
pub trait VowelSequence {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<BaseVowel>;

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl VowelSequence for [BaseVowel] {
    #[inline(always)]
    fn len(&self) -> usize {
        <[BaseVowel]>::len(self)
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<BaseVowel> {
        <[BaseVowel]>::get(self, index).copied()
    }
}

impl VowelSequence for Vec<BaseVowel> {
    #[inline(always)]
    fn len(&self) -> usize {
        Vec::len(self)
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<BaseVowel> {
        <[BaseVowel]>::get(self.as_slice(), index).copied()
    }
}

impl<const N: usize> VowelSequence for [BaseVowel; N] {
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<BaseVowel> {
        <[BaseVowel]>::get(self.as_slice(), index).copied()
    }
}

/// Returns the index of the tone-bearing vowel, or `None` when the nucleus is
/// empty.
///
/// Modern orthography: the newer placement rules.
#[inline]
pub fn analyze_modern<V>(vowels: &V) -> Option<usize>
where
    V: VowelSequence + ?Sized,
{
    let len = vowels.len();

    if len == 0 {
        return None;
    } else if len == 1 {
        return Some(0);
    }

    // The structurally-marked vowel always takes the tone.
    //
    // Examples:
    //   uô  -> ô
    //   ươ  -> ơ
    //   iê  -> ê
    //   ươi -> ơ
    //   uôi -> ô
    //   uyê -> ê
    if let Some(index) = marked_index(vowels) {
        return Some(index);
    }

    match len {
        // Two vowels without a structural mark.
        2 => {
            // oa / oe -> second letter: the first vowel is the /w/ medial,
            // the second is the main vowel (hoá, khoèo).
            if vowels.get(0).zip(vowels.get(1)).is_some_and(|(a, b)| {
                matches!(
                    (a.root(), b.root()),
                    (RootVowel::O, RootVowel::A) | (RootVowel::O, RootVowel::E)
                )
            }) {
                return Some(1);
            }

            // Everything else -> first letter: the main vowel is written with
            // two letters (ia / ua / ưa -> i / u / ư) or is a plain sequence
            // ending in a glide (ai, ao, au, ay, eo, oi, ui, uy, ...).
            Some(0)
        }

        // Three vowels -> middle vowel (oai -> a).
        3 => Some(1),

        // Defensive fallback. A nucleus never holds more than three vowels, so
        // this is reached only by non-compliant `VowelSequence` impls.
        _ => highest_priority_index(vowels),
    }
}

/// Returns the index of the tone-bearing vowel, or `None` when the nucleus is
/// empty.
///
/// Pre-1975 "old style": the tone lands on the second vowel when the syllable
/// is closed (`coda_is_empty == false`), on the first when it is open.
#[inline]
pub fn analyze_old<V>(vowels: &V, coda_is_empty: bool) -> Option<usize>
where
    V: VowelSequence + ?Sized,
{
    let len = vowels.len();

    if len == 0 {
        return None;
    } else if len == 1 {
        return Some(0);
    }

    // The structurally-marked vowel always takes the tone.
    //
    // Examples:
    //   thuế  -> ê
    //   thuở  -> ơ
    //   nước  -> ơ
    //   cuối  -> ô
    if let Some(index) = marked_index(vowels) {
        return Some(index);
    }

    match len {
        // Two vowels:
        //
        // open syllable  -> first
        // closed syllable -> second
        //
        // tòa  -> o
        // toán -> a
        2 if coda_is_empty => Some(0),
        2 => Some(1),

        // Three vowels -> middle vowel.
        //
        // khuỷu -> y
        // oai   -> a
        3 => Some(1),

        // Defensive fallback. A nucleus never holds more than three vowels.
        _ => highest_priority_index(vowels),
    }
}

/// Returns the index of the structurally-marked vowel (ê, ô, ơ, â, ă, ư),
/// preferring the last one; `None` when the nucleus carries no such mark.
#[inline(always)]
fn marked_index<V>(vowels: &V) -> Option<usize>
where
    V: VowelSequence + ?Sized,
{
    let mut index = vowels.len();

    while index > 0 {
        index -= 1;

        if vowels.get(index).is_some_and(|v| v.is_shaped()) {
            return Some(index);
        }
    }

    None
}

/// Defensive fallback: the vowel with the highest placement priority, i.e. the
/// highest [`BaseVowel::id`]. Only reachable with >3 vowels — a nucleus never
/// holds more than three. [`BaseVowel`] ids are unique, so the result never
/// depends on scan order.
#[inline]
fn highest_priority_index<V>(vowels: &V) -> Option<usize>
where
    V: VowelSequence + ?Sized,
{
    let mut best = vowels.get(0)?;
    let mut at = 0;

    for index in 1..vowels.len() {
        let Some(v) = vowels.get(index) else {
            continue;
        };

        if v > best {
            best = v;
            at = index;
        }
    }

    Some(at)
}
