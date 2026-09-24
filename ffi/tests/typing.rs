//! Typing round-trips for each built-in input method and preedit rendering.

mod common;

use common::{Engine, key_event};
use vime::{VimeAction, VimeInputMethod, VimeKey, VimeTonePlacement};

#[test]
fn telex_preedit_and_commit() {
    let mut engine = Engine::create().unwrap();
    let rendered = engine.type_text("vieetj");
    assert_eq!(rendered, "việt");

    let out = engine.commit();
    assert_eq!(out.action, VimeAction::Commit);
    assert_eq!(out.commit.as_deref(), Some("việt"));
    assert!(out.rendered.is_none());
}

#[test]
fn modern_telex_tone_placement() {
    let mut engine = Engine::create_with(VimeInputMethod::Telex, VimeTonePlacement::Modern)
        .expect("engine must be created");
    let rendered = engine.type_text("hoas");
    assert_eq!(rendered, "hoá");
    assert_eq!(engine.commit().commit.as_deref(), Some("hoá"));
}

#[test]
fn old_telex_tone_placement() {
    let mut engine = Engine::create_with(VimeInputMethod::Telex, VimeTonePlacement::Old)
        .expect("engine must be created");
    let rendered = engine.type_text("hoas");
    assert_eq!(rendered, "hóa");
    assert_eq!(engine.commit().commit.as_deref(), Some("hóa"));
}

#[test]
fn vni_round_trip() {
    let mut engine = Engine::create_with(VimeInputMethod::Vni, VimeTonePlacement::Modern)
        .expect("engine must be created");
    let rendered = engine.type_text("hoa1");
    assert_eq!(rendered, "hoá");
    assert_eq!(engine.commit().commit.as_deref(), Some("hoá"));
}

#[test]
fn viqr_round_trip() {
    let mut engine = Engine::create_with(VimeInputMethod::Viqr, VimeTonePlacement::Modern)
        .expect("engine must be created");
    let rendered = engine.type_text("toa'n");
    assert_eq!(rendered, "toán");
    assert_eq!(engine.commit().commit.as_deref(), Some("toán"));
}

#[test]
fn enter_commits_without_suffix() {
    let mut engine = Engine::create().unwrap();
    engine.type_text("vieetj");
    let out = engine.process(key_event(VimeKey::Enter));
    assert_eq!(out.action, VimeAction::Commit);
    assert_eq!(out.commit.as_deref(), Some("việt"));
}

#[test]
fn space_appends_suffix_on_commit() {
    let mut engine = Engine::create().unwrap();
    engine.type_text("vieetj");
    let out = engine.process(key_event(VimeKey::Space));
    assert_eq!(out.action, VimeAction::Commit);
    assert_eq!(out.commit.as_deref(), Some("việt "));
}

#[test]
fn each_keystroke_shows_live_preedit() {
    let mut engine = Engine::create().unwrap();
    let mut seen = Vec::new();
    for ch in "vieetj".chars() {
        let out = engine.process(common::char_event(ch));
        assert_eq!(out.action, VimeAction::UpdatePreedit);
        seen.push(out.rendered.unwrap());
    }
    assert_eq!(seen, ["v", "vi", "vie", "viê", "viêt", "việt"]);
}