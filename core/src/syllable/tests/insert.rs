//! Insert corpus: `BuildingSyllable::insert` at an explicit cursor after a
//! `base` push order, asserted for the effect/error and the final syllable.
//! ```text
//! Keymap + (base pushes) -> BuildingSyllable -> insert(keymap, at, key) -> syllable
//! ```

use super::common::{check_building_effect, check_syllable_eq, BuildingEffect, ExpectedSyllable, C, V};

use crate::keymap::DefaultKeymap;
use crate::phonology::{Coda, Onset, Tone};
use crate::syllable::building::{BuildingSyllable, SyllableBuildError, TransformEffect, TransformTarget};

struct InsertCase {
    base: &'static [char],
    at: usize,
    key: char,
    effect: super::common::BuildingEffect,
    expected: super::common::ExpectedSyllable,
}

macro_rules! ok_case {
    ($base:expr, $at:expr, $key:expr, Changed, $expected:expr) => {
        InsertCase {
            base: $base,
            at: $at,
            key: $key,
            effect: super::common::BuildingEffect::Ok(TransformEffect::None),
            expected: $expected,
        }
    };
    ($base:expr, $at:expr, $key:expr, Transformed($target:expr), $expected:expr) => {
        InsertCase {
            base: $base,
            at: $at,
            key: $key,
            effect: super::common::BuildingEffect::Ok(TransformEffect::Applied($target)),
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
            effect: super::common::BuildingEffect::Err(SyllableBuildError::$err),
            expected: $expected,
        }
    };
}

