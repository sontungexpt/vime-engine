//! Integration tests for the configuration-driven [`Keymap`] layer and its
//! compile-time `u128` bitmask lookups.
//!
//! - `is_tone_key` / `is_shape_key` / `is_stroke_key` are backed by `u128`
//!   bitmasks precomputed in `DefaultKeymap::new`. These tests verify bit
//!   positions for ASCII keys, case-insensitivity via `to_ascii_lowercase`,
//!   and that non-ASCII / out-of-range input is safely rejected without a
//!   shift-overflow panic.

use vime_engine::phonology::{RootVowel, Shape, Tone};
use vime_engine::{DefaultKeymap, Keymap, Rules, ShapeRule, ToneRule};

/// `(key, Tone)` pairs of the Telex layout.
const TELEX_TONES: [(char, Tone); 6] = [
    ('s', Tone::Acute),
    ('f', Tone::Grave),
    ('r', Tone::Hook),
    ('x', Tone::Tilde),
    ('j', Tone::Dot),
    ('z', Tone::Flat),
];

/// `(key, RootVowel, Shape)` triples of the Telex layout.
const TELEX_SHAPES: [(char, RootVowel, Shape); 6] = [
    ('a', RootVowel::A, Shape::Circumflex),
    ('w', RootVowel::A, Shape::Breve),
    ('e', RootVowel::E, Shape::Circumflex),
    ('o', RootVowel::O, Shape::Circumflex),
    ('w', RootVowel::O, Shape::Horn),
    ('w', RootVowel::U, Shape::Horn),
];

/// `(key, Tone)` pairs of the VNI layout.
const VNI_TONES: [(char, Tone); 6] = [
    ('1', Tone::Acute),
    ('2', Tone::Grave),
    ('3', Tone::Hook),
    ('4', Tone::Tilde),
    ('5', Tone::Dot),
    ('0', Tone::Flat),
];

/// `(key, RootVowel, Shape)` triples of the VNI layout.
const VNI_SHAPES: [(char, RootVowel, Shape); 6] = [
    ('6', RootVowel::A, Shape::Circumflex),
    ('7', RootVowel::A, Shape::Breve),
    ('6', RootVowel::E, Shape::Circumflex),
    ('6', RootVowel::O, Shape::Circumflex),
    ('7', RootVowel::O, Shape::Horn),
    ('8', RootVowel::U, Shape::Horn),
];

/// `(key, Tone)` pairs of the VIQR layout.
const VIQR_TONES: [(char, Tone); 6] = [
    ('`', Tone::Grave),
    ('?', Tone::Hook),
    ('~', Tone::Tilde),
    ('\'', Tone::Acute),
    ('.', Tone::Dot),
    ('z', Tone::Flat),
];

/// `(key, RootVowel, Shape)` triples of the VIQR layout.
const VIQR_SHAPES: [(char, RootVowel, Shape); 6] = [
    ('^', RootVowel::A, Shape::Circumflex),
    ('^', RootVowel::E, Shape::Circumflex),
    ('^', RootVowel::O, Shape::Circumflex),
    ('(', RootVowel::A, Shape::Breve),
    ('+', RootVowel::O, Shape::Horn),
    ('+', RootVowel::U, Shape::Horn),
];

/// ASCII characters bound to no role in any shipped layout.
const NEUTRAL: [char; 14] = [
    'b', 'c', 'g', 'h', 'k', 'l', 'm', 'n', 'p', 'q', 't', 'u', 'y', 'v',
];

// ------------------------------------------------------------ shared assertions

/// Verifies every tone key in `tones` decodes and matches case-insensitively.
fn assert_tone_keys(km: impl Keymap, tones: &[(char, Tone)]) {
    for &(key, tone) in tones {
        assert!(km.is_tone_key(key), "{key:?} must be a tone key");
        assert!(
            km.is_tone_key(key.to_ascii_uppercase()),
            "uppercase {key:?} must be a tone key"
        );
        assert_eq!(
            km.decode_tone(key),
            Some(tone),
            "tone decode mismatch for {key:?}"
        );
        assert!(
            !km.is_shape_key(key),
            "tone key {key:?} must not be a shape key"
        );
        assert!(
            !km.is_stroke_key(key),
            "tone key {key:?} must not be a stroke key"
        );
    }
}

/// Verifies every shape rule decodes (on its owner) and matches case-insensitively.
fn assert_shape_keys(km: impl Keymap, shapes: &[(char, RootVowel, Shape)]) {
    for &(key, owner, shape) in shapes {
        assert!(km.is_shape_key(key), "{key:?} must be a shape key");
        assert!(
            km.is_shape_key(key.to_ascii_uppercase()),
            "uppercase {key:?} must be a shape key"
        );
        assert_eq!(
            km.decode_shape(key, owner),
            Some(shape),
            "shape decode mismatch for {key:?} on {owner:?}"
        );
        assert!(
            !km.is_tone_key(key),
            "shape key {key:?} must not be a tone key"
        );
        assert!(
            !km.is_stroke_key(key),
            "shape key {key:?} must not be a stroke key"
        );
        assert_eq!(
            km.decode_shape(key, RootVowel::Y),
            None,
            "shape {key:?} must not apply to Y"
        );
    }
}

