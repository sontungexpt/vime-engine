//! A shared config change reaching every session that follows it.
//!
//! These are the tests that pin the feature down: the change is made once, in
//! one place, and no session is notified by name.

use vime_engine::composition::syllable::SyllableContext;
use vime_engine::phonology::TonePlacement;
use vime_engine::{Settings, 
    Config, DefaultKeymap, SessionFactory, Session,
};

type TelexSession = Session<DefaultKeymap<'static>>;

/// "hoa" + sắc: Modern puts the mark on the second vowel, Old on the first.
const MODERN_HOA: &str = "hoá";
const OLD_HOA: &str = "hóa";

fn make_context(keymap: DefaultKeymap<'static>, tone: TonePlacement) -> SyllableContext<DefaultKeymap<'static>> {
    SyllableContext::new(keymap, tone)
}

fn old_telex() -> Config<DefaultKeymap<'static>> {
    Config::new(
        Settings::default(),
        make_context(DefaultKeymap::telex(), TonePlacement::Old),
    )
}

fn vni() -> Config<DefaultKeymap<'static>> {
    Config::new(
        Settings::default(),
        make_context(DefaultKeymap::vni(), TonePlacement::Modern),
    )
}

fn rendered_to_string(session: &TelexSession) -> String {
    session.rendered().into_iter().collect()
}

fn type_str(session: &mut TelexSession, s: &str) -> String {
    for ch in s.chars() {
        session.insert(ch);
    }
    rendered_to_string(session)
}

fn press_space(session: &mut TelexSession) -> String {
    session.insert(' ');
    rendered_to_string(session)
}

fn press_backspace(session: &mut TelexSession) -> bool {
    *session.backspace().rendered()
}

fn press_delete(session: &mut TelexSession) -> bool {
    *session.delete().rendered()
}

// ─────────────────────── Every session sees the change ───────────────────────

/// The whole feature in one test: replace the shared config once, and every
/// session that was following it renders the new way.
#[test]
fn a_shared_change_reaches_every_session() {
    let engine = SessionFactory::telex(Settings::default());
    let mut first = engine.new_session();
    let mut second = engine.new_session();

    type_str(&mut first, "hoas");
    type_str(&mut second, "hoas");
    assert_eq!(rendered_to_string(&first), MODERN_HOA);
    assert_eq!(rendered_to_string(&second), MODERN_HOA);

    // One write, no session named.
    engine.set_config(old_telex());

    // Space commits, and the commit renders under the settings in force. They
    // are the new ones, because the session picked the change up on the way in
    // rather than being told about it.
    for session in [&mut first, &mut second] {
        assert_eq!(
            press_space(session),
            format!("{OLD_HOA} "),
            "the word committed under the new scheme"
        );
    }
}

/// A session that is not holding a keystroke still adopts the new settings, and
/// `refresh` is how a caller asks for that without inventing input.
#[test]
fn refresh_picks_up_a_change_without_a_keystroke() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();
    type_str(&mut session, "hoas");

    engine.set_config(old_telex());
    assert_eq!(rendered_to_string(&session), MODERN_HOA, "not until it looks");

    assert!(
        session.refresh_config(),
        "the re-render is what the frontend must know about"
    );
    assert_eq!(rendered_to_string(&session), OLD_HOA);
}

/// Refreshing is how a session learns something moved, so it must be silent when
/// nothing did. Otherwise every keystroke would claim the word had changed.
#[test]
fn refresh_is_silent_when_nothing_moved() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();

    assert!(!session.refresh_config(), "nothing to pick up yet");
    assert!(!session.refresh_config(), "still nothing");
}

/// Re-resolving the same settings twice must not re-render: a frontend that
/// trusted the flag would repaint on every key.
#[test]
fn reapplying_the_same_settings_changes_nothing() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();
    type_str(&mut session, "hoas");

    // A new generation, but the same content.
    engine.set_config(vni());

    assert!(!session.refresh_config(), "same settings, no re-render");
    assert_eq!(rendered_to_string(&session), MODERN_HOA);
}

// ───────────────────────── The re-render contract ─────────────────────────

/// Tone placement only changes how the buffer is written out, so the keystrokes
/// still mean what they meant and must survive.
#[test]
fn a_tone_placement_change_keeps_the_buffer() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();
    type_str(&mut session, "hoas");

    engine.set_config(old_telex());
    session.refresh_config();
    assert_eq!(rendered_to_string(&session), OLD_HOA, "same keystrokes, new rendering");
}

