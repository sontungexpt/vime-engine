//! Handles, configuration, and the shared-versus-private config distinction.

mod common;

use vime::{VimeConfig, VIME_INPUT_METHOD_TELEX, VIME_INPUT_METHOD_VNI, VIME_TONE_PLACEMENT_MODERN, VIME_TONE_PLACEMENT_OLD};

/// A factory's sessions follow the shared config, and a change reaches them.
///
/// This is the interesting one: the change happens through the *factory*, and the
/// session has to notice it on its next query without anyone calling anything on
/// it. It is the behaviour most likely to rot, because the cache in between has to
/// be invalidated by the observation and not only by an edit.
#[test]
fn a_shared_config_change_reaches_existing_sessions() {
    let old = VimeConfig {
        tone_placement: VIME_TONE_PLACEMENT_OLD,
        ..Default::default()
    };
    let new = VimeConfig {
        tone_placement: VIME_TONE_PLACEMENT_MODERN,
        ..Default::default()
    };

    let mut factory = common::Factory::create_with(&old).unwrap();
    let mut session = factory.open_session();
    assert_eq!(session.type_text("hoas"), "hóa", "Old places sắc first");

    assert!(factory.set_config(&new));
    assert_eq!(
        session.render_text(),
        "hoá",
        "the next query must show the new config, not the cached old render"
    );
    assert!(session.render_state().is_valid_vietnamese);
}

/// The same change, seen through the full snapshot rather than through
/// `render_text`, because that path has its own cache to invalidate.
#[test]
fn a_shared_config_change_reaches_the_snapshot() {
    let old = VimeConfig {
        input_method: VIME_INPUT_METHOD_TELEX,
        ..Default::default()
    };
    let new = VimeConfig {
        input_method: VIME_INPUT_METHOD_VNI,
        ..Default::default()
    };

    let mut factory = common::Factory::create_with(&old).unwrap();
    let mut session = factory.open_session();
    // The same word, two keymaps. Telex spells sắc with `s`, VNI with `1`; both
    // must render as `toán`, which is only true if the session really swapped.
    assert_eq!(session.type_text("toans"), "toán");
    assert_eq!(session.render_raw_text(), "toans");

    assert!(factory.set_config(&new));
    assert_eq!(
        session.render_raw_text(),
        "toans",
        "the raw keystrokes do not move"
    );
    assert_eq!(session.type_text("toan1"), "toán", "VNI: 1 is sắc");
    assert_eq!(session.render_raw_text(), "toan1");

    // The snapshot path, which has its own view of the same state.
    assert_eq!(session.render_state().text, "toán");
    assert_eq!(session.render_state().raw_text, "toan1");
}

/// A session with a private config ignores the shared one, and going back to the
/// shared one picks up whatever it says at that point.
#[test]
fn a_private_config_shadows_the_shared_one() {
    let shared_old = VimeConfig {
        tone_placement: VIME_TONE_PLACEMENT_OLD,
        ..Default::default()
    };
    let private_modern = VimeConfig {
        tone_placement: VIME_TONE_PLACEMENT_MODERN,
        ..Default::default()
    };

    let mut factory = common::Factory::create_with(&shared_old).unwrap();
    let mut session = factory.open_session_with(&private_modern);
    assert_eq!(session.type_text("hoas"), "hoá", "the private config wins");

    // Changing the shared config must not disturb the private one.
    let shared_modern = VimeConfig {
        tone_placement: VIME_TONE_PLACEMENT_MODERN,
        ..Default::default()
    };
    assert!(factory.set_config(&shared_modern));
    assert_eq!(session.type_text("hoas"), "hoá");

    // Clearing the private config puts it back on the shared one, which is now
    // Modern, so the render is unchanged — and that is the point: the session
    // really did move.
    assert!(session.clear_config());
    assert_eq!(session.type_text("hoas"), "hoá");
}

