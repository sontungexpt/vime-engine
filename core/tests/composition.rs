//! `Composition`'s raw keystroke buffer: what it holds, and where it spills.
//!
//! `raw` is a [`SmallVec`] rather than a `Vec`, so it keeps a whole word inside
//! the struct instead of on the heap. Nothing bounds it — once parsing dies
//! every further key is recorded verbatim, and a user can type forever — so the
//! case that matters here is the boundary: the inline buffer is an optimisation,
//! not a limit, and everything past it has to keep working.
//!
//! [`SmallVec`]: vime_engine::util::vec::SmallVec

use vime_engine::composition::Composition;
use vime_engine::phonology::TonePlacement;
use vime_engine::DefaultKeymap;

/// A [`Composition`] under Telex, which is the keymap these tests type with.
fn telex() -> (Composition, DefaultKeymap<'static>) {
    (Composition::new(), DefaultKeymap::telex())
}

/// Types `text` one key at a time, as a frontend would.
fn type_into(c: &mut Composition, km: &DefaultKeymap<'static>, text: &str) {
    for ch in text.chars() {
        c.insert(km, TonePlacement::Modern, ch);
    }
}

/// The buffer records keystrokes verbatim: one character per key, in order,
/// whatever the parse made of them.
#[test]
fn raw_records_every_keystroke() {
    let (mut c, km) = telex();
    type_into(&mut c, &km, "nguyeen");

    assert_eq!(c.raw(), "nguyeen".chars().collect::<Vec<_>>());
}

/// Past the inline capacity the buffer spills and keeps recording, so the
/// inline arm is a fast path rather than a cap. The keystrokes either side of
/// the boundary are one contiguous sequence, with nothing dropped at the seam.
#[test]
fn raw_spills_past_the_inline_capacity_and_keeps_recording() {
    let (mut c, km) = telex();
    // Comfortably past the inline capacity, and not a multiple of anything
    // that would hide an off-by-one at the seam.
    type_into(&mut c, &km, "nguyeenabcdefghijklmnop");

    let typed: Vec<char> = "nguyeenabcdefghijklmnop".chars().collect();
    assert_eq!(c.raw().len(), typed.len());
    assert_eq!(c.raw(), typed.as_slice());
}

/// Removal crosses the same boundary in the other direction, and leaves an empty
/// buffer rather than a half-drained one.
#[test]
fn raw_shrinks_back_across_the_boundary() {
    let (mut c, km) = telex();
    type_into(&mut c, &km, "nguyeenabcdefghijklmnop");

    for _ in 0.."nguyeenabcdefghijklmnop".chars().count() {
        c.backspace(&km, TonePlacement::Modern);
    }
    assert!(c.raw().is_empty(), "every keystroke removed");

    // And the buffer is reusable afterwards, still recording from empty.
    type_into(&mut c, &km, "ca");
    assert_eq!(c.raw(), "ca".chars().collect::<Vec<_>>());
}
