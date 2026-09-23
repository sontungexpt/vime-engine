//! Shared assertion harness for the syllable-builder integration tests.
//!
//! Two entry-point tests exercise the builder through its public API:
//!
//! * [`push`](../push/index.html) sweeps the behaviour corpus under
//!   [`corpus`](../corpus/index.html) through `BuildingSyllableBuilder::push`.
//! * [`insert`](../insert/index.html) drives `BuildingSyllableBuilder::insert`
//!   at an explicit cursor.
//!
//! Both compare the resulting syllable against an [`ExpectedSyllable`] and
//! report every mismatch through [`check_syllable_eq`], so a failure prints
//! the expected and actual onset / vowels / tone / coda side by side.

use crate::composition::syllable::building::BuildingSyllableBuilder;
use crate::phonology::{BaseVowel, CasedBaseVowel, Coda, Onset, Tone};

/// Field-type shorthands for the dense corpus cases: `(V::A, C::Lower)` reads
/// much faster than `(BaseVowel::VowelCase::Lower)`.
pub use crate::phonology::BaseVowel as V;

/// Vowel case, kept as a tiny local enum so the corpus cases can write
/// `C::Lower` / `C::Upper` without depending on the production casing type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VowelCase {
    Lower,
    Upper,
}

pub use VowelCase as C;

// ─────────────────────────────────────────────────────────────────────────────
// Expected syllable
// ─────────────────────────────────────────────────────────────────────────────

/// The expected semantic state of a syllable: what the builder must report
/// after the whole input has been pushed.
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

/// Compact builders for [`ExpectedSyllable`], one per common shape of a
/// syllable, so every corpus case stays on a single line.
///
/// They fill in the empty `Onset::None` / `Coda::None` / `Tone::Flat` defaults
/// that the exploded struct literal would otherwise spell out on every case.
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

    /// A bare onset with no vowel nucleus yet (`t`, `ngh`, …). `onset_kind` is
    /// `Onset::None` for consonant characters the parser leaves unclassified
    /// (e.g. `q`).
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
    pub const fn onset_vowel(onset_kind: Onset, onset: &'static [char], vowels: &'static [(BaseVowel, VowelCase)], tone: Tone) -> Self {
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
    pub const fn vowel_coda(vowels: &'static [(BaseVowel, VowelCase)], tone: Tone, coda_kind: Coda, coda: &'static [char]) -> Self {
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
    pub const fn full(onset_kind: Onset, onset: &'static [char], vowels: &'static [(BaseVowel, VowelCase)], tone: Tone, coda_kind: Coda, coda: &'static [char]) -> Self {
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
// Assertions
// ─────────────────────────────────────────────────────────────────────────────

/// Renders a syllable as `{ onset, vowels, tone, coda }` for diagnostics.
fn describe(onset_kind: Onset, onset: &[char], vowels: &[(BaseVowel, VowelCase)], tone: Tone, coda_kind: Coda, coda: &[char]) -> String {
    format!("{{ onset: {onset_kind:?} {onset:?}, vowels: {vowels:?}, tone: {tone:?}, coda: {coda_kind:?} {coda:?} }}")
}

/// Compares every field of the builder's syllable against `expected`, building
/// the diagnostic string only when they differ.
pub fn check_syllable_eq(builder: &BuildingSyllableBuilder, expected: &ExpectedSyllable, input: &[char]) -> Result<(), String> {
    let vowel_pair = |v: &CasedBaseVowel| (*v.value(), if v.is_upper() { VowelCase::Upper } else { VowelCase::Lower });

    let matches = builder.onset_kind() == expected.onset_kind
        && builder.onset() == expected.onset
        && builder.vowels().iter().map(vowel_pair).eq(expected.vowels.iter().copied())
        && builder.tone() == expected.tone
        && builder.coda_kind() == expected.coda_kind
        && builder.coda() == expected.coda;

    if matches {
        return Ok(());
    }

    let actual_vowels = builder.vowels().iter().map(vowel_pair).collect::<Vec<_>>();
    Err(format!(
        "input={input:?}\n  expected: {}\n  actual:   {}",
        describe(expected.onset_kind, expected.onset, expected.vowels, expected.tone, expected.coda_kind, expected.coda,),
        describe(builder.onset_kind(), builder.onset(), &actual_vowels, builder.tone(), builder.coda_kind(), builder.coda(),),
    ))
}
