//! Lifecycle tests for the two-phase [`SyllableBuilder`]: the building phase,
//! the fallback into a verbatim dead buffer on a rejected edit, and the
//! recovery to building once the rejected input is removed.
//!
//! The mutators (`push` / `insert` / `remove`) are `pub(crate)`, so these are
//! unit tests: only crate-internal code can drive the dead phase.

use crate::composition::syllable::{InputEffect, SyllableBuilder, SyllableContext};
use crate::keymap::DefaultKeymap;
use crate::phonology::{Onset, TonePlacement};

fn builder() -> SyllableBuilder<DefaultKeymap<'static>> {
    SyllableBuilder::new(SyllableContext::new(
        DefaultKeymap::telex(),
        TonePlacement::Modern,
    ))
}

fn chars(s: &SyllableBuilder<DefaultKeymap<'static>>) -> String {
    let mut out = String::new();
    s.write_to(&mut out);
    out
}

// ─────────────────────────────── Building phase ───────────────────────────────

#[test]
fn initially_building_and_empty() {
    let s = builder();
    assert!(s.is_building());
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);
}

#[test]
fn valid_pushes_stay_in_building_phase() {
    let mut s = builder();
    for ch in ['t', 'a', 'n'] {
        assert_eq!(s.push(ch), InputEffect::StructurallyChanged);
    }

    assert!(s.is_building());
    assert_eq!(chars(&s), "tan");
    assert_eq!(s.onset(), Some(&['t'][..]));
    assert_eq!(s.onset_kind(), Some(Onset::T));
    assert!(s.nucleus().is_some());
    assert!(s.coda().is_some());
}

// ─────────────────────────────── Dead fallback ───────────────────────────────

#[test]
fn rejected_push_falls_back_to_dead() {
    let mut s = builder();
    s.push('a');

    // `z` is neither a vowel, an onset nor a coda char, so the builder rejects
    // it and the accepted prefix carries over into a dead buffer.
    assert_eq!(s.push('z'), InputEffect::StructurallyChanged);

    assert!(!s.is_building());
    assert_eq!(chars(&s), "az");
    assert_eq!(s.len(), 2);

    // The parsed parts are gone once dead.
    assert_eq!(s.onset(), None);
    assert_eq!(s.onset_kind(), None);
    assert_eq!(s.nucleus(), None);
    assert_eq!(s.coda(), None);
    assert_eq!(s.coda_kind(), None);
}

#[test]
fn dead_pushes_are_recorded_verbatim() {
    let mut s = builder();
    s.push('a');
    s.push('z');

    s.push('x');
    s.push('y');

    assert!(!s.is_building());
    assert_eq!(chars(&s), "azxy");
}

#[test]
fn dead_insert_lands_at_the_cursor() {
    let mut s = builder();
    s.push('a');
    s.push('z');

    s.insert(1, 'z');

    assert!(!s.is_building());
    assert_eq!(chars(&s), "azz");
}

#[test]
fn rejected_insert_falls_back_to_dead() {
    let mut s = builder();
    s.push('t');
    s.push('a');

    // Inserting `z` at the onset head is invalid; the whole "ta" prefix is
    // frozen and the rejection recorded at the cursor.
    s.insert(0, 'z');

    assert!(!s.is_building());
    assert_eq!(chars(&s), "zta");
}

/// The dead buffer is verbatim text; a reset discards it.
#[test]
fn reset_recovers_from_dead() {
    let mut s = builder();
    s.push('a');
    s.push('z');
    assert!(!s.is_building());

    s.reset();

    assert!(s.is_building());
    assert!(s.is_empty());
}

// ─────────────────────────────── Recovery ───────────────────────────────

