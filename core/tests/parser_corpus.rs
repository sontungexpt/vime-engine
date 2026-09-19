//! Runs the full data-driven corpus through the current `Composition` pipeline.
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

use corpus::{dead_cases, gi, incomplete, onsets, precomposed, run_all, run_all_dead, run_all_status, syllables, telex_shapes, telex_tones, toggles, tones_shapes, uo_sequences, uppercase, viqr, vni};

use vime_engine::DefaultKeymap;

#[test]
fn telex_corpus() {
    let telex = DefaultKeymap::telex();
    let n = run_all(onsets::CASES, &telex)
        + run_all(telex_tones::CASES, &telex)
        + run_all(telex_shapes::CASES, &telex)
        + run_all(tones_shapes::CASES, &telex)
        + run_all(uo_sequences::CASES, &telex)
        + run_all(precomposed::CASES, &telex)
        + run_all(uppercase::CASES, &telex)
        + run_all(toggles::CASES, &telex)
        + run_all(syllables::CASES, &telex)
        + run_all_status(incomplete::TELEX, &telex)
        + run_all(gi::CASES, &telex);
    assert!(n >= 440, "expected the telex corpus to stay large; got {n} cases");
}

#[test]
fn vni_corpus() {
    let vni = DefaultKeymap::vni();
    let n = run_all(vni::CASES, &vni) + run_all_status(incomplete::VNI, &vni);
    assert!(n >= 55, "expected at least 55 VNI cases; got {n}");
}

#[test]
fn viqr_corpus() {
    let viqr = DefaultKeymap::viqr();
    let n = run_all(viqr::CASES, &viqr);
    assert!(n >= 55, "expected at least 55 VIQr cases; got {n}");
}

#[test]
fn dead_corpus() {
    let telex = DefaultKeymap::telex();
    let vni = DefaultKeymap::vni();
    let n = run_all_dead(&dead_cases::telex_cases(), &telex) + run_all_dead(&dead_cases::vni_cases(), &vni);
    assert!(n >= 35, "expected at least 35 dead cases; got {n}");
}
