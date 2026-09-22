//! Shared, data-driven harness for the `ValidSyllableBuilder::push` corpus.
//!
//! The pipeline under test is just the syllable builder:
//!
//! ```text
//! Keymap + char → ValidSyllableBuilder::push() → syllable state
//! ```
//!
//! Every case pushes characters one at a time and lets the builder classify and
//! transform each one via the keymap. Precomposed Vietnamese vowels (`ạ`, `ắ`,
//! `Ắ`, …) are kept as-is; the corpus never decomposes them.
//!
//! # Case model
//!
//! A [`Case`] is an `input` (the exact push order) plus an [`Outcome`]:
//!
//! * `Outcome::Alive(expected)` — every `push` returns `Ok`, and the final
//!   syllable must match `expected`. Written with `case!`.
//! * `Outcome::Dead(expected)` — some `push` returns `Err`, and the builder must
//!   have rolled back to `expected`. Written with `dead_case!`.
//! * `Outcome::AliveOnly` — every `push` returns `Ok`; the syllable itself is not
//!   inspected. Written with `alive_case!`.
//!
//! The expected syllable uses the compact [`ExpectedSyllable`] constructors
//! ([`vowel`](ExpectedSyllable::vowel),
//! [`consonant`](ExpectedSyllable::consonant),
//! [`onset_vowel`](ExpectedSyllable::onset_vowel),
//! [`vowel_coda`](ExpectedSyllable::vowel_coda),
//! [`full`](ExpectedSyllable::full)):
//!
//! ```text
//! case!(['a', 's'], ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute))
//! ```
//!
//! means `push('a')` then `push('s')` → syllable `{ vowels: [a], tone: acute,
//! … }`.
//!
//! # Corpus layout
//!
//! The behaviour data lives in one module per concern:
//!
//! | module           | covers                                        |
//! |------------------|-----------------------------------------------|
//! | `onsets`         | onsets and plain vowel sequences              |
//! | `telex_tones`    | telex tone keys                               |
//! | `telex_shapes`   | telex shape keys                              |
//! | `tones_shapes`   | tone + shape combinations                     |
//! | `uo_sequences`   | the `uo` / `ươ` special cycles                |
//! | `vni`            | the VNI layout                                |
//! | `viqr`           | the VIQr layout                               |
//! | `precomposed`    | precomposed Vietnamese vowel input            |
//! | `uppercase`      | uppercase input                               |
//! | `toggles`        | toggle / revert behaviour                     |
//! | `gi`             | the ambiguous `gi` onset / nucleus split      |
//! | `dead_cases`     | inputs whose parse `push` rejects             |
//! | `incomplete`     | inputs that keep the builder alive            |
//! | `syllables`      | real Vietnamese syllables (regression corpus) |
//!
//! The `#[test]` entry points live in the crate root integration test
//! (`parser_corpus.rs`); each drives a [`Corpus`] with one keymap, asserts every
//! case, and checks the corpus never silently shrinks.

pub mod dead_cases;
pub mod gi;
pub mod incomplete;
pub mod onsets;
pub mod precomposed;
pub mod syllables;
pub mod telex_shapes;
pub mod telex_tones;
pub mod toggles;
pub mod tones_shapes;
pub mod uo_sequences;
pub mod uppercase;
pub mod viqr;
pub mod vni;

pub mod prelude {
    //! One-line import for the behaviour modules: cases, macros and the
    //! `C` / `V` field shorthands used by every `ExpectedSyllable`.

    pub(crate) use super::{alive_case, case, dead_case, Case, ExpectedSyllable, C, V};
    pub(crate) use vime_engine::phonology::{Coda, Onset, Tone};
}

/// Field-type shorthands for the dense corpus cases: `(V::A, C::Lower)` reads
/// much faster than `(BaseVowel::VowelCase::Lower)`.
pub use vime_engine::phonology::BaseVowel as V;

/// Vowel case, kept as a tiny local enum so the corpus cases can write
/// `C::Lower` / `C::Upper` without depending on the production casing type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VowelCase {
    Lower,
    Upper,
}

pub use VowelCase as C;

// ─────────────────────────────────────────────────────────────────────────────
// Case model
// ─────────────────────────────────────────────────────────────────────────────

/// One corpus case: an input push order and the outcome it must produce.
pub struct Case {
    pub input: &'static [char],
    pub outcome: Outcome,
}

/// What a case must produce.
pub enum Outcome {
    /// Every `push` is `Ok`; the final syllable must match the expected one.
    Alive(ExpectedSyllable),
    /// A `push` fails at some point; the builder must roll back to this state.
    Dead(ExpectedSyllable),
    /// Every `push` is `Ok`; only liveness is asserted, the syllable is not
    /// inspected.
    AliveOnly,
}

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

