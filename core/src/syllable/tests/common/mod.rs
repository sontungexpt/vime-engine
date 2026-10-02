//! Shared assertion harness for the syllable-builder tests.
//!
//! [`ExpectedSyllable`] describes the state a case must produce;
//! [`check_syllable_eq`] and [`check_effect`] compare against it and, on a
//! mismatch, print expected and actual onset / vowels / tone / coda side by side.

use crate::phonology::{BaseVowel, Coda, Onset, Tone, Vowel};
use crate::syllable::building::{
    BuildingSyllable, SyllableBuildError, TransformEffect, TransformTarget,
};
use crate::syllable::InsertOutcome;

/// Shorthand for `BaseVowel`, so dense corpus cases read `(V::A, C::Lower)`.
pub use crate::phonology::BaseVowel as V;

/// Vowel case as a local enum, so cases write `C::Lower` / `C::Upper` without
/// depending on the production casing type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VowelCase {
    Lower,
    Upper,
}

pub use VowelCase as C;

// ─────────────────────────────────────────────────────────────────────────────
// Expected syllable
// ─────────────────────────────────────────────────────────────────────────────

/// The state a case expects the builder to report once the input is fully pushed.
pub struct ExpectedSyllable {
    /// The classified onset cluster kind; `Onset::None` when empty.
    pub onset_kind: Onset,
    /// The raw onset characters (e.g. `['t', 'r']`).
    pub onset: &'static [char],
    /// The vowel nucleus as `(base, case)` pairs, in order.
    pub vowels: &'static [(BaseVowel, VowelCase)],
    /// The tone stored on the syllable.
    pub tone: Tone,
    /// The classified coda cluster kind; `Coda::None` when empty.
    pub coda_kind: Coda,
    /// The raw coda characters (e.g. `['n', 'g']`).
    pub coda: &'static [char],
}

/// Compact constructors for the common syllable shapes, so each corpus case
/// fits on one line; they fill in the `Onset::None` / `Coda::None` /
/// `Tone::Flat` defaults an exploded literal would spell out every time.
impl ExpectedSyllable {
    /// A vowel nucleus with no onset and no coda.
    pub const fn vowel(vowels: &'static [(BaseVowel, VowelCase)], tone: Tone) -> Self {
        Self {
            onset_kind: Onset::None,
            onset: &[],
            vowels,
            tone,
            coda_kind: Coda::None,
            coda: &[],
        }
    }

    /// A bare onset with no nucleus yet (`t`, `ngh`, …); `onset_kind` is
    /// `Onset::None` for consonants the parser leaves unclassified (e.g. `q`).
    pub const fn consonant(onset_kind: Onset, onset: &'static [char]) -> Self {
        Self {
            onset_kind,
            onset,
            vowels: &[],
            tone: Tone::Flat,
            coda_kind: Coda::None,
            coda: &[],
        }
    }

    /// An onset followed by a vowel nucleus, no coda.
    pub const fn onset_vowel(
        onset_kind: Onset,
        onset: &'static [char],
        vowels: &'static [(BaseVowel, VowelCase)],
        tone: Tone,
    ) -> Self {
        Self {
            onset_kind,
            onset,
            vowels,
            tone,
            coda_kind: Coda::None,
            coda: &[],
        }
    }

    /// A vowel nucleus followed by a coda, no onset.
    pub const fn vowel_coda(
        vowels: &'static [(BaseVowel, VowelCase)],
        tone: Tone,
        coda_kind: Coda,
        coda: &'static [char],
    ) -> Self {
        Self {
            onset_kind: Onset::None,
            onset: &[],
            vowels,
            tone,
            coda_kind,
            coda,
        }
    }

    /// The full picture: onset, vowel nucleus and coda.
    pub const fn full(
        onset_kind: Onset,
        onset: &'static [char],
        vowels: &'static [(BaseVowel, VowelCase)],
        tone: Tone,
        coda_kind: Coda,
        coda: &'static [char],
    ) -> Self {
        Self {
            onset_kind,
            onset,
            vowels,
            tone,
            coda_kind,
            coda,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Edit effects
// ─────────────────────────────────────────────────────────────────────────────

/// The result an `insert` / `remove` case must produce, tagged so a case table
/// can state success and failure with the same field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Ok(InsertOutcome),
    Err(SyllableBuildError),
}

/// Compares a case's expected [`Effect`] with the actual result, labelling
/// both on a mismatch.
pub fn check_effect(
    expected: &Effect,
    got: Result<InsertOutcome, SyllableBuildError>,
    label: &str,
) -> Result<(), String> {
    match (expected, got) {
        (Effect::Ok(want), Ok(actual)) if want == &actual => Ok(()),
        (Effect::Err(want), Err(actual)) if want == &actual => Ok(()),
        (expected, got) => Err(format!("{label}: expected {expected:?}, got {got:?}")),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Building module test harness (uses TransformEffect)
// ─────────────────────────────────────────────────────────────────────────────

/// Effect type for `BuildingSyllable` operations, which return `TransformEffect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingEffect {
    Ok(TransformEffect),
    Err(SyllableBuildError),
}

/// Compares expected effect with actual result for building module tests.
pub fn check_building_effect(
    expected: &BuildingEffect,
    got: Result<TransformEffect, SyllableBuildError>,
    label: &str,
) -> Result<(), String> {
    match (expected, got) {
        (BuildingEffect::Ok(want), Ok(actual)) if want == &actual => Ok(()),
        (BuildingEffect::Err(want), Err(actual)) if want == &actual => Ok(()),
        (expected, got) => Err(format!("{label}: expected {expected:?}, got {got:?}")),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Assertions
// ─────────────────────────────────────────────────────────────────────────────

/// Renders a syllable as `{ onset, vowels, tone, coda }` for diagnostics.
fn describe(
    onset_kind: Onset,
    onset: &[char],
    vowels: &[(BaseVowel, VowelCase)],
    tone: Tone,
    coda_kind: Coda,
    coda: &[char],
) -> String {
    format!("{{ onset: {onset_kind:?} {onset:?}, vowels: {vowels:?}, tone: {tone:?}, coda: {coda_kind:?} {coda:?} }}")
}

/// Compares every field of the builder's syllable against `expected`, building
/// the diagnostic string only when they differ.
pub fn check_syllable_eq(
    builder: &BuildingSyllable,
    expected: &ExpectedSyllable,
    input: &[char],
) -> Result<(), String> {
    let vowel_pair = |v: &Vowel| {
        (
            v.base(),
            if v.is_upper() {
                VowelCase::Upper
            } else {
                VowelCase::Lower
            },
        )
    };

    let matches = builder.onset_kind() == expected.onset_kind
        && builder.onset() == expected.onset
        && builder
            .nucleus()
            .iter()
            .map(vowel_pair)
            .eq(expected.vowels.iter().copied())
        && builder.tone() == expected.tone
        && builder.coda_kind() == expected.coda_kind
        && builder.coda() == expected.coda;

    if matches {
        return Ok(());
    }

    let actual_vowels = builder.nucleus().iter().map(vowel_pair).collect::<Vec<_>>();
    Err(format!(
        "input={input:?}\n  expected: {}\n  actual:   {}",
        describe(
            expected.onset_kind,
            expected.onset,
            expected.vowels,
            expected.tone,
            expected.coda_kind,
            expected.coda,
        ),
        describe(
            builder.onset_kind(),
            builder.onset(),
            &actual_vowels,
            builder.tone(),
            builder.coda_kind(),
            builder.coda(),
        ),
    ))
}
