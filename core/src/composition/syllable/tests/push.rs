//! Runs the full data-driven corpus through `BuildingSyllableBuilder::push`.
//!
//! Behaviour data and the [`Corpus`] runner live under `corpus/`; the shared
//! `ExpectedSyllable` model and `check_syllable_eq` come from `common`. The
//! entry points below sweep each module's rows with one keymap and assert the
//! corpus never silently shrinks.

use super::corpus::{dead_cases, gi, incomplete, onsets, precomposed, syllables, telex_shapes, telex_tones, toggles, tones_shapes, uo_sequences, uppercase, viqr, vni, Corpus};

use crate::keymap::DefaultKeymap;

#[test]
fn telex_corpus() {
    let telex = DefaultKeymap::telex();
    let n = Corpus::new(&telex)
        .with(onsets::CASES)
        .with(telex_tones::CASES)
        .with(telex_shapes::CASES)
        .with(tones_shapes::CASES)
        .with(uo_sequences::CASES)
        .with(precomposed::CASES)
        .with(uppercase::CASES)
        .with(toggles::CASES)
        .with(syllables::CASES)
        .with(incomplete::TELEX)
        .with(gi::CASES)
        .finish("telex corpus");
    assert!(n >= 440, "expected the telex corpus to stay large; got {n} cases");
}

#[test]
fn vni_corpus() {
    let vni = DefaultKeymap::vni();
    let n = Corpus::new(&vni).with(vni::CASES).with(incomplete::VNI).finish("vni corpus");
    assert!(n >= 55, "expected at least 55 VNI cases; got {n}");
}

#[test]
fn viqr_corpus() {
    let viqr = DefaultKeymap::viqr();
    let n = Corpus::new(&viqr).with(viqr::CASES).finish("viqr corpus");
    assert!(n >= 55, "expected at least 55 VIQr cases; got {n}");
}

#[test]
fn dead_corpus() {
    let telex = DefaultKeymap::telex();
    let vni = DefaultKeymap::vni();
    let n = Corpus::new(&telex).with(dead_cases::TELEX).finish("telex dead corpus") + Corpus::new(&vni).with(dead_cases::VNI).finish("vni dead corpus");
    assert!(n >= 35, "expected at least 35 dead cases; got {n}");
}