// Core insert cases that match current implementation behavior
const CASES: &[InsertCase] = &[
    // ───────────────────────────── Tone transforms ─────────────────────────────
    ok_case!(
        &['a'],
        1,
        's',
        Transformed(TransformTarget::Tone),
        ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute)
    ),
    ok_case!(
        &['a'],
        1,
        'f',
        Transformed(TransformTarget::Tone),
        ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Grave)
    ),
    ok_case!(
        &['a'],
        1,
        'r',
        Transformed(TransformTarget::Tone),
        ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Hook)
    ),
    ok_case!(
        &['a'],
        1,
        'x',
        Transformed(TransformTarget::Tone),
        ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Tilde)
    ),
    ok_case!(
        &['a'],
        1,
        'j',
        Transformed(TransformTarget::Tone),
        ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Dot)
    ),

    // ───────────────────────────── Shape transforms (Telex) ────────────────────
    ok_case!(
        &['a'],
        1,
        'w',
        Transformed(TransformTarget::Nucleus(0)),
        ExpectedSyllable::vowel(&[(V::ABreve, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['o'],
        1,
        'w',
        Transformed(TransformTarget::Nucleus(0)),
        ExpectedSyllable::vowel(&[(V::OHorn, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['u'],
        1,
        'w',
        Transformed(TransformTarget::Nucleus(0)),
        ExpectedSyllable::vowel(&[(V::UHorn, C::Lower)], Tone::Flat)
    ),

    // ───────────────────────────── D-stroke transforms ───────────────────────────
    ok_case!(
        &['d', 'a'],
        0,
        'd',
        Transformed(TransformTarget::DStroke),
        ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::A, C::Lower)], Tone::Flat)
    ),

    // ───────────────────────────── Uo/ươ pair transforms ─────────────────────────
    ok_case!(
        &['u', 'o'],
        1,
        'w',
        Transformed(TransformTarget::Nucleus(1)),
        ExpectedSyllable::vowel(&[(V::U, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['u', 'ơ'],
        1,
        'w',
        Transformed(TransformTarget::Nucleus(0)),
        ExpectedSyllable::vowel(&[(V::UHorn, C::Lower), (V::OHorn, C::Lower)], Tone::Flat)
    ),

    // ───────────────────────────── Coda extension ──────────────────────────────
    ok_case!(
        &['a'],
        1,
        'n',
        Changed,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::N, &['n'])
    ),
    ok_case!(
        &['a'],
        1,
        'c',
        Changed,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::C, &['c'])
    ),
    ok_case!(
        &['a', 'c'],
        2,
        'h',
        Changed,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Ch, &['c', 'h'])
    ),
    ok_case!(
        &['a'],
        1,
        't',
        Changed,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::T, &['t'])
    ),
    ok_case!(
        &['a'],
        1,
        'p',
        Changed,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::P, &['p'])
    ),
    ok_case!(
        &['a', 'n'],
        2,
        'g',
        Changed,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),

    // ───────────────────────────── Onset extension ──────────────────────────────
    ok_case!(
        &['a'],
        0,
        't',
        Changed,
        ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::A, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['a'],
        0,
        'h',
        Changed,
        ExpectedSyllable::onset_vowel(Onset::H, &['h'], &[(V::A, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['a'],
        0,
        'g',
        Changed,
        ExpectedSyllable::onset_vowel(Onset::G, &['g'], &[(V::A, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['a'],
        0,
        'd',
        Changed,
        ExpectedSyllable::onset_vowel(Onset::D, &['d'], &[(V::A, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['t', 'a'],
        1,
        'h',
        Changed,
        ExpectedSyllable::onset_vowel(Onset::Th, &['t', 'h'], &[(V::A, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['n', 'g', 'a'],
        2,
        'h',
        Changed,
        ExpectedSyllable::onset_vowel(
            Onset::Ngh,
            &['n', 'g', 'h'],
            &[(V::A, C::Lower)],
            Tone::Flat
        )
    ),

    // ───────────────────────────── Nucleus extension ────────────────────────────
    ok_case!(
        &['a'],
        1,
        'i',
        Changed,
        ExpectedSyllable::vowel(&[(V::A, C::Lower), (V::I, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['a'],
        1,
        'o',
        Changed,
        ExpectedSyllable::vowel(&[(V::A, C::Lower), (V::O, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['a'],
        1,
        'u',
        Changed,
        ExpectedSyllable::vowel(&[(V::A, C::Lower), (V::U, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['o', 'a'],
        2,
        'i',
        Changed,
        ExpectedSyllable::vowel(
            &[(V::O, C::Lower), (V::A, C::Lower), (V::I, C::Lower)],
            Tone::Flat
        )
    ),
    ok_case!(
        &['u', 'ơ'],
        2,
        'c',
        Changed,
        ExpectedSyllable::vowel_coda(
            &[(V::UHorn, C::Lower), (V::OHorn, C::Lower)],
            Tone::Flat,
            Coda::C,
            &['c']
        )
    ),

    // ───────────────────────────── Insert at empty ──────────────────────────────
    ok_case!(
        &[],
        0,
        'a',
        Changed,
        ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &[],
        0,
        't',
        Changed,
        ExpectedSyllable::consonant(Onset::T, &['t'])
    ),

    // ───────────────────────────── Gi formation ────────────────────────────────
    ok_case!(
        &['g', 'i', 'n'],
        2,
        'a',
        Changed,
        ExpectedSyllable::full(
            Onset::Gi,
            &['g', 'i'],
            &[(V::A, C::Lower)],
            Tone::Flat,
            Coda::N,
            &['n']
        )
    ),
    ok_case!(
        &['g', 'i'],
        2,
        'a',
        Changed,
        ExpectedSyllable::onset_vowel(Onset::Gi, &['g', 'i'], &[(V::A, C::Lower)], Tone::Flat)
    ),
    ok_case!(
        &['g', 'i'],
        2,
        'n',
        Changed,
        ExpectedSyllable::full(
            Onset::G,
            &['g'],
            &[(V::I, C::Lower)],
            Tone::Flat,
            Coda::N,
            &['n']
        )
    ),
    ok_case!(
        &['g', 'i', 'a', 'n'],
        4,
        'h',
        Changed,
        ExpectedSyllable::full(
            Onset::Gi,
            &['g', 'i'],
            &[(V::A, C::Lower)],
            Tone::Flat,
            Coda::Nh,
            &['n', 'h']
        )
    ),

    // ───────────────────────────── Complex full syllables ───────────────────────
    ok_case!(
        &['a', 'n'],
        0,
        't',
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
        &['a', 'c'],
        0,
        't',
        Changed,
        ExpectedSyllable::full(
            Onset::T,
            &['t'],
            &[(V::A, C::Lower)],
            Tone::Flat,
            Coda::C,
            &['c']
        )
    ),
    ok_case!(
        &['a', 'c'],
        2,
        'h',
        Changed,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Ch, &['c', 'h'])
    ),
    ok_case!(
        &['o', 'a', 'n', 's'],
        0,
        't',
        Changed,
        ExpectedSyllable::full(
            Onset::T,
            &['t'],
            &[(V::O, C::Lower), (V::A, C::Lower)],
            Tone::Acute,
            Coda::N,
            &['n']
        )
    ),

    // ───────────────────────────── Uppercase handling ───────────────────────────
    ok_case!(
        &['t', 'a'],
        1,
        'O',
        Changed,
        ExpectedSyllable::onset_vowel(
            Onset::T,
            &['t'],
            &[(V::O, C::Upper), (V::A, C::Lower)],
            Tone::Flat
        )
    ),
    ok_case!(
        &['a'],
        0,
        'G',
        Changed,
        ExpectedSyllable::onset_vowel(Onset::G, &['G'], &[(V::A, C::Lower)], Tone::Flat)
    ),

    // ───────────────────────────── Error cases ────────────────────────────────
    err_case!(
        &['a'],
        0,
        'q',
        InvalidOnset,
        ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat)
    ),
    err_case!(
        &['u'],
        0,
        'q',
        InvalidOnset,
        ExpectedSyllable::vowel(&[(V::U, C::Lower)], Tone::Flat)
    ),
    err_case!(
        &['a'],
        0,
        'f',
        InvalidOnset,
        ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Flat)
    ),
    err_case!(
        &['o', 'a'],
        1,
        'i',
        InvalidNucleus,
        ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower)], Tone::Flat)
    ),
    err_case!(
        &['o', 'a', 'i'],
        2,
        'u',
        InvalidNucleus,
        ExpectedSyllable::vowel(
            &[(V::O, C::Lower), (V::A, C::Lower), (V::I, C::Lower)],
            Tone::Flat
        )
    ),
    err_case!(
        &['a', 'n', 'g'],
        2,
        'h',
        InvalidCoda,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::Ng, &['n', 'g'])
    ),
    err_case!(
        &['a', 'c'],
        1,
        'h',
        InvalidCoda,
        ExpectedSyllable::vowel_coda(&[(V::A, C::Lower)], Tone::Flat, Coda::C, &['c'])
    ),
    err_case!(
        &['đ', 'a'],
        0,
        'd',
        InvalidOnset,
        ExpectedSyllable::onset_vowel(Onset::D, &['d'], &[(V::A, C::Lower)], Tone::Flat)
    ),
    err_case!(
        &['t', 'a'],
        0,
        'h',
        InvalidOnset,
        ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::A, C::Lower)], Tone::Flat)
    ),
    err_case!(
        &['t', 'a'],
        1,
        's',
        InvalidOnset,
        ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::A, C::Lower)], Tone::Flat)
    ),
    err_case!(
        &['o', 'a'],
        1,
        'i',
        InvalidNucleus,
        ExpectedSyllable::vowel(&[(V::O, C::Lower), (V::A, C::Lower)], Tone::Flat)
    ),
    err_case!(
        &['o', 'a', 'i'],
        2,
        'u',
        InvalidNucleus,
        ExpectedSyllable::vowel(
            &[(V::O, C::Lower), (V::A, C::Lower), (V::I, C::Lower)],
            Tone::Flat
        )
    ),
    err_case!(
        &['o', 'a', 'n'],
        1,
        'i',
        InvalidNucleus,
        ExpectedSyllable::vowel_coda(
            &[(V::O, C::Lower), (V::A, C::Lower)],
            Tone::Flat,
            Coda::N,
            &['n']
        )
    ),
    err_case!(
        &['o', 'a', 'n', 's'],
        1,
        'i',
        InvalidNucleus,
        ExpectedSyllable::vowel_coda(
            &[(V::O, C::Lower), (V::A, C::Lower)],
            Tone::Acute,
            Coda::N,
            &['n']
        )
    ),
];

fn run_case(keymap: &DefaultKeymap, case: &InsertCase) -> Result<(), String> {
    let mut builder = BuildingSyllable::default();

    for &ch in case.base {
        builder
            .push(keymap, ch)
            .map_err(|e| format!("base push({ch:?}) failed: {e:?}"))?;
    }

    if case.at > builder.len() {
        return Err(format!(
            "base={:?} insert(at={}, {:?}) FAILED: builder len={} (at={})",
            case.base, case.at, case.key, builder.len(), case.at
        ));
    }

    let effect = builder.insert(keymap, case.at, case.key);
    let label = format!(
        "base={:?} insert(at={}, {:?})",
        case.base, case.at, case.key
    );

    super::common::check_building_effect(&case.effect, effect, &label)?;
    check_syllable_eq(&builder, &case.expected, case.base)
}

#[test]
fn insert_cases() {
    let telex = DefaultKeymap::telex();

    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|case| run_case(&telex, case).err())
        .collect();

    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// Additional edge case tests for insert
#[test]
fn insert_at_boundaries() {
    let telex = DefaultKeymap::telex();

    // Insert at position 0 (before everything)
    let mut builder = BuildingSyllable::default();
    builder.push(&telex, 'a').unwrap();
    builder.insert(&telex, 0, 't').unwrap();
    assert_eq!(builder.onset(), &['t']);
    assert_eq!(builder.toneless_nucleus().len(), 1);

    // Insert at end position
    let mut builder = BuildingSyllable::default();
    builder.push(&telex, 't').unwrap();
    builder.push(&telex, 'a').unwrap();
    builder.insert(&telex, 2, 'n').unwrap();
    assert_eq!(builder.coda(), &['n']);
}

#[test]
fn insert_tone_on_multi_vowel() {
    let telex = DefaultKeymap::telex();

    // Tone on multi-vowel nucleus should go to correct position per placement
    let mut builder = BuildingSyllable::default();
    for ch in ['h', 'o', 'a'] {
        builder.push(&telex, ch).unwrap();
    }
    // Modern: tone on 'a', Old: tone on 'o'
    builder.insert(&telex, 3, 's').unwrap();
    let modern = builder.to_chars(crate::phonology::TonePlacement::Modern).iter().collect::<String>();
    let old = builder.to_chars(crate::phonology::TonePlacement::Old).iter().collect::<String>();
    assert_eq!(modern, "hoá");
    assert_eq!(old, "hóa");
}