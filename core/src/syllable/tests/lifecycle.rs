//! Lifecycle tests for the two-phase [`Syllable`]: the building phase,
//! the fallback into a verbatim dead buffer on a rejected edit, and the
//! recovery to building once the rejected input is removed.
//!
//! The mutators (`push` / `insert` / `remove`) are `pub(crate)`, so these are
//! unit tests: only crate-internal code can drive the dead phase.

use crate::keymap::DefaultKeymap;
use crate::phonology::{Onset, TonePlacement};
use crate::syllable::{InputEffect, Syllable};

/// The keymap every test here parses under. A [`Syllable`] does not hold one,
/// so each operation is handed it.
fn keymap() -> DefaultKeymap<'static> {
    DefaultKeymap::telex()
}

/// The tone-placement scheme every test here renders under.
const TONE: TonePlacement = TonePlacement::Modern;

fn builder() -> Syllable {
    Syllable::new()
}

fn chars(s: &Syllable) -> String {
    let mut out = String::new();
    s.write_to(TONE, &mut out);
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
        assert_eq!(
            s.push(&keymap(), TONE, ch),
            InputEffect::StructurallyChanged
        );
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
    s.push(&keymap(), TONE, 'a');

    // `z` is neither a vowel, an onset nor a coda char, so the builder rejects
    // it and the accepted prefix carries over into a dead buffer.
    assert_eq!(
        s.push(&keymap(), TONE, 'z'),
        InputEffect::StructurallyChanged
    );

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
    s.push(&keymap(), TONE, 'a');
    s.push(&keymap(), TONE, 'z');

    s.push(&keymap(), TONE, 'x');
    s.push(&keymap(), TONE, 'y');

    assert!(!s.is_building());
    assert_eq!(chars(&s), "azxy");
}

#[test]
fn dead_insert_lands_at_the_cursor() {
    let mut s = builder();
    s.push(&keymap(), TONE, 'a');
    s.push(&keymap(), TONE, 'z');

    s.insert(&keymap(), TONE, 1, 'z');

    assert!(!s.is_building());
    assert_eq!(chars(&s), "azz");
}

#[test]
fn rejected_insert_falls_back_to_dead() {
    let mut s = builder();
    s.push(&keymap(), TONE, 't');
    s.push(&keymap(), TONE, 'a');

    // Inserting `z` at the onset head is invalid; the whole "ta" prefix is
    // frozen and the rejection recorded at the cursor.
    s.insert(&keymap(), TONE, 0, 'z');

    assert!(!s.is_building());
    assert_eq!(chars(&s), "zta");
}

/// The dead buffer is verbatim text; a reset discards it.
#[test]
fn reset_recovers_from_dead() {
    let mut s = builder();
    s.push(&keymap(), TONE, 'a');
    s.push(&keymap(), TONE, 'z');
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
    s.push(&keymap(), TONE, 'a');
    s.push(&keymap(), TONE, 'z');
    s.push(&keymap(), TONE, 'g');
    assert!(!s.is_building());
    assert_eq!(chars(&s), "azg");

    // Still rejected (`z` remains) -> dead, verbatim text now "az".
    s.remove(&keymap(), TONE, 2);
    assert!(!s.is_building());
    assert_eq!(chars(&s), "az");

    // The last rejected char goes -> "a" parses again.
    s.remove(&keymap(), TONE, 1);
    assert!(s.is_building());
    assert_eq!(chars(&s), "a");
    assert!(s.nucleus().is_some());
}

/// Removing an accepted char instead leaves only rejected input behind, which
/// can never re-parse, so the buffer stays dead.
#[test]
fn removing_accepted_char_keeps_dead() {
    let mut s = builder();
    s.push(&keymap(), TONE, 'a');
    s.push(&keymap(), TONE, 'z');
    assert_eq!(chars(&s), "az");

    s.remove(&keymap(), TONE, 0);

    assert!(!s.is_building());
    assert_eq!(chars(&s), "z");
}

/// On the building path a removal stays building.
#[test]
fn remove_on_building_path() {
    let mut s = builder();
    for ch in ['t', 'a'] {
        s.push(&keymap(), TONE, ch);
    }

    s.remove(&keymap(), TONE, 1);

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
        s.push(&keymap(), TONE, ch);
    }
    assert!(s.is_building());
    let mut out = String::new();
    s.write_to(TONE, &mut out);
    assert_eq!(out, s.to_chars(TONE).iter().collect::<String>());

    // Dead phase: verbatim buffer, reached after a rejected character.
    s.push(&keymap(), TONE, 'z');
    assert!(!s.is_building());
    let mut out = String::new();
    s.write_to(TONE, &mut out);
    assert_eq!(out, s.to_chars(TONE).iter().collect::<String>());
}