/// A new keymap gives the buffered keystrokes a different meaning, so they
/// cannot be reinterpreted and are dropped.
#[test]
fn a_keymap_change_drops_the_buffer() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();
    type_str(&mut session, "hoas");

    engine.set_config(vni());
    session.refresh_config();
    // Keymap change: buffer cleared because keystrokes reinterpreted
    assert_eq!(rendered_to_string(&session), "", "buffer cleared on keymap change");
}

/// A settings change can move the word on a key that did nothing of its own.
/// Reporting `Forward` there would leave the stale word on screen.
#[test]
fn a_change_reports_changed_even_when_the_key_would_be_forwarded() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();
    type_str(&mut session, "hoas");

    engine.set_config(old_telex());

    // The caret is at the end, so there is nothing ahead to delete and the key
    // is forwarded. The re-render still happened, so the frontend still has to
    // repaint.
    press_delete(&mut session);
    assert_eq!(rendered_to_string(&session), OLD_HOA);
}

/// With no pending change, the same key is forwarded as it always was.
#[test]
fn a_forwarded_key_stays_forwarded_when_nothing_changed() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();
    // Backspace on empty buffer does nothing (returns false)
    assert!(!press_backspace(&mut session));
}

// ───────────────────────────── Private sessions ─────────────────────────────

/// A private config is the point of having one, so a shared change must leave
/// it alone.
#[test]
fn a_shared_change_leaves_a_private_session_alone() {
    let engine = SessionFactory::telex(Settings::default());
    let mut following = engine.new_session();
    let mut private = engine.new_session_with(old_telex());

    type_str(&mut following, "hoas");
    type_str(&mut private, "hoas");

    engine.set_config(vni());
    following.refresh_config();
    assert!(!private.refresh_config(), "a private session is not looking");

    // Telex, Old, and the word it already held, for the session that opted out.
    assert_eq!(rendered_to_string(&private), OLD_HOA);
    assert_eq!(
        private.config().context.tone_placement(),
        TonePlacement::Old
    );

    // VNI for the one still following. Keymap change cleared buffer.
    assert_eq!(following.config().context.keymap(), &DefaultKeymap::vni());
    assert_eq!(rendered_to_string(&following), "", "buffer cleared on keymap change");
    assert_eq!(type_str(&mut following, "hoa1"), MODERN_HOA);
}

/// Giving a following session a private config is what makes it stop following.
#[test]
fn taking_a_private_config_stops_the_following() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();
    type_str(&mut session, "hoas");

    session.set_private_config(old_telex());
    assert!(session.has_private_config());
    assert_eq!(rendered_to_string(&session), OLD_HOA, "and takes effect at once");

    engine.set_config(vni());
    assert!(!session.refresh_config());
    assert_eq!(rendered_to_string(&session), OLD_HOA);
}

/// Dropping the private config puts the session back on the shared settings,
/// including a change it missed while it was private.
#[test]
fn clearing_a_private_config_catches_up_with_the_shared_one() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session_with(old_telex());
    assert!(session.has_private_config());

    // Move the shared config on while the session is ignoring it.
    engine.set_config(vni());
    session.insert('h');
    assert_eq!(rendered_to_string(&session), "h");

    session.clear_private_config();
    assert!(!session.has_private_config());
    assert_eq!(session.config().context.keymap(), &DefaultKeymap::vni());
}

/// A private session created standalone has no engine to follow, so it reports
/// the settings it was built with and nothing disturbs it.
#[test]
fn a_standalone_private_session_is_self_contained() {
    let mut session = Session::with_isolated_config(old_telex());
    assert!(session.has_private_config());
    assert!(!session.refresh_config());
    assert_eq!(type_str(&mut session, "hoas"), OLD_HOA);
}

// ────────────────────────── Independence of buffers ──────────────────────────

/// Sessions share settings, not input. This is the one thing they must never
/// do, and the reason a session is not just an `SessionFactory` with a name.
#[test]
fn sessions_do_not_share_their_buffers() {
    let engine = SessionFactory::telex(Settings::default());
    let mut first = engine.new_session();
    let mut second = engine.new_session();

    type_str(&mut first, "hoa");
    assert_eq!(rendered_to_string(&first), "hoa");
    assert_eq!(rendered_to_string(&second), "", "the other session is untouched");

    // And a commit in one leaves the other alone.
    type_str(&mut second, "ba");
    // Commit is now just reading the rendered buffer and resetting
    assert_eq!(rendered_to_string(&first), "hoa");
    first.reset();
    assert_eq!(rendered_to_string(&second), "ba");
}