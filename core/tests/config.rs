//! Where a session's settings come from: the shared config it is created from,
//! and the private config it can be given instead.

use vime_engine::composition::syllable::SyllableContext;
use vime_engine::phonology::TonePlacement;
use vime_engine::{Settings, 
    Config, DefaultKeymap, SessionFactory, Key, KeyEvent, KeyStates, Result, Session,
};

type TelexSession = Session<DefaultKeymap<'static>>;

/// A session that never set a private config, so it reports as following the
/// shared one.
fn following() -> TelexSession {
    SessionFactory::telex(Settings::default()).new_session()
}

fn type_str(session: &mut TelexSession, s: &str) -> String {
    for ch in s.chars() {
        session.process_key(KeyEvent {
            key: Key::Character(ch),
            states: KeyStates::empty(),
        });
    }
    session.parsed()
}

// "hoa" + sắc: the two schemes place the mark on different vowels
// (Modern 2-vowel rule: second vowel; Old open syllable: first vowel).
const MODERN_HOA: &str = "hoá";
const OLD_HOA: &str = "hóa";

#[test]
fn a_new_session_follows_the_shared_config() {
    let mut session = following();
    assert_eq!(type_str(&mut session, "hoas"), MODERN_HOA);
}

#[test]
fn a_session_config_selects_the_tone_placement_at_construction() {
    let mut session = Session::with_isolated_config(Config::new(
        Settings::default(),
        SyllableContext::new(DefaultKeymap::telex(), TonePlacement::Old),
    ));
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
    let mut private = engine.new_session_with(Config::new(
        Settings::default(),
        SyllableContext::new(DefaultKeymap::telex(), TonePlacement::Old),
    ));

    assert_eq!(type_str(&mut following, "hoas"), MODERN_HOA);
    assert_eq!(type_str(&mut private, "hoas"), OLD_HOA);
}

#[test]
fn only_a_session_with_a_private_config_reports_as_private() {
    let engine = SessionFactory::telex(Settings::default());
    assert!(!engine.new_session().has_private_config());
    assert!(
        engine
            .new_session_with(Config::from_keymap(
                Settings::default(),
                DefaultKeymap::telex()
            ))
            .has_private_config()
    );
}

// ─────────────────────────── Caret movement ───────────────────────────

/// Presses a non-character key and reports what the session did with it.
fn press(session: &mut TelexSession, key: Key) -> Result {
    session.process_key(KeyEvent {
        key,
        states: KeyStates::empty(),
    })
}

/// The caret predicates decide whether a move is consumed or forwarded, so
/// their boundary conditions are what the frontend contract rests on: the
/// caret is always either movable left or movable right, and never both, and
/// `can_move_right` goes false only once the caret is at the very end.
#[test]
fn caret_predicates_track_the_raw_buffer() {
    let mut session = following();

    // Type "toa": the caret sits after the final character.
    type_str(&mut session, "toa");
    assert_eq!(press(&mut session, Key::Left), Result::CursorMoved);
    assert_eq!(press(&mut session, Key::Left), Result::CursorMoved);
    assert_eq!(press(&mut session, Key::Left), Result::CursorMoved);

    // Caret now at position 0: further left is forwarded, not consumed.
    assert_eq!(press(&mut session, Key::Left), Result::Forward);

    // And back to the end, where right is forwarded.
    assert_eq!(press(&mut session, Key::Right), Result::CursorMoved);
    assert_eq!(press(&mut session, Key::Right), Result::CursorMoved);
    assert_eq!(press(&mut session, Key::Right), Result::CursorMoved);
    assert_eq!(press(&mut session, Key::Right), Result::Forward);
}

/// Moving the caret must not alter the rendered text, and resetting must put
/// the caret back to the start.
#[test]
fn caret_moves_leave_the_render_unchanged() {
    let mut session = following();
    type_str(&mut session, "hoas");
    let before = session.parsed();

    for key in [Key::Left, Key::Left, Key::Right] {
        press(&mut session, key);
    }
    assert_eq!(session.parsed(), before);

    // After a reset the buffer is empty, so there is nowhere to move.
    session.reset();
    assert_eq!(session.parsed(), "");
}
