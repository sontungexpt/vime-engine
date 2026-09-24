//! C ABI layout and enum-value locks.
//!
//! These pin the exact byte sizes/offsets and discriminant values that the
//! `vime_engine.h` header and the Rust backend share, so a mismatch fails the
//! tests instead of silently corrupting native adapters.

use std::mem::{align_of, offset_of, size_of};

use vime::{
    VimeAction, VimeInputMethod, VimeKey, VimeKeyEvent, VimeOutput, VimeTonePlacement,
};

#[test]
fn scalar_enum_widths() {
    assert_eq!(size_of::<VimeAction>(), 4);
    assert_eq!(size_of::<VimeInputMethod>(), 4);
    assert_eq!(size_of::<VimeTonePlacement>(), 4);
    assert_eq!(size_of::<VimeKey>(), size_of::<u32>());
    assert_eq!(size_of::<VimeKey>(), 4);
}

#[test]
fn key_event_layout() {
    assert_eq!(size_of::<VimeKeyEvent>(), 12);
    assert_eq!(offset_of!(VimeKeyEvent, key), 0);
    assert_eq!(offset_of!(VimeKeyEvent, character), 4);
    assert_eq!(offset_of!(VimeKeyEvent, states), 8);
}

#[test]
fn output_layout() {
    assert_eq!(size_of::<VimeOutput>(), 24);
    assert_eq!(align_of::<VimeOutput>(), 8);
    assert_eq!(offset_of!(VimeOutput, action), 0);
    assert_eq!(offset_of!(VimeOutput, rendered), 8);
    assert_eq!(offset_of!(VimeOutput, commit), 16);
}

#[test]
fn action_discriminants() {
    assert_eq!(VimeAction::Forward as u32, 0);
    assert_eq!(VimeAction::Noop as u32, 1);
    assert_eq!(VimeAction::UpdatePreedit as u32, 2);
    assert_eq!(VimeAction::Commit as u32, 3);
}

#[test]
fn input_method_discriminants() {
    assert_eq!(VimeInputMethod::Telex as u32, 1);
    assert_eq!(VimeInputMethod::Vni as u32, 2);
    assert_eq!(VimeInputMethod::Viqr as u32, 3);
}

#[test]
fn tone_placement_discriminants() {
    assert_eq!(VimeTonePlacement::Modern as u32, 1);
    assert_eq!(VimeTonePlacement::Old as u32, 2);
}

#[test]
fn key_discriminants() {
    assert_eq!(VimeKey::None as u32, 0);
    assert_eq!(VimeKey::Backspace as u32, 1);
    assert_eq!(VimeKey::Delete as u32, 2);
    assert_eq!(VimeKey::Left as u32, 3);
    assert_eq!(VimeKey::Right as u32, 4);
    assert_eq!(VimeKey::Enter as u32, 5);
    assert_eq!(VimeKey::Escape as u32, 6);
    assert_eq!(VimeKey::Tab as u32, 7);
    assert_eq!(VimeKey::Space as u32, 8);
}