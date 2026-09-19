//! J. Invalid / dead cases: inputs that leave the parse dead.
//!
//! Each entry spells out the full observable state a `Composition` must report
//! after the whole input was appended: the raw `input()` buffer (the row's own
//! characters), the semantic syllable kept at the moment the parse died, the
//! parse phase at death, and the expected `FallbackState` — the recorded
//! `SyllableParseIssue` and the exact `Accepted` / `Rejected` sequence frozen
//! in `FallbackState::buffer` (accepted syllable characters, then the killer
//! and any characters appended afterwards). The harness builds the expected
//! `FallbackState` from the row's `(issue, toneless)` data at runtime (`vec!`)
//! and compares it structurally with a single `assert_eq!`.
//!
//! Notes vs. the old corpus:
//!
//! * The old `InvalidCharacter` variant is gone. A non-letter typed *after* a
//!   vowel falls through `append_vowel_literal` and kills as `InvalidCoda`;
//!   the old "keep any non-letter as a literal custom onset" behaviour no
//!   longer exists, so symbols now die in the onset phase as `InvalidOnset`.
//! * `q` waits for a plain `u`: every other vowel after `q` kills as
//!   `InvalidOnset` (a precomposed `ư` is not a plain `u`).

use super::{dead_case, DeadCase, ExpectedSyllable};
use super::{C, V};
use vime_engine::composition::{CharStatus, ParseAppendingPhase, ParseError, ParseP
use vime_engine::phonology::{Coda, Onset, Tone};
pub fn telex_cases() -> Vec<DeadCase> {
    vec![
        dead_case!(
            ['b', 'c', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::B, &['b']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('b'), CharStatus::Rejected('c'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['w', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::vowel(&[], Tone::Flat),
            ParseError::InvalidOnset,
            &[CharStatus::Rejected('w'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['z', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::vowel(&[], Tone::Flat),
            ParseError::InvalidOnset,
            &[CharStatus::Rejected('z'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['j', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::vowel(&[], Tone::Flat),
            ParseError::InvalidOnset,
            &[CharStatus::Rejected('j'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['f', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::vowel(&[], Tone::Flat),
            ParseError::InvalidOnset,
            &[CharStatus::Rejected('f'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['g', 'r', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::G, &['g']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('g'), CharStatus::Rejected('r'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['t', 'r', 'g', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::Tr, &['t', 'r']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('t'), CharStatus::Accepted('r'), CharStatus::Rejected('g'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['k', 'h', 'h', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::Kh, &['k', 'h']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('k'), CharStatus::Accepted('h'), CharStatus::Rejected('h'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['n', 'g', 'g', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::Ng, &['n', 'g']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('n'), CharStatus::Accepted('g'), CharStatus::Rejected('g'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['t', 'r', 'l', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::Tr, &['t', 'r']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('t'), CharStatus::Accepted('r'), CharStatus::Rejected('l'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['n', 'g', 'c', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::Ng, &['n', 'g']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('n'), CharStatus::Accepted('g'), CharStatus::Rejected('c'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['p', 'h', 'f', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::Ph, &['p', 'h']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('p'), CharStatus::Accepted('h'), CharStatus::Rejected('f'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['b', 'd', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::B, &['b']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('b'), CharStatus::Rejected('d'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['m', 'n', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::M, &['m']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('m'), CharStatus::Rejected('n'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['q', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::None, &['q']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('q'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['q', 'e'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::None, &['q']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('q'), CharStatus::Rejected('e')]
        ),
        dead_case!(
            ['q', 'i'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::None, &['q']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('q'), CharStatus::Rejected('i')]
        ),
        dead_case!(
            ['q', 'o'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::None, &['q']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('q'), CharStatus::Rejected('o')]
        ),
        dead_case!(
            ['q', 'ư'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::None, &['q']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('q'), CharStatus::Rejected('ư')]
        ),
        dead_case!(
            ['Q', 'A'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::None, &['Q']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('Q'), CharStatus::Rejected('A')]
        ),
        dead_case!(
            ['Q', 'E'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::None, &['Q']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('Q'), CharStatus::Rejected('E')]
        ),
        dead_case!(['?'], ParseAppendingPhase::Onset, ExpectedSyllable::vowel(&[], Tone::Flat), ParseError::InvalidOnset, &[CharStatus::Rejected('?')]),
        dead_case!(['!'], ParseAppendingPhase::Onset, ExpectedSyllable::vowel(&[], Tone::Flat), ParseError::InvalidOnset, &[CharStatus::Rejected('!')]),
        dead_case!(['#'], ParseAppendingPhase::Onset, ExpectedSyllable::vowel(&[], Tone::Flat), ParseError::InvalidOnset, &[CharStatus::Rejected('#')]),
        dead_case!(
            ['c', 'h', '?'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::Ch, &['c', 'h']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('c'), CharStatus::Accepted('h'), CharStatus::Rejected('?')]
        ),
        dead_case!(
            ['c', 'h', '0'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::Ch, &['c', 'h']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('c'), CharStatus::Accepted('h'), CharStatus::Rejected('0')]
        ),
        dead_case!(
            ['a', 'i', 'u', 'n'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower), (V::I, C::Lower)], Tone::Flat),
            ParseError::InvalidNucleus,
            &[CharStatus::Accepted('a'), CharStatus::Accepted('i'), CharStatus::Rejected('u'), CharStatus::Rejected('n')]
        ),
        dead_case!(
            ['a', 'o', 'i', 'u'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower), (V::O, C::Lower)], Tone::Flat),
            ParseError::InvalidNucleus,
            &[CharStatus::Accepted('a'), CharStatus::Accepted('o'), CharStatus::Rejected('i'), CharStatus::Rejected('u')]
        ),
        dead_case!(
            ['b', 'a', 'o', 'i', 'u', 'n'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::A, C::Lower), (V::O, C::Lower)], Tone::Flat),
            ParseError::InvalidNucleus,
            &[
                CharStatus::Accepted('b'),
                CharStatus::Accepted('a'),
                CharStatus::Accepted('o'),
                CharStatus::Rejected('i'),
                CharStatus::Rejected('u'),
                CharStatus::Rejected('n')
            ]
        ),
        dead_case!(
            ['a', 'k', 't'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('k'), CharStatus::Rejected('t')]
        ),
        dead_case!(
            ['a', 'b', 'd'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('b'), CharStatus::Rejected('d')]
        ),
        dead_case!(
            ['a', 'n', 'n'],
            ParseAppendingPhase::Coda,
            ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::N, &['n']),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Accepted('n'), CharStatus::Rejected('n')]
        ),
        dead_case!(
            ['c', 'h', 'a', 't', 'c'],
            ParseAppendingPhase::Coda,
            ExpectedSyllable::syllable(Onset::Ch, &['c', 'h'], &[(V::A, C::Lower)], Tone::Flat, Coda::T, &['t']),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('c'), CharStatus::Accepted('h'), CharStatus::Accepted('a'), CharStatus::Accepted('t'), CharStatus::Rejected('c')]
        ),
        dead_case!(
            ['a', 'm', 'h'],
            ParseAppendingPhase::Coda,
            ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::M, &['m']),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Accepted('m'), CharStatus::Rejected('h')]
        ),
        dead_case!(
            ['a', 'n', 'g', 'g'],
            ParseAppendingPhase::Coda,
            ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g']),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Accepted('n'), CharStatus::Accepted('g'), CharStatus::Rejected('g')]
        ),
        dead_case!(
            ['a', 't', 'c'],
            ParseAppendingPhase::Coda,
            ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::T, &['t']),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Accepted('t'), CharStatus::Rejected('c')]
        ),
        dead_case!(
            ['o', 'n', 'm'],
            ParseAppendingPhase::Coda,
            ExpectedSyllable::vowel_coda(&[(V::O, C::Lower)], Tone::Flat, Coda::N, &['n']),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('o'), CharStatus::Accepted('n'), CharStatus::Rejected('m')]
        ),
        dead_case!(
            ['a', 'p', 'h'],
            ParseAppendingPhase::Coda,
            ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::P, &['p']),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Accepted('p'), CharStatus::Rejected('h')]
        ),
        dead_case!(
            ['a', 'n', 'g', 'h'],
            ParseAppendingPhase::Coda,
            ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g']),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Accepted('n'), CharStatus::Accepted('g'), CharStatus::Rejected('h')]
        ),
        dead_case!(
            ['a', 't', 't'],
            ParseAppendingPhase::Coda,
            ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::T, &['t']),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Accepted('t'), CharStatus::Rejected('t')]
        ),
        dead_case!(
            ['a', 'k', 'k'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('k'), CharStatus::Rejected('k')]
        ),
        // moved from `viqr`
        dead_case!(
            ['a', '?'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('?')]
        ),
        dead_case!(
            ['a', '$'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('$')]
        ),
        dead_case!(
            ['c', 'h', 'a', '0'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::onset_vowel(Onset::Ch, &['c', 'h'], &[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('c'), CharStatus::Accepted('h'), CharStatus::Accepted('a'), CharStatus::Rejected('0')]
        ),
        dead_case!(
            ['a', 'n', 'o'],
            ParseAppendingPhase::Coda,
            ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::N, &['n']),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Accepted('n'), CharStatus::Rejected('o')]
        ),
        dead_case!(
            ['â', 'a'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidNucleus,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['a', 'w', 'w'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('w')]
        ),
        dead_case!(
            ['e', 'e', 'e'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::E, C::Lower)], Tone::Flat),
            ParseError::InvalidNucleus,
            &[CharStatus::Accepted('e'), CharStatus::Rejected('e')]
        ),
        dead_case!(
            ['u', 'w', 'w'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::U, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('u'), CharStatus::Rejected('w')]
        ),
        dead_case!(
            ['o', 'w', 'w'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('o'), CharStatus::Rejected('w')]
        ),
        dead_case!(
            ['u', 'o', 'w', 'w', 'w'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::O, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('u'), CharStatus::Accepted('o'), CharStatus::Rejected('w')]
        ),
        dead_case!(
            ['ư', 'ô'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Flat),
            ParseError::InvalidNucleus,
            &[CharStatus::Accepted('ư'), CharStatus::Rejected('ô')]
        ),
        dead_case!(
            ['u', 'o', 'w', 'e'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat),
            ParseError::InvalidNucleus,
            &[CharStatus::Accepted('u'), CharStatus::Accepted('ơ'), CharStatus::Rejected('e')]
        ),
        dead_case!(
            ['u', 'o', 'w', 'a'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat),
            ParseError::InvalidNucleus,
            &[CharStatus::Accepted('u'), CharStatus::Accepted('ơ'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['ư', 'ơ', 'w'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::O, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('u'), CharStatus::Accepted('o'), CharStatus::Rejected('w')]
        ),
        dead_case!(
            ['ắ', 's'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('ă'), CharStatus::Rejected('s')]
        ),
        dead_case!(
            ['ấ', 's'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::ACircumflex, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('â'), CharStatus::Rejected('s')]
        ),
        dead_case!(
            ['ộ', 'j'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::OCircumflex, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('ô'), CharStatus::Rejected('j')]
        ),
        dead_case!(
            ['ắ', 'w'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('w')]
        ),
        dead_case!(
            ['ấ', 'a'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute),
            ParseError::InvalidNucleus,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['Ắ', 's'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::ABreve, C::Upper)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('Ă'), CharStatus::Rejected('s')]
        ),
        dead_case!(
            ['Ắ', 'w'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Upper)], Tone::Acute),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('A'), CharStatus::Rejected('w')]
        ),
        dead_case!(
            ['a', 's', 's'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('s')]
        ),
        dead_case!(
            ['á', 's'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('s')]
        ),
        dead_case!(
            ['à', 'f'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('f')]
        ),
        dead_case!(
            ['ă', 'w'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('w')]
        ),
        dead_case!(
            ['ê', 'e'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::E, C::Lower)], Tone::Flat),
            ParseError::InvalidNucleus,
            &[CharStatus::Accepted('e'), CharStatus::Rejected('e')]
        ),
        dead_case!(
            ['ơ', 'w'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('o'), CharStatus::Rejected('w')]
        ),
        dead_case!(
            ['ư', 'w'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::U, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('u'), CharStatus::Rejected('w')]
        ),
        dead_case!(
            ['ẻ', 'r'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::E, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('e'), CharStatus::Rejected('r')]
        ),
        dead_case!(
            ['õ', 'x'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('o'), CharStatus::Rejected('x')]
        ),
        dead_case!(
            ['ị', 'j'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::I, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('i'), CharStatus::Rejected('j')]
        ),
        dead_case!(
            ['d', 'd', 'd'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::D, &['d']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('d'), CharStatus::Rejected('d')]
        ),
        dead_case!(
            ['D', 'D', 'D'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::D, &['D']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('D'), CharStatus::Rejected('D')]
        ),
        dead_case!(
            ['o', 's', 's'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::O, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('o'), CharStatus::Rejected('s')]
        ),
        dead_case!(
            ['i', 'x', 'x'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::I, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('i'), CharStatus::Rejected('x')]
        ),
        dead_case!(
            ['u', 'r', 'r'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::U, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('u'), CharStatus::Rejected('r')]
        ),
        dead_case!(
            ['q', 'a', 'b', 'c'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::None, &['q']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('q'), CharStatus::Rejected('a'), CharStatus::Rejected('b'), CharStatus::Rejected('c')]
        ),
        dead_case!(
            ['a', 't', 't', 't'],
            ParseAppendingPhase::Coda,
            ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::T, &['t']),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Accepted('t'), CharStatus::Rejected('t'), CharStatus::Rejected('t')]
        ),
        dead_case!(
            ['a', 'w', 'w', 'w'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('w'), CharStatus::Rejected('w')]
        ),
        dead_case!(
            ['d', 'd', 'd', 'd'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::D, &['d']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('d'), CharStatus::Rejected('d'), CharStatus::Rejected('d')]
        ),
    ]
}

pub fn vni_cases() -> Vec<DeadCase> {
    vec![
        dead_case!(
            ['1', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::vowel(&[], Tone::Flat),
            ParseError::InvalidOnset,
            &[CharStatus::Rejected('1'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['9', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::vowel(&[], Tone::Flat),
            ParseError::InvalidOnset,
            &[CharStatus::Rejected('9'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['6', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::vowel(&[], Tone::Flat),
            ParseError::InvalidOnset,
            &[CharStatus::Rejected('6'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['7', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::vowel(&[], Tone::Flat),
            ParseError::InvalidOnset,
            &[CharStatus::Rejected('7'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['a', '9'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('9')]
        ),
        dead_case!(
            ['a', 'b', 'd'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('b'), CharStatus::Rejected('d')]
        ),
        dead_case!(
            ['a', '0'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('0')]
        ),
        dead_case!(
            ['b', 'c', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::B, &['b']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('b'), CharStatus::Rejected('c'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['w', 'a'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::vowel(&[], Tone::Flat),
            ParseError::InvalidOnset,
            &[CharStatus::Rejected('w'), CharStatus::Rejected('a')]
        ),
        dead_case!(
            ['d', '9', '9'],
            ParseAppendingPhase::Onset,
            ExpectedSyllable::consonant(Onset::D, &['d']),
            ParseError::InvalidOnset,
            &[CharStatus::Accepted('d'), CharStatus::Rejected('9')]
        ),
        dead_case!(
            ['a', 't', 't'],
            ParseAppendingPhase::Coda,
            ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::T, &['t']),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Accepted('t'), CharStatus::Rejected('t')]
        ),
        dead_case!(
            ['a', 'k', 'k'],
            ParseAppendingPhase::Vowel,
            ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat),
            ParseError::InvalidCoda,
            &[CharStatus::Accepted('a'), CharStatus::Rejected('k'), CharStatus::Rejected('k')]
        ),
    ]
}
