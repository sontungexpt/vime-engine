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

use vime::{VimeConfig, VIME_INPUT_METHOD_TELEX, VIME_INPUT_METHOD_VNI, VIME_INPUT_METHOD_VIQR, VIME_TONE_PLACEMENT_MODERN, VIME_TONE_PLACEMENT_OLD, VimeRenderState};

/// `VimeConfig { u32; u32; }` — two 4-byte integers.
///
/// The u32s match C's `int`-sized enums on every 64-bit ABI, which is the assumption
/// these tests lock; `c_abi.rs` verifies it with a C compiler rather than taking
/// it on faith.
#[test]
fn config_layout_matches_the_header() {
    assert_eq!(size_of::<VimeConfig>(), 8);
    assert_eq!(align_of::<VimeConfig>(), 4);
    assert_eq!(offset_of!(VimeConfig, input_method), 0);
    assert_eq!(offset_of!(VimeConfig, tone_placement), 4);
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

/// The discriminants the header writes. The constants start at 1, leaving 0 as
/// "unset".
#[test]
fn raw_values_match_the_header() {
    assert_eq!(VIME_INPUT_METHOD_TELEX, 1);
    assert_eq!(VIME_INPUT_METHOD_VNI, 2);
    assert_eq!(VIME_INPUT_METHOD_VIQR, 3);
    assert_eq!(VIME_TONE_PLACEMENT_MODERN, 1);
    assert_eq!(VIME_TONE_PLACEMENT_OLD, 2);

    // The types are u32 aliases, so they are 4 bytes.
    assert_eq!(size_of::<vime::VimeInputMethod>(), 4);
    assert_eq!(size_of::<vime::VimeTonePlacement>(), 4);
}

/// `VIME_CONFIG_INIT` is Telex / Modern, and a factory created
/// with no config at all behaves the same way. Modern puts the sắc on the second
/// vowel of `hoa`; Old would put it on the first, so the render says which one is
/// in force.
#[test]
fn a_configless_factory_matches_the_headers_init() {
    let init = VimeConfig::default();
    assert_eq!(init.input_method, VIME_INPUT_METHOD_TELEX);
    assert_eq!(init.tone_placement, VIME_TONE_PLACEMENT_MODERN);

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
