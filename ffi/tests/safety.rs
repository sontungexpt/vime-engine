//! Robustness of the C boundary against misbehaving callers: NULL handles,
//! invalid Unicode payloads. Every case must degrade to `Forward`/NULL, never
//! crash or corrupt state.
//!
//! Out-of-range enum *discriminants* are also guarded by explicit `_` arms in
//! the Rust entry points, but a `repr(u32)` enum holding an invalid value can
//! only be built with `transmute`, which aborts in debug builds — so those
//! arms are validated structurally (compiler exhaustiveness) rather than by a
//! dedicated runtime test.

mod common;

use std::ptr;

use common::Engine;
use vime::{VimeAction, VimeInputMethod, VimeKey, VimeKeyEvent, VimeOutput, VimeTonePlacement};

#[test]
fn null_handle_is_safe_everywhere() {
    unsafe {
        vime::vime_destroy(ptr::null_mut());

        let reset = vime::vime_reset(ptr::null_mut());
        assert_eq!(reset.action, VimeAction::Forward);
        assert!(reset.rendered.is_null());
        assert!(reset.commit.is_null());

        let commit = vime::vime_commit(ptr::null_mut());
        assert_eq!(commit.action, VimeAction::Forward);

        let key = vime::vime_process_key(
            ptr::null_mut(),
            VimeKeyEvent {
                key: VimeKey::None,
                character: 'a' as u32,
                states: 0,
            },
        );
        assert_eq!(key.action, VimeAction::Forward);

        let method = vime::vime_set_input_method(ptr::null_mut(), VimeInputMethod::Telex);
        assert_eq!(method.action, VimeAction::Forward);

        let tone = vime::vime_set_tone_placement(ptr::null_mut(), VimeTonePlacement::Modern);
        assert_eq!(tone.action, VimeAction::Forward);
    }
}

#[test]
fn invalid_unicode_character_is_rejected() {
    let mut engine = Engine::create().unwrap();
    let out = engine.process(VimeKeyEvent {
        key: VimeKey::None,
        character: 0x11_0000, // beyond the valid Unicode range
        states: 0,
    });
    assert_eq!(out.action, VimeAction::Forward);
    assert!(out.rendered.is_none());
    assert!(out.commit.is_none());
}

#[test]
fn vime_output_default_is_forward() {
    let out = VimeOutput::default();
    assert_eq!(out.action, VimeAction::Forward);
    assert!(out.rendered.is_null());
    assert!(out.commit.is_null());
}

#[test]
fn modifier_states_do_not_crash() {
    let mut engine = Engine::create().unwrap();
    for states in [0, 1, 2, 4, 8, 16, 32, 64, 128, 0xFF] {
        let out = engine.process(VimeKeyEvent {
            key: VimeKey::None,
            character: 'v' as u32,
            states,
        });
        assert_eq!(out.action, VimeAction::UpdatePreedit);
    }
    assert_eq!(engine.commit().commit.as_deref(), Some("vvvvvvvvvv"));
}