/// Asserts each stroke key is a stroke key and nothing else.
fn assert_stroke_keys(km: impl Keymap, strokes: &[char]) {
    for &key in strokes {
        assert!(km.is_stroke_key(key), "{key:?} must be a stroke key");
        assert!(
            km.is_stroke_key(key.to_ascii_uppercase()),
            "uppercase {key:?} must be a stroke key"
        );
        assert!(
            !km.is_tone_key(key),
            "stroke key {key:?} must not be a tone key"
        );
        assert!(
            !km.is_shape_key(key),
            "stroke key {key:?} must not be a shape key"
        );
    }
}

/// Asserts the characters in `neutral` trigger no role and no transform.
///
/// `layout` names the keymap under test so a failure points at one layout.
fn assert_neutral(layout: &str, km: impl Keymap, neutral: &[char]) {
    for &ch in neutral {
        assert!(
            !km.is_tone_key(ch),
            "{layout}: {ch:?} must not be a tone key"
        );
        assert!(
            !km.is_shape_key(ch),
            "{layout}: {ch:?} must not be a shape key"
        );
        assert!(
            !km.is_stroke_key(ch),
            "{layout}: {ch:?} must not be a stroke key"
        );
        assert!(
            !km.is_transform_key(ch),
            "{layout}: {ch:?} must not be a transform key"
        );
        assert_eq!(
            km.decode_tone(ch),
            None,
            "{layout}: {ch:?} must not decode as a tone"
        );
        assert_eq!(
            km.decode_shape(ch, RootVowel::A),
            None,
            "{layout}: {ch:?} must not decode as a shape"
        );
    }
}

// ------------------------------------------------------------------ telex mask

#[test]
fn telex_tone_keys_match_layout() {
    assert_tone_keys(DefaultKeymap::telex(), &TELEX_TONES);
}

#[test]
fn telex_shape_keys_match_layout() {
    assert_shape_keys(DefaultKeymap::telex(), &TELEX_SHAPES);
}

#[test]
fn telex_stroke_key_matches_layout() {
    assert_stroke_keys(DefaultKeymap::telex(), &['d']);
}

#[test]
fn telex_transform_key_matches_classification() {
    let km = DefaultKeymap::telex();
    for &(key, _) in &TELEX_TONES {
        assert!(km.is_transform_key(key));
    }
    for &(key, _, _) in &TELEX_SHAPES {
        assert!(km.is_transform_key(key));
    }
    assert!(km.is_transform_key('d'));
}

// ------------------------------------------------------------------ vni mask

#[test]
fn vni_tone_keys_match_layout() {
    assert_tone_keys(DefaultKeymap::vni(), &VNI_TONES);
}

#[test]
fn vni_shape_keys_match_layout() {
    assert_shape_keys(DefaultKeymap::vni(), &VNI_SHAPES);
}

#[test]
fn vni_stroke_key_matches_layout() {
    assert_stroke_keys(DefaultKeymap::vni(), &['9']);
}

// ------------------------------------------------------------------ viqr mask

#[test]
fn viqr_tone_keys_match_layout() {
    assert_tone_keys(DefaultKeymap::viqr(), &VIQR_TONES);
}

#[test]
fn viqr_shape_keys_match_layout() {
    assert_shape_keys(DefaultKeymap::viqr(), &VIQR_SHAPES);
}

#[test]
fn viqr_stroke_key_matches_layout() {
    assert_stroke_keys(DefaultKeymap::viqr(), &['d']);
}

// ----------------------------------------------------------- cross-layout mask

#[test]
fn mask_is_case_insensitive_for_punctuation_too() {
    // `to_ascii_lowercase` only folds A-Z; punctuation/digits are stored with
    // bit 0x20 possibly set, so they must map to themselves in the bitmask.
    let viqr = DefaultKeymap::viqr();
    for &ch in &['^', '`', '~', '(', '+', '?', '.'] {
        assert!(
            viqr.is_shape_key(ch) || viqr.is_tone_key(ch),
            "viqr {ch:?} must be bound"
        );
    }
}

/// `^` and `~` differ only in bit 0x20, so they are the pair that catches a
/// lookup which lowercases arithmetically (`| 0x20`) instead of calling
/// `to_ascii_lowercase`.
///
/// `^` is a VIQR *shape* key and `~` a VIQR *tone* key. Folding the two
/// together makes `^` report as the `~` tone bit, which silently drops every
/// circumflex in VIQR. The shipped `to_ascii_lowercase` keeps them apart, and
/// this is the test that says so — previously the only evidence was an A/B
/// bench asserting a rejected branchless variant was wrong.
#[test]
fn viqr_caret_and_tilde_do_not_collide() {
    let viqr = DefaultKeymap::viqr();

    assert!(viqr.is_shape_key('^'), "'^' must be a shape key");
    assert!(
        !viqr.is_tone_key('^'),
        "'^' must not be mistaken for the '~' tone key"
    );

    assert!(viqr.is_tone_key('~'), "'~' must be a tone key");
    assert!(!viqr.is_shape_key('~'), "'~' must not be a shape key");

    // The fold that would break it, spelled out: 0x5E | 0x20 == 0x7E == '~'.
    assert_eq!(b'^' | 0x20, b'~', "the collision this test guards is real");
}

