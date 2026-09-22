//! E. The special `uo` / `ươ` cycles.

use super::prelude::*;

pub const CASES: &[Case] = &[
    case!(['u', 'o'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::O, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'w'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'w', 'w'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'o'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Flat)),
    case!(['ư', 'o'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::O, C::Lower)], Tone::Flat)),
    case!(['ư', 'ơ'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'ơ'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'o', 'i'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'o', 'i', 's'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Acute)),
    case!(['u', 'o', 'o', 'i', 'r'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Hook)),
    case!(['u', 'o', 'o', 'i', 'x'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Tilde)),
    case!(['u', 'o', 'o', 'i', 'j'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Dot)),
    case!(['u', 'o', 'w', 'i'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'w', 'i', 's'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Acute)),
    case!(['u', 'o', 'w', 'i', 'f'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Grave)),
    case!(['u', 'o', 'w', 'i', 'r'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Hook)),
    case!(['u', 'o', 'w', 'i', 'x'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Tilde)),
    case!(['u', 'o', 'w', 'u'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::U, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'w', 'u', 'j'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::U, C::Lower)], Tone::Dot)),
    case!(['u', 'o', 'w', 'w', 'i'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'o', 'n', 'g'], ExpectedSyllable::vowel_coda(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])),
    case!(
        ['u', 'o', 'o', 'n', 'g', 's'],
        ExpectedSyllable::vowel_coda(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Acute, Coda::Ng, &['n', 'g'])
    ),
    case!(
        ['m', 'u', 'o', 'o', 'n', 's'],
        ExpectedSyllable::full(Onset::M, &['m'], &[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    case!(['u', 'o', 'w', 'o', 'n', 'g'], ExpectedSyllable::vowel_coda(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])),
    case!(['u', 'o', 'w', 's'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Acute)),
    case!(['u', 'o', 'w', 'f'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Grave)),
    case!(
        ['t', 'h', 'u', 'o', 'w', 'n', 'g'],
        ExpectedSyllable::full(Onset::Th, &['t', 'h'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    case!(['u', 'o', 'w', 'c', 's'], ExpectedSyllable::vowel_coda(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Acute, Coda::C, &['c'])),
    case!(
        ['t', 'u', 'o', 'w', 'c', 's'],
        ExpectedSyllable::full(Onset::T, &['t'], &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(
        ['t', 'h', 'i', 'e', 'e', 'c', 's'],
        ExpectedSyllable::full(Onset::Th, &['t', 'h'], &[(V::I, C::Lower), (V::ECircumflex, C::Lower)], Tone::Acute, Coda::C, &['c'])
    ),
    case!(['u', 'o', 'w', 'o'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Flat)),
    case!(['u', 'o', 'w', 's', 'i'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Acute)),
    case!(['u', 'o', 'o', 's', 'i'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OCircumflex, C::Lower), (V::I, C::Lower)], Tone::Acute)),
    case!(
        ['u', 'o', 'o', 'n', 'g', 'x'],
        ExpectedSyllable::vowel_coda(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Tilde, Coda::Ng, &['n', 'g'])
    ),
    case!(['u', 'o', 'o', 'n', 'g', 'j'], ExpectedSyllable::vowel_coda(&[(V::U, C::Lower), (V::OCircumflex, C::Lower)], Tone::Dot, Coda::Ng, &['n', 'g'])),
    case!(['u', 'o', 'w', 'n', 'g'], ExpectedSyllable::vowel_coda(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])),
    case!(['u', 'o', 'w', 't', 'j'], ExpectedSyllable::vowel_coda(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Dot, Coda::T, &['t'])),
    case!(['u', 'o', 'w', 'c', 'f'], ExpectedSyllable::vowel_coda(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Grave, Coda::C, &['c'])),
];
