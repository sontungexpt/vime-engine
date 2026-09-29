//! Editing keys: backspace, delete and caret navigation through the C ABI.
//!
//! Deletions are transactional in the engine: a removal that would leave an
//! invalid nucleus is rolled back (e.g. `"vi"` minus the only vowel no-ops),
//! so these tests assert the observable action outcomes rather than assuming
//! raw parity.

mod common;

use common::{SessionFactory, Session, char_event, key_event};
use vime::{VimeAction, VimeKey};

#[test]
fn backspace_removes_last_character() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("viet");

    let out = session.process(key_event(VimeKey::Backspace));
    assert_eq!(out.action, VimeAction::Changed);
    assert_eq!(out.rendered.as_deref(), Some("vie"));
}

#[test]
fn backspace_steps_back_through_a_syllable() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("viet");

    session.process(key_event(VimeKey::Backspace));
    let out = session.process(key_event(VimeKey::Backspace));
    assert_eq!(out.action, VimeAction::Changed);
    assert_eq!(out.rendered.as_deref(), Some("vi"));
}

#[test]
fn repeated_backspace_eventually_forwards() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("viet");

    // Keep backspacing until the engine reports nothing left to delete.
    let forwarded =
        (0..8).any(|_| session.process(key_event(VimeKey::Backspace)).action == VimeAction::Forward);
    assert!(forwarded, "backspacing to empty must eventually forward");
}

#[test]
fn delete_does_not_apply_at_end() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("viet");
    assert_eq!(
        session.process(key_event(VimeKey::Delete)).action,
        VimeAction::Forward
    );
}

#[test]
fn left_at_buffer_start_forwards() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("viet");

    // Park the caret at the start first (the caret begins at the end after typing).
    for _ in 0..4 {
        session.process(key_event(VimeKey::Left));
    }
    assert_eq!(
        session.process(key_event(VimeKey::Left)).action,
        VimeAction::Forward
    );
}

#[test]
fn right_at_buffer_end_forwards() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("viet");
    assert_eq!(
        session.process(key_event(VimeKey::Right)).action,
        VimeAction::Forward
    );
}

/// The word is produced lazily: `vime_session_process_key` must not render it, and
/// `vime_session_render` must yield the same text the eager field used to carry. A
/// frontend that never asks must never pay for the render.
#[test]
fn parsed_is_fetched_lazily_and_stays_valid() {
    use std::ffi::CStr;

    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("viet");

    // SAFETY: the handle is live and owned by `session`.
    let rendered = unsafe {
        let ptr = vime::vime_session_render(session.handle());
        assert!(!ptr.is_null());
        // Asking again yields the same cached text, still valid.
        let again = vime::vime_session_render(session.handle());
        assert_eq!(ptr, again, "repeated reads hit the cache");
        unsafe { CStr::from_ptr(ptr).to_str() }.unwrap().to_string()
    };
    assert_eq!(rendered, "viet");

    // The cache is invalidated by the next state change, and the new text is
    // rendered on demand.
    session.process(key_event(VimeKey::Backspace));
    // SAFETY: as above.
    let after = unsafe {
        let ptr = vime::vime_session_render(session.handle());
        assert!(!ptr.is_null());
        unsafe { CStr::from_ptr(ptr).to_str() }.unwrap().to_string()
    };
    assert_eq!(after, "vie");
}