//! Engine lifecycle: create/destroy, input-method and tone-placement
//! switching, reset and commit semantics.

mod common;

use common::{Engine, key_event};
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
fn reset_returns_updated_empty_preedit() {
    let mut engine = Engine::create().unwrap();
    engine.type_text("viet");
    let out = engine.reset();
    assert_eq!(out.action, VimeAction::UpdatePreedit);
    assert_eq!(out.rendered.as_deref(), Some(""));
    assert!(out.commit.is_none());
}

#[test]
fn switching_method_clears_pending_buffer() {
    let mut engine = Engine::create().unwrap();
    engine.type_text("too");
    let out = engine.set_input_method(VimeInputMethod::Vni);
    assert_eq!(out.action, VimeAction::UpdatePreedit);
    assert_eq!(out.rendered.as_deref(), Some(""));
    assert!(out.commit.is_none());
}

#[test]
fn switching_placement_renders_without_text_commit() {
    let mut engine = Engine::create_with(VimeInputMethod::Telex, VimeTonePlacement::Modern)
        .expect("engine must be created");
    engine.type_text("hoas");
    let out = engine.set_tone_placement(VimeTonePlacement::Old);
    assert_eq!(out.action, VimeAction::UpdatePreedit);
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
    assert_eq!(committed.commit.as_deref(), Some("hóa"));
    assert!(committed.rendered.is_none());
}

#[test]
fn navigate_empty_buffer_forwards() {
    let mut engine = Engine::create().unwrap();
    assert_eq!(engine.process(key_event(VimeKey::Left)).action, VimeAction::Forward);
    assert_eq!(engine.process(key_event(VimeKey::Right)).action, VimeAction::Forward);
    assert_eq!(engine.process(key_event(VimeKey::Backspace)).action, VimeAction::Forward);
    assert_eq!(engine.process(key_event(VimeKey::Delete)).action, VimeAction::Forward);
}