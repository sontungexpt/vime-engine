//! Insert-path corpus: `BuildingSyllableBuilder::insert` at an explicit cursor.
//!
//! The pipeline under test is:
//!
//! ```text
//! Keymap + (base pushes) → BuildingSyllableBuilder → insert(keymap, at, key) → syllable
//! ```
//!
//! The insertion index is absolute over the concatenation
//! `onset ++ vowels ++ coda`:
//!
//! * `at >= len` delegates to `push` (append).
//! * `at <= onset.len()` edits the onset, then falls back to the nucleus at the
//!   onset boundary.
//! * `at <= onset.len() + vowels.len()` edits the nucleus (cursor-bound
//!   transforms apply to vowels strictly *left* of the cursor).
//! * otherwise the cursor sits in the coda.
//!
//! Every case is a `base` push order plus one `(at, key)` insert, checked for
//! the returned `InputEffect`/`SyllableError` **and** the final syllable
//! (transform side-effects that survive an `Err` are asserted exactly).

use super::common::{check_syllable_eq, ExpectedSyllable, C, V};

use crate::composition::syllable::building::{BuildingSyllableBuilder, SyllableError};
use crate::composition::syllable::InputEffect;
use crate::keymap::DefaultKeymap;
use crate::phonology::{Coda, Onset, Tone};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Effect {
    Ok(InputEffect),
    Err(SyllableError),
}

struct InsertCase {
    base: &'static [char],
    at: usize,
    key: char,
    effect: Effect,
    expected: ExpectedSyllable,
}

macro_rules! ok_case {
    ($base:expr, $at:expr, $key:expr, $eff:ident, $expected:expr) => {
        InsertCase {
            base: $base,
            at: $at,
            key: $key,
            effect: Effect::Ok(InputEffect::$eff),
            expected: $expected,
        }
    };
}

macro_rules! err_case {
    ($base:expr, $at:expr, $key:expr, $err:ident, $expected:expr) => {
        InsertCase {
            base: $base,
            at: $at,
            key: $key,
            effect: Effect::Err(SyllableError::$err),
            expected: $expected,
        }
    };
}

const CASES: &[InsertCase] = &[
    // ─────────────────────── Append (at >= len == push) ───────────────────────
    ok_case!(&['a'], 1, 's', Transformed, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute)),
    ok_case!(&['a'], 1, 'n', StructurallyChanged, ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::N, &['n'])),
    ok_case!(&['a'], 1, 'i', StructurallyChanged, ExpectedSyllable::vowel(&[(V::A, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    ok_case!(&['a', 'n'], 2, 'g', StructurallyChanged, ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])),
    ok_case!(&['o', 'a'], 2, 'i', StructurallyChanged, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    ok_case!(
        &['u', 'ơ'],
        2,
        'c',
        StructurallyChanged,
        ExpectedSyllable::vowel_coda(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat, Coda::C, &['c'])
    ),
    ok_case!(&[], 0, 'a', StructurallyChanged, ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat)),
    // ─────────────────────────────── Onset region ───────────────────────────────
    ok_case!(&['t', 'a'], 1, 'h', StructurallyChanged, ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::A, C::Lower)], Tone::Flat)),
    ok_case!(&['n', 'g', 'a'], 2, 'h', StructurallyChanged, ExpectedSyllable::onset_vowel(Onset::Ngh, &['n', 'g', 'h'], &[(V::A, C::Lower)], Tone::Flat)),
    ok_case!(&['t', 'a'], 1, 'o', StructurallyChanged, ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::O, C::Lower), (V::A, C::Lower)], Tone::Flat)),
    ok_case!(&['a'], 0, 't', StructurallyChanged, ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::A, C::Lower)], Tone::Flat)),
    // ─────────────────────────────── Vowel region ───────────────────────────────
    ok_case!(&['o', 'i'], 1, 'a', StructurallyChanged, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    ok_case!(&['i'], 1, 'e', StructurallyChanged, ExpectedSyllable::vowel(&[(V::I, C::Lower), (V::E, C::Lower)], Tone::Flat)),
    ok_case!(&['o', 'i'], 1, 'w', Transformed, ExpectedSyllable::vowel(&[(V::OHorn, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    ok_case!(&['o', 'i'], 1, 's', Transformed, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::I, C::Lower)], Tone::Acute)),
    ok_case!(&['u', 'o'], 1, 'w', Transformed, ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)),
    // ─────────────────────────────── Error / dead ───────────────────────────────
    err_case!(&['t', 'a'], 0, 'h', InvalidOnset, ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::A, C::Lower)], Tone::Flat)),
    err_case!(&['t', 'a'], 1, 's', InvalidOnset, ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::A, C::Lower)], Tone::Flat)),
    err_case!(&['o', 'a'], 1, 'i', InvalidNucleus, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower)], Tone::Flat)),
    err_case!(&['o', 'a', 'i'], 2, 'u', InvalidNucleus, ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower), (V::I, C::Lower)], Tone::Flat)),
    err_case!(&['a', 'n', 'g'], 2, 'h', InvalidCoda, ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])),
    err_case!(&['a', 'c'], 1, 'h', InvalidCoda, ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::C, &['c'])),
    // A reverted d-stroke survives the failed literal insert: đa + d@0 → da.
    err_case!(&['đ', 'a'], 0, 'd', InvalidOnset, ExpectedSyllable::onset_vowel(Onset::D, &['d'], &[(V::A, C::Lower)], Tone::Flat)),
];

fn run_case(keymap: &DefaultKeymap, case: &InsertCase) -> Result<(), String> {
    let mut builder = BuildingSyllableBuilder::default();

    for &ch in case.base {
        builder.push(keymap, ch).map_err(|e| format!("base push({ch:?}) failed: {e:?}"))?;
    }

    let effect = builder.insert(keymap, case.at, case.key);
    let label = format!("base={:?} insert(at={}, {:?})", case.base, case.at, case.key);

    match (&case.effect, effect) {
        (Effect::Ok(expected), Ok(got)) if expected == &got => {}
        (Effect::Err(expected), Err(got)) if expected == &got => {}
        (Effect::Ok(expected), got) => {
            return Err(format!("{label}: expected Ok({expected:?}), got {got:?}"));
        }
        (Effect::Err(expected), got) => {
            return Err(format!("{label}: expected Err({expected:?}), got {got:?}"));
        }
    }

    check_syllable_eq(&builder, &case.expected, case.base)
}

#[test]
fn insert_cases() {
    let telex = DefaultKeymap::telex();

    let failures: Vec<String> = CASES.iter().filter_map(|case| run_case(&telex, case).err()).collect();

    if !failures.is_empty() {
        panic!("{} failing insert case(s):\n\n{}", failures.len(), failures.join("\n\n"));
    }

    assert!(CASES.len() >= 21, "expected the insert corpus to stay sizable; got {}", CASES.len());
}
