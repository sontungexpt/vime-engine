//! Engine lifecycle: create/destroy, input-method and tone-placement
//! switching, reset and commit semantics.

mod common;

use common::{key_event, Engine};
use vime::{VimeAction, VimeInputMethod, VimeKey, VimeTonePlacement};

#[test]
fn create_and_destroy() {
    let engine = Engine::create().expect("vime_create must succeed");
    drop(engine);
}

#[test]
fn create_with_all_methods() {
    for method in [
        VimeInputMethod::Telex,
        VimeInputMethod::Vni,
        VimeInputMethod::Viqr,
    ] {
        let engine =
            Engine::create_with(method, VimeTonePlacement::Modern).expect("engine must be created");
        drop(engine);
    }
}

#[test]
fn commit_on_empty_buffer_forwards() {
    let mut engine = Engine::create().unwrap();
    let out = engine.commit();
    assert_eq!(out.action, VimeAction::Forward);
    assert!(out.rendered.is_none());
    assert!(out.commit.is_none());
}

#[test]
fn reset_returns_updated_empty_buffer() {
    let mut engine = Engine::create().unwrap();
    engine.type_text("viet");
    let out = engine.reset();
    assert_eq!(out.action, VimeAction::Changed);
    assert_eq!(out.rendered.as_deref(), Some(""));
    assert!(out.commit.is_none());
}

#[test]
fn switching_method_clears_pending_buffer() {
    let mut engine = Engine::create().unwrap();
    engine.type_text("too");
    let out = engine.set_input_method(VimeInputMethod::Vni);
    assert_eq!(out.action, VimeAction::Changed);
    assert_eq!(out.rendered.as_deref(), Some(""));
    assert!(out.commit.is_none());
}

#[test]
fn switching_placement_renders_without_text_commit() {
    let mut engine = Engine::create_with(VimeInputMethod::Telex, VimeTonePlacement::Modern)
        .expect("engine must be created");
    engine.type_text("hoas");
    let out = engine.set_tone_placement(VimeTonePlacement::Old);
    assert_eq!(out.action, VimeAction::Changed);
    assert_eq!(out.rendered.as_deref(), Some("hóa"));
    assert!(out.commit.is_none());
}

#[test]
fn switch_then_commit_uses_new_placement() {
    let mut engine = Engine::create_with(VimeInputMethod::Telex, VimeTonePlacement::Modern)
        .expect("engine must be created");
    engine.type_text("hoas");
    engine.set_tone_placement(VimeTonePlacement::Old);
    let committed = engine.commit();
    assert_eq!(committed.action, VimeAction::Commit);
    assert_eq!(engine.committed().as_deref(), Some("hóa"));
}

#[test]
fn navigate_empty_buffer_forwards() {
    let mut engine = Engine::create().unwrap();
    assert_eq!(
        engine.process(key_event(VimeKey::Left)).action,
        VimeAction::Forward
    );
    assert_eq!(
        engine.process(key_event(VimeKey::Right)).action,
        VimeAction::Forward
    );
    assert_eq!(
        engine.process(key_event(VimeKey::Backspace)).action,
        VimeAction::Forward
    );
    assert_eq!(
        engine.process(key_event(VimeKey::Delete)).action,
        VimeAction::Forward
    );
}

/// A state change made outside `vime_process_key` must still invalidate the
/// cached word, or a lazy `vime_parsed` hands back text from before the
/// change. Both config setters are state changes of exactly that kind.
#[test]
fn config_changes_invalidate_the_cached_word() {
    use std::ffi::CStr;

    let mut engine = Engine::create_with(VimeInputMethod::Telex, VimeTonePlacement::Modern)
        .expect("engine must be created");
    engine.type_text("hoas");

    // SAFETY: the handle is live and owned by `engine`.
    let read = |engine: &mut Engine| unsafe {
        let ptr = vime::vime_parsed(engine.raw());
        assert!(!ptr.is_null());
        CStr::from_ptr(ptr).to_str().unwrap().to_string()
    };

    // Cached under the modern scheme.
    assert_eq!(read(&mut engine), "hoá");

    // Switching placement re-renders the pending vowels; the cache must not
    // still answer with the modern rendering.
    engine.set_tone_placement(VimeTonePlacement::Old);
    assert_eq!(read(&mut engine), "hóa");

    // Switching the method clears the buffer; the cache must not still answer
    // with the pre-clear text.
    engine.type_text("too");
    engine.set_input_method(VimeInputMethod::Vni);
    assert_eq!(read(&mut engine), "");
}

/// The word buffer is reused across renders rather than reallocated, so the
/// returned pointer must stay correct after a state change: the new text has to
/// land in the same live buffer the frontend is still holding.
#[test]
fn reused_word_buffer_serves_the_current_text() {
    use std::ffi::CStr;

    let mut engine = Engine::create().unwrap();
    let typed = engine.type_text("tiengs");

    // SAFETY: the handle is live and owned by `engine`.
    let read = |engine: &mut Engine| unsafe {
        let ptr = vime::vime_parsed(engine.raw());
        assert!(!ptr.is_null());
        CStr::from_ptr(ptr).to_str().unwrap().to_string()
    };

    // The buffer must agree with what typing last reported, and repeated reads
    // must not disturb it.
    assert_eq!(read(&mut engine), typed);
    assert_eq!(read(&mut engine), typed);
    assert_eq!(read(&mut engine), typed);

    // Each backspace must land new text in the same reused buffer.
    let mut previous = typed;
    for _ in 0..4 {
        engine.process(key_event(VimeKey::Backspace));
        let now = read(&mut engine);
        assert_ne!(now, previous, "backspace must change the word");
        previous = now;
    }

    // And an emptied buffer must render as empty, not as stale text.
    for _ in 0..8 {
        engine.process(key_event(VimeKey::Backspace));
    }
    assert_eq!(read(&mut engine), "");
}
