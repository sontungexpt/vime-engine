//! K. Real Vietnamese syllables (telex) — the long-term regression corpus.

use super::{case, ExpectedSyllable, TestCase};
use super::{C, V};
use vime_engine::composition::ParseAppendingPhase;
use vime_engine::phonology::{Coda, Onset, Tone};

pub const CASES: &[TestCase] = &[
    case!(
        ['t', 'h', 'ủ', 'y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower)], Tone::Hook)
    ),
    case!(['h', 'ủ', 'y'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::H, &['h'], &[(V::U, C::Lower), (V::Y, C::Lower)], Tone::Hook)),
    case!(
        ['k', 'h', 'u', 'y', 'u', 'r'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Kh, &['k', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::U, C::Lower)], Tone::Hook)
    ),
    case!(
        ['t', 'h', 'u', 'y', 'r'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower)], Tone::Hook)
    ),
    case!(
        ['t', 'h', 'u', 'o', 'w', 'r'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Hook)
    ),
    case!(
        ['c', 'h', 'u', 'y', 'ệ', 'n'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ch, &['c', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Dot, Coda::N, &['n'])
    ),
    case!(
        ['t', 'h', 'u', 'y', 'ê', 'n'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    case!(
        ['n', 'g', 'ư', 'ơ', 'i', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ng, &['n', 'g'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['n', 'g', 'u', 'o', 'w', 'i', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ng, &['n', 'g'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['m', 'u', 'ố', 'n'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::M, &['m'], &[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    case!(['q', 'u', 'á'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::A, C::Lower)], Tone::Acute)),
    case!(
        ['q', 'u', 'o', 'o', 'c', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Qu, &['q', 'u'], &[(V::OCircumflex, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(
        ['n', 'u', 'o', 'w', 'c', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::N, &['n'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(
        ['t', 'i', 'e', 'e', 'n', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::T, &['t'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower)], Tone::Grave, Coda::N, &['n'])
    ),
    case!(
        ['c', 'ư', 'ờ', 'i'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['n', 'g', 'h', 'i', 'a', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ngh, &['n', 'g', 'h'], &[(V::I, C::Lower), (V::A, C::Lower)], Tone::Grave)
    ),
    case!(['g', 'i', 'ờ'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::OHorn, C::Lower)], Tone::Grave)),
    case!(['c', 'h', 'à', 'n'], ParseAppendingPha ExpectedSyllable::syllable(Onset::Ch, &['c', 'h'], &[(V::A, C::Lower)], Tone::Grave, Coda::N, &['n'])),
    case!(['a', 'n', 'h'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Nh, &['n', 'h'])),
    case!(['e', 'm'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::E, C::Lower)], Tone::Flat, Coda::M, &['m'])),
    case!(['ơ', 'n'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::OHorn, C::Lower)], Tone::Flat, Coda::N, &['n'])),
    case!(['ô', 'n', 'g'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::OCircumflex, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])),
    case!(['s', 'ư', 'a', 'r'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::S, &['s'], &[(V::UHorn, C::Lower), (V::A, C::Lower)], Tone::Hook)),
    case!(['m', 'ư', 'a', 's'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::UHorn, C::Lower), (V::A, C::Lower)], Tone::Acute)),
    case!(['q', 'u', 'y'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower)], Tone::Flat)),
    case!(['q', 'u', 'y', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower)], Tone::Acute)),
    case!(
        ['t', 'o', 'a', 'n', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::T, &['t'], &[(V::O, C::Lower), (V::A, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    case!(['t', 'o', 'a', 'f'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::O, C::Lower), (V::A, C::Lower)], Tone::Grave)),
    case!(['h', 'o', 'a', 'j'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::H, &['h'], &[(V::O, C::Lower), (V::A, C::Lower)], Tone::Dot)),
    case!(['a', 'o', 's'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower), (V::O, C::Lower)], Tone::Acute)),
    case!(
        ['y', 'e', 'e', 'u', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::Y, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Acute)
    ),
    case!(['y', 'ê', 'u'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::Y, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['y', 'ê', 'u', 'f'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::Y, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Grave)),
    case!(['n', 'h', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Nh, &['n', 'h'], &[(V::A, C::Lower)], Tone::Flat)),
    case!(['n', 'h', 'a', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Nh, &['n', 'h'], &[(V::A, C::Lower)], Tone::Grave)),
    case!(['n', 'h', 'a', 'r'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Nh, &['n', 'h'], &[(V::A, C::Lower)], Tone::Hook)),
    case!(['n', 'h', 'a', 'j'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Nh, &['n', 'h'], &[(V::A, C::Lower)], Tone::Dot)),
    case!(
        ['n', 'h', 'i', 'ê', 'u', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Nh, &['n', 'h'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Grave)
    ),
    case!(
        ['n', 'h', 'ậ', 't'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Nh, &['n', 'h'], &[(V::ACircumflex, C::Lower)], Tone::Dot, Coda::T, &['t'])
    ),
    case!(
        ['n', 'h', 'a', 'n', 'h', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Nh, &['n', 'h'], &[(V::A, C::Lower)], Tone::Acute, Coda::Nh, &['n', 'h'])
    ),
    case!(
        ['c', 'h', 'u', 'y', 'e', 'e', 'n', 'r'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ch, &['c', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Hook, Coda::N, &['n'])
    ),
    case!(
        ['t', 'h', 'u', 'y', 'e', 'e', 'n', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Grave, Coda::N, &['n'])
    ),
    case!(
        ['d', 'u', 'y', 'e', 'e', 'n'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::D, &['d'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    case!(
        ['n', 'g', 'u', 'y', 'e', 'e', 'n'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ng, &['n', 'g'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    case!(
        ['n', 'g', 'u', 'y', 'e', 'e', 'n', 'j'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ng, &['n', 'g'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Dot, Coda::N, &['n'])
    ),
    case!(
        ['k', 'h', 'u', 'y', 'a'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Kh, &['k', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::A, C::Lower)], Tone::Flat)
    ),
    case!(
        ['p', 'h', 'u', 'o', 'w', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ph, &['p', 'h'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['s', 'u', 'o', 'w', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::S, &['s'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['h', 'u', 'o', 'w', 'n', 'g', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['t', 'u', 'o', 'w', 'n', 'g', 'r'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::T, &['t'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Hook, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['d', 'd', 'u', 'o', 'w', 'n', 'g', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Đ, &['đ'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['t', 'r', 'u', 'o', 'w', 'n', 'g', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Tr, &['t', 'r'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['x', 'u', 'o', 'o', 'n', 'g', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::X, &['x'], &[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['c', 'u', 'w', 'u', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::UHorn, C::Lower), (V::U, C::Lower)], Tone::Acute)
    ),
    case!(
        ['h', 'u', 'w', 'u', 'x'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::H, &['h'], &[(V::UHorn, C::Lower), (V::U, C::Lower)], Tone::Tilde)
    ),
    case!(
        ['d', 'u', 'w', 'a', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::D, &['d'], &[(V::UHorn, C::Lower), (V::A, C::Lower)], Tone::Acute)
    ),
    case!(
        ['h', 'o', 'o', 'n', 'g', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::OCircumflex, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['s', 'o', 'o', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::S, &['s'], &[(V::OCircumflex, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['d', 'd', 'o', 'o', 'n', 'g', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Đ, &['đ'], &[(V::OCircumflex, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['t', 'h', 'o', 'o', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Th, &['t', 'h'], &[(V::OCircumflex, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['p', 'h', 'o', 'n', 'g', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ph, &['p', 'h'], &[(V::O, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['n', 'h', 'u', 'w', 'n', 'g', 'x'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Nh, &['n', 'h'], &[(V::UHorn, C::Lower)], Tone::Tilde, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['l', 'a', 'a', 'u'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::L, &['l'], &[(V::ACircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)
    ),
    case!(
        ['c', 'u', 'o', 'o', 'i', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Acute)
    ),
    case!(
        ['c', 'u', 'o', 'o', 'i', 'j'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Dot)
    ),
    case!(
        ['c', 'h', 'u', 'o', 'o', 'i', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ch, &['c', 'h'], &[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Acute)
    ),
    case!(
        ['t', 'i', 'e', 'e', 'u'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)
    ),
    case!(
        ['t', 'h', 'i', 'e', 'e', 'u', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Acute)
    ),
    case!(
        ['c', 'h', 'i', 'e', 'e', 'u', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ch, &['c', 'h'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Grave)
    ),
    case!(
        ['d', 'd', 'i', 'e', 'e', 'u', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Đ, &['đ'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Grave)
    ),
    case!(
        ['m', 'i', 'e', 'e', 'n'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::M, &['m'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    case!(
        ['p', 'h', 'i', 'e', 'e', 'n'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ph, &['p', 'h'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    case!(
        ['h', 'i', 'e', 'e', 'n'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    case!(
        ['v', 'i', 'e', 'e', 't', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::V, &['v'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower)], Tone::Acute, Coda::T, &['t'])
    ),
    case!(
        ['b', 'u', 'o', 'w', 'c', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::B, &['b'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(
        ['t', 'o', 'a', 'n', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::T, &['t'], &[(V::O, C::Lower), (V::A, C::Lower)], Tone::Grave, Coda::N, &['n'])
    ),
    case!(
        ['c', 'h', 'o', 'a', 'n', 'g', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ch, &['c', 'h'], &[(V::O, C::Lower), (V::A, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['h', 'o', 'a', 'w', 'c', 'j'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::O, C::Lower), (V::ABreve, C::Lower)], Tone::Dot, Coda::C, &['c'])
    ),
    case!(
        ['g', 'i', 'a', 'o', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::A, C::Lower), (V::O, C::Lower)], Tone::Acute)
    ),
    case!(
        ['t', 'h', 'u', 'e', 'e'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat)
    ),
    case!(
        ['t', 'h', 'u', 'e', 'e', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::ECircumflex, C::Lower)], Tone::Acute)
    ),
    case!(
        ['t', 'h', 'u', 'e', 'e', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::ECircumflex, C::Lower)], Tone::Grave)
    ),
    case!(
        ['k', 'h', 'o', 'e', 'r'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Kh, &['k', 'h'], &[(V::O, C::Lower), (V::E, C::Lower)], Tone::Hook)
    ),
    case!(['q', 'u', 'e', 'e'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(
        ['c', 'h', 'i', 'u', 'j'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ch, &['c', 'h'], &[(V::I, C::Lower), (V::U, C::Lower)], Tone::Dot)
    ),
    case!(['m', 'a', 'n', 'h', 'j'], ParseAppendingPha ExpectedSyllable::syllable(Onset::M, &['m'], &[(V::A, C::Lower)], Tone::Dot, Coda::Nh, &['n', 'h'])),
    case!(
        ['k', 'h', 'a', 'a', 'n', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Kh, &['k', 'h'], &[(V::ACircumflex, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    case!(
        ['n', 'g', 'a', 'w', 'n'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ng, &['n', 'g'], &[(V::ABreve, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    case!(['b', 'a', 'w', 'n'], ParseAppendingPha ExpectedSyllable::syllable(Onset::B, &['b'], &[(V::ABreve, C::Lower)], Tone::Flat, Coda::N, &['n'])),
    case!(
        ['t', 'h', 'u', 'y', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower)], Tone::Acute)
    ),
    case!(['h', 'u', 'y', 'r'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::H, &['h'], &[(V::U, C::Lower), (V::Y, C::Lower)], Tone::Hook)),
    case!(
        ['t', 'h', 'a', 'a', 'y', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Grave)
    ),
    case!(
        ['d', 'd', 'a', 'a', 's', 'y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Đ, &['đ'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Acute)
    ),
    case!(
        ['m', 'a', 'a', 's', 'y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Acute)
    ),
    case!(['a', 'a', 's', 'y'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Acute)),
    case!(
        ['d', 'a', 'a', 'y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::D, &['d'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)
    ),
    case!(
        ['m', 'a', 'a', 'y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)
    ),
    case!(
        ['t', 'a', 'a', 'y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)
    ),
    case!(
        ['x', 'a', 'a', 'y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::X, &['x'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)
    ),
    case!(
        ['k', 'e', 'e', 'u'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::K, &['k'], &[(V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)
    ),
    case!(
        ['d', 'i', 'e', 'e', 'u', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::D, &['d'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Grave)
    ),
    case!(
        ['k', 'h', 'i', 'e', 'e', 'u'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Kh, &['k', 'h'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)
    ),
    case!(
        ['t', 'h', 'i', 'u'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::I, C::Lower), (V::U, C::Lower)], Tone::Flat)
    ),
    case!(
        ['d', 'd', 'e', 'e', 'u', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Đ, &['đ'], &[(V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Grave)
    ),
    case!(
        ['s', 'a', 'a', 'u'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::S, &['s'], &[(V::ACircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)
    ),
    case!(
        ['c', 'a', 'a', 'u'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::ACircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)
    ),
    case!(
        ['c', 'a', 'a', 'u', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::ACircumflex, C::Lower), (V::U, C::Lower)], Tone::Grave)
    ),
    case!(
        ['t', 'r', 'a', 'a', 'u'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Tr, &['t', 'r'], &[(V::ACircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)
    ),
    case!(
        ['b', 'a', 'a', 'u', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::ACircumflex, C::Lower), (V::U, C::Lower)], Tone::Grave)
    ),
    case!(
        ['d', 'd', 'a', 'a', 'u', 's'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Đ, &['đ'], &[(V::ACircumflex, C::Lower), (V::U, C::Lower)], Tone::Acute)
    ),
    case!(
        ['c', 'h', 'u', 'w', 'n', 'g', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ch, &['c', 'h'], &[(V::UHorn, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['n', 'g', 'u', 'w', 'n', 'g', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ng, &['n', 'g'], &[(V::UHorn, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['m', 'u', 'w', 'n', 'g', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::M, &['m'], &[(V::UHorn, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['c', 'u', 'w', 'n', 'g', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::C, &['c'], &[(V::UHorn, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['c', 'u', 'w', 'n', 'g', 'r'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::C, &['c'], &[(V::UHorn, C::Lower)], Tone::Hook, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['c', 'u', 'w', 'n', 'g', 'x'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::C, &['c'], &[(V::UHorn, C::Lower)], Tone::Tilde, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['s', 'u', 'w', 'n', 'g', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::S, &['s'], &[(V::UHorn, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['n', 'o', 'o', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::N, &['n'], &[(V::OCircumflex, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['l', 'o', 'o', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::L, &['l'], &[(V::OCircumflex, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['n', 'o', 'o', 'n', 'g', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::N, &['n'], &[(V::OCircumflex, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['c', 'o', 'o', 'n', 'g', 'j'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::C, &['c'], &[(V::OCircumflex, C::Lower)], Tone::Dot, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['t', 'r', 'o', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Tr, &['t', 'r'], &[(V::O, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['t', 'r', 'o', 'o', 'n', 'g', 'j'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Tr, &['t', 'r'], &[(V::OCircumflex, C::Lower)], Tone::Dot, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['n', 'o', 'n', 'g', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::N, &['n'], &[(V::O, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['t', 'r', 'o', 'n', 'g', 'j'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Tr, &['t', 'r'], &[(V::O, C::Lower)], Tone::Dot, Coda::Ng, &['n', 'g'])
    ),
    case!(['c', 'o', 'n', 'g', 'j'], ParseAppendingPha ExpectedSyllable::syllable(Onset::C, &['c'], &[(V::O, C::Lower)], Tone::Dot, Coda::Ng, &['n', 'g'])),
    case!(
        ['c', 'h', 'o', 'o', 'n', 'g', 's'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ch, &['c', 'h'], &[(V::OCircumflex, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['c', 'h', 'o', 'o', 'n', 'g', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ch, &['c', 'h'], &[(V::OCircumflex, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['l', 'u', 'o', 'w', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::L, &['l'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['l', 'u', 'o', 'w', 'n', 'g', 'j'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::L, &['l'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Dot, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['c', 'h', 'u', 'o', 'w', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ch, &['c', 'h'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['v', 'u', 'o', 'w', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::V, &['v'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['x', 'u', 'o', 'w', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::X, &['x'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['t', 'u', 'o', 'w', 'n', 'g', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::T, &['t'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['h', 'u', 'o', 'w', 'n', 'g', 'r'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Hook, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['p', 'h', 'u', 'o', 'w', 'n', 'g', 'j'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ph, &['p', 'h'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Dot, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['q', 'u', 'y', 'n', 'h'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower)], Tone::Flat, Coda::Nh, &['n', 'h'])
    ),
    case!(
        ['q', 'u', 'y', 'n', 'h', 'f'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower)], Tone::Grave, Coda::Nh, &['n', 'h'])
    ),
    case!(
        ['q', 'u', 'y', 'e', 'e', 'n', 'r'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Hook, Coda::N, &['n'])
    ),
    case!(
        ['q', 'u', 'y', 'e', 'e', 't', 'j'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Dot, Coda::T, &['t'])
    ),
    case!(
        ['n', 'h', 'u', 'y', 'e', 'e', 'n', 'x'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Nh, &['n', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Tilde, Coda::N, &['n'])
    ),
    case!(
        ['r', 'i', 'e', 'e', 'u'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::R, &['r'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)
    ),
    case!(
        ['t', 'r', 'e', 'e', 'u'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Tr, &['t', 'r'], &[(V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)
    ),
    case!(['r', 'i', 'u', 's'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::R, &['r'], &[(V::I, C::Lower), (V::U, C::Lower)], Tone::Acute)),
    case!(['b', 'o', 'w', 'i'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(
        ['n', 'o', 'o', 'i', 'r'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::N, &['n'], &[(V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Hook)
    ),
    case!(['m', 'o', 'i', 'f'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::O, C::Lower), (V::I, C::Lower)], Tone::Grave)),
    case!(['c', 'u', 'i', 'f'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::U, C::Lower), (V::I, C::Lower)], Tone::Grave)),
    case!(['v', 'u', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::V, &['v'], &[(V::U, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['o', 'a', 'i', 'r'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower), (V::I, C::Lower)], Tone::Hook)),
    case!(
        ['x', 'o', 'a', 'y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::X, &['x'], &[(V::O, C::Lower), (V::A, C::Lower), (V::Y, C::Lower)], Tone::Flat)
    ),
    case!(
        ['t', 'h', 'o', 'a', 'i', 'r'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::O, C::Lower), (V::A, C::Lower), (V::I, C::Lower)], Tone::Hook)
    ),
    case!(
        ['o', 'a', 'w', 'c', 'j'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::O, C::Lower), (V::ABreve, C::Lower)], Tone::Dot, Coda::C, &['c'])
    ),
    case!(
        ['b', 'u', 'o', 'o', 'i', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['r', 'u', 'o', 'o', 'i', 'f'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::R, &['r'], &[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(['m', 'u', 'w', 'c', 's'], ParseAppendingPha ExpectedSyllable::syllable(Onset::M, &['m'], &[(V::UHorn, C::Lower)], Tone::Acute, Coda::C, &['c'])),
    case!(
        ['u', 'y', 'e', 'e', 'n', 'r'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Hook, Coda::N, &['n'])
    ),
];
