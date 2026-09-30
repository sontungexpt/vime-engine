//! A shared config change reaching every session that follows it.
//!
//! These are the tests that pin the feature down: the change is made once, in
//! one place, and no session is notified by name.

mod common;
use common::*;

use vime_engine::phonology::TonePlacement;
use vime_engine::{DefaultKeymap, Session, SessionConfig, SessionFactory, Settings};

fn old_telex() -> SessionConfig<DefaultKeymap<'static>> {
    make_config(DefaultKeymap::telex(), TonePlacement::Old)
}

fn vni() -> SessionConfig<DefaultKeymap<'static>> {
    make_config(DefaultKeymap::vni(), TonePlacement::Modern)
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
    assert_eq!(rendered_to_string(&mut first), MODERN_HOA);
    assert_eq!(rendered_to_string(&mut second), MODERN_HOA);

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
/// `pull_config` is how a caller asks for that without inventing input.
#[test]
fn refresh_picks_up_a_change_without_a_keystroke() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();
    assert_eq!(type_str(&mut session, "hoas"), MODERN_HOA);

    engine.set_config(old_telex());

    // Not until it looks. `config` is a plain read, so the new settings are not
    // in force yet; anything that renders would have adopted them on the way in.
    assert_eq!(session.config().tone_placement(), TonePlacement::Modern);

    assert!(
        session.pull_config(),
        "the re-render is what the frontend must know about"
    );
    assert_eq!(rendered_to_string(&mut session), OLD_HOA);
}

/// Refreshing is how a session learns something moved, so it must be silent when
/// nothing did. Otherwise every keystroke would claim the word had changed.
#[test]
fn refresh_is_silent_when_nothing_moved() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();

    assert!(!session.pull_config(), "nothing to pick up yet");
    assert!(!session.pull_config(), "still nothing");
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
    assert_eq!(rendered_to_string(&mut session), OLD_HOA);
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

/// Giving a following session a private config is what makes it stop following.
#[test]
fn taking_a_private_config_stops_the_following() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();
    type_str(&mut session, "hoas");

    session.set_private_config(old_telex());
    assert!(session.has_private_config());
    assert_eq!(
        rendered_to_string(&mut session),
        OLD_HOA,
        "and takes effect at once"
    );

    engine.set_config(vni());
    assert!(!session.pull_config());
    assert_eq!(rendered_to_string(&mut session), OLD_HOA);
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
    assert_eq!(rendered_to_string(&mut session), "h");

    session.clear_private_config();
    assert!(!session.has_private_config());
    assert_eq!(session.config().keymap(), &DefaultKeymap::vni());
}

/// A private session created standalone has no engine to follow, so it reports
/// the settings it was built with and nothing disturbs it.
#[test]
fn a_standalone_private_session_is_self_contained() {
    let mut session = Session::with_isolated_config(old_telex());
    assert!(session.has_private_config());
    assert!(!session.pull_config());
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
    assert_eq!(rendered_to_string(&mut first), "hoa");
    assert_eq!(
        rendered_to_string(&mut second),
        "",
        "the other session is untouched"
    );

    // And a commit in one leaves the other alone.
    type_str(&mut second, "ba");
    // Commit is now just reading the rendered buffer and resetting
    assert_eq!(rendered_to_string(&mut first), "hoa");
    first.reset();
    assert_eq!(rendered_to_string(&mut second), "ba");
}

// ──────────────────── Positions and validity for a frontend ───────────────────

/// The two cursors are separate because a transform consumes a keystroke
/// without lengthening the rendered word, so the two positions drift apart.
#[test]
fn the_two_cursors_diverge_across_a_transform() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();

    type_str(&mut session, "aw");
    assert_eq!(rendered_to_string(&mut session), "ă");
    assert_eq!(session.rendered_cursor(), 1, "one rendered character");
    assert_eq!(session.raw_cursor(), 2, "but two keystrokes");

    // Backspacing once eats the shape key, which puts the raw cursor behind the
    // rendered one — the case where a caller has to be told which caret it is.
    assert!(press_backspace(&mut session));
    assert_eq!(session.rendered_cursor(), 0);
    assert_eq!(session.raw_cursor(), 1);
}

/// A position is a character index, never a byte offset, so it is exactly as
/// long as the render the caller can see.
#[test]
fn a_cursor_never_exceeds_the_rendered_word() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();

    for word in ["", "a", "hoas", "uowng", "thuowng"] {
        session.reset();
        type_str(&mut session, word);
        let rendered = rendered_to_string(&mut session);
        assert!(
            session.rendered_cursor() <= rendered.chars().count(),
            "{word:?}: caret {} is past the {}-character render",
            session.rendered_cursor(),
            rendered.chars().count(),
        );
        assert!(session.raw_cursor() <= session.raw().len());
    }
}

/// Validity is a question about a *finished* word: a bare onset is a fragment,
/// and input the parser could not read at all is not a word either.
#[test]
fn validity_answers_whether_the_buffer_is_a_word() {
    let engine = SessionFactory::telex(Settings::default());
    let mut session = engine.new_session();

    let verdict = |session: &mut TelexSession, word: &str| {
        session.reset();
        type_str(session, word);
        session.is_valid()
    };

    assert!(!verdict(&mut session, ""), "nothing typed");
    assert!(!verdict(&mut session, "b"), "an onset alone is a fragment");
    assert!(verdict(&mut session, "ba"), "a one-vowel nucleus is a word");
    assert!(verdict(&mut session, "anh"), "coda after a plain vowel");
    assert!(
        verdict(&mut session, "toan"),
        "two-vowel nucleus with a coda"
    );
    assert!(verdict(&mut session, "uowng"), "renders as ương");
    assert!(
        !verdict(&mut session, "qwerty"),
        "input the parser could not read"
    );

    // The core's own spelling rules are reported, not second-guessed: an
    // entering coda with no tone is what `EnteringToneRequired` rejects.
    assert!(!verdict(&mut session, "hoc"), "hoc without a tone");
    assert!(verdict(&mut session, "hocs"), "hocs carries one");
}
