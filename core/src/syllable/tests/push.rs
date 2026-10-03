//! Runs the data-driven corpus through `BuildingSyllable::push`.
//!
//! Each test sweeps the `corpus/` slices with one keymap and asserts the
//! corpus never silently shrinks.
//!
//! Test organization:
//! - `telex_corpus`: Full Telex keymap test (all corpus slices)
//! - `vni_corpus`: VNI keymap test (VNI + incomplete cases)
//! - `viqr_corpus`: VIQR keymap test (VIQR cases)
//! - `dead_corpus`: Rejected input fallback tests (Telex + VNI)

use super::corpus::{
    dead_cases, gi, incomplete, onsets, precomposed, syllables, telex_shapes, telex_tones, toggles,
    tones_shapes, uo_sequences, uppercase, viqr, vni, Corpus,
};

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
    assert!(
        n >= 440,
        "expected the telex corpus to stay large; got {n} cases"
    );
}

#[test]
fn vni_corpus() {
    let vni = DefaultKeymap::vni();
    let n = Corpus::new(&vni)
        .with(vni::CASES)
        .with(incomplete::VNI)
        .finish("vni corpus");
    assert!(n >= 55, "expected at least 55 VNI cases; got {n}");
}

#[test]
fn viqr_corpus() {
    let viqr = DefaultKeymap::viqr();
    let n = Corpus::new(&viqr).with(viqr::CASES).finish("viqr corpus");
    assert!(n >= 55, "expected at least 55 VIQR cases; got {n}");
}

#[test]
fn dead_corpus() {
    let telex = DefaultKeymap::telex();
    let vni = DefaultKeymap::vni();
    let n = Corpus::new(&telex)
        .with(dead_cases::TELEX)
        .finish("telex dead corpus")
        + Corpus::new(&vni)
            .with(dead_cases::VNI)
            .finish("vni dead corpus");
    assert!(n >= 35, "expected at least 35 dead cases; got {n}");
}

/// Edge case: single vowel through public Syllable API
#[test]
fn push_single_vowel() {
    let telex = DefaultKeymap::telex();
    let mut syl = crate::syllable::Syllable::new();
    syl.push(&telex, crate::phonology::TonePlacement::Modern, 'a');
    assert!(syl.toneless_nucleus().is_some());
    assert_eq!(syl.toneless_nucleus().unwrap().len(), 1);
}

/// Edge case: maximum nucleus length (3 vowels)
#[test]
fn push_max_nucleus() {
    let telex = DefaultKeymap::telex();
    let mut syl = crate::syllable::Syllable::new();
    for ch in ['i', 'e', 'u'] {
        syl.push(&telex, crate::phonology::TonePlacement::Modern, ch);
    }
    assert_eq!(syl.toneless_nucleus().unwrap().len(), 3);
    // Fourth vowel should fail (go to dead)
    syl.push(&telex, crate::phonology::TonePlacement::Modern, 'a');
    assert!(!syl.is_building());
}

/// Edge case: maximum onset length
#[test]
fn push_max_onset() {
    let telex = DefaultKeymap::telex();
    let mut syl = crate::syllable::Syllable::new();
    // "ngh" is max onset (3 chars)
    for ch in ['n', 'g', 'h'] {
        syl.push(&telex, crate::phonology::TonePlacement::Modern, ch);
    }
    assert_eq!(syl.onset().unwrap().len(), 3);
    // Fourth onset char should fail (go to dead)
    syl.push(&telex, crate::phonology::TonePlacement::Modern, 't');
    assert!(!syl.is_building());
}

/// Edge case: maximum coda length
#[test]
fn push_max_coda() {
    let telex = DefaultKeymap::telex();
    let mut syl = crate::syllable::Syllable::new();
    // Build nucleus first
    syl.push(&telex, crate::phonology::TonePlacement::Modern, 'a');
    // "ng" is max coda (2 chars)
    for ch in ['n', 'g'] {
        syl.push(&telex, crate::phonology::TonePlacement::Modern, ch);
    }
    assert_eq!(syl.coda().unwrap().len(), 2);
    // Third coda char should fail (go to dead)
    syl.push(&telex, crate::phonology::TonePlacement::Modern, 't');
    assert!(!syl.is_building());
}