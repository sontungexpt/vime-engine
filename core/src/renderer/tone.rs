use crate::{phonology::{RootVowel, Shape}, Syllable};

/// Tone-placement orthography: the modern standard or the pre-1975 "old style".
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Orthography {
    /// Modern standard orthography ("học sinh" placement).
    #[default]
    Modern,
    /// Older orthography ("học sinh" placement).
    Old,
}

/// Analyzes a syllable and returns the index of its tone-bearing vowel,
/// or `None` when the nucleus is empty.
///
/// This is the single entry point for tone placement. It is kept independent
/// of any renderer so callers can ask "where would the tone mark go" without
/// building output text.
pub fn analyze(syllable: &Syllable, orthography: Orthography) -> Option<usize> {
    match orthography {
        Orthography::Modern => tone_position_modern(syllable),
        Orthography::Old => tone_position_old(syllable),
    }
}

/// Modern orthography: tone position with the newer placement rules.
#[inline]
fn tone_position_modern(syllable: &Syllable) -> Option<usize> {
    let vowels = &syllable.vowels;
    let len = vowels.len();

    if len == 0 {
        return None;
    } else if len == 1 {
        return Some(0);
    }

    let mut index = len;

    // A vowel carrying a structural mark is the tone-bearing vowel.
    //
    // Examples:
    //   uô  -> ô
    //   ươ  -> ơ
    //   iê  -> ê
    //   ươi -> ơ
    //   uôi -> ô
    //
    // The structural vowel takes precedence over positional rules.
    while index > 0 {
        index -= 1;

        if vowels[index].value.shape() != Shape::None {
            return Some(index);
        }
    }

    if len == 2 {
        // oa / oe -> second letter.
        //
        // In these sequences the first vowel represents the
        // medial /w/, while the second vowel is the main vowel.
        if matches!(
            (vowels[0].value.root(), vowels[1].value.root()),
            (RootVowel::O, RootVowel::A) | (RootVowel::O, RootVowel::E)
        ) {
            return Some(1);
        }

        // uy -> first letter, the /w/ medial keeps the tone
        // (thủy, hủy, khuỷ).

        // Main vowel written with two letters:
        //
        // ia / ua / ưa -> first letter

        // Other two-vowel sequences without a structural mark
        // have the first vowel as the tone-bearing vowel:
        //
        // ai, ao, au, ay, eo, oi, ui, ...
        return Some(0);
    }

    if len == 3 {
        // Normal three-vowel nuclei:
        //
        // oai -> a
        // uôi -> ô
        // ươi -> ơ
        // iêu -> ê
        // uyê -> ê
        //
        // Structural vowels were already handled above.
        return Some(1);
    }

    // Defensive fallback.
    //
    // Syllable currently permits at most three vowels, so this
    // should not normally be reached. BaseVowel priority is only
    // a fallback and is NOT the primary orthographic rule.
    let mut position = len - 1;

    for index in (0..len).rev() {
        if vowels[index].value.id() < vowels[position].value.id() {
            position = index;
        }
    }

    Some(position)
}

/// Old orthography: tone position with the older placement rules.
#[inline]
fn tone_position_old(syllable: &Syllable) -> Option<usize> {
    let vowels = &syllable.vowels;
    let len = vowels.len();

    if len == 0 {
        return None;
    } else if len == 1 {
        return Some(0);
    }

    // Structural vowels always take the tone mark.
    //
    // Examples:
    //   thuế  -> ê
    //   thuở  -> ơ
    //   nước  -> ơ
    //   cuối  -> ô
    //
    // This rule overrides the positional rules below.
    let mut index = len;

    while index > 0 {
        index -= 1;

        if vowels[index].value.shape() != Shape::None {
            return Some(index);
        }
    }

    match len {
        // Two vowels:
        //
        // open syllable  -> first
        // closed syllable -> second
        //
        // tòa  -> o
        // toán -> a
        2 if syllable.coda_chars.is_empty() => Some(0),
        2 => Some(1),

        // Three vowels -> middle vowel.
        //
        // khuỷu -> y
        // oai   -> a
        // ...
        3 => Some(1),

        // Defensive fallback. Syllable currently supports
        // at most three vowels.
        _ => {
            let mut position = 0;

            for index in 1..len {
                if vowels[index].value.id() < vowels[position].value.id() {
                    position = index;
                }
            }

            Some(position)
        }
    }
}