/// Keys bound to no role in any shipped layout, plus the characters that must
/// never reach a bitmask shift: the ASCII range edges and non-ASCII scalars.
#[test]
fn unbound_keys_are_rejected_in_every_layout() {
    let telex = DefaultKeymap::telex();
    let vni = DefaultKeymap::vni();
    let viqr = DefaultKeymap::viqr();
    let layouts = [("telex", &telex), ("vni", &vni), ("viqr", &viqr)];

    let rejected: Vec<char> = NEUTRAL
        .iter()
        .copied()
        .chain(['\0', '\x7F']) // ASCII range edges
        .chain(['đ', 'Đ', 'ư', 'ơ', 'â', '　', '😀', '\u{10FFFF}'])
        .collect();

    for (name, km) in layouts {
        assert_neutral(name, *km, &rejected);
    }
}

// ------------------------------------------------------------------ validation

/// Asserts `build` panics with a layout-invalid message.
fn assert_invalid(build: impl FnOnce() -> Rules<'static>) {
    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(build));
    assert!(panicked.is_err(), "invalid layout must panic");
}

#[test]
fn rules_reject_duplicate_tone_key() {
    assert_invalid(|| {
        Rules::new(
            &[
                ToneRule {
                    key: b's',
                    tone: Tone::Acute,
                },
                ToneRule {
                    key: b's',
                    tone: Tone::Grave,
                },
            ],
            &[],
            &[],
        )
    });
}

#[test]
fn rules_reject_tone_shape_collision() {
    assert_invalid(|| {
        Rules::new(
            &[ToneRule {
                key: b's',
                tone: Tone::Acute,
            }],
            &[ShapeRule {
                key: b's',
                on: RootVowel::A,
                shape: Shape::Breve,
            }],
            &[],
        )
    });
}

#[test]
fn rules_reject_shape_same_owner_duplicate() {
    assert_invalid(|| {
        Rules::new(
            &[],
            &[
                ShapeRule {
                    key: b'w',
                    on: RootVowel::A,
                    shape: Shape::Breve,
                },
                ShapeRule {
                    key: b'w',
                    on: RootVowel::A,
                    shape: Shape::Circumflex,
                },
            ],
            &[],
        )
    });
}

#[test]
fn rules_allow_multi_owner_shape_reuse() {
    // Rule 4: the same shape key MAY map different owners (e.g. w -> ư/ơ).
    let rules = Rules::new(
        &[],
        &[
            ShapeRule {
                key: b'w',
                on: RootVowel::A,
                shape: Shape::Breve,
            },
            ShapeRule {
                key: b'w',
                on: RootVowel::O,
                shape: Shape::Horn,
            },
            ShapeRule {
                key: b'w',
                on: RootVowel::U,
                shape: Shape::Horn,
            },
        ],
        &[],
    );
    assert_eq!(rules.shapes.len(), 3);
}

#[test]
fn rules_reject_duplicate_stroke() {
    assert_invalid(|| Rules::new(&[], &[], &[b'z', b'z']));
}

#[test]
fn rules_reject_stroke_tone_collision() {
    assert_invalid(|| {
        Rules::new(
            &[ToneRule {
                key: b'z',
                tone: Tone::Acute,
            }],
            &[],
            &[b'z'],
        )
    });
}

#[test]
fn rules_reject_stroke_shape_collision() {
    assert_invalid(|| {
        Rules::new(
            &[],
            &[ShapeRule {
                key: b'z',
                on: RootVowel::A,
                shape: Shape::Breve,
            }],
            &[b'z'],
        )
    });
}

#[test]
fn rules_reject_non_ascii_key() {
    // Every key is a `u8`; a rule carrying a byte outside ASCII must panic
    // rather than silently alias an ASCII slot in the bitmask.
    const HIGH_TONES: [ToneRule; 3] = [
        ToneRule {
            key: 0xFF,
            tone: Tone::Acute,
        },
        ToneRule {
            key: 0x80,
            tone: Tone::Acute,
        },
        ToneRule {
            key: 0xF0,
            tone: Tone::Acute,
        },
    ];
    const HIGH_SHAPE: [ShapeRule; 1] = [ShapeRule {
        key: 0xF0,
        on: RootVowel::A,
        shape: Shape::Breve,
    }];

    for rules in &HIGH_TONES {
        assert_invalid(|| Rules::new(std::slice::from_ref(rules), &[], &[]));
    }
    assert_invalid(|| Rules::new(&[], &HIGH_SHAPE, &[]));
    assert_invalid(|| Rules::new(&[], &[], &[0x80]));
}
