//! Runs the full data-driven corpus through the real `Parser` pipeline.
//!
//! The corpus data and the harness live under `corpus/`:
//!
//! * `corpus/mod.rs`          — shared types, macros and runner helpers
//! * `corpus/onsets` …        — one module of behaviour data per concern
//!
//! The entry points below run each module's cases and assert the corpus never
//! silently shrinks. Add new inputs to the module matching the behaviour they
//! exercise; keep the bodies of the tests here as small as possible.

mod corpus;

use corpus::{
    dead_cases, onsets, precomposed, run_all, run_all_dead, syllables, telex_shapes, telex_tones,
    toggles, tones_shapes, uo_sequences, uppercase, viqr, vni,
};

use vime_engine::ConfiguredRuleEngine;

#[test]
fn telex_corpus() {
    let telex = ConfiguredRuleEngine::telex();
    let n = run_all(onsets::CASES, &telex)
        + run_all(telex_tones::CASES, &telex)
        + run_all(telex_shapes::CASES, &telex)
        + run_all(tones_shapes::CASES, &telex)
        + run_all(uo_sequences::CASES, &telex)
        + run_all(precomposed::CASES, &telex)
        + run_all(uppercase::CASES, &telex)
        + run_all(toggles::CASES, &telex)
        + run_all(syllables::CASES, &telex);
    assert!(
        n >= 430,
        "expected the telex corpus to stay large; got {n} cases"
    );
}

#[test]
fn vni_corpus() {
    let vni = ConfiguredRuleEngine::vni();
    let n = run_all(vni::CASES, &vni);
    assert!(n >= 55, "expected at least 55 VNI cases; got {n}");
}

#[test]
fn viqr_corpus() {
    let viqr = ConfiguredRuleEngine::viqr();
    let n = run_all(viqr::CASES, &viqr);
    assert!(n >= 55, "expected at least 55 VIQr cases; got {n}");
}

#[test]
fn dead_corpus() {
    let telex = ConfiguredRuleEngine::telex();
    let vni = ConfiguredRuleEngine::vni();
    let n = run_all_dead(dead_cases::TELEX, &telex) + run_all_dead(dead_cases::VNI, &vni);
    assert!(n >= 30, "expected at least 30 dead cases; got {n}");
}
