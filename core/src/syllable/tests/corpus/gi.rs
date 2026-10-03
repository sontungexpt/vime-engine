//! The `gi` family: `gi` is the onset only when another vowel follows the `i`.
//!
//! ```text
//! g → onset G    gi → onset G + nucleus I    gia → onset Gi + nucleus A
//! ```
//!
//! Section groups:
//!   1. Base cases: g, gi, gia
//!   2. Gi + various vowels
//!   3. Tone on Gi onset
//!   4. Gi with coda

use super::prelude::*;

pub const CASES: &[Case] = &[
    // ═══════════════════════════════════════════
    // 1. Base cases: g, gi, gia
    // ═══════════════════════════════════════════
    case!(['g'], ExpectedSyllable::consonant(Onset::G, &['g'])),
    case!(['g','i'], ExpectedSyllable::onset_vowel(Onset::G, &['g'], &[(V::I,C::Lower)], Tone::Flat)),
    case!(['g','i','a'], ExpectedSyllable::onset_vowel(Onset::Gi, &['g','i'], &[(V::A,C::Lower)], Tone::Flat)),

    // ═══════════════════════════════════════════
    // 2. Gi + various vowels
    // ═══════════════════════════════════════════
    case!(['g','i','e'], ExpectedSyllable::onset_vowel(Onset::Gi, &['g','i'], &[(V::E,C::Lower)], Tone::Flat)),
    case!(['g','i','o'], ExpectedSyllable::onset_vowel(Onset::Gi, &['g','i'], &[(V::O,C::Lower)], Tone::Flat)),

    // ═══════════════════════════════════════════
    // 3. Gi + triphthongs
    // ════════════════════════════════════════════
    case!(['g','i','a','o'], ExpectedSyllable::onset_vowel(Onset::Gi, &['g','i'], &[(V::A,C::Lower),(V::O,C::Lower)], Tone::Flat)),

    // ═══════════════════════════════════════════
    // 4. Tone on Gi onset
    // ═══════════════════════════════════════════
    case!(['g','i','s'], ExpectedSyllable::onset_vowel(Onset::G, &['g'], &[(V::I,C::Lower)], Tone::Acute)),
    case!(['g','i','a','s'], ExpectedSyllable::onset_vowel(Onset::Gi, &['g','i'], &[(V::A,C::Lower)], Tone::Acute)),
];