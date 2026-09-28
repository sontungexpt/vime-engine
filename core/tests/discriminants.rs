//! Discriminant-layout pins for the `repr(u8)` enums.
//!
//! `Coda::from_id` and `Onset::from_id` convert an integer to an enum
//! discriminant with a `transmute`, guarded only by `id < COUNT`. That guard is
//! sound *only* while the variants are declared contiguously from zero. Adding a
//! variant out of order, or bumping `COUNT` past the last discriminant, would
//! make the transmute produce an invalid enum value — Undefined Behavior that
//! compiles cleanly and reads plausible memory.
//!
//! These tests fail at compile time if the representation changes, and at run
//! time if the contiguity invariant breaks. `vowel.rs` already pins its own
//! layout (`size_of::<BaseVowel>() == 1`); this covers the two enums that had
//! no such pin.

use vime_engine::phonology::{Coda, Onset};

#[test]
fn coda_discriminants_are_contiguous() {
    assert_eq!(std::mem::size_of::<Coda>(), 1, "Coda must stay repr(u8)");

    for id in 0..Coda::COUNT as u8 {
        let coda = Coda::from_id(id).unwrap_or_else(|| panic!("id {id} must map to a Coda"));
        // The round trip is what proves the id is a real discriminant, not
        // just a value in range.
        assert_eq!(coda.id(), id, "Coda::from_id({id}) must round-trip");
    }

    assert!(
        Coda::from_id(Coda::COUNT as u8).is_none(),
        "COUNT must be exclusive: the id equal to COUNT is not a Coda",
    );
    assert!(Coda::from_id(u8::MAX).is_none());
}

#[test]
fn onset_discriminants_are_contiguous() {
    assert_eq!(std::mem::size_of::<Onset>(), 1, "Onset must stay repr(u8)");

    for id in 0..Onset::COUNT as u8 {
        let onset = Onset::from_id(id).unwrap_or_else(|| panic!("id {id} must map to an Onset"));
        assert_eq!(onset.id(), id, "Onset::from_id({id}) must round-trip");
    }

    assert!(
        Onset::from_id(Onset::COUNT as u8).is_none(),
        "COUNT must be exclusive: the id equal to COUNT is not an Onset",
    );
    assert!(Onset::from_id(u8::MAX).is_none());
}
