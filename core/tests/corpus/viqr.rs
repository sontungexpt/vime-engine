//! F2. The VIQr layout.
//!
//! VIQr layout (config/input/default/viqr.rs):
//!   tones `` ` `` (grave), `?` (hook), `~` (tilde), `'` (acute), `.` (dot),
//!   `z` (flat reset)
//!   shapes `^` = circumflex (a e o), `(` = breve (a), `+` = horn (o u)
//!   stroke `d` = d → đ

use super::{case, ExpectedSyllable, TestCase};
use super::{C, V};
use vime_engine::composition::ParseAppendingPhase;
use vime_engine::phonology::{Coda, Onset, Tone};

pub const CASES: &[TestCase] = &[
    case!(['a', '\''], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute)),
    case!(['a', '`'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Grave)),
    case!(['a', '?'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Hook)),
    case!(['a', '~'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Tilde)),
    case!(['a', '.'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Dot)),
    case!(['e', '\''], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::E, C::Lower)], Tone::Acute)),
    case!(['e', '`'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::E, C::Lower)], Tone::Grave)),
    case!(['o', '?'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Hook)),
    case!(['u', '~'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower)], Tone::Tilde)),
    case!(['i', '.'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::I, C::Lower)], Tone::Dot)),
    case!(['y', '?'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::Y, C::Lower)], Tone::Hook)),
    case!(['y', '`'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::Y, C::Lower)], Tone::Grave)),
    case!(['á', 'z'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat)),
    case!(['ấ', 'z'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['a', '\'', '`'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Grave)),
    case!(['a', 'n', '\''], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Acute, Coda::N, &['n'])),
    case!(['a', 'm', '`'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Grave, Coda::M, &['m'])),
    case!(['a', 't', '.'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Dot, Coda::T, &['t'])),
    case!(['a', '^'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['a', '('], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Flat)),
    case!(['e', '^'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['o', '^'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Flat)),
    case!(['o', '+'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', '+'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Flat)),
    case!(['a', '^', '\''], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Acute)),
    case!(['a', '^', '?'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Hook)),
    case!(['a', '(', '\''], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Acute)),
    case!(['a', '(', '.'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Dot)),
    case!(['o', '^', '.'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Dot)),
    case!(['o', '+', '\''], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Acute)),
    case!(['o', '+', '?'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Hook)),
    case!(['u', '+', '\''], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Acute)),
    case!(['u', '+', '~'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Tilde)),
    case!(['a', '\'', '^'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Acute)),
    case!(['d', 'd'], ParseAppendingPhase::Onset, ExpectedSyllable::consonant(Onset::DStroke, &['đ'])),
    case!(['d', 'd', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::A, C::Lower)], Tone::Flat)),
    case!(['d', 'd', 'a', '\''], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::A, C::Lower)], Tone::Acute)),
    case!(['D', 'd'], ParseAppendingPhase::Onset, ExpectedSyllable::consonant(Onset::DStroke, &['Đ'])),
    case!(['q', 'u', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::A, C::Lower)], Tone::Flat)),
    case!(['q', 'u', 'a', '\''], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::A, C::Lower)], Tone::Acute)),
    case!(['q', 'u', 'y'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower)], Tone::Flat)),
    case!(['u', 'o', '+'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'o', '+', 'i'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(
        ['u', 'o', '+', 'i', '\''],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Acute)
    ),
    case!(['u', 'o', '^', 'i'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['ư', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::A, C::Lower)], Tone::Flat)),
    case!(
        ['t', 'h', 'ư', 'a', '?'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::UHorn, C::Lower), (V::A, C::Lower)], Tone::Hook)
    ),
    case!(
        ['n', 'g', 'u', 'o', '+', 'i', '`'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ng, &['n', 'g'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['n', 'u', 'o', '+', 'c', '\''],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::N, &['n'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(
        ['d', 'd', 'u', 'o', '+', 'c', '.'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::DStroke, &['đ'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Dot, Coda::C, &['c'])
    ),
    case!(
        ['q', 'u', 'y', 'e', '^', 't', '\''],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Acute, Coda::T, &['t'])
    ),
    case!(['h', 'o', 'i', '?'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::H, &['h'], &[(V::O, C::Lower), (V::I, C::Lower)], Tone::Hook)),
    case!(
        ['d', 'd', 'e', '^', 'n', '\''],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::DStroke, &['đ'], &[(V::ECircumflex, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    case!(
        ['v', 'i', 'e', '^', 't', '.'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::V, &['v'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower)], Tone::Dot, Coda::T, &['t'])
    ),
    case!(
        ['t', 'h', 'a', 'n', 'h', '`'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Th, &['t', 'h'], &[(V::A, C::Lower)], Tone::Grave, Coda::Nh, &['n', 'h'])
    ),
    case!(['s', 'o', 'n', 'g'], ParseAppendingPha ExpectedSyllable::syllable(Onset::S, &['s'], &[(V::O, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])),
    case!(['x', 'i', 'n', 'h'], ParseAppendingPha ExpectedSyllable::syllable(Onset::X, &['x'], &[(V::I, C::Lower)], Tone::Flat, Coda::Nh, &['n', 'h'])),
    case!(
        ['t', 'h', 'u', 'e', '^', '\''],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::ECircumflex, C::Lower)], Tone::Acute)
    ),
    case!(
        ['t', 'h', 'u', 'e', '^'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat)
    ),
    case!(
        ['h', 'o', 'a', '(', 'c', '.'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::O, C::Lower), (V::ABreve, C::Lower)], Tone::Dot, Coda::C, &['c'])
    ),
    case!(
        ['k', 'h', 'o', 'e', '?'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Kh, &['k', 'h'], &[(V::O, C::Lower), (V::E, C::Lower)], Tone::Hook)
    ),
    case!(['d', 'd', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::I, C::Lower)], Tone::Flat)),
    case!(['q', 'u', 'e', '^'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['y', 'e', '^', 'u'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::Y, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['m', 'u', '+', 'a'], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::UHorn, C::Lower), (V::A, C::Lower)], Tone::Flat)),
    case!(
        ['h', 'u', '+', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::UHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(['a', 'n', 'h', '`'], ParseAppendingPhase::Coda, ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Grave, Coda::Nh, &['n', 'h'])),
    case!(['b', 'a', 'n', '?'], ParseAppendingPhase::Coda, ExpectedSyllable::syllable(Onset::B, &['b'], &[(V::A, C::Lower)], Tone::Hook, Coda::N, &['n'])),
    case!(
        ['m', 'u', 'o', '+', 'i', '`'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['c', 'u', 'o', '+', 'i', '`'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['k', 'h', 'o', '^', 'n', 'g'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Kh, &['k', 'h'], &[(V::OCircumflex, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['h', 'o', '^', 'n', 'g', '`'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::OCircumflex, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['s', 'o', '^', 'n', 'g', '\''],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::S, &['s'], &[(V::OCircumflex, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])
    ),
    case!(['m', 'a', 'n', 'h', '.'], ParseAppendingPha ExpectedSyllable::syllable(Onset::M, &['m'], &[(V::A, C::Lower)], Tone::Dot, Coda::Nh, &['n', 'h'])),
    case!(['x', 'a', 'n', 'h'], ParseAppendingPha ExpectedSyllable::syllable(Onset::X, &['x'], &[(V::A, C::Lower)], Tone::Flat, Coda::Nh, &['n', 'h'])),
    case!(['n', 'h', 'a', '`'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Nh, &['n', 'h'], &[(V::A, C::Lower)], Tone::Grave)),
    case!(['b', 'a', '`'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::A, C::Lower)], Tone::Grave)),
    case!(['a', '^', 'y', '\''], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Acute)),
    case!(
        ['d', 'a', '^', 'y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::D, &['d'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)
    ),
    case!(
        ['m', 'a', '^', 'y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)
    ),
    case!(
        ['c', 'a', '^', 'y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)
    ),
    case!(
        ['k', 'h', 'u', 'y', 'a'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Kh, &['k', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::A, C::Lower)], Tone::Flat)
    ),
    case!(
        ['c', 'h', 'u', 'y', 'e', '^', 'n', '.'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ch, &['c', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Dot, Coda::N, &['n'])
    ),
    case!(
        ['n', 'g', 'u', 'y', 'e', '^', 'n'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Ng, &['n', 'g'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    case!(
        ['y', 'e', '^', 'u', '\''],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::vowel(&[(V::Y, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Acute)
    ),
    case!(
        ['k', 'h', 'o', 'e'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Kh, &['k', 'h'], &[(V::O, C::Lower), (V::E, C::Lower)], Tone::Flat)
    ),
    case!(['q', 'u', 'y', '\''], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower)], Tone::Acute)),
    case!(['l', 'a', '.'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::L, &['l'], &[(V::A, C::Lower)], Tone::Dot)),
    case!(['h', 'o', 'c', '.'], ParseAppendingPhase::Coda, ExpectedSyllable::syllable(Onset::H, &['h'], &[(V::O, C::Lower)], Tone::Dot, Coda::C, &['c'])),
    case!(['t', 'h', 'e', '^', '\''], ParseAppendingPhas ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::ECircumflex, C::Lower)], Tone::Acute)),
    case!(['i', 'e', 'u'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::I, C::Lower), (V::E, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['o', 'a', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['o', 'a', 'u'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['o', 'e', 'o'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::E, C::Lower), (V::O, C::Lower)], Tone::Flat)),
    case!(['u', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['u', 'y', 'u'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::Y, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['u', 'y', 'a'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::Y, C::Lower), (V::A, C::Lower)], Tone::Flat)),
    case!(['u', 'a', '^'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['u', 'a', '^', 'y'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)),
    case!(['u', 'y', 'e', '^'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['o', 'a', '('], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::ABreve, C::Lower)], Tone::Flat)),
    case!(['o', '^', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['o', '+', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['u', '+', 'i'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
];
