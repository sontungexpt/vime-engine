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

/// The versioning contract, exercised: a NULL config, a zeroed struct, and a
/// struct carrying the current size must all produce the same engine, and a
/// size the library does not recognise must be rejected rather than read.
#[test]
fn config_accepted_in_every_valid_form() {
    use vime::{VimeConfig, VimeInputMethod, VimeTonePlacement};

    // NULL.
    // SAFETY: NULL is one of the two forms the entry point accepts.
    let null = unsafe {
        vime::vime_create_with_config(
            std::ptr::null(),
            VimeInputMethod::Vni,
            VimeTonePlacement::Old,
        )
    };
    assert!(!null.is_null(), "NULL config must build an engine");
    // SAFETY: live handle from the call above.
    unsafe { vime::vime_destroy(null) };

    // A zeroed struct: struct_size 0 means "every default".
    let zeroed = VimeConfig {
        struct_size: 0,
        auto_restore_english: false,
    };
    // SAFETY: the struct is readable and 0 is an accepted size.
    let h = unsafe {
        vime::vime_create_with_config(&zeroed, VimeInputMethod::Telex, VimeTonePlacement::Modern)
    };
    assert!(!h.is_null(), "a zeroed config must build an engine");
    // SAFETY: live handle from the call above.
    unsafe { vime::vime_destroy(h) };

    // The current size, which is what VIME_CONFIG_INIT produces.
    let current = VimeConfig {
        struct_size: std::mem::size_of::<VimeConfig>() as u32,
        auto_restore_english: false,
    };
    // SAFETY: the struct is readable and the size is the current one.
    let h = unsafe {
        vime::vime_create_with_config(&current, VimeInputMethod::Viqr, VimeTonePlacement::Modern)
    };
    assert!(!h.is_null(), "a current-size config must build an engine");
    // SAFETY: live handle from the call above.
    unsafe { vime::vime_destroy(h) };
}

/// Revision 1 predates `auto_restore_english`, so a caller reporting size 4
/// never wrote that byte. It must be ignored rather than read as stack garbage,
/// which for this field would silently invert the engine default.
#[test]
fn a_revision_1_config_does_not_supply_the_new_field() {
    use vime::VimeConfig;

    // Deliberately `false`, which is NOT the engine default. If the field were
    // read from this struct the result would be the opposite of the default.
    let rev1 = VimeConfig {
        struct_size: 4,
        auto_restore_english: false,
    };
    // SAFETY: the struct is readable and 4 is the revision-1 size.
    let read = unsafe { VimeConfig::read(&rev1) }.expect("revision 1 must be accepted");
    assert_eq!(
        read.auto_restore_english,
        vime_engine::Config::default().auto_restore_english,
        "a revision-1 caller must get the engine default, not its unwritten byte"
    );

    // Revision 2 does supply the field, and its value must be honoured.
    let rev2 = VimeConfig {
        struct_size: std::mem::size_of::<VimeConfig>() as u32,
        auto_restore_english: false,
    };
    // SAFETY: the struct is readable and the size is the current one.
    let read = unsafe { VimeConfig::read(&rev2) }.expect("revision 2 must be accepted");
    assert!(
        !read.auto_restore_english,
        "a revision-2 caller's value must be read, not overridden"
    );
}

/// The C default and the engine default must agree, since a caller that says
/// nothing gets the C struct's values and the engine has to be built from them.
#[test]
fn config_defaults_match_the_engine() {
    assert_eq!(
        vime::VimeConfig::default_config().to_engine_config(),
        vime_engine::Config::default(),
        "a caller passing no config must get the engine's own defaults"
    );
}

/// A `struct_size` that is neither 0 nor a size this library knows describes a
/// layout from a future revision. Reading past it would be reading uninitialised
/// or foreign memory, so it is rejected.
#[test]
fn config_rejects_a_size_it_does_not_recognise() {
    use vime::{VimeConfig, VimeInputMethod, VimeTonePlacement};

    let bogus = VimeConfig {
        struct_size: 3,
        auto_restore_english: false,
    };
    // SAFETY: the struct is readable; 3 is the value under test.
    let h = unsafe {
        vime::vime_create_with_config(&bogus, VimeInputMethod::Telex, VimeTonePlacement::Modern)
    };
    assert!(h.is_null(), "an unrecognised struct_size must be rejected");
}