macro_rules! case {
    ([$($ch:expr),* $(,)?], $syllable:expr $(,)?) => {
        $crate::corpus::Case {
            input: &[$($ch),*],
            outcome: $crate::corpus::Outcome::Alive($syllable),
        }
    };
}

macro_rules! dead_case {
    ([$($ch:expr),* $(,)?], $syllable:expr $(,)?) => {
        $crate::corpus::Case {
            input: &[$($ch),*],
            outcome: $crate::corpus::Outcome::Dead($syllable),
        }
    };
}

/// A `case!`-style entry for a checkpoint that must stay *alive*: every push
/// is `Ok`, but the final syllable is not inspected.
macro_rules! alive_case {
    ([$($ch:expr),* $(,)?]) => {
        $crate::corpus::Case {
            input: &[$($ch),*],
            outcome: $crate::corpus::Outcome::AliveOnly,
        }
    };
}

pub(crate) use alive_case;
pub(crate) use case;
pub(crate) use dead_case;

// ─────────────────────────────────────────────────────────────────────────────
// Runners
// ─────────────────────────────────────────────────────────────────────────────

use vime_engine::composition::ValidSyllableBuilder;
use vime_engine::phonology::{BaseVowel, CasedBaseVowel, Coda, Onset, Tone};
use vime_engine::Keymap;

/// Pushes every character in order, requiring each `push` to be accepted, and
/// hands back the resulting builder.
fn push_all<KM: Keymap>(keymap: &KM, input: &[char]) -> Result<ValidSyllableBuilder, String> {
    let mut builder = ValidSyllableBuilder::default();

    for &ch in input {
        builder.push(keymap, ch).map_err(|e| format!("input={input:?}: push({ch:?}) unexpectedly failed: {e:?}"))?;
    }

    Ok(builder)
}

/// Pushes every character in order and checks the final syllable against
/// `expected`.
fn run_expect<KM: Keymap>(input: &[char], expected: &ExpectedSyllable, keymap: &KM) -> Result<(), String> {
    let builder = push_all(keymap, input)?;
    check_syllable_eq(&builder, expected, input)
}

/// Pushes every character in order, then requires the builder to be dead: some
/// `push` returns `Err`, and afterwards it must have rolled back to `expected`.
fn run_dead<KM: Keymap>(input: &[char], expected: &ExpectedSyllable, keymap: &KM) -> Result<(), String> {
    let mut builder = ValidSyllableBuilder::default();

    let failed = input.iter().any(|&ch| builder.push(keymap, ch).is_err());
    if !failed {
        return Err(format!("input={input:?}: expected a push to fail, but all were accepted",));
    }

    check_syllable_eq(&builder, expected, input)
}

/// Pushes every character in order, requiring the builder to stay alive; the
/// syllable itself is not inspected.
fn run_alive<KM: Keymap>(input: &[char], keymap: &KM) -> Result<(), String> {
    push_all(keymap, input).map(|_| ())
}

/// Runs a single case against `keymap`.
fn run_case<KM: Keymap>(case: &Case, keymap: &KM) -> Result<(), String> {
    match &case.outcome {
        Outcome::Alive(expected) => run_expect(case.input, expected, keymap),
        Outcome::Dead(expected) => run_dead(case.input, expected, keymap),
        Outcome::AliveOnly => run_alive(case.input, keymap),
    }
}

/// A keymap-bound sweep over several corpus slices that collects every failure
/// and reports them all at once, instead of stopping at the first mismatch.
pub struct Corpus<'a, KM: Keymap> {
    keymap: &'a KM,
    cases: usize,
    failures: Vec<String>,
}

impl<'a, KM: Keymap> Corpus<'a, KM> {
    /// Starts a sweep over `keymap` with an empty failure list.
    pub fn new(keymap: &'a KM) -> Self {
        Self { keymap, cases: 0, failures: Vec::new() }
    }

    /// Sweeps an additional corpus slice, collecting any failures.
    pub fn with(mut self, cases: &[Case]) -> Self {
        self.cases += cases.len();
        for case in cases {
            if let Err(e) = run_case(case, self.keymap) {
                self.failures.push(e);
            }
        }
        self
    }

    /// Panics with every collected failure (labelled), or returns how many cases
    /// ran when the sweep is clean.
    pub fn finish(self, label: &str) -> usize {
        if !self.failures.is_empty() {
            panic!("{label}: {} failing case(s):\n\n{}", self.failures.len(), self.failures.join("\n\n"),);
        }
        self.cases
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
pub fn check_syllable_eq(builder: &ValidSyllableBuilder, expected: &ExpectedSyllable, input: &[char]) -> Result<(), String> {
    let vowel_pair = |v: &CasedBaseVowel| (v.value, if v.uppercase { VowelCase::Upper } else { VowelCase::Lower });

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
