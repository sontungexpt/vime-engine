//! Behaviour corpus for `BuildingSyllable::push`: the [`Case`] / [`Outcome`]
//! model, the `case!` macros, the runners and one data module per concern.
//! ```text
//! Keymap + char → BuildingSyllable::push() → syllable state
//! ```

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
    //! One-line import for the behaviour modules: cases, macros, the shared
    //! `ExpectedSyllable` / `C` / `V` and the phonology types.

    pub(crate) use super::super::common::{ExpectedSyllable, C, V};
    pub(crate) use super::{alive_case, case, dead_case, Case};
    pub(crate) use crate::phonology::{Coda, Onset, Tone};
}

use super::common::{check_syllable_eq, ExpectedSyllable};

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
    /// Every `push` is `Ok`; only liveness is asserted.
    AliveOnly,
}

/// A live case: push every char in order, then the syllable must match.
macro_rules! case {
    ([$($ch:expr),* $(,)?], $syllable:expr $(,)?) => {
        $crate::syllable::tests::corpus::Case {
            input: &[$($ch),*],
            outcome: $crate::syllable::tests::corpus::Outcome::Alive($syllable),
        }
    };
}

/// A case where some `push` is rejected; the builder must roll back to `expected`.
macro_rules! dead_case {
    ([$($ch:expr),* $(,)?], $syllable:expr $(,)?) => {
        $crate::syllable::tests::corpus::Case {
            input: &[$($ch),*],
            outcome: $crate::syllable::tests::corpus::Outcome::Dead($syllable),
        }
    };
}

/// A checkpoint that must stay *alive*: every push is `Ok`, the syllable unchecked.
macro_rules! alive_case {
    ([$($ch:expr),* $(,)?]) => {
        $crate::syllable::tests::corpus::Case {
            input: &[$($ch),*],
            outcome: $crate::syllable::tests::corpus::Outcome::AliveOnly,
        }
    };
}

pub(crate) use alive_case;
pub(crate) use case;
pub(crate) use dead_case;

// ─────────────────────────────────────────────────────────────────────────────
// Runners
// ─────────────────────────────────────────────────────────────────────────────

use crate::keymap::Keymap;
use crate::syllable::building::BuildingSyllable;

/// Pushes every character in order, requiring each `push` to be accepted.
fn push_all<KM: Keymap>(keymap: &KM, input: &[char]) -> Result<BuildingSyllable, String> {
    let mut builder = BuildingSyllable::default();

    for &ch in input {
        builder
            .push(keymap, ch)
            .map_err(|e| format!("input={input:?}: push({ch:?}) unexpectedly failed: {e:?}"))?;
    }

    Ok(builder)
}

/// Pushes every character in order, then checks the final syllable against `expected`.
fn run_expect<KM: Keymap>(
    input: &[char],
    expected: &ExpectedSyllable,
    keymap: &KM,
) -> Result<(), String> {
    let builder = push_all(keymap, input)?;
    check_syllable_eq(&builder, expected, input)
}

/// Requires some `push` to fail, then checks the rolled-back syllable against `expected`.
fn run_dead<KM: Keymap>(
    input: &[char],
    expected: &ExpectedSyllable,
    keymap: &KM,
) -> Result<(), String> {
    let mut builder = BuildingSyllable::default();

    let failed = input.iter().any(|&ch| builder.push(keymap, ch).is_err());
    if !failed {
        return Err(format!(
            "input={input:?}: expected a push to fail, but all were accepted",
        ));
    }

    check_syllable_eq(&builder, expected, input)
}

/// Pushes every character in order, requiring the builder to stay alive.
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
        Self {
            keymap,
            cases: 0,
            failures: Vec::new(),
        }
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
            panic!(
                "{label}: {} failing case(s):\n\n{}",
                self.failures.len(),
                self.failures.join("\n\n"),
            );
        }
        self.cases
    }
}