/// `write_to` appends, so it must not clobber what is already in the buffer.
#[test]
fn write_to_appends_rather_than_replaces() {
    let mut s = builder();
    for ch in "toi".chars() {
        s.push(&keymap(), TONE, ch);
    }

    let mut out = String::from("prefix:");
    s.write_to(TONE, &mut out);
    assert_eq!(
        out,
        format!("prefix:{}", s.to_chars(TONE).iter().collect::<String>())
    );

    // And in the dead phase too.
    s.push(&keymap(), TONE, 'z');
    let mut out = String::from("dead:");
    s.write_to(TONE, &mut out);
    assert_eq!(
        out,
        format!("dead:{}", s.to_chars(TONE).iter().collect::<String>())
    );
}

/// The dead phase writes its characters in the order they were typed,
/// accepted and rejected alike.
#[test]
fn write_to_walks_dead_buffer_in_order() {
    let mut s = builder();
    for ch in "toizq".chars() {
        s.push(&keymap(), TONE, ch);
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
    s.write_to(TONE, &mut out);
    assert_eq!(out, "");

    // A dead buffer that still holds nothing.
    s.push(&keymap(), TONE, 'z');
    s.remove(&keymap(), TONE, 0);
    let mut out = String::new();
    s.write_to(TONE, &mut out);
    assert!(out.is_empty());
}

/// The dead phase's own `write_to` must agree with its `to_chars`, and append
/// rather than replace — the same contract the building phase's `write_to` has.
#[test]
fn dead_write_to_matches_to_chars_and_appends() {
    let mut s = builder();
    for ch in "toizq".chars() {
        s.push(&keymap(), TONE, ch);
    }
    assert!(!s.is_building());

    let mut out = String::from("head|");
    s.write_to(TONE, &mut out);
    assert_eq!(
        out,
        format!("head|{}", s.to_chars(TONE).iter().collect::<String>())
    );
    assert_eq!(out, "head|toizq");
}

// ────────────────────── Explicit configuration arguments ──────────────────────

/// The builder stores no configuration, so the tone-placement scheme is the
/// caller's to choose at render time. One parsed syllable, two renderings.
#[test]
fn tone_placement_is_supplied_per_render() {
    let telex = DefaultKeymap::telex();
    let mut s = Syllable::new();
    for ch in "hoas".chars() {
        s.push(&telex, TONE, ch);
    }
    assert!(s.is_building());

    // Modern and Old put the mark on opposite vowels of the open `oa` nucleus,
    // so the same parsed state renders two different words.
    let modern: String = s.to_chars(TonePlacement::Modern).into_iter().collect();
    let old: String = s.to_chars(TonePlacement::Old).into_iter().collect();
    assert_eq!(modern, "hoá");
    assert_eq!(old, "hóa");

    // `write_to` and `iter_chars` take the same argument and agree with
    // `to_chars`.
    let mut out = String::new();
    s.write_to(TonePlacement::Old, &mut out);
    assert_eq!(out, old);
    assert_eq!(
        s.iter_chars(TonePlacement::Modern).collect::<String>(),
        modern
    );
}

/// Likewise for the keymap: it is the caller's to choose per operation, and the
/// same keystrokes mean different things under different layouts.
#[test]
fn keymap_is_supplied_per_operation() {
    // Telex binds `w` as a shape key, so it transforms the vowel.
    let telex = DefaultKeymap::telex();
    let mut s = Syllable::new();
    s.push(&telex, TONE, 'a');
    assert_eq!(s.push(&telex, TONE, 'w'), InputEffect::Transformed);
    assert!(s.is_building());
    assert_eq!(s.to_chars(TONE).iter().collect::<String>(), "ă");

    // VIQR binds no shape on `w`, so the very same keystrokes are taken as
    // literal input — and `w` cannot follow the vowel in a syllable, so the
    // builder falls back to the dead buffer.
    let viqr = DefaultKeymap::viqr();
    let mut v = Syllable::new();
    v.push(&viqr, TONE, 'a');
    assert_eq!(v.push(&viqr, TONE, 'w'), InputEffect::StructurallyChanged);
    assert!(!v.is_building());
    assert_eq!(v.to_chars(TONE).iter().collect::<String>(), "aw");
}