/// Removing the last rejected char makes the remaining buffer a valid syllable
/// again, so the builder re-parses it and switches back to the building phase.
#[test]
fn removing_rejected_chars_returns_to_building() {
    let mut s = builder();
    s.push('a');
    s.push('z');
    s.push('g');
    assert!(!s.is_building());
    assert_eq!(chars(&s), "azg");

    // Still rejected (`z` remains) -> dead, verbatim text now "az".
    s.remove(2);
    assert!(!s.is_building());
    assert_eq!(chars(&s), "az");

    // The last rejected char goes -> "a" parses again.
    s.remove(1);
    assert!(s.is_building());
    assert_eq!(chars(&s), "a");
    assert!(s.nucleus().is_some());
}

/// Removing an accepted char instead leaves only rejected input behind, which
/// can never re-parse, so the buffer stays dead.
#[test]
fn removing_accepted_char_keeps_dead() {
    let mut s = builder();
    s.push('a');
    s.push('z');
    assert_eq!(chars(&s), "az");

    s.remove(0);

    assert!(!s.is_building());
    assert_eq!(chars(&s), "z");
}

/// On the building path a removal stays building.
#[test]
fn remove_on_building_path() {
    let mut s = builder();
    for ch in ['t', 'a'] {
        s.push(ch);
    }

    s.remove(1);

    assert!(s.is_building());
    assert_eq!(chars(&s), "t");
}

// ─────────────────────────────── Rendering ───────────────────────────────

/// `write_to` must append exactly what `to_chars` yields, in both phases.
#[test]
fn write_to_matches_to_chars_in_both_phases() {
    // Building phase: inline render.
    let mut s = builder();
    for ch in "nguye".chars() {
        s.push(ch);
    }
    assert!(s.is_building());
    let mut out = String::new();
    s.write_to(&mut out);
    assert_eq!(out, s.to_chars().iter().collect::<String>());

    // Dead phase: verbatim buffer, reached after a rejected character.
    s.push('z');
    assert!(!s.is_building());
    let mut out = String::new();
    s.write_to(&mut out);
    assert_eq!(out, s.to_chars().iter().collect::<String>());
}

/// `write_to` appends, so it must not clobber what is already in the buffer.
#[test]
fn write_to_appends_rather_than_replaces() {
    let mut s = builder();
    for ch in "toi".chars() {
        s.push(ch);
    }

    let mut out = String::from("prefix:");
    s.write_to(&mut out);
    assert_eq!(out, format!("prefix:{}", s.to_chars().iter().collect::<String>()));

    // And in the dead phase too.
    s.push('z');
    let mut out = String::from("dead:");
    s.write_to(&mut out);
    assert_eq!(out, format!("dead:{}", s.to_chars().iter().collect::<String>()));
}

/// The dead phase writes its characters in the order they were typed,
/// accepted and rejected alike.
#[test]
fn write_to_walks_dead_buffer_in_order() {
    let mut s = builder();
    for ch in "toizq".chars() {
        s.push(ch);
    }
    assert!(!s.is_building());
    assert_eq!(chars(&s), "toizq");
}

/// An empty syllable writes nothing in either phase.
#[test]
fn write_to_on_empty_syllable() {
    let mut s = builder();
    assert!(s.is_building());
    let mut out = String::new();
    s.write_to(&mut out);
    assert_eq!(out, "");

    // A dead buffer that still holds nothing.
    s.push('z');
    s.remove(0);
    let mut out = String::new();
    s.write_to(&mut out);
    assert!(out.is_empty());
}

/// The dead phase's own `write_to` must agree with its `to_chars`, and append
/// rather than replace — the same contract the building phase's `write_to` has.
#[test]
fn dead_write_to_matches_to_chars_and_appends() {
    let mut s = builder();
    for ch in "toizq".chars() {
        s.push(ch);
    }
    assert!(!s.is_building());

    let mut out = String::from("head|");
    s.write_to(&mut out);
    assert_eq!(out, format!("head|{}", s.to_chars().iter().collect::<String>()));
    assert_eq!(out, "head|toizq");
}
