//! C ABI layout and enum-value locks.
//!
//! The compile-time assertions in `src/config.rs` and `src/render.rs` already
//! fail the build on a layout change. These repeat them at run time, next to the
//! behavioural tests, so `cargo test` alone reports a mismatch as a test failure
//! rather than as a compiler error somewhere else — and, more usefully, so a
//! reviewer can read the whole ABI contract in one file.
//!
//! The other half of the verification is `tests/c_abi.rs`, which compiles a real C
//! program against `include/vime_engine.h`. These tests check that Rust agrees
//! with itself; that one checks that C agrees too.

mod common;

use std::mem::{align_of, offset_of, size_of};

use vime::{VimeConfig, VimeInputMethod, VimeRenderState, VimeTonePlacement};

/// `VimeConfig { bool; enum; enum; }` — one byte, three of padding, two enums.
///
/// The enums are `int`-sized in C on every 64-bit ABI, which is the assumption
/// these tests lock; `c_abi.rs` verifies it with a C compiler rather than taking
/// it on faith.
#[test]
fn config_layout_matches_the_header() {
    assert_eq!(size_of::<VimeConfig>(), 12);
    assert_eq!(align_of::<VimeConfig>(), 4);
    assert_eq!(offset_of!(VimeConfig, auto_restore_english), 0);
    assert_eq!(offset_of!(VimeConfig, input_method), 4);
    assert_eq!(offset_of!(VimeConfig, tone_placement), 8);
}

/// Two pointers, six `size_t`s, a `bool` — and the `bool` lands on a 72-byte total
/// with seven bytes of tail padding.
#[test]
fn render_state_layout_matches_the_header() {
    assert_eq!(size_of::<VimeRenderState>(), 72);
    assert_eq!(align_of::<VimeRenderState>(), 8);
    assert_eq!(offset_of!(VimeRenderState, text), 0);
    assert_eq!(offset_of!(VimeRenderState, raw_text), 8);
    assert_eq!(offset_of!(VimeRenderState, cursor_byte_idx), 16);
    assert_eq!(offset_of!(VimeRenderState, cursor_char_idx), 24);
    assert_eq!(offset_of!(VimeRenderState, raw_cursor_byte_idx), 32);
    assert_eq!(offset_of!(VimeRenderState, raw_cursor_char_idx), 40);
    assert_eq!(offset_of!(VimeRenderState, bytes_to_delete), 48);
    assert_eq!(offset_of!(VimeRenderState, chars_to_delete), 56);
    assert_eq!(offset_of!(VimeRenderState, is_valid_vietnamese), 64);
}

/// `bool` is one byte in C and in Rust, so the C header's `true`/`false` reach
/// the engine as the same two values Rust uses. Worth a test, because a `bool`
/// that is anything but 0/1 is undefined behaviour on the C side.
#[test]
fn a_bool_crosses_the_boundary_unchanged() {
    assert_eq!(size_of::<bool>(), 1);
    for flag in [true, false] {
        let config = VimeConfig {
            auto_restore_english: flag,
            ..VimeConfig::default()
        };
        let bytes: [u8; size_of::<VimeConfig>()] = {
            let mut out = [0u8; size_of::<VimeConfig>()];
            // SAFETY: `VimeConfig` is plain old data — a `bool` and two enums,
            // both `repr(C)` and `Copy` — so every byte pattern is a valid value
            // and the copy cannot miss a padding byte being uninitialised.
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &config as *const VimeConfig as *const u8,
                    out.as_mut_ptr(),
                    out.len(),
                )
            };
            out
        };
        assert_eq!(bytes[0], u8::from(flag), "C expects 0 or 1");
    }
}

/// The discriminants the header writes. `VimeInputMethod` and `VimeTonePlacement`
/// start at 1, leaving 0 as "unset".
#[test]
fn enum_discriminants_match_the_header() {
    assert_eq!(VimeInputMethod::Telex as u32, 1);
    assert_eq!(VimeInputMethod::Vni as u32, 2);
    assert_eq!(VimeInputMethod::Viqr as u32, 3);
    assert_eq!(VimeTonePlacement::Modern as u32, 1);
    assert_eq!(VimeTonePlacement::Old as u32, 2);

    assert_eq!(size_of::<VimeInputMethod>(), 4);
    assert_eq!(size_of::<VimeTonePlacement>(), 4);
}

/// `VIME_CONFIG_INIT` is Telex / Modern / auto-restore, and a factory created
/// with no config at all behaves the same way. Modern puts the sắc on the second
/// vowel of `hoa`; Old would put it on the first, so the render says which one is
/// in force.
#[test]
fn a_configless_factory_matches_the_headers_init() {
    let init = VimeConfig::default();
    assert!(init.auto_restore_english);
    assert_eq!(init.input_method, VimeInputMethod::Telex);
    assert_eq!(init.tone_placement, VimeTonePlacement::Modern);

    let mut default_factory = common::Factory::create().expect("a default factory");
    let mut default_session = default_factory.open_session();
    assert_eq!(default_session.type_text("hoas"), "hoá", "Telex, modern");

    let mut init_factory = common::Factory::create_with(&init).expect("a factory from init");
    let mut init_session = init_factory.open_session();
    assert_eq!(
        init_session.type_text("hoas"),
        "hoá",
        "the same as no config"
    );
}
