//! F2. The VIQr layout.
//!
//! VIQr layout (config/input/default/viqr.rs):
//!   tones `` ` `` (grave), `?` (hook), `~` (tilde), `'` (acute), `.` (dot),
//!   `z` (flat reset)
//!   shapes `^` = circumflex (a e o), `(` = breve (a), `+` = horn (o u)
//!   stroke `d` = d → đ

use super::prelude::*;

pub const CASES: &[Case] = &[
    case!(['a', '\''], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute)),
    case!(['a', '`'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Grave)),
    case!(['a', '?'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Hook)),
    case!(['a', '~'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Tilde)),
    case!(['a', '.'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Dot)),
    case!(['e', '\''], ExpectedSyllable::vowel(&[(V::E, C::Lower)], Tone::Acute)),
    case!(['e', '`'], ExpectedSyllable::vowel(&[(V::E, C::Lower)], Tone::Grave)),
    case!(['o', '?'], ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Hook)),
    case!(['u', '~'], ExpectedSyllable::vowel(&[(V::U, C::Lower)], Tone::Tilde)),
    case!(['i', '.'], ExpectedSyllable::vowel(&[(V::I, C::Lower)], Tone::Dot)),
    case!(['y', '?'], ExpectedSyllable::vowel(&[(V::Y, C::Lower)], Tone::Hook)),
    case!(['y', '`'], ExpectedSyllable::vowel(&[(V::Y, C::Lower)], Tone::Grave)),
    case!(['á', 'z'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat)),
    case!(['ấ', 'z'], ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['a', '\'', '`'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Grave)),
    case!(['a', 'n', '\''], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Acute, Coda::N, &['n'])),
    case!(['a', 'm', '`'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Grave, Coda::M, &['m'])),
    case!(['a', 't', '.'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Dot, Coda::T, &['t'])),
    case!(['a', '^'], ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['a', '('], ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Flat)),
    case!(['e', '^'], ExpectedSyllable::vowel(&[(V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['o', '^'], ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Flat)),
    case!(['o', '+'], ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', '+'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Flat)),
    case!(['a', '^', '\''], ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Acute)),
    case!(['a', '^', '?'], ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Hook)),
    case!(['a', '(', '\''], ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Acute)),
    case!(['a', '(', '.'], ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Dot)),
    case!(['o', '^', '.'], ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Dot)),
    case!(['o', '+', '\''], ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Acute)),
    case!(['o', '+', '?'], ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Hook)),
    case!(['u', '+', '\''], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Acute)),
    case!(['u', '+', '~'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Tilde)),
    case!(['a', '\'', '^'], ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Acute)),
    case!(['d', 'd'], ExpectedSyllable::consonant(Onset::DStroke, &['đ'])),
    case!(['d', 'd', 'a'], ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::A, C::Lower)], Tone::Flat)),
    case!(['d', 'd', 'a', '\''], ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::A, C::Lower)], Tone::Acute)),
    case!(['D', 'd'], ExpectedSyllable::consonant(Onset::DStroke, &['Đ'])),
    case!(['q', 'u', 'a'], ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::A, C::Lower)], Tone::Flat)),
    case!(['q', 'u', 'a', '\''], ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::A, C::Lower)], Tone::Acute)),
    case!(['q', 'u', 'y'], ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower)], Tone::Flat)),
    case!(['u', 'o', '+'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'o', '+', 'i'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['u', 'o', '+', 'i', '\''], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Acute)),
    case!(['u', 'o', '^', 'i'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['ư', 'a'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::A, C::Lower)], Tone::Flat)),
    case!(['t', 'h', 'ư', 'a', '?'], ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::UHorn, C::Lower), (V::A, C::Lower)], Tone::Hook)),
    case!(
        ['n', 'g', 'u', 'o', '+', 'i', '`'],
        ExpectedSyllable::onset_vowel(Onset::Ng, &['n', 'g'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['n', 'u', 'o', '+', 'c', '\''],
        ExpectedSyllable::full(Onset::N, &['n'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(
        ['d', 'd', 'u', 'o', '+', 'c', '.'],
        ExpectedSyllable::full(Onset::DStroke, &['đ'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Dot, Coda::C, &['c'])
    ),
    case!(
        ['q', 'u', 'y', 'e', '^', 't', '\''],
        ExpectedSyllable::full(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Acute, Coda::T, &['t'])
    ),
    case!(['h', 'o', 'i', '?'], ExpectedSyllable::onset_vowel(Onset::H, &['h'], &[(V::O, C::Lower), (V::I, C::Lower)], Tone::Hook)),
    case!(['d', 'd', 'e', '^', 'n', '\''], ExpectedSyllable::full(Onset::DStroke, &['đ'], &[(V::ECircumflex, C::Lower)], Tone::Acute, Coda::N, &['n'])),
    case!(
        ['v', 'i', 'e', '^', 't', '.'],
        ExpectedSyllable::full(Onset::V, &['v'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower)], Tone::Dot, Coda::T, &['t'])
    ),
    case!(['t', 'h', 'a', 'n', 'h', '`'], ExpectedSyllable::full(Onset::Th, &['t', 'h'], &[(V::A, C::Lower)], Tone::Grave, Coda::Nh, &['n', 'h'])),
    case!(['s', 'o', 'n', 'g'], ExpectedSyllable::full(Onset::S, &['s'], &[(V::O, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])),
    case!(['x', 'i', 'n', 'h'], ExpectedSyllable::full(Onset::X, &['x'], &[(V::I, C::Lower)], Tone::Flat, Coda::Nh, &['n', 'h'])),
    case!(
        ['t', 'h', 'u', 'e', '^', '\''],
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::ECircumflex, C::Lower)], Tone::Acute)
    ),
    case!(['t', 'h', 'u', 'e', '^'], ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::U, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['h', 'o', 'a', '(', 'c', '.'], ExpectedSyllable::full(Onset::H, &['h'], &[(V::O, C::Lower), (V::ABreve, C::Lower)], Tone::Dot, Coda::C, &['c'])),
    case!(['k', 'h', 'o', 'e', '?'], ExpectedSyllable::onset_vowel(Onset::Kh, &['k', 'h'], &[(V::O, C::Lower), (V::E, C::Lower)], Tone::Hook)),
    case!(['d', 'd', 'i'], ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::I, C::Lower)], Tone::Flat)),
    case!(['q', 'u', 'e', '^'], ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['y', 'e', '^', 'u'], ExpectedSyllable::vowel(&[(V::Y, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['m', 'u', '+', 'a'], ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::UHorn, C::Lower), (V::A, C::Lower)], Tone::Flat)),
    case!(['h', 'u', '+', 'n', 'g'], ExpectedSyllable::full(Onset::H, &['h'], &[(V::UHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])),
    case!(['a', 'n', 'h', '`'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Grave, Coda::Nh, &['n', 'h'])),
    case!(['b', 'a', 'n', '?'], ExpectedSyllable::full(Onset::B, &['b'], &[(V::A, C::Lower)], Tone::Hook, Coda::N, &['n'])),
    case!(
        ['m', 'u', 'o', '+', 'i', '`'],
        ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(
        ['c', 'u', 'o', '+', 'i', '`'],
        ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)
    ),
    case!(['k', 'h', 'o', '^', 'n', 'g'], ExpectedSyllable::full(Onset::Kh, &['k', 'h'], &[(V::OCircumflex, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])),
    case!(['h', 'o', '^', 'n', 'g', '`'], ExpectedSyllable::full(Onset::H, &['h'], &[(V::OCircumflex, C::Lower)], Tone::Grave, Coda::Ng, &['n', 'g'])),
    case!(['s', 'o', '^', 'n', 'g', '\''], ExpectedSyllable::full(Onset::S, &['s'], &[(V::OCircumflex, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])),
    case!(['m', 'a', 'n', 'h', '.'], ExpectedSyllable::full(Onset::M, &['m'], &[(V::A, C::Lower)], Tone::Dot, Coda::Nh, &['n', 'h'])),
    case!(['x', 'a', 'n', 'h'], ExpectedSyllable::full(Onset::X, &['x'], &[(V::A, C::Lower)], Tone::Flat, Coda::Nh, &['n', 'h'])),
    case!(['n', 'h', 'a', '`'], ExpectedSyllable::onset_vowel(Onset::Nh, &['n', 'h'], &[(V::A, C::Lower)], Tone::Grave)),
    case!(['b', 'a', '`'], ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::A, C::Lower)], Tone::Grave)),
    case!(['a', '^', 'y', '\''], ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Acute)),
    case!(['d', 'a', '^', 'y'], ExpectedSyllable::onset_vowel(Onset::D, &['d'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)),
    case!(['m', 'a', '^', 'y'], ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)),
    case!(['c', 'a', '^', 'y'], ExpectedSyllable::onset_vowel(Onset::C, &['c'], &[(V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)),
    case!(
        ['k', 'h', 'u', 'y', 'a'],
        ExpectedSyllable::onset_vowel(Onset::Kh, &['k', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::A, C::Lower)], Tone::Flat)
    ),
    case!(
        ['c', 'h', 'u', 'y', 'e', '^', 'n', '.'],
        ExpectedSyllable::full(Onset::Ch, &['c', 'h'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Dot, Coda::N, &['n'])
    ),
    case!(
        ['n', 'g', 'u', 'y', 'e', '^', 'n'],
        ExpectedSyllable::full(Onset::Ng, &['n', 'g'], &[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    case!(['y', 'e', '^', 'u', '\''], ExpectedSyllable::vowel(&[(V::Y, C::Lower), (V::ECircumflex, C::Lower), (V::U, C::Lower)], Tone::Acute)),
    case!(['k', 'h', 'o', 'e'], ExpectedSyllable::onset_vowel(Onset::Kh, &['k', 'h'], &[(V::O, C::Lower), (V::E, C::Lower)], Tone::Flat)),
    case!(['q', 'u', 'y', '\''], ExpectedSyllable::onset_vowel(Onset::Qu, &['q', 'u'], &[(V::Y, C::Lower)], Tone::Acute)),
    case!(['l', 'a', '.'], ExpectedSyllable::onset_vowel(Onset::L, &['l'], &[(V::A, C::Lower)], Tone::Dot)),
    case!(['h', 'o', 'c', '.'], ExpectedSyllable::full(Onset::H, &['h'], &[(V::O, C::Lower)], Tone::Dot, Coda::C, &['c'])),
    case!(['t', 'h', 'e', '^', '\''], ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::ECircumflex, C::Lower)], Tone::Acute)),
    case!(['i', 'e', 'u'], ExpectedSyllable::vowel(&[(V::I, C::Lower), (V::E, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['o', 'a', 'i'], ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['o', 'a', 'u'], ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['o', 'e', 'o'], ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::E, C::Lower), (V::O, C::Lower)], Tone::Flat)),
    case!(['u', 'i'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['u', 'y', 'u'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::Y, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['u', 'y', 'a'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::Y, C::Lower), (V::A, C::Lower)], Tone::Flat)),
    case!(['u', 'a', '^'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::ACircumflex, C::Lower)], Tone::Flat)),
    case!(['u', 'a', '^', 'y'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::ACircumflex, C::Lower), (V::Y, C::Lower)], Tone::Flat)),
    case!(['u', 'y', 'e', '^'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::Y, C::Lower), (V::ECircumflex, C::Lower)], Tone::Flat)),
    case!(['o', 'a', '('], ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::ABreve, C::Lower)], Tone::Flat)),
    case!(['o', '^', 'i'], ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['o', '+', 'i'], ExpectedSyllable::vowel(&[(V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['u', '+', 'i'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
];
