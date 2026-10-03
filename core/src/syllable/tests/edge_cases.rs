//! Edge case tests for syllable builder: boundary conditions, invalid
//! sequences, and stress tests that don't fit in the main corpus.

use super::common::{check_syllable_eq, ExpectedSyllable, C, V};
use crate::keymap::DefaultKeymap;
use crate::phonology::{Coda, Onset, Tone, TonePlacement};
use crate::syllable::building::BuildingSyllable;

const TONE: TonePlacement = TonePlacement::Modern;

fn keymap() -> DefaultKeymap<'static> {
    DefaultKeymap::telex()
}

/// Run a single push sequence and return the builder
fn build(keys: &[char]) -> BuildingSyllable {
    let mut builder = BuildingSyllable::default();
    for &ch in keys {
        let _ = builder.push(&keymap(), ch);
    }
    builder
}

#[test]
fn empty_syllable() {
    let builder = BuildingSyllable::default();
    assert!(builder.toneless_nucleus().is_empty());
    assert_eq!(builder.onset().len(), 0);
    assert_eq!(builder.coda().len(), 0);
    assert_eq!(builder.len(), 0);
    assert_eq!(builder.tone(), Tone::Flat);
}

#[test]
fn max_onset_length() {
    // Onset max is 3 chars (ngh)
    let mut builder = build(&['n', 'g', 'h', 'a']);
    assert_eq!(builder.onset().len(), 3);
    assert_eq!(builder.onset_kind(), Onset::Ngh);
    // 4th onset char should fail
    let mut builder2 = build(&['n', 'g', 'h']);
    assert!(builder2.push(&keymap(), 't').is_err());
}

#[test]
fn max_coda_length() {
    // Coda max is 2 chars (ng, nh, ch)
    let mut builder = build(&['a', 'n', 'g']);
    assert_eq!(builder.coda().len(), 2);
    assert_eq!(builder.coda_kind(), Coda::Ng);
    // 3rd coda char should fail
    let mut builder2 = build(&['a', 'n', 'g']);
    assert!(builder2.push(&keymap(), 't').is_err());
}

#[test]
fn q_waits_for_u() {
    // 'q' alone is not a valid onset, needs 'u'
    let mut builder = build(&['q']);
    assert!(builder.push(&keymap(), 'a').is_err()); // q + a is invalid
    builder.push(&keymap(), 'u').unwrap(); // q + u = Qu
    assert_eq!(builder.onset_kind(), Onset::Qu);
}

#[test]
fn render_roundtrip_modern_old() {
    // Modern and Old placement on same syllable
    use crate::syllable::Syllable;
    let telex = keymap();
    let mut syl = Syllable::new();
    for ch in ['h', 'o', 'a', 'f'] {
        syl.push(&telex, TonePlacement::Modern, ch);
    }
    let modern = syl.to_chars(TonePlacement::Modern).iter().collect::<String>();
    let old = syl.to_chars(TonePlacement::Old).iter().collect::<String>();
    assert_eq!(modern, "hoà");
    assert_eq!(old, "hòa");
}

#[test]
fn write_to_appends() {
    use crate::syllable::Syllable;
    let telex = keymap();
    let mut syl = Syllable::new();
    for ch in ['n', 'g', 'u', 'y', 'e', 'n', 's'] {
        syl.push(&telex, TonePlacement::Modern, ch);
    }
    let mut out = String::from("prefix|");
    syl.write_to(TonePlacement::Modern, &mut out);
    assert_eq!(out, "prefix|nguýen");
}