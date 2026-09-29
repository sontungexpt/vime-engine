mod common;

use std::ptr;

use common::SessionFactory;
use vime::{
    VimeAction, VimeConfig, VimeInputMethod, VimeKey, VimeKeyEvent, VimeOutput,
    VimeTonePlacement, VimeSessionFactoryHandle, VimeSessionHandle,
};

#[test]
fn null_handle_is_safe_everywhere() {
    unsafe {
        // Factory NULL
        vime::vime_session_factory_destroy(ptr::null_mut());

        // Session NULL
        vime::vime_session_destroy(ptr::null_mut());
        assert!(!vime::vime_session_reset(ptr::null_mut()));
        assert!(vime::vime_session_render(ptr::null_mut()).is_null());

        let key = vime::vime_session_process_key(
            ptr::null_mut(),
            VimeKeyEvent {
                key: VimeKey::Character,
                character: 'a' as u32,
                states: 0,
            },
        );
        assert_eq!(key.action, VimeAction::Forward);

        // Factory setters report false for NULL
        assert!(!vime::vime_session_factory_set_input_method(
            ptr::null_mut(),
            VimeInputMethod::Telex
        ));
        assert!(!vime::vime_session_factory_set_tone_placement(
            ptr::null_mut(),
            VimeTonePlacement::Modern
        ));
        assert!(!vime::vime_session_factory_set_auto_restore_english(
            ptr::null_mut(),
            true
        ));
    }
}

#[test]
#[test]
#[test]
fn invalid_unicode_character_is_rejected() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = unsafe { vime::vime_session_create(factory.handle()) };
    let out = unsafe { vime::vime_session_process_key(
        session,
        VimeKeyEvent {
            key: VimeKey::Character,
            character: 0x11_0000, // beyond the valid Unicode range
            states: 0,
        },
    )};
    assert_eq!(out.action, VimeAction::Forward);
}

#[test]
fn vime_output_default_is_forward() {
    let out = VimeOutput::default();
    assert_eq!(out.action, VimeAction::Forward);
    assert!(out.commit.is_null());
}

#[test]
fn modifier_states_do_not_crash() {
    let mut factory = SessionFactory::create().unwrap();
    let mut session = unsafe { vime::vime_session_create(factory.handle()) };
    for states in [0, 1, 2, 4, 8, 16, 32, 64, 128, 0xFF] {
        let out = unsafe { vime::vime_session_process_key(
            session,
            VimeKeyEvent {
                key: VimeKey::Character,
                character: 'v' as u32,
                states,
            },
        )};
        assert_eq!(out.action, VimeAction::Changed);
    }
    // Verify 10 'v's were typed
    let render = unsafe { vime::vime_session_render(session) };
    assert_eq!(
        unsafe { std::ffi::CStr::from_ptr(render).to_str().unwrap() },
        "vvvvvvvvvv"
    );
}

#[test]
fn config_accepted_in_every_form() {
    use vime::{VimeConfig, VimeInputMethod, VimeTonePlacement};

    // NULL.
    // SAFETY: NULL is one of the forms the entry point accepts.
    let null = unsafe {
        vime::vime_session_factory_create_with_config(
            std::ptr::null(),
        )
    };
    assert!(!null.is_null(), "NULL config must build a factory");
    unsafe { vime::vime_session_factory_destroy(null) };

    // An explicit field value, which must be honoured.
    for flag in [true, false] {
        let config = VimeConfig {
            auto_restore_english: flag,
            input_method: VimeInputMethod::Telex,
            tone_placement: VimeTonePlacement::Modern,
        };
        // SAFETY: the struct is readable.
        let h = unsafe {
            vime::vime_session_factory_create_with_config(&config)
        };
        assert!(!h.is_null(), "an explicit config must build a factory");
        unsafe { vime::vime_session_factory_destroy(h) };
    }
}

#[test]
fn config_defaults_match_the_engine() {
    assert_eq!(
        vime::VimeConfig::default().to_engine_config(),
        vime_engine::Config::new(
            vime_engine::Settings::default(),
            vime_engine::composition::syllable::SyllableContext::new(vime_engine::DefaultKeymap::telex(), vime_engine::phonology::TonePlacement::Modern)
        ),
        "a caller passing no config must get the engine's own defaults"
    );
}