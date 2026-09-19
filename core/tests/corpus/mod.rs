//! Shared, data-driven harness for the `Composition` corpus tests.
//!
//! The real pipeline under test is:
//!
//! ```text
//! Keymap → Composition::append() → Syllable
//! ```
//!
//! Every case lists the *exact* order in which characters are appended; the
//! composition classifies and transforms each one itself via the mapping.
//! Precomposed Vietnamese vowels (`ạ`, `ắ`, `Ắ`, …) are kept as-is; the corpus
//! never decomposes them.
//!
//! # The expected model
//!
//! Each row spells out the full observable state a [`Composition`] must report
//! after the whole input has been appended — an [`ExpectedComposition`]:
//!
//! 1. `input` — the raw appended-character buffer (`Composition::input()`);
//!    every character ever appended, including transform keys and the
//!    characters that killed the parse or were appended after a fallback.
//! 2. `syllable` — the semantic [`ExpectedSyllable`]: onset, vowel nucleus,
//!    tone and coda, compared field-by-field.
//! 3. `phase` — the parsing phase (`Composition::syllable_parse_phase()`):
//!    `Onset` → `Vowel` → `Coda`.
//! 4. `fallback` — the fallback state (`Composition::fallback()`): `None`
//!    while the parse is alive, or `Some(FallbackState)` once it dies — the
//!    recorded [`FallbackReason`] and the exact `Accepted` / `Rejected`
//!    character sequence frozen in `FallbackState::buffer` (accepted syllable
//!    characters, then the killer and any characters appended afterwards).
//!
//! The expected `FallbackState` is a real production value: the harness
//! `fallback_state` helper builds one per fixture row (production `Buffer::push`
//! is public), so the runner can compare the observed fallback state against
//! `expected.fallback` with a single structural `assert_eq!` — reason, every
//! `CharStatus`, and their order are all covered by the derived `PartialEq`.
//! Dead fixtures are built at runtime (a populated `Buffer` is not const),
//! while live fixtures stay `&'static` const arrays with `fallback: None`.
//!
//! # Reading a case
//!
//! Live rows use the compact [`ExpectedSyllable`] constructors
//! ([`vowel`](ExpectedSyllable::vowel),
//! [`consonant`](ExpectedSyllable::consonant),
//! [`onset_vowel`](ExpectedSyllable::onset_vowel),
//! [`vowel_coda`](ExpectedSyllable::vowel_coda),
//! [`syllable`](ExpectedSyllable::syllable)) and list the final parse phase
//! explicitly:
//!
//! ```text
//! case!(['a', 's'], SyllableParsePhase::Vowel,
//!       ExpectedSyllable::vowel(&[(V::A, C::Lower)], Tone::Acute))
//! ```
//!
//! means `append('a')` then `append('s')` → syllable `{ vowels: [a], tone:
//! acute, … }`, phase `Vowel`, no fallback state. Dead rows carry the
//! syllable/phase/issue/toneless explicitly (`dead_case!`). Rows that must
//! stay alive without inspecting the syllable use `status_case!`.
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
//! | `dead_cases`     | full observable state of dead parses          |
//! | `incomplete`     | inputs that leave the parse alive (`none`)    |
//! | `syllables`      | real Vietnamese syllables (regression corpus) |
//!
//! The `#[test]` entry points live in the crate root integration test
//! (`parser_corpus.rs`); they sum the per-module counts and assert the corpus
//! never silently shrinks.

use vime_engine::{
    composition::{Buffer, CharStatus, FallbackReason, FallbackState, ParseAppendingPhase, ParseError, ParseP
    phonology::{BaseVowel, Case, Coda, Onset, Tone},
    Composition, DefaultInputKeymap,
};

/// Field-type shorthands for the data modules: on the dense corpus rows
/// `(V::A, C::Lower)` reads much faster than `(BaseVowel::A, Case::Lower)`.
pub use vime_engine::phonology::{BaseVowel as V, Case as C};

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

// ─────────────────────────────────────────────────────────────────────────────
// Test data types
// ─────────────────────────────────────────────────────────────────────────────

/// The expected semantic state of a live syllable: what [`Composition::syllable`]
/// must report after the whole input has been appended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedSyllable {
    /// The classified onset cluster kind; `Onset::None` when empty.
    pub onset_kind: Onset,
    /// The raw onset characters (e.g. `['t', 'r']`).
    pub onset: &'static [char],
    /// The vowel nucleus as `(base, case)` pairs, in order.
    pub vowels: &'static [(BaseVowel, Case)],
    /// The tone stored on the syllable.
    pub tone: Tone,
    /// The classified coda cluster kind; `Coda::None` when empty.
    pub coda_kind: Coda,
    /// The raw coda characters (e.g. `['n', 'g']`).
    pub coda: &'static [char],
}

