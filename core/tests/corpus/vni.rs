//! F. The VNI layout.
//!
//! VNI layout (config/input/default/vni.rs):
//!   tones 1..=5 = acute/grave/hook/tilde/dot, 0 = flat reset
//!   shapes 6 = circumflex (a e o), 7 = breve(a) / horn(o), 8 = horn(u)
//!   stroke 9 = d → đ

use super::{case, ExpectedSyllable, TestCase};
use super::{C, V};
use vime_engine::composition::ParseAppendingPhase;
use vime_engine::phonology::{Coda, Onset, Tone};

pub const CASES: &[TestCase] = &[
    case!(['a', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute)),
    case!(['a', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Grave)),
    case!(['a', '3'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Hook)),
    case!(['a', '4'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Tilde)),
    case!(['a', '5'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Dot)),
    case!(['e', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::E, C::Lower)], Tone::Acute)),
    case!(['e', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::E, C::Lower)], Tone::Grave)),
    case!(['i', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::I, C::Lower)], Tone::Acute)),
    case!(['o', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Acute)),
    case!(['u', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower)], Tone::Acute)),
    case!(['u', '5'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower)], Tone::Dot)),
    case!(['o', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Grave)),
    case!(['u', '3'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower)], Tone::Hook)),
    case!(['i', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::I, C::Lower)], Tone::Grave)),
    case!(['i', '3'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::I, C::Lower)], Tone::Hook)),
    case!(['y', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::Y, C::Lower)], Tone::Acute)),
    case!(['y', '4'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::Y, C::Lower)], Tone::Tilde)),
    case!(['a', '1', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Grave)),
    case!(['a', '1', '0'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat)),
    case!(['a', '2', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute)),
    case!(['a', '3', '5'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Dot)),
    case!(['a', '6'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['a', '7'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Flat)),
    case!(['e', '6'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['o', '6'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Flat)),
    case!(['o', '7'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', '8'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Flat)),
    case!(['a', '7', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Acute)),
    case!(['a', '7', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Grave)),
    case!(['a', '7', '5'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Dot)),
    case!(['a', '6', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Acute)),
    case!(['o', '6', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Acute)),
    case!(['o', '6', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Grave)),
    case!(['o', '7', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Grave)),
    case!(['o', '7', '3'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Hook)),
    case!(['e', '6', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower)], Tone::Grave)),
    case!(['u', '8', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Acute)),
    case!(['u', '8', '4'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Tilde)),
    case!(['u', '8', '5'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Dot)),
    case!(['a', '1', '7'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Acute)),
    case!(['a', '1', '6'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Acute)),
    case!(['d', '9', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::A, C::Lower)], Tone::Flat)),
    case!(['d', '9', 'a', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::A, C::Lower)], Tone::Acute)),
    case!(['D', '9', 'A'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::DStroke, &['Đ'], &[(V::A, C::Upper)], Tone::Flat)),
    case!(['u', 'o', '7'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'o', '7', 'i'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(
        ['u', 'o', '7', 'i', '1'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Acute)
    ),
    case!(['u', 'o', '6', 'i'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['u', 'o', '6'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Flat)),
    case!(['q', 'u', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::A, C::Lower)], Tone::Flat)),
    case!(['q', 'u', 'a', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::A, C::Lower)], Tone::Acute)),
    case!(
        ['m', 'u', 'o', '6', 'n', '1'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::M, &['m'], &[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    case!(
        ['c', 'h', 'o', 'a', '1'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ch, &['c', 'h'], &[(V::O, C::Lower), (V::A, C::Lower)], Tone::Acute)
    ),
    case!(['d', '9', 'e', 'p', '5'], ParseAppendingPha ExpectedSyllable::syllable(Onset::Đ, &['đ'], &[(V::E, C::Lower)], Tone::Dot, Coda::P, &['p'])),
    case!(
        ['v', 'i', 'e', '6', 't', '5'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::V, &['v'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower)], Tone::Dot, Coda::T, &['t'])
    ),
    case!(
        ['n', 'g', 'ư', 'ơ', 'i', '2'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ng, &['n', 'g'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['t', 'h', 'ư', 'ơ', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Th, &['t', 'h'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(['x', 'o', 'n', 'g'], ParseAppendingPha ExpectedSyllable::syllable(Onset::X, &['x'], &[(V::O, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])),
    case!(
        ['t', 'h', 'u', 'o', '7', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Th, &['t', 'h'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['n', 'u', 'o', '7', 'c', '1'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::N, &['n'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(
        ['t', 'r', 'u', '8', 'n', 'g', '1'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Tr, &['t', 'r'], &[(V::UHorn, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['q', 'u', 'y', 'e', '6', 't', '1'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Acute, Coda::T, &['t'])
    ),
    case!(
        ['s', 'o', '6', 'n', 'g', '1'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::S, &['s'], &[(V::OCircumflex, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['n', 'g', 'h', 'e', '6', '2'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ngh, &['n', 'g', 'h'], &[(V::ECircumflex, C::Lower)], Tone::Grave)
    ),
    case!(['x', 'i', 'n', 'h'], ParseAppendingPha ExpectedSyllable::syllable(Onset::X, &['x'], &[(V::I, C::Lower)], Tone::Flat, Coda::Nh, &['n', 'h'])),
    case!(['m', 'a', 'n', 'g'], ParseAppendingPha ExpectedSyllable::syllable(Onset::M, &['m'], &[(V::A, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])),
    case!(
        ['t', 'r', 'a', '7', 'n', 'g', '1'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Tr, &['t', 'r'], &[(V::ABreve, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])
    ),
    case!(['n', 'o', 'i', '1'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::N, &['n'], &[(V::O, C::Lower), (V::I, C::Lower)], Tone::Acute)),
    case!(['b', 'a', 'n', '2'], ParseAppendingPhase::Coda, ExpectedSyllable::syllable(Onset::B, &['b'], &[(V::A, C::Lower)], Tone::Grave, Coda::N, &['n'])),
    case!(['c', 'h', 'u', '8', '4'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::Ch, &['c', 'h'], &[(V::UHorn, C::Lower)], Tone::Tilde)),
    case!(['m', 'e', '5'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::E, C::Lower)], Tone::Dot)),
    case!(
        ['s', 'u', '8', 'a', '3'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::S, &['s'], &[(V::UHorn, C::Lower), (V::A, C::Lower)], Tone::Hook)
    ),
    case!(
        ['d', '9', 'u', '8', 'c', '1'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::DStroke, &['đ'], &[(V::UHorn, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(['h', 'o', 'c', '5'], ParseAppendingPhase::Coda, ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::O, C::Lower)], Tone::Dot, Coda::C, &['c'])),
    case!(['q', 'u', 'y', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower)], Tone::Acute)),
    case!(
        ['t', 'i', 'e', '6', 'n', '1'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::T, &['t'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    case!(
        ['n', 'h', 'a', 'n', 'h'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Nh, &['n', 'h'], &[(V::A, C::Lower)], Tone::Flat, Coda::Nh, &['n', 'h'])
    ),
    case!(['c', 'h', 'e', '6'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::Ch, &['c', 'h'], &[(V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(
        ['m', 'o', '6', 'i'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Flat)
    ),
    case!(
        ['d', '9', 'o', '6', 't', '1'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::DStroke, &['đ'], &[(V::OCircumflex, C::Lower)], Tone::Acute, Coda::T, &['t'])
    ),
    case!(['m', 'a', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::A, C::Lower)], Tone::Acute)),
    case!(
        ['t', 'h', 'u', 'e', '6', '1'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::ECircumflex, C::Lower)], Tone::Acute)
    ),
    case!(
        ['t', 'h', 'u', 'e', '6'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat)
    ),
    case!(
        ['h', 'o', 'a', '7', 'c', '5'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::O, C::Lower), (V::ABreve, C::Lower)], Tone::Dot, Coda::C, &['c'])
    ),
    case!(
        ['k', 'h', 'o', 'e', '3'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Kh, &['k', 'h'], &[(V::O, C::Lower), (V::E, C::Lower)], Tone::Hook)
    ),
    case!(['y', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::Y, C::Lower)], Tone::Grave)),
    case!(['y', '3'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::Y, C::Lower)], Tone::Hook)),
    case!(['y', '5'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::Y, C::Lower)], Tone::Dot)),
    case!(['d', '9', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::I, C::Lower)], Tone::Flat)),
    case!(['d', '9', 'i', '1'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::I, C::Lower)], Tone::Acute)),
    case!(
        ['t', 'h', 'a', 'n', 'h', '2'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Th, &['t', 'h'], &[(V::A, C::Lower)], Tone::Grave, Coda::Nh, &['n', 'h'])
    ),
    case!(
        ['n', 'g', 'h', 'e', '6', '1'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ngh, &['n', 'g', 'h'], &[(V::ECircumflex, C::Lower)], Tone::Acute)
    ),
    case!(
        ['s', 'u', '8', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::S, &['s'], &[(V::UHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['t', 'h', 'i', 'e', '6', 'u', '1'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Acute)
    ),
    case!(['b', 'a', 'n', 'h'], ParseAppendingPha ExpectedSyllable::syllable(Onset::B, &['b'], &[(V::A, C::Lower)], Tone::Flat, Coda::Nh, &['n', 'h'])),
    case!(['n', 'g', 'h', 'e'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Ngh, &['n', 'g', 'h'], &[(V::E, C::Lower)], Tone::Flat)),
    case!(['m', 'o', '5', 'i'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::O, C::Lower), (V::I, C::Lower)], Tone::Dot)),
    case!(
        ['m', 'u', 'o', '7', 'i', '2'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['c', 'u', 'o', '7', 'i', '2'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['u', 'o', '7', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::vowel_coda(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['h', 'u', 'o', '7', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['h', 'u', 'o', '7', 'n', 'g', '3'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Hook, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['c', 'h', 'u', 'o', '7', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ch, &['c', 'h'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['v', 'u', 'o', '7', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::V, &['v'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['n', 'g', 'u', 'y', 'e', '6', 'n', '5'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ng, &['n', 'g'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Dot, Coda::N, &['n'])
    ),
    case!(
        ['d', 'u', 'y', 'e', '6', 't', '5'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::D, &['d'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Dot, Coda::T, &['t'])
    ),
    case!(
        ['d', '9', 'u', 'o', '7', 'c', '5'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::DStroke, &['đ'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Dot, Coda::C, &['c'])
    ),
    case!(
        ['k', 'h', 'a', '1', 'c'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Kh, &['k', 'h'], &[(V::A, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(
        ['k', 'h', 'a', '1', 'c', 'h'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Kh, &['k', 'h'], &[(V::A, C::Lower)], Tone::Acute, Coda::Ch, &['c', 'h'])
    ),
    case!(['m', 'a', '5', 'n', 'h'], ParseAppendingPha ExpectedSyllable::syllable(Onset::M, &['m'], &[(V::A, C::Lower)], Tone::Dot, Coda::Nh, &['n', 'h'])),
    case!(['s', 'a', '5', 'c', 'h'], ParseAppendingPha ExpectedSyllable::syllable(Onset::S, &['s'], &[(V::A, C::Lower)], Tone::Dot, Coda::Ch, &['c', 'h'])),
    case!(['y', 'e', '6', 'u'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::Y, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(
        ['y', 'e', '6', 'u', '1'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::Y, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Acute)
    ),
    case!(
        ['t', 'o', 'a', 'n', '2'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::T, &['t'], &[(V::O, C::Lower), (V::A, C::Lower)], Tone::Grave, Coda::N, &['n'])
    ),
    case!(
        ['h', 'o', 'a', 'n', '2'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::O, C::Lower), (V::A, C::Lower)], Tone::Grave, Coda::N, &['n'])
    ),
    case!(
        ['q', 'u', 'y', 'e', '6', 'n', '3'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Hook, Coda::N, &['n'])
    ),
    case!(
        ['t', 'u', 'y', 'e', '6', 't', '1'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::T, &['t'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Acute, Coda::T, &['t'])
    ),
    case!(['n', 'h', 'a', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Nh, &['n', 'h'], &[(V::A, C::Lower)], Tone::Grave)),
    case!(['l', 'a', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::L, &['l'], &[(V::A, C::Lower)], Tone::Grave)),
    case!(['b', 'a', '2'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::A, C::Lower)], Tone::Grave)),
    case!(
        ['V', 'i', 'e', '6', 't', '5'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::V, &['V'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower)], Tone::Dot, Coda::T, &['t'])
    ),
    case!(
        ['k', 'h', 'o', 'a', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Kh, &['k', 'h'], &[(V::O, C::Lower), (V::A, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(['i', 'e', 'u'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::I, C::Lower), (V::E, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['o', 'a', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['o', 'a', 'u'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['o', 'e', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::E, C::Lower), (V::O, C::Lower)], Tone::Flat)),
    case!(['u', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['u', 'a', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::A, C::Lower), (V::O, C::Lower)], Tone::Flat)),
    case!(['u', 'y', 'u'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::Y, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['u', 'y', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::Y, C::Lower), (V::A, C::Lower)], Tone::Flat)),
    case!(['u', 'a', '6'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['u', 'a', '6', 'y'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)),
    case!(['u', 'y', 'e', '6'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['o', 'a', '7'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::ABreve, C::Lower)], Tone::Flat)),
    case!(['o', '6', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['o', '7', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['u', '8', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
];
