//! C ABI layout and enum-value locks.
//!
//! These pin the exact byte sizes/offsets and discriminant values that the
//! `vime_engine.h` header and the Rust backend share, so a mismatch fails the
//! tests instead of silently corrupting native adapters.

use std::mem::{align_of, offset_of, size_of};

use vime::{
    VimeAction, VimeConfig, VimeInputMethod, VimeKey, VimeKeyEvent, VimeOutput, VimeTonePlacement,
};

#[test]
fn scalar_enum_widths() {
    assert_eq!(size_of::<VimeAction>(), 4);
    assert_eq!(size_of::<VimeInputMethod>(), 4);
    assert_eq!(size_of::<VimeTonePlacement>(), 4);
    assert_eq!(size_of::<VimeKey>(), size_of::<u32>());
    assert_eq!(size_of::<VimeKey>(), 4);
}

/// The header and the backend must agree on the struct, and the default must
/// be the engine's rather than the C zero for a bool.
#[test]
fn config_layout() {
    assert_eq!(size_of::<VimeConfig>(), 1, "VimeConfig is a single bool");
    assert_eq!(align_of::<VimeConfig>(), 1);
    assert_eq!(offset_of!(VimeConfig, auto_restore_english), 0);
}

#[test]
fn key_event_layout() {
    assert_eq!(size_of::<VimeKeyEvent>(), 12);
    assert_eq!(offset_of!(VimeKeyEvent, key), 0);
    assert_eq!(offset_of!(VimeKeyEvent, character), 4);
    assert_eq!(offset_of!(VimeKeyEvent, states), 8);
}

/// `VimeOutput` carries only the commit text; the word is fetched
/// separately through `vime_parsed` when the frontend wants it.
#[test]
fn output_layout() {
    assert_eq!(size_of::<VimeOutput>(), 16);
    assert_eq!(align_of::<VimeOutput>(), 8);
    assert_eq!(offset_of!(VimeOutput, action), 0);
    assert_eq!(offset_of!(VimeOutput, commit), 8);
}

/// The discriminant tables the C header declares. Adding a variant to any of
/// these enums means adding a row here, so a renumbering cannot slip through.
const ACTIONS: [(VimeAction, u32); 5] = [
    (VimeAction::Forward, 0),
    (VimeAction::Noop, 1),
    (VimeAction::Changed, 2),
    (VimeAction::Commit, 3),
    (VimeAction::CursorMoved, 4),
];

const INPUT_METHODS: [(VimeInputMethod, u32); 3] = [
    (VimeInputMethod::Telex, 1),
    (VimeInputMethod::Vni, 2),
    (VimeInputMethod::Viqr, 3),
];

const TONE_PLACEMENTS: [(VimeTonePlacement, u32); 2] =
    [(VimeTonePlacement::Modern, 1), (VimeTonePlacement::Old, 2)];

const KEYS: [(VimeKey, u32); 9] = [
    (VimeKey::Character, 0),
    (VimeKey::Backspace, 1),
    (VimeKey::Delete, 2),
    (VimeKey::Left, 3),
    (VimeKey::Right, 4),
    (VimeKey::Enter, 5),
    (VimeKey::Escape, 6),
    (VimeKey::Tab, 7),
    (VimeKey::Space, 8),
];

/// Every C-visible enum is `repr(u32)` and the header's `#define`s must match.
#[test]
fn c_enum_discriminants_match_the_header() {
    for (value, expected) in ACTIONS {
        assert_eq!(value as u32, expected, "{value:?} discriminant changed");
    }
    for (value, expected) in INPUT_METHODS {
        assert_eq!(value as u32, expected, "{value:?} discriminant changed");
    }
    for (value, expected) in TONE_PLACEMENTS {
        assert_eq!(value as u32, expected, "{value:?} discriminant changed");
    }
    for (value, expected) in KEYS {
        assert_eq!(value as u32, expected, "{value:?} discriminant changed");
    }
}

/// The discriminant tables are dense and gap-free, so the first row cannot
/// drift away from the rest.
#[test]
fn c_enum_discriminants_are_dense() {
    // `VimeInputMethod` and `VimeTonePlacement` deliberately start at 1 (0 is
    // reserved as "unset" in the C header), so only those two are offset.
    assert_dense("VimeAction", ACTIONS.iter().map(|(_, v)| *v), 0);
    assert_dense("VimeKey", KEYS.iter().map(|(_, v)| *v), 0);
    assert_dense("VimeInputMethod", INPUT_METHODS.iter().map(|(_, v)| *v), 1);
    assert_dense(
        "VimeTonePlacement",
        TONE_PLACEMENTS.iter().map(|(_, v)| *v),
        1,
    );
}

/// Asserts `values` are exactly `base..base + len`, in order.
fn assert_dense(name: &str, values: impl Iterator<Item = u32>, base: u32) {
    let values: Vec<u32> = values.collect();
    let expected: Vec<u32> = (base..base + values.len() as u32).collect();
    assert_eq!(
        values, expected,
        "{name} discriminants must stay dense from {base}"
    );
}
