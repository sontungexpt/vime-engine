//! C. Telex shape keys.

use super::{case, ExpectedSyllable, TestCase};
use super::{C, V};
use vime_engine::composition::ParseAppendingPhase;
use vime_engine::phonology::{Coda, Onset, Tone};

pub const CASES: &[TestCase] = &[
    case!(['a', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Flat)),
    case!(['a', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['e', 'e'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['o', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Flat)),
    case!(['o', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Flat)),
    case!(['â', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Flat)),
    case!(['ă', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['c', 'h', 'a', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Ch, &['c', 'h'], &[(V::ABreve, C::Lower)], Tone::Flat)),
    case!(['c', 'h', 'a', 'a'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::Ch, &['c', 'h'], &[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['c', 'o', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::OCircumflex, C::Lower)], Tone::Flat)),
    case!(['b', 'u', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::UHorn, C::Lower)], Tone::Flat)),
    case!(['t', 'h', 'o', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::OHorn, C::Lower)], Tone::Flat)),
    case!(
        ['t', 'o', 'i', 'o'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Flat)
    ),
    case!(['a', 'n', 'w'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::ABreve, C::Lower)], Tone::Flat, Coda::N, &['n'])),
    case!(['a', 'n', 'a'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::ACircumflex, C::Lower)], Tone::Flat, Coda::N, &['n'])),
    case!(['o', 'w', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Flat)),
    case!(['o', 'o', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Flat)),
    case!(['o', 'o', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::O, C::Lower)], Tone::Flat)),
    case!(['k', 'e', 'e'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::K, &['k'], &[(V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['c', 'h', 'e', 'e'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::Ch, &['c', 'h'], &[(V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['t', 'h', 'e', 'e'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['n', 'g', 'a', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Ng, &['n', 'g'], &[(V::ABreve, C::Lower)], Tone::Flat)),
    case!(['d', 'd', 'a', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::ABreve, C::Lower)], Tone::Flat)),
    case!(['b', 'u', 'w', 'n'], ParseAppendingPha ExpectedSyllable::syllable(Onset::B, &['b'], &[(V::UHorn, C::Lower)], Tone::Flat, Coda::N, &['n'])),
    case!(['t', 'h', 'a', 'a'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['e', 'e', 'u'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['o', 'o', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['o', 'w', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['u', 'w', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::A, C::Lower)], Tone::Flat)),
    case!(['a', 'a', 'n'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::ACircumflex, C::Lower)], Tone::Flat, Coda::N, &['n'])),
    case!(['b', 'a', 'w', 'n'], ParseAppendingPha ExpectedSyllable::syllable(Onset::B, &['b'], &[(V::ABreve, C::Lower)], Tone::Flat, Coda::N, &['n'])),
    case!(['u', 'w', 'n'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::UHorn, C::Lower)], Tone::Flat, Coda::N, &['n'])),
    case!(['l', 'a', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::L, &['l'], &[(V::ABreve, C::Lower)], Tone::Flat)),
    case!(['r', 'a', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::R, &['r'], &[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['h', 'a', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::H, &['h'], &[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['y', 'e', 'e'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['o', 'a', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::ABreve, C::Lower)], Tone::Flat)),
    case!(
        ['n', 'g', 'a', 'a', 'n'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ng, &['n', 'g'], &[(V::ACircumflex, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    case!(
        ['t', 'h', 'a', 'a', 'n'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Th, &['t', 'h'], &[(V::ACircumflex, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    case!(['u', 'a', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['u', 'a', 'a', 'y'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)),
    case!(['u', 'y', 'e', 'e'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['u', 'w', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
];
