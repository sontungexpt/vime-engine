//! Typing round-trips for each built-in input method and word rendering.

mod common;

use common::{SessionFactory, Session, char_event, key_event};
use vime::{VimeAction, VimeConfig, VimeInputMethod, VimeKey, VimeTonePlacement};

#[test]
fn telex_word_and_commit() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    let rendered = session.type_text("vieetj");
    assert_eq!(rendered, "việt");

    let out = session.commit();
    assert_eq!(out.action, VimeAction::Commit);
    assert_eq!(out.commit.as_deref(), Some("việt"));
    // Committing clears the word.
    assert_eq!(
        session.process(key_event(VimeKey::Enter)).action,
        VimeAction::Forward
    );
}

#[test]
fn modern_telex_tone_placement() {
    let mut factory = SessionFactory::create_with(VimeConfig {
        input_method: VimeInputMethod::Telex,
        tone_placement: VimeTonePlacement::Modern,
        auto_restore_english: true,
    }).unwrap();
    let mut session = factory.open_session();
    let rendered = session.type_text("hoas");
    assert_eq!(rendered, "hoá");
    assert_eq!(session.commit_and_read().as_deref(), Some("hoá"));
}

#[test]
fn old_telex_tone_placement() {
    let mut factory = SessionFactory::create_with(VimeConfig {
        input_method: VimeInputMethod::Telex,
        tone_placement: VimeTonePlacement::Old,
        auto_restore_english: true,
    }).unwrap();
    let mut session = factory.open_session();
    let rendered = session.type_text("hoas");
    assert_eq!(rendered, "hóa");
    assert_eq!(session.commit_and_read().as_deref(), Some("hóa"));
}

#[test]
fn vni_round_trip() {
    let mut factory = SessionFactory::create_with(VimeConfig {
        input_method: VimeInputMethod::Vni,
        tone_placement: VimeTonePlacement::Modern,
        auto_restore_english: true,
    }).unwrap();
    let mut session = factory.open_session();
    let rendered = session.type_text("hoa1");
    assert_eq!(rendered, "hoá");
    assert_eq!(session.commit_and_read().as_deref(), Some("hoá"));
}

#[test]
fn viqr_round_trip() {
    let mut factory = SessionFactory::create_with(VimeConfig {
        input_method: VimeInputMethod::Viqr,
        tone_placement: VimeTonePlacement::Modern,
        auto_restore_english: true,
    }).unwrap();
    let mut session = factory.open_session();
    let rendered = session.type_text("toa'n");
    assert_eq!(rendered, "toán");
    assert_eq!(session.commit_and_read().as_deref(), Some("toán"));
}

#[test]
fn enter_commits_without_suffix() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("vieetj");
    let out = session.process(key_event(VimeKey::Enter));
    assert_eq!(out.action, VimeAction::Commit);
    assert_eq!(out.commit.as_deref(), Some("việt"));
}

#[test]
fn space_appends_suffix_on_commit() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    session.type_text("vieetj");
    let out = session.process(key_event(VimeKey::Space));
    assert_eq!(out.action, VimeAction::Commit);
    assert_eq!(out.commit.as_deref(), Some("việt "));
}

#[test]
fn each_keystroke_shows_live_word() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = factory.open_session();
    let mut seen = Vec::new();
    for ch in "vieetj".chars() {
        let out = session.process(char_event(ch));
        assert_eq!(out.action, VimeAction::Changed);
        seen.push(out.rendered.unwrap());
    }
    assert_eq!(seen, ["v", "vi", "vie", "viê", "viêt", "việt"]);
}