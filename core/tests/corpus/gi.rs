//! The `gi` family.
//!
//! A syllable prefix of `gi` is ambiguous: if another vowel follows the `i`,
//! `gi` becomes the onset (`Onset::Gi`), otherwise `g` stays the onset and the
//! `i` is the nucleus. Each entry below asserts the exact final semantic state
//! seen through the `Composition` syllable accessors:
//!
//! ```text
//! g    → onset G
//! gi   → onset G + nucleus I
//! gia  → onset Gi + nucleus A
//! ```

use super::{case, ExpectedSyllable, TestCase};
use super::{C, V};
use vime_engine::composition::ParseAppendingPhase;
use vime_engine::phonology::{Onset, Tone};

pub const CASES: &[TestCase] = &[
    case!(['g'], ParseAppendingPhase::Onset, ExpectedSyllable::consonant(Onset::G, &['g'])),
    case!(['g', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::G, &['g'], &[(V::I, C::Lower)], Tone::Flat)),
    case!(['g', 'i', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::A, C::Lower)], Tone::Flat)),
    case!(['g', 'i', 'e'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::E, C::Lower)], Tone::Flat)),
    case!(['g', 'i', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::O, C::Lower)], Tone::Flat)),
    case!(
        ['g', 'i', 'a', 'o'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::A, C::Lower), (V::O, C::Lower)], Tone::Flat)
    ),
    case!(['g', 'i', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::G, &['g'], &[(V::I, C::Lower)], Tone::Acute)),
    case!(['g', 'i', 'a', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::A, C::Lower)], Tone::Acute)),
];
