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

/// The VIQR punctuation keys, each with the one role it holds: `(key,
/// is_shape, is_tone)`.
///
/// `to_ascii_lowercase` folds only A-Z, so a key is stored under the bit its
/// *own* byte occupies. For punctuation that bit already has 0x20 set, which is
/// exactly what makes an arithmetic fold (`| 0x20`) collapse these keys onto
/// each other. Pinned to the exact role rather than to "some role", so a key
/// that lands in the wrong mask fails even though it is still bound to
/// something.
const VIQR_PUNCTUATION: [(char, bool, bool); 8] = [
    ('^', true, false),
    ('(', true, false),
    ('+', true, false),
    ('`', false, true),
    ('\'', false, true),
    ('?', false, true),
    ('~', false, true),
    ('.', false, true),
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

/// Every VIQR punctuation key is bound, and to exactly one mask. Punctuation has
/// no upper case, so there is no fold to survive here — the property under test
/// is the one the `0x20` bit would break: a key whose bit already has that bit
/// set must not be reachable through the neighbour it would fold onto.
#[test]
fn viqr_punctuation_keys_hold_exactly_one_role() {
    let viqr = DefaultKeymap::viqr();
    for &(ch, is_shape, is_tone) in &VIQR_PUNCTUATION {
        assert_eq!(viqr.is_shape_key(ch), is_shape, "viqr {ch:?}: shape role");
        assert_eq!(viqr.is_tone_key(ch), is_tone, "viqr {ch:?}: tone role");
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

/// Asserts `build` is rejected, with `expected` somewhere in the panic message.
///
/// The message is the assertion, not the panic. Every guard in `Rules::new`
/// panics with its own text, so matching it is what says *which* rule rejected
/// the layout. A bare `is_err()` would pass just as happily if the constructor
/// panicked for an unrelated reason — a bad index, say — which is precisely the
/// class of bug these guards exist to make unreachable.
fn assert_invalid(build: impl FnOnce() -> Rules<'static>, expected: &str) {
    let payload = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(build)) {
        Ok(_) => panic!("an invalid layout must be rejected, not accepted"),
        Err(payload) => payload,
    };

    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or_else(|| panic!("the panic payload was not a string"));

    assert!(
        message.contains(expected),
        "expected the rejection to mention {expected:?}, got {message:?}"
    );
}

#[test]
fn rules_reject_duplicate_tone_key() {
    assert_invalid(
        || {
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
        },
        "a tone key maps to multiple tones",
    );
}

#[test]
fn rules_reject_tone_shape_collision() {
    assert_invalid(
        || {
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
        },
        "a key maps to both a tone and a shape",
    );
}

#[test]
fn rules_reject_shape_same_owner_duplicate() {
    assert_invalid(
        || {
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
        },
        "a shape key applies multiple shapes to one owner",
    );
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
    assert_invalid(|| Rules::new(&[], &[], b"zz"), "a stroke key is duplicated");
}

#[test]
fn rules_reject_stroke_tone_collision() {
    assert_invalid(
        || {
            Rules::new(
                &[ToneRule {
                    key: b'z',
                    tone: Tone::Acute,
                }],
                &[],
                b"z",
            )
        },
        "a stroke key maps to both a stroke and a tone",
    );
}

#[test]
fn rules_reject_stroke_shape_collision() {
    assert_invalid(
        || {
            Rules::new(
                &[],
                &[ShapeRule {
                    key: b'z',
                    on: RootVowel::A,
                    shape: Shape::Breve,
                }],
                b"z",
            )
        },
        "a stroke key maps to both a stroke and a shape",
    );
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

    for rule in &HIGH_TONES {
        assert_invalid(
            || Rules::new(std::slice::from_ref(rule), &[], &[]),
            "key must be ASCII",
        );
    }
    assert_invalid(|| Rules::new(&[], &HIGH_SHAPE, &[]), "key must be ASCII");
    assert_invalid(|| Rules::new(&[], &[], &[0x80]), "key must be ASCII");
}