/// A session's own config can be replaced and cleared too.
#[test]
fn a_private_config_can_be_replaced() {
    let modern = VimeConfig::default();
    let old = VimeConfig {
        tone_placement: VIME_TONE_PLACEMENT_OLD,
        ..Default::default()
    };

    let mut factory = common::Factory::create().unwrap();
    let mut session = factory.open_session_with(&modern);
    assert_eq!(session.type_text("hoas"), "hoá");

    assert!(session.set_config(&old));
    assert_eq!(session.type_text("hoas"), "hóa");

    assert!(session.clear_config());
    assert_eq!(
        session.type_text("hoas"),
        "hoá",
        "back on the shared Modern"
    );
}

/// A session outlives the factory that made it, which is what makes a session
/// handle worth having on its own: a frontend can keep the composing session
/// across a settings reload that replaces the factory.
#[test]
fn a_session_outlives_its_factory() {
    let mut session = {
        let mut factory = common::Factory::create().unwrap();
        let mut session = factory.open_session();
        assert_eq!(session.type_text("tieengs"), "tiếng");
        session
    };
    // The factory is gone; the session still works.
    assert_eq!(session.render_text(), "tiếng");
    assert_eq!(session.type_text("toan"), "toan");
    assert!(session.insert('a'));
}

/// Independent sessions do not share a buffer, which is the whole point of
/// having a factory at all.
#[test]
fn sessions_from_one_factory_are_independent() {
    let mut factory = common::Factory::create().unwrap();
    let mut a = factory.open_session();
    let mut b = factory.open_session();

    assert_eq!(a.type_text("tieengs"), "tiếng");
    assert_eq!(b.type_text("toan"), "toan");

    assert_eq!(a.render_text(), "tiếng");
    assert_eq!(b.render_text(), "toan");

    a.reset();
    assert_eq!(a.render_text(), "");
    assert_eq!(b.render_text(), "toan", "reset is per session");
}

/// Two sessions with different private configs out of one factory.
#[test]
fn private_configs_differ_within_one_factory() {
    let modern = VimeConfig::default();
    let old = VimeConfig {
        tone_placement: VIME_TONE_PLACEMENT_OLD,
        ..Default::default()
    };
    let mut factory = common::Factory::create().unwrap();
    let mut modern_session = factory.open_session_with(&modern);
    let mut old_session = factory.open_session_with(&old);

    assert_eq!(modern_session.type_text("hoas"), "hoá");
    assert_eq!(old_session.type_text("hoas"), "hóa");
}

/// A factory with no config and one built from `VIME_CONFIG_INIT` behave the
/// same, so a caller may pass the macro or pass nothing.
#[test]
fn init_and_no_config_agree() {
    let mut a = common::Factory::create().unwrap().open_session();
    let mut b = common::Factory::create_with(&common::config_init())
        .unwrap()
        .open_session();
    for word in ["tieengs", "hoas", "toan", "dduongf", "nguoiwf"] {
        assert_eq!(a.type_text(word), b.type_text(word), "typing {word:?}");
    }
}

/// The string a caller gets back is stable until the next call on that session,
/// which is what the header promises. Reading it twice without editing returns
/// the same pointer and the same bytes.
#[test]
fn the_text_pointer_is_stable_until_the_next_edit() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("tieengs");

    let first = session.render_text_ptr().unwrap();
    let second = session.render_text_ptr().unwrap();
    assert_eq!(first, second, "no edit, so no re-render, so no new pointer");
    assert_eq!(first, "tiếng");

    // An edit is what makes the pointer move.
    session.insert('a');
    let third = session.render_text_ptr().unwrap();
    assert_ne!(first, third);
    assert_eq!(third, "tiếnga");
}

/// The snapshot's own pointer is likewise stable, and it is the same object
/// across calls because it lives in the handle.
#[test]
fn the_snapshot_pointer_is_stable() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("tieengs");
    let first = session.render_state_ptr();
    let second = session.render_state_ptr();
    assert_eq!(first, second);
    assert!(!first.is_null());
}
