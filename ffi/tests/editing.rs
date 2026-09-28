//! Editing keys: backspace, delete and caret navigation through the C ABI.
//!
//! Deletions are transactional in the engine: a removal that would leave an
//! invalid nucleus is rolled back (e.g. `"vi"` minus the only vowel no-ops),
//! so these tests assert the observable action outcomes rather than assuming
//! raw parity.

mod common;

use common::{key_event, Engine};
use vime::{VimeAction, VimeKey};

#[test]
fn backspace_removes_last_character() {
    let mut engine = Engine::create().unwrap();
    engine.type_text("viet");

    let out = engine.process(key_event(VimeKey::Backspace));
    assert_eq!(out.action, VimeAction::UpdatePreedit);
    assert_eq!(out.rendered.as_deref(), Some("vie"));
}

#[test]
fn backspace_steps_back_through_a_syllable() {
    let mut engine = Engine::create().unwrap();
    engine.type_text("viet");

    engine.process(key_event(VimeKey::Backspace));
    let out = engine.process(key_event(VimeKey::Backspace));
    assert_eq!(out.action, VimeAction::UpdatePreedit);
    assert_eq!(out.rendered.as_deref(), Some("vi"));
}

#[test]
fn repeated_backspace_eventually_forwards() {
    let mut engine = Engine::create().unwrap();
    engine.type_text("viet");

    // Keep backspacing until the engine reports nothing left to delete.
    let forwarded =
        (0..8).any(|_| engine.process(key_event(VimeKey::Backspace)).action == VimeAction::Forward);
    assert!(forwarded, "backspacing to empty must eventually forward");
}

#[test]
fn delete_does_not_apply_at_end() {
    let mut engine = Engine::create().unwrap();
    engine.type_text("viet");
    assert_eq!(
        engine.process(key_event(VimeKey::Delete)).action,
        VimeAction::Forward
    );
}

#[test]
fn left_at_buffer_start_forwards() {
    let mut engine = Engine::create().unwrap();
    engine.type_text("viet");

    // Park the caret at the start first (the caret begins at the end after typing).
    for _ in 0..4 {
        engine.process(key_event(VimeKey::Left));
    }
    assert_eq!(
        engine.process(key_event(VimeKey::Left)).action,
        VimeAction::Forward
    );
}

#[test]
fn right_at_buffer_end_forwards() {
    let mut engine = Engine::create().unwrap();
    engine.type_text("viet");
    assert_eq!(
        engine.process(key_event(VimeKey::Right)).action,
        VimeAction::Forward
    );
}

/// The preedit is produced lazily: `vime_process_key` must not render it, and
/// `vime_preedit` must yield the same text the eager field used to carry. A
/// frontend that never asks must never pay for the render.
#[test]
fn preedit_is_fetched_lazily_and_stays_valid() {
    use std::ffi::CStr;

    let mut engine = Engine::create().unwrap();
    engine.type_text("viet");

    // SAFETY: the handle is live and owned by `engine`.
    let rendered = unsafe {
        let ptr = vime::vime_preedit(engine.raw());
        assert!(!ptr.is_null());
        // Asking again yields the same cached text, still valid.
        let again = vime::vime_preedit(engine.raw());
        assert_eq!(ptr, again, "repeated reads hit the cache");
        CStr::from_ptr(ptr).to_str().unwrap().to_string()
    };
    assert_eq!(rendered, "viet");

    // The cache is invalidated by the next state change, and the new text is
    // rendered on demand.
    engine.process(key_event(VimeKey::Backspace));
    // SAFETY: as above.
    let after = unsafe {
        let ptr = vime::vime_preedit(engine.raw());
        assert!(!ptr.is_null());
        CStr::from_ptr(ptr).to_str().unwrap().to_string()
    };
    assert_eq!(after, "vie");
}
