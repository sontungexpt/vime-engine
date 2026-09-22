//! J. Invalid / dead cases: inputs that kill the parse.
//!
//! A dead case (`dead_case!`) is an input where some `push` returns `Err`; when
//! it does, the builder rolls back and keeps the syllable listed in the case.
//! The runner expands the case into: push everything in order, require a
//! failure, then compare the rolled-back syllable against the expected one.
//!
//! Notes vs. the old telex-on-composition corpus:
//!
//! * A non-letter typed *after* a vowel kills as `InvalidCoda`; a non-letter
//!   typed in the onset phase kills as `InvalidOnset`.
//! * `q` waits for a plain `u`: every other vowel after `q` kills as
//!   `InvalidOnset` (a precomposed `ư` is not a plain `u`).

use super::prelude::*;

/// Telex dead cases (run against the telex keymap).
pub const TELEX: &[Case] = &[
    dead_case!(['b', 'c', 'a'], ExpectedSyllable::consonant(Onset::B, &['b']),),
    dead_case!(['w', 'a'], ExpectedSyllable::vowel(&[], Tone::Flat),),
    dead_case!(['z', 'a'], ExpectedSyllable::vowel(&[], Tone::Flat),),
    dead_case!(['j', 'a'], ExpectedSyllable::vowel(&[], Tone::Flat),),
    dead_case!(['f', 'a'], ExpectedSyllable::vowel(&[], Tone::Flat),),
    dead_case!(['g', 'r', 'a'], ExpectedSyllable::consonant(Onset::G, &['g']),),
    dead_case!(['t', 'r', 'g', 'a'], ExpectedSyllable::consonant(Onset::Tr, &['t', 'r']),),
    dead_case!(['k', 'h', 'h', 'a'], ExpectedSyllable::consonant(Onset::Kh, &['k', 'h']),),
    dead_case!(['n', 'g', 'g', 'a'], ExpectedSyllable::consonant(Onset::Ng, &['n', 'g']),),
    dead_case!(['t', 'r', 'l', 'a'], ExpectedSyllable::consonant(Onset::Tr, &['t', 'r']),),
    dead_case!(['n', 'g', 'c', 'a'], ExpectedSyllable::consonant(Onset::Ng, &['n', 'g']),),
    dead_case!(['p', 'h', 'f', 'a'], ExpectedSyllable::consonant(Onset::Ph, &['p', 'h']),),
    dead_case!(['b', 'd', 'a'], ExpectedSyllable::consonant(Onset::B, &['b']),),
    dead_case!(['m', 'n', 'a'], ExpectedSyllable::consonant(Onset::M, &['m']),),
    dead_case!(['q', 'a'], ExpectedSyllable::consonant(Onset::None, &['q']),),
    dead_case!(['q', 'e'], ExpectedSyllable::consonant(Onset::None, &['q']),),
    dead_case!(['q', 'i'], ExpectedSyllable::consonant(Onset::None, &['q']),),
    dead_case!(['q', 'o'], ExpectedSyllable::consonant(Onset::None, &['q']),),
    dead_case!(['q', 'ư'], ExpectedSyllable::consonant(Onset::None, &['q']),),
    dead_case!(['Q', 'A'], ExpectedSyllable::consonant(Onset::None, &['Q']),),
    dead_case!(['Q', 'E'], ExpectedSyllable::consonant(Onset::None, &['Q']),),
    dead_case!(['?'], ExpectedSyllable::vowel(&[], Tone::Flat)),
    dead_case!(['!'], ExpectedSyllable::vowel(&[], Tone::Flat)),
    dead_case!(['#'], ExpectedSyllable::vowel(&[], Tone::Flat)),
    dead_case!(['c', 'h', '?'], ExpectedSyllable::consonant(Onset::Ch, &['c', 'h']),),
    dead_case!(['c', 'h', '0'], ExpectedSyllable::consonant(Onset::Ch, &['c', 'h']),),
    dead_case!(['a', 'i', 'u', 'n'], ExpectedSyllable::vowel(&[(V::A, C::Lower), (V::I, C::Lower)], Tone::Flat),),
    dead_case!(['a', 'o', 'i', 'u'], ExpectedSyllable::vowel(&[(V::A, C::Lower), (V::O, C::Lower)], Tone::Flat),),
    dead_case!(['b', 'a', 'o', 'i', 'u', 'n'], ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::A, C::Lower), (V::O, C::Lower)], Tone::Flat),),
    dead_case!(['a', 'k', 't'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['a', 'b', 'd'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['a', 'n', 'n'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::N, &['n']),),
    dead_case!(['c', 'h', 'a', 't', 'c'], ExpectedSyllable::full(Onset::Ch, &['c', 'h'], &[(V::A, C::Lower)], Tone::Flat, Coda::T, &['t']),),
    dead_case!(['a', 'm', 'h'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::M, &['m']),),
    dead_case!(['a', 'n', 'g', 'g'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g']),),
    dead_case!(['a', 't', 'c'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::T, &['t']),),
    dead_case!(['o', 'n', 'm'], ExpectedSyllable::vowel_coda(&[(V::O, C::Lower)], Tone::Flat, Coda::N, &['n']),),
    dead_case!(['a', 'p', 'h'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::P, &['p']),),
    dead_case!(['a', 'n', 'g', 'h'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g']),),
    dead_case!(['a', 't', 't'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::T, &['t']),),
    dead_case!(['a', 'k', 'k'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    // moved from `viqr`
    dead_case!(['a', '?'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['a', '$'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['c', 'h', 'a', '0'], ExpectedSyllable::onset_vowel(Onset::Ch, &['c', 'h'], &[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['a', 'n', 'o'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::N, &['n']),),
    dead_case!(['â', 'a'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['a', 'w', 'w'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['e', 'e', 'e'], ExpectedSyllable::vowel(&[(V::E, C::Lower)], Tone::Flat),),
    dead_case!(['u', 'w', 'w'], ExpectedSyllable::vowel(&[(V::U, C::Lower)], Tone::Flat),),
    dead_case!(['o', 'w', 'w'], ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Flat),),
    dead_case!(['u', 'o', 'w', 'w', 'w'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::O, C::Lower)], Tone::Flat),),
    dead_case!(['ư', 'ô'], ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Flat),),
    dead_case!(['u', 'o', 'w', 'e'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat),),
    dead_case!(['u', 'o', 'w', 'a'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat),),
    dead_case!(['ư', 'ơ', 'w'], ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::O, C::Lower)], Tone::Flat),),
    dead_case!(['ắ', 's'], ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Flat),),
    dead_case!(['ấ', 's'], ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Flat),),
    dead_case!(['ộ', 'j'], ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Flat),),
    dead_case!(['ắ', 'w'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute),),
    dead_case!(['ấ', 'a'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute),),
    dead_case!(['Ắ', 's'], ExpectedSyllable::vowel(&[(V::ABreve, C::Upper)], Tone::Flat),),
    dead_case!(['Ắ', 'w'], ExpectedSyllable::vowel(&[(V::A, C::Upper)], Tone::Acute),),
    dead_case!(['a', 's', 's'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['á', 's'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['à', 'f'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['ă', 'w'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['ê', 'e'], ExpectedSyllable::vowel(&[(V::E, C::Lower)], Tone::Flat),),
    dead_case!(['ơ', 'w'], ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Flat),),
    dead_case!(['ư', 'w'], ExpectedSyllable::vowel(&[(V::U, C::Lower)], Tone::Flat),),
    dead_case!(['ẻ', 'r'], ExpectedSyllable::vowel(&[(V::E, C::Lower)], Tone::Flat),),
    dead_case!(['õ', 'x'], ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Flat),),
    dead_case!(['ị', 'j'], ExpectedSyllable::vowel(&[(V::I, C::Lower)], Tone::Flat),),
    dead_case!(['d', 'd', 'd'], ExpectedSyllable::consonant(Onset::D, &['d']),),
    dead_case!(['D', 'D', 'D'], ExpectedSyllable::consonant(Onset::D, &['D']),),
    dead_case!(['o', 's', 's'], ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Flat),),
    dead_case!(['i', 'x', 'x'], ExpectedSyllable::vowel(&[(V::I, C::Lower)], Tone::Flat),),
    dead_case!(['u', 'r', 'r'], ExpectedSyllable::vowel(&[(V::U, C::Lower)], Tone::Flat),),
    dead_case!(['q', 'a', 'b', 'c'], ExpectedSyllable::consonant(Onset::None, &['q']),),
    dead_case!(['a', 't', 't', 't'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::T, &['t']),),
    dead_case!(['a', 'w', 'w', 'w'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['d', 'd', 'd', 'd'], ExpectedSyllable::consonant(Onset::D, &['d']),),
];

/// VNI dead cases (run against the VNI keymap).
pub const VNI: &[Case] = &[
    dead_case!(['1', 'a'], ExpectedSyllable::vowel(&[], Tone::Flat),),
    dead_case!(['9', 'a'], ExpectedSyllable::vowel(&[], Tone::Flat),),
    dead_case!(['6', 'a'], ExpectedSyllable::vowel(&[], Tone::Flat),),
    dead_case!(['7', 'a'], ExpectedSyllable::vowel(&[], Tone::Flat),),
    dead_case!(['a', '9'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['a', 'b', 'd'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['a', '0'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
    dead_case!(['b', 'c', 'a'], ExpectedSyllable::consonant(Onset::B, &['b']),),
    dead_case!(['w', 'a'], ExpectedSyllable::vowel(&[], Tone::Flat),),
    dead_case!(['d', '9', '9'], ExpectedSyllable::consonant(Onset::D, &['d']),),
    dead_case!(['a', 't', 't'], ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::T, &['t']),),
    dead_case!(['a', 'k', 'k'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),),
];
