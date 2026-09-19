//! I. Toggle / revert behaviour.
//!
//! Transform keys produce one of three effects:
//!   Applied      → the transform took effect (tone/shape set)
//!   Reverted     → an equal tone/shape was toggled off; the key then falls
//!                  through as a literal and lands in the coda
//!   NotApplicable→ nothing to transform; the key falls through as literal
//! The expected strings below reflect that *exact* semantics.

use super::{case, ExpectedSyllable, TestCase};
use super::{C, V};
use vime_engine::composition::ParseAppendingPhase;
use vime_engine::phonology::{Onset, Tone};

pub const CASES: &[TestCase] = &[
    case!(['a', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute)),
    case!(['á', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Grave)),
    case!(['à', 'r'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Hook)),
    case!(['ả', 'x'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Tilde)),
    case!(['ã', 'j'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Dot)),
    case!(['ả', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Grave)),
    case!(['á', 'z'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat)),
    case!(['ạ', 'z'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat)),
    case!(['ô', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::O, C::Lower)], Tone::Flat)),
    case!(['a', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['â', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Flat)),
    case!(['ă', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['a', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Flat)),
    case!(['o', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Flat)),
    case!(['a', 's', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Acute)),
    case!(['ă', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Acute)),
    case!(['a', 'w', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Acute)),
    case!(['d', 'd'], ParseAppendingPhase::Onset, ExpectedSyllable::consonant(Onset::Đ, &['đ'])),
    case!(['D', 'D'], ParseAppendingPhase::Onset, ExpectedSyllable::consonant(Onset::Đ, &['Đ'])),
    case!(['d', 'd', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Đ, &['đ'], &[(V::A, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'w', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['s'], ParseAppendingPhase::Onset, ExpectedSyllable::consonant(Onset::S, &['s'])),
    case!(['o', 's', 'r'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Hook)),
];
