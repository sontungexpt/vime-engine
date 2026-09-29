//! Engine lifecycle: create/destroy, input-method and tone-placement
//! switching, reset and commit semantics.

mod common;

use common::{SessionFactory, Session, char_event, key_event};
use vime::{VimeAction, VimeConfig, VimeInputMethod, VimeKey, VimeTonePlacement};

#[test]
fn create_and_destroy_factory() {
    let factory = SessionFactory::create().expect("vime_session_factory_create must succeed");
    drop(factory);
}

#[test]
fn create_factory_with_all_methods() {
    for method in [
        VimeInputMethod::Telex,
        VimeInputMethod::Vni,
        VimeInputMethod::Viqr,
    ] {
        let factory = SessionFactory::create_with(VimeConfig {
            input_method: method,
            tone_placement: VimeTonePlacement::Modern,
            auto_restore_english: true,
        }).expect("factory must be created");
        drop(factory);
    }
}

#[test]
fn commit_on_empty_buffer_forwards() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    let out = session.commit();
    assert_eq!(out.action, VimeAction::Forward);
    assert!(out.rendered.is_none());
    assert!(out.commit.is_none());
}

#[test]
fn reset_returns_updated_empty_buffer() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("viet");
    let out = session.reset();
    assert_eq!(out.action, VimeAction::Changed);
    assert_eq!(out.rendered.as_deref(), Some(""));
    assert!(out.commit.is_none());
}

#[test]
fn switching_method_clears_pending_buffer() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("too");
    let out = session.set_input_method(VimeInputMethod::Vni);
    assert_eq!(out.action, VimeAction::Changed);
    assert_eq!(out.rendered.as_deref(), Some(""));
    assert!(out.commit.is_none());
}

/// `vime_session_reset` invalidates the cached word on its own path.
#[test]
fn reset_invalidates_the_cached_word() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("viet");

    // Prime the cache so a stale read would be observable.
    let before = session.word().unwrap();
    assert_eq!(before, "viet");

    assert!(session.reset_flag(), "reset must report success");

    // The word must now be empty, not the cached "viet".
    let after = session.word().unwrap();
    assert_eq!(after, "");
}

#[test]
fn switching_placement_renders_without_text_commit() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("hoas");
    let out = session.set_tone_placement(VimeTonePlacement::Old);
    assert_eq!(out.action, VimeAction::Changed);
    assert_eq!(out.rendered.as_deref(), Some("hóa"));
    assert!(out.commit.is_none());
}

#[test]
fn switch_then_commit_uses_new_placement() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("hoas");
    session.set_tone_placement(VimeTonePlacement::Old);
    let committed = session.commit();
    assert_eq!(committed.action, VimeAction::Commit);
    assert_eq!(committed.commit.as_deref(), Some("hóa"));
}

#[test]
fn navigate_empty_buffer_forwards() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    assert_eq!(
        session.process(key_event(VimeKey::Left)).action,
        VimeAction::Forward
    );
    assert_eq!(
        session.process(key_event(VimeKey::Right)).action,
        VimeAction::Forward
    );
    assert_eq!(
        session.process(key_event(VimeKey::Backspace)).action,
        VimeAction::Forward
    );
    assert_eq!(
        session.process(key_event(VimeKey::Delete)).action,
        VimeAction::Forward
    );
}

/// A state change made outside `vime_session_process_key` must still invalidate the
/// cached word, or a lazy `vime_session_render` hands back text from before the
/// change. Both config setters are state changes of exactly that kind.
#[test]
fn config_changes_invalidate_the_cached_word() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("hoas");

    // Cached under the modern scheme.
    assert_eq!(session.word().expect("a word must be reported"), "hoá");

    // Switching placement re-renders the pending vowels; the cache must not
    // still answer with the modern rendering.
    session.set_tone_placement(VimeTonePlacement::Old);
    assert_eq!(session.word().expect("a word must be reported"), "hóa");

    // Switching the method clears the buffer; the cache must not still answer
    // with the pre-clear text.
    session.type_text("too");
    session.set_input_method(VimeInputMethod::Vni);
    assert_eq!(session.word().expect("a word must be reported"), "");
}

/// The word buffer is reused across renders rather than reallocated, so the
/// returned pointer must stay correct after a state change: the new text has to
/// land in the same live buffer the frontend is still holding.
#[test]
fn reused_word_buffer_serves_the_current_text() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    let typed = session.type_text("tiengs");

    // The buffer must agree with what typing last reported, and repeated reads
    // must not disturb it.
    assert_eq!(session.word().expect("a word must be reported"), typed);
    assert_eq!(session.word().expect("a word must be reported"), typed);
    assert_eq!(session.word().expect("a word must be reported"), typed);

    // Each backspace must land new text in the same reused buffer.
    let mut previous = typed;
    for _ in 0..4 {
        session.process(key_event(VimeKey::Backspace));
        let now = session.word().expect("a word must be reported");
        assert_ne!(now, previous, "backspace must change the word");
        previous = now;
    }

    // And an emptied buffer must render as empty, not as stale text.
    for _ in 0..8 {
        session.process(key_event(VimeKey::Backspace));
    }
    assert_eq!(session.word().expect("a word must be reported"), "");
}