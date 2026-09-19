//! E. The special `uo` / `ươ` cycles.

use super::{case, ExpectedSyllable, TestCase};
use super::{C, V};
use vime_engine::composition::ParseAppendingPhase;
use vime_engine::phonology::{Coda, Onset, Tone};

pub const CASES: &[TestCase] = &[
    case!(['u', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::O, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'w', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Flat)),
    case!(['ư', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::O, C::Lower)], Tone::Flat)),
    case!(['ư', 'ơ'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'ơ'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'o', 'i'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(
        ['u', 'o', 'o', 'i', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Acute)
    ),
    case!(
        ['u', 'o', 'o', 'i', 'r'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Hook)
    ),
    case!(
        ['u', 'o', 'o', 'i', 'x'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Tilde)
    ),
    case!(
        ['u', 'o', 'o', 'i', 'j'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Dot)
    ),
    case!(['u', 'o', 'w', 'i'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(
        ['u', 'o', 'w', 'i', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Acute)
    ),
    case!(
        ['u', 'o', 'w', 'i', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['u', 'o', 'w', 'i', 'r'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Hook)
    ),
    case!(
        ['u', 'o', 'w', 'i', 'x'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Tilde)
    ),
    case!(['u', 'o', 'w', 'u'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(
        ['u', 'o', 'w', 'u', 'j'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::U, C::Lower)], Tone::Dot)
    ),
    case!(
        ['u', 'o', 'w', 'w', 'i'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)
    ),
    case!(
        ['u', 'o', 'o', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['u', 'o', 'o', 'n', 'g', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['m', 'u', 'o', 'o', 'n', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::M, &['m'], &[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    case!(
        ['u', 'o', 'w', 'o', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(['u', 'o', 'w', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Acute)),
    case!(['u', 'o', 'w', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Grave)),
    case!(
        ['t', 'h', 'u', 'o', 'w', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Th, &['t', 'h'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['u', 'o', 'w', 'c', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(
        ['t', 'u', 'o', 'w', 'c', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::T, &['t'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(
        ['t', 'h', 'i', 'e', 'e', 'c', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Th, &['t', 'h'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(['u', 'o', 'w', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Flat)),
    case!(
        ['u', 'o', 'w', 's', 'i'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Acute)
    ),
    case!(
        ['u', 'o', 'o', 's', 'i'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Acute)
    ),
    case!(
        ['u', 'o', 'o', 'n', 'g', 'x'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Tilde, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['u', 'o', 'o', 'n', 'g', 'j'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Dot, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['u', 'o', 'w', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['u', 'o', 'w', 't', 'j'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Dot, Coda::T, &['t'])
    ),
    case!(
        ['u', 'o', 'w', 'c', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Grave, Coda::C, &['c'])
    ),
];
