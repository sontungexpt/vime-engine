//! Discriminant contiguity pins for `repr(u8)` enums (`Coda`, `Onset`).
//! `from_id` uses transmute guarded by `id < COUNT`; breaks if variants aren't contiguous from 0.
//! Catches UB at compile/run time. `vowel.rs` covers `BaseVowel`.

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
