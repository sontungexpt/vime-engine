//! Vowel codec and classification tests.
//!
//! The public vowel surface covers:
//!
//! - 6 [`RootVowel`]s
//! - 4 [`Shape`]s
//! - 12 [`BaseVowel`]s
//! - 6 [`Tone`]s
//! - 2 cases
//! - 144 precomposed Vietnamese vowel characters in total.
//!
//! The tests verify:
//!
//! - `Default`/`is_some` semantics for the marker enums
//! - `BaseVowel` ID ordering, round-trip, packed bit layout and one-byte size
//! - the `(root, shape)` -> ID table, including every invalid combination
//! - exhaustive encode/decode bijection
//! - exact lowercase/uppercase surface forms
//! - one exhaustive scan over the Unicode scalar range checking decoder
//!   coverage, round-trips and `is_vowel` consistency
//! - rejection of non-vowels
//! - `Vowel` field layout, accessors, mutating setters, copying `with_*`
//!   builders, `remove_tone` / `remove_shape` and their `without_*` copies
//! - `Eq` / `Hash` behavior, and `BaseVowel` ordering by priority ID. `Vowel`
//!   and `Tone` are deliberately unordered - only `BaseVowel` implements
//!   `Ord`, because it is the only one of the three whose stored value and
//!   priority order disagree.
//! - const-evaluability of the codec
//! - shape replacement and `is_plain` / `is_shaped` consistency
//!
//! Iteration helpers below walk the tables in canonical order (`root`/`base`
//! by priority ID, then `tone` by ID, then lowercase before uppercase) so no
//! test has to repeat the nested loops.

use std::collections::HashSet;

use vime_engine::phonology::{
    decode_vowel, encode_vowel, is_vowel, BaseVowel, BaseVowelId, RootVowel, Shape, Tone, Vowel,
};

/// Root letters in declaration order.
const ROOTS: &[RootVowel] = &[
    RootVowel::A,
    RootVowel::E,
    RootVowel::I,
    RootVowel::O,
    RootVowel::U,
    RootVowel::Y,
];

/// Diacritic shapes in declaration order.
const SHAPES: &[Shape] = &[Shape::None, Shape::Circumflex, Shape::Breve, Shape::Horn];

const BASES: &[BaseVowel] = &[
    BaseVowel::Y,
    BaseVowel::U,
    BaseVowel::I,
    BaseVowel::E,
    BaseVowel::O,
    BaseVowel::A,
    BaseVowel::UHorn,
    BaseVowel::ACircumflex,
    BaseVowel::OCircumflex,
    BaseVowel::ABreve,
    BaseVowel::ECircumflex,
    BaseVowel::OHorn,
];

const TONES: &[Tone] = &[
    Tone::Flat,
    Tone::Acute,
    Tone::Grave,
    Tone::Hook,
    Tone::Tilde,
    Tone::Dot,
];

/// Letter cases, lowercase first: case is the low bit of the packed value.
const CASES: [bool; 2] = [false, true];

const BASE_COUNT: usize = BASES.len();
const TONE_COUNT: usize = TONES.len();
const VOWEL_COUNT: usize = BASE_COUNT * TONE_COUNT * CASES.len();

/// Every `(root, shape)` pair with the [`BaseVowel`] Vietnamese allows.
///
/// Rows follow [`ROOTS`] then [`SHAPES`] (the declaration order of both enums,
/// which is the order the lookup table is indexed by). `None` marks a
/// combination that does not exist.
#[rustfmt::skip]
const EXPECTED_PARTS: [(RootVowel, Shape, Option<BaseVowel>); ROOTS.len() * SHAPES.len()] = [
    // A
    (RootVowel::A, Shape::None,       Some(BaseVowel::A)),
    (RootVowel::A, Shape::Circumflex, Some(BaseVowel::ACircumflex)),
    (RootVowel::A, Shape::Breve,      Some(BaseVowel::ABreve)),
    (RootVowel::A, Shape::Horn,       None),
    // E
    (RootVowel::E, Shape::None,       Some(BaseVowel::E)),
    (RootVowel::E, Shape::Circumflex, Some(BaseVowel::ECircumflex)),
    (RootVowel::E, Shape::Breve,      None),
    (RootVowel::E, Shape::Horn,       None),
    // I
    (RootVowel::I, Shape::None,       Some(BaseVowel::I)),
    (RootVowel::I, Shape::Circumflex, None),
    (RootVowel::I, Shape::Breve,      None),
    (RootVowel::I, Shape::Horn,       None),
    // O
    (RootVowel::O, Shape::None,       Some(BaseVowel::O)),
    (RootVowel::O, Shape::Circumflex, Some(BaseVowel::OCircumflex)),
    (RootVowel::O, Shape::Breve,      None),
    (RootVowel::O, Shape::Horn,       Some(BaseVowel::OHorn)),
    // U
    (RootVowel::U, Shape::None,       Some(BaseVowel::U)),
    (RootVowel::U, Shape::Circumflex, None),
    (RootVowel::U, Shape::Breve,      None),
    (RootVowel::U, Shape::Horn,       Some(BaseVowel::UHorn)),
    // Y
    (RootVowel::Y, Shape::None,       Some(BaseVowel::Y)),
    (RootVowel::Y, Shape::Circumflex, None),
    (RootVowel::Y, Shape::Breve,      None),
    (RootVowel::Y, Shape::Horn,       None),
];

