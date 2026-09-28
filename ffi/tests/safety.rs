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

        assert!(!vime::vime_reset(ptr::null_mut()));

        // The lazy word accessor must be NULL-safe too.
        assert!(vime::vime_parsed(ptr::null_mut()).is_null());

        let key = vime::vime_process_key(
            ptr::null_mut(),
            VimeKeyEvent {
                key: VimeKey::Character,
                character: 'a' as u32,
                states: 0,
            },
        );
        assert_eq!(key.action, VimeAction::Forward);

        // The config setters report success, and a null handle is not success.
        assert!(!vime::vime_set_input_method(
            ptr::null_mut(),
            VimeInputMethod::Telex
        ));
        assert!(!vime::vime_set_tone_placement(
            ptr::null_mut(),
            VimeTonePlacement::Modern
        ));
    }
}

#[test]
fn invalid_unicode_character_is_rejected() {
    let mut engine = Engine::create().unwrap();
    let out = engine.process(VimeKeyEvent {
        key: VimeKey::Character,
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
    assert!(out.commit.is_null());
}

#[test]
fn modifier_states_do_not_crash() {
    let mut engine = Engine::create().unwrap();
    for states in [0, 1, 2, 4, 8, 16, 32, 64, 128, 0xFF] {
        let out = engine.process(VimeKeyEvent {
            key: VimeKey::Character,
            character: 'v' as u32,
            states,
        });
        assert_eq!(out.action, VimeAction::Changed);
    }
    assert_eq!(engine.commit_and_read().as_deref(), Some("vvvvvvvvvv"));
}
