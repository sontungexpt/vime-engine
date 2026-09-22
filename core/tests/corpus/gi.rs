//! The `gi` family.
//!
//! A syllable prefix of `gi` is ambiguous: if another vowel follows the `i`,
//! `gi` becomes the onset (`Onset::Gi`), otherwise `g` stays the onset and the
//! `i` is the nucleus. Each entry below asserts the exact final semantic state
//! the builder must report after all pushes:
//!
//! ```text
//! g    → onset G
//! gi   → onset G + nucleus I
//! gia  → onset Gi + nucleus A
//! ```

use super::prelude::*;

pub const CASES: &[Case] = &[
    case!(['g'], ExpectedSyllable::consonant(Onset::G, &['g'])),
    case!(['g', 'i'], ExpectedSyllable::onset_vowel(Onset::G, &['g'], &[(V::I, C::Lower)], Tone::Flat)),
    case!(['g', 'i', 'a'], ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::A, C::Lower)], Tone::Flat)),
    case!(['g', 'i', 'e'], ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::E, C::Lower)], Tone::Flat)),
    case!(['g', 'i', 'o'], ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::O, C::Lower)], Tone::Flat)),
    case!(['g', 'i', 'a', 'o'], ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::A, C::Lower), (V::O, C::Lower)], Tone::Flat)),
    case!(['g', 'i', 's'], ExpectedSyllable::onset_vowel(Onset::G, &['g'], &[(V::I, C::Lower)], Tone::Acute)),
    case!(['g', 'i', 'a', 's'], ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::A, C::Lower)], Tone::Acute)),
];
