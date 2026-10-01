//! The `gi` family: `gi` is the onset only when another vowel follows the `i`.
//!
//! ```text
//! g → onset G    gi → onset G + nucleus I    gia → onset Gi + nucleus A
//! ```

use super::prelude::*;

pub const CASES: &[Case] = &[
    case!(['g'], ExpectedSyllable::consonant(Onset::G, &['g'])),
    case!(
        ['g', 'i'],
        ExpectedSyllable::onset_vowel(Onset::G, &['g'], &[(V::I, C::Lower)], Tone::Flat)
    ),
    case!(
        ['g', 'i', 'a'],
        ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::A, C::Lower)], Tone::Flat)
    ),
    case!(
        ['g', 'i', 'e'],
        ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::E, C::Lower)], Tone::Flat)
    ),
    case!(
        ['g', 'i', 'o'],
        ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::O, C::Lower)], Tone::Flat)
    ),
    case!(
        ['g', 'i', 'a', 'o'],
        ExpectedSyllable::onset_vowel(
            Onset::Gi,
            &['g', 'i'],
            &[(V::A, C::Lower), (V::O, C::Lower)],
            Tone::Flat
        )
    ),
    case!(
        ['g', 'i', 's'],
        ExpectedSyllable::onset_vowel(Onset::G, &['g'], &[(V::I, C::Lower)], Tone::Acute)
    ),
    case!(
        ['g', 'i', 'a', 's'],
        ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::A, C::Lower)], Tone::Acute)
    ),
];
