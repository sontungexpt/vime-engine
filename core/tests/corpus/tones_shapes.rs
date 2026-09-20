//! D. Tone + shape combinations.

use super::{case, ExpectedSyllable, TestCase};
use super::{C, V};
use vime_engine::composition::ParseAppendingPhase;
use vime_engine::phonology::{Coda, Onset, Tone};

pub const CASES: &[TestCase] = &[
    case!(['a', 'w', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Acute)),
    case!(['a', 'w', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Grave)),
    case!(['a', 'w', 'r'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Hook)),
    case!(['a', 'w', 'x'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Tilde)),
    case!(['a', 'w', 'j'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Dot)),
    case!(['a', 'a', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Acute)),
    case!(['a', 'a', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Grave)),
    case!(['a', 'a', 'r'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Hook)),
    case!(['a', 'a', 'x'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Tilde)),
    case!(['a', 'a', 'j'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Dot)),
    case!(['e', 'e', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower)], Tone::Acute)),
    case!(['e', 'e', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower)], Tone::Grave)),
    case!(['e', 'e', 'r'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower)], Tone::Hook)),
    case!(['e', 'e', 'x'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower)], Tone::Tilde)),
    case!(['e', 'e', 'j'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower)], Tone::Dot)),
    case!(['o', 'o', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Acute)),
    case!(['o', 'o', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Grave)),
    case!(['o', 'o', 'r'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Hook)),
    case!(['o', 'o', 'x'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Tilde)),
    case!(['o', 'o', 'j'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Dot)),
    case!(['o', 'w', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Acute)),
    case!(['o', 'w', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Grave)),
    case!(['o', 'w', 'r'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Hook)),
    case!(['o', 'w', 'x'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Tilde)),
    case!(['o', 'w', 'j'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Dot)),
    case!(['u', 'w', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Acute)),
    case!(['u', 'w', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Grave)),
    case!(['u', 'w', 'r'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Hook)),
    case!(['u', 'w', 'x'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Tilde)),
    case!(['u', 'w', 'j'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Dot)),
    case!(['a', 's', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Acute)),
    case!(['a', 'f', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Grave)),
    case!(['a', 's', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Acute)),
    case!(['e', 's', 'e'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower)], Tone::Acute)),
    case!(['o', 's', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Acute)),
    case!(['o', 's', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Acute)),
    case!(['u', 's', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Acute)),
    case!(['a', 'w', 's', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Grave)),
    case!(['a', 'a', 's', 'r'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Hook)),
    case!(
        ['c', 'h', 'a', 'w', 'n', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ch, &['c', 'h'], &[(V::ABreve, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    case!(
        ['m', 'a', 'a', 'n', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::M, &['m'], &[(V::ACircumflex, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    case!(['k', 'h', 'a', 'a', 's'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::Kh, &['k', 'h'], &[(V::ACircumflex, C::Lower)], Tone::Acute)),
    case!(['a', 'a', 'n', 's'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::ACircumflex, C::Lower)], Tone::Acute, Coda::N, &['n'])),
    case!(['e', 'e', 'n', 'j'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::ECircumflex, C::Lower)], Tone::Dot, Coda::N, &['n'])),
    case!(['o', 'w', 'n', 'f'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::OHorn, C::Lower)], Tone::Grave, Coda::N, &['n'])),
    case!(['ấ', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Acute)),
    case!(['e', 'e', 'u', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Grave)),
    case!(['o', 'o', 'i', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Grave)),
    case!(['u', 'w', 'a', 'r'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::A, C::Lower)], Tone::Hook)),
    case!(
        ['b', 'a', 'w', 'n', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::B, &['b'], &[(V::ABreve, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    case!(
        ['b', 'a', 'a', 'n', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::B, &['b'], &[(V::ACircumflex, C::Lower)], Tone::Grave, Coda::N, &['n'])
    ),
    case!(
        ['o', 'a', 'w', 't', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::O, C::Lower), (V::ABreve, C::Lower)], Tone::Acute, Coda::T, &['t'])
    ),
    case!(
        ['l', 'o', 'a', 'w', 't', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::L, &['l'], &[(V::O, C::Lower), (V::ABreve, C::Lower)], Tone::Acute, Coda::T, &['t'])
    ),
    case!(
        ['t', 'h', 'o', 'o', 'i', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Acute)
    ),
    case!(
        ['d', 'd', 'a', 'a', 'u', 'r'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::ACircumflex, C::Lower), (V::U, C::Lower)], Tone::Hook)
    ),
    case!(
        ['c', 'u', 'o', 'o', 'n', 'j'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::C, &['c'], &[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Dot, Coda::N, &['n'])
    ),
    case!(
        ['b', 'a', 'a', 'y', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Acute)
    ),
];
