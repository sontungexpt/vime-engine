//! Remove-path corpus: `BuildingSyllableBuilder::remove` at an explicit index.
//!
//! The pipeline under test is:
//!
//! ```text
//! Keymap + (base pushes) → BuildingSyllableBuilder → remove(at) → syllable
//! ```
//!
//! The removal index is absolute over the concatenation
//! `onset ++ vowels ++ coda`:
//!
//! * `at < onset.len()` deletes from the onset (`gi` degrades the `i` into a
//!   vowel; every other legal onset cluster is down-closed, so generic onset
//!   removals always succeed).
//! * `at < onset.len() + vowels.len()` deletes a vowel, clearing the tone when
//!   it targeted that exact vowel. A single vowel under a coda cannot go (an
//!   empty nucleus is only legal coda-less), so that edit fails.
//! * otherwise the cursor deletes from the coda.
//!
//! Every case asserts the returned `InputEffect`/`SyllableBuildError` **and**
//! the final syllable — a rejected removal must leave the base untouched.

use super::common::{check_syllable_eq, ExpectedSyllable, C, V};

use crate::composition::syllable::building::{BuildingSyllable, SyllableBuildError};
use crate::composition::syllable::InputEffect;
use crate::keymap::DefaultKeymap;
use crate::phonology::{Coda, Onset, Tone, TonePlacement};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Effect {
    Ok(InputEffect),
    Err(SyllableBuildError),
}

struct RemoveCase {
    base: &'static [char],
    at: usize,
    effect: Effect,
    expected: ExpectedSyllable,
}

macro_rules! ok_case {
    ($base:expr, $at:expr, $eff:ident, $expected:expr) => {
        RemoveCase {
            base: $base,
            at: $at,
            effect: Effect::Ok(InputEffect::$eff),
            expected: $expected,
        }
    };
}

macro_rules! err_case {
    ($base:expr, $at:expr, $err:ident, $expected:expr) => {
        RemoveCase {
            base: $base,
            at: $at,
            effect: Effect::Err(SyllableBuildError::$err),
            expected: $expected,
        }
    };
}

