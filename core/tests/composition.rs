//! `Composition` raw buffer tests: inline `SmallVec` behavior, spill boundary, shrink-back, cursor clamping independence.

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
// ───────────────────────── moving several positions at once ─────────────────────────

/// A typed word leaves both cursors at the end, which is where a counted sweep
/// starts from.
fn typed(word: &str) -> Composition {
    let (mut c, km) = telex();
    type_into(&mut c, &km, word);
    c
}

/// Moving left by `n` lands exactly `n` positions back while the buffer allows
/// it, and reports that the cursor moved.
#[test]
fn moving_left_by_n_lands_exactly_n_back() {
    let mut c = typed("nga");
    assert_eq!((c.rendered_cursor(), c.raw_cursor()), (3, 3));

    let moved = c.move_cursor_left_by(2);
    assert!(*moved.rendered() && *moved.raw());
    assert_eq!((c.rendered_cursor(), c.raw_cursor()), (1, 1));
}

/// Past the start the movement clamps rather than wrapping, and still counts as a
/// move: the caret did go somewhere. Once it is at the start there is nothing
/// left to give, which is what tells a frontend the key was forwarded.
#[test]
fn moving_left_past_the_start_clamps_and_still_reports_true() {
    let mut c = typed("nga");

    let moved = c.move_cursor_left_by(99);
    assert!(*moved.rendered() && *moved.raw());
    assert_eq!((c.rendered_cursor(), c.raw_cursor()), (0, 0));

    let again = c.move_cursor_left_by(1);
    assert!(
        !*again.rendered() && !*again.raw(),
        "already at the start, so nothing moved"
    );
}

/// `usize::MAX` is what a frontend forwarding a garbage count would produce, so
/// the arithmetic has to saturate rather than wrap around the buffer.
#[test]
fn a_saturating_count_does_not_overflow() {
    let mut c = typed("nga");

    assert!(*c.move_cursor_left_by(usize::MAX).rendered());
    assert_eq!((c.rendered_cursor(), c.raw_cursor()), (0, 0));

    assert!(*c.move_cursor_right_by(usize::MAX).rendered());
    assert_eq!((c.rendered_cursor(), c.raw_cursor()), (3, 3));

    // And the same huge count from the end is still a no-op, not a wrap.
    assert!(!*c.move_cursor_right_by(usize::MAX).rendered());
    assert_eq!((c.rendered_cursor(), c.raw_cursor()), (3, 3));
}

/// Zero asks to move nowhere, so it reports nothing moved.
#[test]
fn moving_by_zero_moves_nothing() {
    let mut c = typed("nga");

    let moved = c.move_cursor_left_by(0);
    assert!(!*moved.rendered() && !*moved.raw());
    assert_eq!((c.rendered_cursor(), c.raw_cursor()), (3, 3));
}

/// The two cursors are clamped independently, because a transform makes them
/// disagree about where the ends are: `a` + `w` renders one character from two
/// keystrokes, so the parsed cursor runs out of room first.
#[test]
fn the_two_cursors_clamp_independently() {
    let mut c = typed("aw"); // renders as "ă": one character, two keystrokes
    assert_eq!((c.rendered_cursor(), c.raw_cursor()), (1, 2));

    let first = c.move_cursor_left_by(1);
    assert!(*first.rendered() && *first.raw());
    assert_eq!((c.rendered_cursor(), c.raw_cursor()), (0, 1));

    // The parsed cursor is at its start now; the raw one still has a step left,
    // and reports so independently rather than following its neighbour.
    let second = c.move_cursor_left_by(1);
    assert!(!*second.rendered(), "the rendered cursor is already at 0");
    assert!(*second.raw(), "the raw cursor still had room to step back");
    assert_eq!((c.rendered_cursor(), c.raw_cursor()), (0, 0));
}
