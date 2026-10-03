//! Behaviour corpus for `BuildingSyllable::push`: the [`Case`] / [`Outcome`]
//! model, the `case!` macros, the runners and one data module per concern.
//!
//! **Data flow:** Keymap + char → `BuildingSyllable::push()` → syllable state
//!
//! **Corpus organization:**
//! - `onsets.rs`      — single consonant, clusters, qu, gi, plain vowels
//! - `telex_tones.rs` — Telex tone keys (s f r x j) on all vowel types
//! - `telex_shapes.rs` — Telex shape keys (w ^ ( ) for ơ ô ư ă ơ̂)
//! - `tones_shapes.rs` — tone + shape combinations on same vowel
//! - `uo_sequences.rs` — uo / ươ normalization cycles (w key)
//! - `incomplete.rs`  — alive checkpoints (parse can continue)
//! - `dead_cases.rs`  — rejected input rollbacks (InvalidOnset/Nucleus/Coda)
//! - `gi.rs`          — gi onset formation edge cases
//! - `toggles.rs`     — transform revert/toggle behaviour
//! - `uppercase.rs`   — case preservation through transforms
//! - `precomposed.rs` — precomposed Vietnamese characters as input
//! - `vni.rs`         — VNI layout (digits for tones/shapes/stroke)
//! - `viqr.rs`        — VIQR layout (punctuation for tones/shapes)
//! - `syllables.rs`   — real Vietnamese words (regression corpus)

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
    //! One-line import for behaviour modules: cases, macros, shared types.
    pub(crate) use super::super::common::{ExpectedSyllable, C, V};
    pub(crate) use super::{alive_case, case, dead_case, Case};
    pub(crate) use crate::phonology::{Coda, Onset, Tone};
}

use super::common::{check_syllable_eq, ExpectedSyllable};

// ════════════════════════════════════════════════════════════════════════════
// Case Model
// ═══════════════════════════════════════════════════════════════════════════

/// One corpus case: an input push sequence and the outcome it must produce.
pub struct Case {
    /// The keystroke sequence to push.
    pub input: &'static [char],
    /// The expected outcome after pushing all characters.
    pub outcome: Outcome,
}

/// What a case must produce after all pushes.
pub enum Outcome {
    /// Every `push` is `Ok`; the final syllable must match `expected`.
    Alive(ExpectedSyllable),
    /// Some `push` fails; the builder rolls back to `expected` (the accepted prefix).
    Dead(ExpectedSyllable),
    /// Every `push` is `Ok`; only liveness is checked (no state comparison).
    AliveOnly,
}

/// A live case: push every char in order, then the syllable must match exactly.
macro_rules! case {
    ([$($ch:expr),* $(,)?], $syllable:expr $(,)?) => {
        $crate::syllable::tests::corpus::Case {
            input: &[$($ch),*],
            outcome: $crate::syllable::tests::corpus::Outcome::Alive($syllable),
        }
    };
}

/// A dead case: some `push` fails; builder rolls back to `expected` (accepted prefix).
macro_rules! dead_case {
    ([$($ch:expr),* $(,)?], $syllable:expr $(,)?) => {
        $crate::syllable::tests::corpus::Case {
            input: &[$($ch),*],
            outcome: $crate::syllable::tests::corpus::Outcome::Dead($syllable),
        }
    };
}

/// A checkpoint that must stay alive: every push is `Ok`, state unchecked.
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

// ════════════════════════════════════════════════════════════════════════════
// Runners
// ════════════════════════════════════════════════════════════════════════════

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

/// Pushes every character, then checks the final syllable against `expected`.
fn run_expect<KM: Keymap>(
    input: &[char],
    expected: &ExpectedSyllable,
    keymap: &KM,
) -> Result<(), String> {
    let builder = push_all(keymap, input)?;
    check_syllable_eq(&builder, expected, input)
}

/// Requires some `push` to fail, then checks the rolled-back syllable.
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

/// Pushes every character, requiring the builder to stay alive (no state check).
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