/// Expected lowercase surface forms.
///
/// Rows are ordered by [`BaseVowel`] priority ID.
/// Columns are ordered by [`Tone`] ID.
#[rustfmt::skip]
const ALL_LOWER: [[char; TONE_COUNT]; BASE_COUNT] = [
    // Y
    ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'],
    // U
    ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'],
    // I
    ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'],
    // E
    ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'],
    // O
    ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'],
    // A
    ['a', 'á', 'à', 'ả', 'ã', 'ạ'],
    // UHorn
    ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự'],
    // ACircumflex
    ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'],
    // OCircumflex
    ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ'],
    // ABreve
    ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'],
    // ECircumflex
    ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ'],
    // OHorn
    ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ'],
];

/// Shaped spellings and the same spelling with the shape stripped.
const STRIPPED: [(char, char); 12] = [
    ('ấ', 'á'),
    ('Ậ', 'Ạ'),
    ('ừ', 'ù'),
    ('Ử', 'Ủ'),
    ('ợ', 'ọ'),
    ('Ệ', 'Ẹ'),
    ('ặ', 'ạ'),
    ('ê', 'e'),
    ('ô', 'o'),
    ('ă', 'a'),
    ('ư', 'u'),
    ('ơ', 'o'),
];

/// Characters that must never decode as a Vietnamese vowel.
const INVALID: [char; 31] = [
    // ASCII consonants
    'b', 'c', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x',
    'z', // Vietnamese stroke
    'đ', 'Đ', // Digits / punctuation / whitespace
    '0', '9', '!', '@', '#', ' ', '\n', '\t', // Non-Vietnamese Latin letter
    'å',
];

// ───────────────────────────── Iteration helpers ──────────────────────────────

/// Runs `f` for every `(x, y)` pair, `x` in the outer loop.
pub fn each_pair<A: Copy, B: Copy>(xs: &[A], ys: &[B], mut f: impl FnMut(A, B)) {
    for &x in xs {
        for &y in ys {
            f(x, y);
        }
    }
}

/// Runs `f` for every `(base, shape)` pair.
pub fn each_base_shape(f: impl FnMut(BaseVowel, Shape)) {
    each_pair(BASES, SHAPES, f);
}

/// Runs `f` for every `(root, shape)` pair.
pub fn each_root_shape(f: impl FnMut(RootVowel, Shape)) {
    each_pair(ROOTS, SHAPES, f);
}

/// Runs `f` for every `(base, tone, upper)` triple.
pub fn each_vowel(mut f: impl FnMut(BaseVowel, Tone, bool)) {
    for &base in BASES {
        for &tone in TONES {
            for &upper in &CASES {
                f(base, tone, upper);
            }
        }
    }
}

// ───────────────────────────────── BaseVowelId ──────────────────────────

#[test]
fn base_vowel_id_count_is_correct() {
    assert_eq!(BaseVowelId::COUNT, 12);
}

#[test]
fn base_vowel_id_discriminants_are_dense() {
    let mut seen = [false; BaseVowelId::COUNT];
    for &vowel in BASES {
        let id = vowel.id();
        let raw = id as u8 as usize;
        assert!(raw < BaseVowelId::COUNT, "{vowel:?} id out of range");
        assert!(!seen[raw], "id {raw} is duplicated");
        seen[raw] = true;
    }
    assert!(seen.iter().all(|&s| s), "some ids are unused");
}

#[test]
fn base_vowel_id_from_u8_returns_correct_variants() {
    for &vowel in BASES {
        let id = vowel.id();
        assert_eq!(BaseVowelId::from_u8(id as u8), Some(id));
    }
}

#[test]
fn base_vowel_id_from_u8_returns_none_for_out_of_bounds() {
    assert!(BaseVowelId::from_u8(BaseVowelId::COUNT as u8).is_none());
    assert!(BaseVowelId::from_u8(u8::MAX).is_none());
}

#[test]
fn base_vowel_id_from_u8_unchecked_is_safe_for_valid_ids() {
    for &vowel in BASES {
        let id = vowel.id();
        let reconstructed = unsafe { BaseVowelId::from_u8_unchecked(id as u8) };
        assert_eq!(reconstructed, id);
    }
}

#[test]
fn base_vowel_id_from_u8_is_inverse_of_id() {
    for vowel in BASES {
        let id = vowel.id();
        assert_eq!(BaseVowelId::from_u8(id as u8), Some(id));
    }
}

#[test]
fn base_vowel_id_as_u8_round_trips() {
    for &vowel in BASES {
        let id = vowel.id();
        let raw = id as u8;
        assert_eq!(BaseVowelId::from_u8(raw), Some(id));
        assert_eq!(unsafe { BaseVowelId::from_u8_unchecked(raw) }, id);
    }
}

#[test]
fn base_vowel_id_ordering_matches_discriminant_order() {
    let ids: Vec<BaseVowelId> = BASES.iter().copied().map(|b| b.id()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(sorted, ids);

    for pair in ids.windows(2) {
        assert!(pair[0] < pair[1]);
    }

    assert_eq!(ids.iter().copied().max(), Some(BaseVowelId::OHorn));
    assert_eq!(ids.iter().copied().min(), Some(BaseVowelId::Y));
}

#[test]
fn base_vowel_id_from_id_round_trips() {
    for &vowel in BASES {
        let id = vowel.id();
        assert_eq!(BaseVowel::from_id(id), Some(vowel));
    }
}

#[path = "vowel/codec.rs"]
mod codec;
#[path = "vowel/layout.rs"]
mod layout;
#[path = "vowel/properties.rs"]
mod properties;
