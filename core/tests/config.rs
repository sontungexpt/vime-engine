//! Where a session's settings come from: the shared config it is created from,
//! and the private config it can be given instead.

mod common;
use common::*;

use vime_engine::phonology::TonePlacement;
use vime_engine::{DefaultKeymap, Session, SessionFactory, Settings};

/// A session that never set a private config, so it reports as following the
/// shared one.
fn following() -> TelexSession {
    SessionFactory::telex(Settings::default()).new_session()
}

#[test]
fn a_new_session_follows_the_shared_config() {
    let mut session = following();
    assert_eq!(type_str(&mut session, "hoas"), MODERN_HOA);
}

#[test]
fn a_session_config_selects_the_tone_placement_at_construction() {
    let mut session =
        Session::with_isolated_config(make_config(DefaultKeymap::telex(), TonePlacement::Old));
    assert_eq!(type_str(&mut session, "hoas"), OLD_HOA);
}

#[test]
fn the_engines_keymap_reaches_every_session_it_creates() {
    let engine = SessionFactory::vni(Settings::default());
    let mut first = engine.new_session();
    let mut second = engine.new_session();

    assert_eq!(type_str(&mut first, "hoa1"), MODERN_HOA);
    assert_eq!(type_str(&mut second, "hoa1"), MODERN_HOA);
}

#[test]
fn a_private_config_overrides_only_its_own_session() {
    let engine = SessionFactory::telex(Settings::default());
    let mut following = engine.new_session();
    let mut private =
        engine.new_session_with(make_config(DefaultKeymap::telex(), TonePlacement::Old));

    assert_eq!(type_str(&mut following, "hoas"), MODERN_HOA);
    assert_eq!(type_str(&mut private, "hoas"), OLD_HOA);
}

#[test]
fn only_a_session_with_a_private_config_reports_as_private() {
    let engine = SessionFactory::telex(Settings::default());
    assert!(!engine.new_session().has_private_config());
    assert!(engine
        .new_session_with(make_config(DefaultKeymap::telex(), TonePlacement::Modern))
        .has_private_config());
}

// ─────────────────────────── Caret movement ───────────────────────────

/// The caret predicates decide whether a move is consumed or forwarded, so
/// their boundary conditions are what the frontend contract rests on: the
/// caret is always either movable left or movable right, and never both, and
/// `can_move_right` goes false only once the caret is at the very end.
#[test]
fn caret_predicates_track_the_raw_buffer() {
    let mut session = following();

    // Type "toa": the caret sits after the final character.
    type_str(&mut session, "toa");
    assert!(*session.move_cursor_left().rendered());
    assert!(*session.move_cursor_left().rendered());
    assert!(*session.move_cursor_left().rendered());

    // Caret now at position 0: further left is forwarded, not consumed.
    assert!(!*session.move_cursor_left().rendered());

    // And back to the end, where right is forwarded.
    assert!(*session.move_cursor_right().rendered());
    assert!(*session.move_cursor_right().rendered());
    assert!(*session.move_cursor_right().rendered());
    assert!(!*session.move_cursor_right().rendered());
}

/// Counted movement is the same movement, N times: the session delegates to the
/// composition rather than keeping a second path, so a frontend can implement
/// word-at-a-time jumps without asking the engine for new behaviour.
///
/// The over-count matters more than the exact step here — it is what says the
/// endpoints are reached rather than overshot.
#[test]
fn counted_caret_movement_reaches_the_same_endpoints() {
    let mut session = following();
    type_str(&mut session, "nga");
    assert_eq!(session.rendered_cursor(), 3);

    // Four back from a three-character word clamps at the start.
    assert!(*session.move_cursor_left_by(4).rendered());
    assert_eq!((session.rendered_cursor(), session.raw_cursor()), (0, 0));

    // Nothing left to give, so a frontend forwards the key.
    assert!(!*session.move_cursor_left_by(1).rendered());

    // And right by an over-count lands on the end, never past it.
    assert!(*session.move_cursor_right_by(99).rendered());
    assert_eq!((session.rendered_cursor(), session.raw_cursor()), (3, 3));
    assert!(!*session.move_cursor_right_by(99).rendered());
}

/// Moving the caret must not alter the rendered text, and resetting must put
/// the caret back to the start.
#[test]
fn caret_moves_leave_the_render_unchanged() {
    let mut session = following();
    type_str(&mut session, "hoas");
    let before = rendered_to_string(&mut session);

    for _ in 0..3 {
        session.move_cursor_left();
        session.move_cursor_left();
        session.move_cursor_right();
    }
    assert_eq!(rendered_to_string(&mut session), before);

    // After a reset the buffer is empty, so there is nowhere to move.
    session.reset();
    assert_eq!(rendered_to_string(&mut session), "");
}