const CASES: &[RemoveCase] = &[
    // ─────────────────────────────── Onset region ───────────────────────────────
    // `tan` minus `t` → `an`.
    ok_case!(
        &['t', 'a', 'n'],
        0,
        StructurallyChanged,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    // `than` minus `h` → `tan`; minus `t` → `han` (a lone `h` is a legal onset).
    ok_case!(
        &['t', 'h', 'a', 'n'],
        1,
        StructurallyChanged,
        ExpectedSyllable::full(
            Onset::T,
            &['t'],
            &[(V::A, C::Lower)],
            Tone::Flat,
            Coda::N,
            &['n']
        )
    ),
    ok_case!(
        &['t', 'h', 'a', 'n'],
        0,
        StructurallyChanged,
        ExpectedSyllable::full(
            Onset::H,
            &['h'],
            &[(V::A, C::Lower)],
            Tone::Flat,
            Coda::N,
            &['n']
        )
    ),
    // Every member of the `ngh` cluster can go: the rest stays a valid onset.
    ok_case!(
        &['n', 'g', 'h', 'i', 'a'],
        0,
        StructurallyChanged,
        ExpectedSyllable::onset_vowel(
            Onset::Gh,
            &['g', 'h'],
            &[(V::I, C::Lower), (V::A, C::Lower)],
            Tone::Flat
        )
    ),
    ok_case!(
        &['n', 'g', 'h', 'i', 'a'],
        1,
        StructurallyChanged,
        ExpectedSyllable::onset_vowel(
            Onset::Nh,
            &['n', 'h'],
            &[(V::I, C::Lower), (V::A, C::Lower)],
            Tone::Flat
        )
    ),
    ok_case!(
        &['n', 'g', 'h', 'i', 'a'],
        2,
        StructurallyChanged,
        ExpectedSyllable::onset_vowel(
            Onset::Ng,
            &['n', 'g'],
            &[(V::I, C::Lower), (V::A, C::Lower)],
            Tone::Flat
        )
    ),
    // ────────────────── `gi` cluster: `i` becomes a vowel ──────────────────
    // Removing `g` pushes the lone `i` onto the nucleus head: `gian` → `ian`.
    ok_case!(
        &['g', 'i', 'a', 'n'],
        0,
        StructurallyChanged,
        ExpectedSyllable::vowel_coda(
            &[(V::I, C::Lower), (V::A, C::Lower)],
            Tone::Flat,
            Coda::N,
            &['n']
        )
    ),
    ok_case!(
        &['g', 'i', 'a'],
        0,
        StructurallyChanged,
        ExpectedSyllable::vowel(&[(V::I, C::Lower), (V::A, C::Lower)], Tone::Flat)
    ),
    // The tone riding on the resulting `i` survives the demotion.
    ok_case!(
        &['g', 'i', 'a', 'n', 's'],
        0,
        StructurallyChanged,
        ExpectedSyllable::vowel_coda(
            &[(V::I, C::Lower), (V::A, C::Lower)],
            Tone::Acute,
            Coda::N,
            &['n']
        )
    ),
    // Removing the `i` instead leaves a plain `g` onset.
    ok_case!(
        &['g', 'i', 'a', 'n'],
        1,
        StructurallyChanged,
        ExpectedSyllable::full(
            Onset::G,
            &['g'],
            &[(V::A, C::Lower)],
            Tone::Flat,
            Coda::N,
            &['n']
        )
    ),
    ok_case!(
        &['g', 'i', 'a', 'n', 's'],
        1,
        StructurallyChanged,
        ExpectedSyllable::full(
            Onset::G,
            &['g'],
            &[(V::A, C::Lower)],
            Tone::Acute,
            Coda::N,
            &['n']
        )
    ),
    ok_case!(
        &['g', 'i'],
        1,
        StructurallyChanged,
        ExpectedSyllable::consonant(Onset::G, &['g'])
    ),
    // ─────────────────── `gi` removal that breaks the nucleus ───────────────────
    // `i` + a dead tail under a `gi` onset: `giao` minus `g` needs `i` in a
    // nucleus that cannot host `io`, so the whole removal is rolled back.
    err_case!(
        &['g', 'i', 'o', 'a'],
        0,
        InvalidNucleus,
        ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::O, C::Lower), (V::A, C::Lower)], Tone::Flat)
    ),
    // Same rejection when the nucleus is already full: `gioai` minus `g` has
    // nowhere to put the `i`, and must not overflow the vowel buffer.
    err_case!(
        &['g', 'i', 'o', 'a', 'i'],
        0,
        InvalidNucleus,
        ExpectedSyllable::onset_vowel(
            Onset::Gi,
            &['g', 'i'],
            &[(V::O, C::Lower), (V::A, C::Lower), (V::I, C::Lower)],
            Tone::Flat
        )
    ),
    err_case!(
        &['g', 'i', 'o', 'i'],
        0,
        InvalidNucleus,
        ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::O, C::Lower), (V::I, C::Lower)], Tone::Flat)
    ),
    // ─────────────────────────────── Vowel region ───────────────────────────────
    // `oai` minus the middle vowel → `oi`.
    ok_case!(
        &['o', 'a', 'i'],
        1,
        StructurallyChanged,
        ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::I, C::Lower)], Tone::Flat)
    ),
    // A single vowel under a coda is undeletable: the nucleus may not go empty.
    err_case!(
        &['t', 'a', 'n'],
        1,
        InvalidNucleus,
        ExpectedSyllable::full(Onset::T, &['t'], &[(V::A, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    err_case!(
        &['t', 'h', 'a', 'n'],
        2,
        InvalidNucleus,
        ExpectedSyllable::full(
            Onset::Th,
            &['t', 'h'],
            &[(V::A, C::Lower)],
            Tone::Flat,
            Coda::N,
            &['n']
        )
    ),
    err_case!(
        &['a', 'c', 'h'],
        0,
        InvalidNucleus,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Ch, &['c', 'h'])
    ),
    // Coda-less single vowels can go, leaving a bare onset: `ta` → `t`.
    ok_case!(
        &['t', 'a'],
        1,
        StructurallyChanged,
        ExpectedSyllable::consonant(Onset::T, &['t'])
    ),
    // Removing the vowel the tone sits on drops the tone; removing a different
    // vowel keeps it. For `oán` + a coda, the tone tracks the second vowel.
    ok_case!(
        &['o', 'a', 'n', 's'],
        1,
        StructurallyChanged,
        ExpectedSyllable::vowel_coda(&[(V::O, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    ok_case!(
        &['o', 'a', 'n', 's'],
        0,
        StructurallyChanged,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    // ─────────────────────────────── Coda region ───────────────────────────────
    // `tan` minus `n` → `ta`; `ach` minus `h` → `ac`; `oang` minus `g` → `oan`.
    ok_case!(
        &['t', 'a', 'n'],
        2,
        StructurallyChanged,
        ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::A, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['a', 'c', 'h'],
        2,
        StructurallyChanged,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::C, &['c'])
    ),
    ok_case!(
        &['o', 'a', 'n', 'g'],
        3,
        StructurallyChanged,
        ExpectedSyllable::vowel_coda(&[(V::O, C::Lower), (V::A, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    // Removing the head of a cluster leaves `h` / `g`, which a coda cannot be.
    err_case!(
        &['a', 'c', 'h'],
        1,
        InvalidCoda,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Ch, &['c', 'h'])
    ),
    err_case!(
        &['o', 'a', 'n', 'g'],
        2,
        InvalidCoda,
        ExpectedSyllable::vowel_coda(
            &[(V::O, C::Lower), (V::A, C::Lower)],
            Tone::Flat,
            Coda::Ng,
            &['n', 'g']
        )
    ),
];

fn run_case(keymap: &DefaultKeymap, case: &RemoveCase) -> Result<(), String> {
    let mut builder = BuildingSyllable::default();

    for &ch in case.base {
        builder
            .push(keymap, ch)
            .map_err(|e| format!("base push({ch:?}) failed: {e:?}"))?;
    }

    let effect = builder.remove(case.at, TonePlacement::Modern);
    let label = format!("base={:?} remove(at={})", case.base, case.at);

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
fn remove_cases() {
    let telex = DefaultKeymap::telex();

    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|case| run_case(&telex, case).err())
        .collect();

    if !failures.is_empty() {
        panic!(
            "{} failing remove case(s):\n\n{}",
            failures.len(),
            failures.join("\n\n")
        );
    }

    assert!(
        CASES.len() >= 24,
        "expected the remove corpus to stay sizable; got {}",
        CASES.len()
    );
}

/// A realistic backspace sequence: deleting the coda, then demoting `gi`, then
/// clearing an empty onset char, walks `gian` down to `ia` step by step.
#[test]
fn remove_sequence() {
    let telex = DefaultKeymap::telex();
    let mut builder = BuildingSyllable::default();

    for &ch in &['g', 'i', 'a', 'n'] {
        builder.push(&telex, ch).unwrap();
    }

    assert_eq!(builder.remove(3, TonePlacement::Modern), Ok(InputEffect::StructurallyChanged));
    check_syllable_eq(
        &builder,
        &ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::A, C::Lower)], Tone::Flat),
        &['g', 'i', 'a'],
    )
    .unwrap();

    assert_eq!(builder.remove(0, TonePlacement::Modern), Ok(InputEffect::StructurallyChanged));
    check_syllable_eq(
        &builder,
        &ExpectedSyllable::vowel(&[(V::I, C::Lower), (V::A, C::Lower)], Tone::Flat),
        &['i', 'a'],
    )
    .unwrap();
}