/// Compact builders for [`ExpectedSyllable`], one per common shape of a
/// syllable, so every corpus row stays on a single line.
///
/// They fill in the empty `Onset::None` / `Coda::None` / `Tone::Flat` defaults
/// that the exploded struct literal would otherwise spell out on every row.
impl ExpectedSyllable {
    /// A vowel nucleus with no onset and no coda.
    pub const fn vowel(vowels: &'static [(BaseVowel, Case)], tone: Tone) -> Self {
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
    pub const fn onset_vowel(onset_kind: Onset, onset: &'static [char], vowels: &'static [(BaseVowel, Case)], tone: Tone) -> Self {
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
    pub const fn vowel_coda(vowels: &'static [(BaseVowel, Case)], tone: Tone, coda_kind: Coda, coda: &'static [char]) -> Self {
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
    pub const fn syllable(onset_kind: Onset, onset: &'static [char], vowels: &'static [(BaseVowel, Case)], tone: Tone, coda_kind: Coda, coda: &'static [char]) -> Self {
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

/// The full observable state a `Composition` must report after `input` has been
/// appended: the raw appended-char buffer, the semantic syllable, the parsing
/// phase, and the fallback state (`None` while the parse is alive).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedComposition {
    /// The exact characters in `Composition::input()`.
    pub input: &'static [char],
    /// The semantic [`ExpectedSyllable`].
    pub syllable: ExpectedSyllable,
    /// The parsing phase (`Composition::syllable_parse_phase()`).
    pub phase: ParseAppendingPhase,
    /// `None` while alive; `Some(FallbackState)` after the composition fell back.
    pub fallback: Option<FallbackState>,
}

/// Builds the expected fallback state of a dead fixture: the recorded
/// [`SyllableParseIssue`] and the exact `Accepted` / `Rejected` character
/// sequence of `FallbackState::buffer`, in order.
///
/// Fixtures keep the statuses as a plain slice; this helper turns them into a
/// production `FallbackState` (using the public `Buffer::push`), so the runner
/// can compare the observed state with a single structural `assert_eq!`.
pub fn fallback_state(issue: ParseError, buffer: &[CharStatus]) -> FallbackState {
    let mut buf = Buffer::new();
    for &status in buffer {
        buf.push(status);
    }
    FallbackState {
        reason: FallbackReason::ParseFailed(issue),
        buffer: buf,
    }
}

/// An output case. `input` is the exact append order; `expected` is the full
/// observable state the composition must report afterwards.
pub struct TestCase {
    pub input: &'static [char],
    pub expected: ExpectedComposition,
}

/// A case that must end in a dead parse (`fallback() == Some`). `input` is the
/// exact append order; `expected` carries the syllable kept at death, the
/// phase at death and the fallback `FallbackState`.
pub struct DeadCase {
    pub input: &'static [char],
    pub expected: ExpectedComposition,
}

/// A case that must end in an alive parse (`fallback() == None`).
pub struct StatusCase {
    pub input: &'static [char],
}

macro_rules! case {
    ([$($ch:expr),* $(,)?], $phase:expr, $syllable:expr $(,)?) => {
        $crate::corpus::TestCase {
            input: &[$($ch),*],
            expected: $crate::corpus::ExpectedComposition {
                input: &[$($ch),*],
                syllable: $syllable,
                phase: $phase,
                fallback: None,
            },
        }
    };
}

macro_rules! dead_case {
    ([$($ch:expr),* $(,)?], $phase:expr, $syllable:expr, $issue:expr, $toneless:expr $(,)?) => {
        $crate::corpus::DeadCase {
            input: &[$($ch),*],
            expected: $crate::corpus::ExpectedComposition {
                input: &[$($ch),*],
                syllable: $syllable,
                phase: $phase,
                fallback: Some($crate::corpus::fallback_state($issue, $toneless)),
            },
        }
    };
}

/// A `case!`-style entry for any checkpoint that must stay *alive*
/// (`fallback() == None`). Runs through `run_all_status` / `run_status`.
macro_rules! status_case {
    ([$($ch:expr),* $(,)?]) => {
        $crate::corpus::StatusCase {
            input: &[$($ch),*],
        }
    };
}

pub(crate) use case;
pub(crate) use dead_case;
pub(crate) use status_case;

// ─────────────────────────────────────────────────────────────────────────────
// Assertion helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Compares the raw appended-character buffer against `expected`.
fn assert_input_eq(parser: &Composition<DefaultInputKeymap<'_>>, expected: &[char], input: &[char]) {
    assert_eq!(parser.input().items(), expected, "input={input:?}: unexpected appended-char buffer",);
}

/// Compares every field of the parsed syllable against `expected`. The parse
/// phase is *not* part of the syllable and is asserted separately (the phase
/// can diverge from the syllable shape after a kill, e.g. a two-vowel `uo`
/// prefix left in the `Vowel` phase).
pub fn assert_syllable_eq(parser: &Composition<DefaultInputKeymap<'_>>, expected: &ExpectedSyllable, input: &[char]) {
    let syllable = parser.syllable();

    assert_eq!(syllable.onset.kind(), expected.onset_kind, "input={input:?}: unexpected onset kind",);
    assert_eq!(syllable.onset.chars(), expected.onset, "input={input:?}: unexpected onset characters",);

    let actual_vowels: Vec<(BaseVowel, Case)> = syllable.vowels.iter().map(|v| (v.value, v.case)).collect();
    assert_eq!(actual_vowels, expected.vowels, "input={input:?}: unexpected vowel nucleus",);

    assert_eq!(syllable.tone, expected.tone, "input={input:?}: unexpected tone",);

    assert_eq!(syllable.coda.kind(), expected.coda_kind, "input={input:?}: unexpected coda kind",);
    assert_eq!(syllable.coda.chars(), expected.coda, "input={input:?}: unexpected coda characters",);
}

/// Asserts the four observable pieces of a composition in order: the raw
/// `input()` buffer, the semantic syllable, the parse phase, and the fallback
/// state — compared as a whole with a single structural `assert_eq!` against
/// the expected `FallbackState` (reason + every `CharStatus`, in order).
pub fn assert_composition_eq(parser: &Composition<DefaultInputKeymap<'_>>, expected: &ExpectedComposition, input: &[char]) {
    assert_input_eq(parser, expected.input, input);

    assert_syllable_eq(parser, &expected.syllable, input);

    assert_eq!(parser.syllable_parse_phase(), expected.phase, "input={input:?}: unexpected parse phase",);

    assert_eq!(
        parser.fallback().as_ref(),
        expected.fallback.as_ref(),
        "\ninput = {input:?}\n  expected fallback state = {:?}\n  actual                  = {:?}",
        expected.fallback,
        parser.fallback(),
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Runner helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Appends every character in order and asserts the parser accepts the whole
/// input without dying, and that the four observable pieces (raw input buffer,
/// syllable, phase, `fallback() == None`) match `case.expected`.
pub fn run_case(case: &TestCase, mapping: &DefaultInputKeymap<'_>) {
    let mut parser = Composition::new(*mapping);

    for &ch in case.input {
        parser.append(ch);
    }

    assert_composition_eq(&parser, &case.expected, case.input);

    assert!(!parser.is_empty(), "input={:?} left the composition empty", case.input,);
}

/// Appends every character in order and asserts the composition stays alive,
/// i.e. `fallback() == None`. This is what `status_case!` means today.
pub fn run_status(case: &StatusCase, mapping: &DefaultInputKeymap<'_>) {
    let mut parser = Composition::new(*mapping);

    for &ch in case.input {
        parser.append(ch);
    }

    assert!(parser.fallback().is_none(), "\ninput = {:?}\n  expected: alive (fallback == None)\n  actual:   {:?}", case.input, parser.fallback(),);

    assert_eq!(parser.input().items(), case.input, "input={:?}: unexpected appended-char buffer", case.input);

    assert!(!parser.is_empty(), "input={:?} left the composition empty", case.input,);
}

/// Appends every character in order and asserts the composition ends dead
/// (`fallback() == Some`) and that the four observable pieces (raw input
/// buffer, syllable kept at death, phase at death, and the `FallbackState`)
/// match `case.expected`.
pub fn run_dead(case: &DeadCase, mapping: &DefaultInputKeymap<'_>) {
    let mut parser = Composition::new(*mapping);

    for &ch in case.input {
        parser.append(ch);
    }

    assert!(parser.fallback().is_some(), "\ninput = {:?}\n  expected: dead (fallback == Some)\n  actual:   None", case.input,);

    assert_composition_eq(&parser, &case.expected, case.input);

    assert!(!parser.is_empty(), "input={:?} left the composition empty", case.input,);
}

/// Runs every case in a corpus slice and returns how many were executed.
pub fn run_all<'a>(cases: &'a [TestCase], mapping: &DefaultInputKeymap<'a>) -> usize {
    for case in cases {
        run_case(case, mapping);
    }
    cases.len()
}

/// Runs every dead case in a corpus slice and returns how many were executed.
pub fn run_all_dead<'a>(cases: &'a [DeadCase], mapping: &DefaultInputKeymap<'a>) -> usize {
    for case in cases {
        run_dead(case, mapping);
    }
    cases.len()
}

/// Runs every alive-status checkpoint in a corpus slice and returns how many
/// were executed.
pub fn run_all_status<'a>(cases: &'a [StatusCase], mapping: &DefaultInputKeymap<'a>) -> usize {
    for case in cases {
        run_status(case, mapping);
    }
    cases.len()
}
