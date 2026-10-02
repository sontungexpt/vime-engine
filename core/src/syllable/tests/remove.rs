//! Remove corpus: `BuildingSyllable::remove` at an explicit cursor after a
//! `base` push order, asserted for the effect/error and the final syllable.

use super::common::{check_building_effect, check_syllable_eq, BuildingEffect, ExpectedSyllable, C, V};

use crate::keymap::DefaultKeymap;
use crate::phonology::{Coda, Onset, Tone, TonePlacement};
use crate::syllable::building::{BuildingSyllable, SyllableBuildError, TransformEffect, TransformTarget};

struct RemoveCase {
    base: &'static [char],
    at: usize,
    effect: super::common::BuildingEffect,
    expected: super::common::ExpectedSyllable,
}

macro_rules! ok_case {
    ($base:expr, $at:expr, Changed, $expected:expr) => {
        RemoveCase {
            base: $base,
            at: $at,
            effect: super::common::BuildingEffect::Ok(TransformEffect::None),
            expected: $expected,
        }
    };
    ($base:expr, $at:expr, Transformed($target:expr), $expected:expr) => {
        RemoveCase {
            base: $base,
            at: $at,
            effect: super::common::BuildingEffect::Ok(TransformEffect::Applied($target)),
            expected: $expected,
        }
    };
}

macro_rules! err_case {
    ($base:expr, $at:expr, $err:ident, $expected:expr) => {
        RemoveCase {
            base: $base,
            at: $at,
            effect: super::common::BuildingEffect::Err(SyllableBuildError::$err),
            expected: $expected,
        }
    };
}

const CASES: &[RemoveCase] = &[
    ok_case!(
        &['t', 'a', 'n'],
        0,
        Changed,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    ok_case!(
        &['t', 'h', 'a', 'n'],
        1,
        Changed,
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
        Changed,
        ExpectedSyllable::full(
            Onset::H,
            &['h'],
            &[(V::A, C::Lower)],
            Tone::Flat,
            Coda::N,
            &['n']
        )
    ),
    ok_case!(
        &['n', 'g', 'h', 'i', 'a'],
        0,
        Changed,
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
        Changed,
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
        Changed,
        ExpectedSyllable::onset_vowel(
            Onset::Ng,
            &['n', 'g'],
            &[(V::I, C::Lower), (V::A, C::Lower)],
            Tone::Flat
        )
    ),
    ok_case!(
        &['g', 'i', 'a', 'n'],
        0,
        Changed,
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
        Changed,
        ExpectedSyllable::vowel(&[(V::I, C::Lower), (V::A, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['g', 'i', 'a', 'n', 's'],
        0,
        Changed,
        ExpectedSyllable::vowel_coda(
            &[(V::I, C::Lower), (V::A, C::Lower)],
            Tone::Acute,
            Coda::N,
            &['n']
        )
    ),
    ok_case!(
        &['g', 'i', 'a', 'n'],
        1,
        Changed,
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
        Changed,
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
        Changed,
        ExpectedSyllable::consonant(Onset::G, &['g'])
    ),
    err_case!(
        &['g', 'i', 'o', 'a'],
        0,
        InvalidNucleus,
        ExpectedSyllable::onset_vowel(
            Onset::Gi,
            &['g', 'i'],
            &[(V::O, C::Lower), (V::A, C::Lower)],
            Tone::Flat
        )
    ),
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
        ExpectedSyllable::onset_vowel(
            Onset::Gi,
            &['g', 'i'],
            &[(V::O, C::Lower), (V::I, C::Lower)],
            Tone::Flat
        )
    ),
    ok_case!(
        &['o', 'a', 'i'],
        1,
        Changed,
        ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::I, C::Lower)], Tone::Flat)
    ),
    err_case!(
        &['t', 'a', 'n'],
        1,
        InvalidNucleus,
        ExpectedSyllable::full(
            Onset::T,
            &['t'],
            &[(V::A, C::Lower)],
            Tone::Flat,
            Coda::N,
            &['n']
        )
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
    ok_case!(
        &['t', 'a'],
        1,
        Changed,
        ExpectedSyllable::consonant(Onset::T, &['t'])
    ),
    ok_case!(
        &['o', 'a', 'n', 's'],
        1,
        Changed,
        ExpectedSyllable::vowel_coda(&[(V::O, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    ok_case!(
        &['o', 'a', 'n', 's'],
        0,
        Changed,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Acute, Coda::N, &['n'])
    ),
    ok_case!(
        &['t', 'a', 'n'],
        2,
        Changed,
        ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::A, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['a', 'c', 'h'],
        2,
        Changed,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::C, &['c'])
    ),
    ok_case!(
        &['o', 'a', 'n', 'g'],
        3,
        Changed,
        ExpectedSyllable::vowel_coda(
            &[(V::O, C::Lower), (V::A, C::Lower)],
            Tone::Flat,
            Coda::N,
            &['n']
        )
    ),
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

fn run_case(case: &RemoveCase) -> Result<(), String> {
    let mut builder = BuildingSyllable::default();

    for &ch in case.base {
        builder
            .push(&DefaultKeymap::telex(), ch)
            .map_err(|e| format!("base push({ch:?}) failed: {e:?}"))?;
    }

    let effect_result = builder.remove(case.at, TonePlacement::Modern);
    let label = format!(
        "base={:?} remove at {:?}",
        case.base, case.at
    );

    // Convert Result<(), E> to Result<TransformEffect, E> for check_building_effect
    let effect_for_check = effect_result.map(|_| TransformEffect::None);
    super::common::check_building_effect(&case.effect, effect_for_check, &label)?;
    check_syllable_eq(&builder, &case.expected, &case.base)
}

#[test]
fn remove_corpus() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|case| run_case(case).err())
        .collect();

    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
