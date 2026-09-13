//! Shared, data-driven harness for the `Parser` corpus tests.
//!
//! The real pipeline under test is:
//!
//! ```text
//! RuleEngine → Parser::push() → Syllable → Renderer
//! ```
//!
//! Every case lists the *exact* order in which characters are pushed; the
//! parser classifies each one itself via the mapping. Precomposed Vietnamese
//! vowels (`ạ`, `ắ`, `Ắ`, …) are kept as-is; the corpus never decomposes
//! them.
//!
//! # Reading a case
//!
//! ```text
//! case!(['a', 's'], "á"),
//! ```
//!
//! means:
//!
//! ```text
//! push('a')   // Literal: plain vowel
//! push('s')   // Transform: acute tone
//! → "á"
//! ```
//!
//! Anything the mapping does *not* treat as a transform key is handled as a
//! literal, exactly like `Engine` does.
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
//! | `toggles`       | toggle / revert behaviour                     |
//! | `dead_cases`     | inputs that leave the parse in a dead status  |
//! | `syllables`      | real Vietnamese syllables (regression corpus) |
//!
//! The `#[test]` entry points live in the crate root integration test
//! (`parser_corpus.rs`); they sum the per-module counts and assert the corpus
//! never silently shrinks.

use vime_engine::{ConfiguredRuleEngine, DefaultRenderer, ParseStatus, Parser, Renderer};

pub mod dead_cases;
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

pub struct TestCase {
    pub input: &'static [char],
    pub expected: &'static str,
}

pub struct DeadCase {
    pub input: &'static [char],
    pub expected: ParseStatus,
}

macro_rules! case {
    ([$($ch:expr),* $(,)?], $expected:expr) => {
        TestCase {
            input: &[$($ch),*],
            expected: $expected,
        }
    };
}

macro_rules! dead_case {
    ([$($ch:expr),* $(,)?], $expected:expr) => {
        DeadCase {
            input: &[$($ch),*],
            expected: $expected,
        }
    };
}

pub(crate) use case;
pub(crate) use dead_case;

// ─────────────────────────────────────────────────────────────────────────────
// Runner helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Pushes every character in order, then renders the resulting syllable.
pub fn run_case(case: &TestCase, mapping: &ConfiguredRuleEngine<'_>) {
    let mut parser = Parser::new(*mapping);

    for &ch in case.input {
        parser.push(ch);
    }

    let rendered = DefaultRenderer::default().render(parser.syllable());

    assert_eq!(
        rendered,
        case.expected,
        "\ninput={:?}\n  expected = {:?}\n  actual   = {:?}\n  syllable = {:?}\n  status   = {:?}\n  phase    = {:?}",
        case.input,
        case.expected,
        rendered,
        parser.syllable(),
        parser.status(),
        parser.phase(),
    );
}

/// Pushes every character in order and asserts the parser ends up in the
/// expected (dead) status. The syllable that a dead parse renders is not
/// compared, only the `ParseStatus`.
pub fn run_dead(case: &DeadCase, mapping: &ConfiguredRuleEngine<'_>) {
    let mut parser = Parser::new(*mapping);

    for &ch in case.input {
        parser.push(ch);
    }

    assert_eq!(
        parser.status(),
        case.expected,
        "\ninput={:?}\n  expected status = {:?}\n  actual status   = {:?}\n  phase = {:?}\n  syllable = {:?}",
        case.input,
        case.expected,
        parser.status(),
        parser.phase(),
        parser.syllable(),
    );
}

/// Runs every case in a corpus slice and returns how many were executed.
pub fn run_all<'a>(cases: &'a [TestCase], mapping: &ConfiguredRuleEngine<'a>) -> usize {
    for case in cases {
        run_case(case, mapping);
    }
    cases.len()
}

/// Runs every dead case in a corpus slice and returns how many were executed.
pub fn run_all_dead<'a>(cases: &'a [DeadCase], mapping: &ConfiguredRuleEngine<'a>) -> usize {
    for case in cases {
        run_dead(case, mapping);
    }
    cases.len()
}
