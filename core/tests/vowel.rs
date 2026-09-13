#![cfg(test)]

//! Vowel codec and classification tests.
//!
//! The public vowel codec covers:
//!
//! - 12 [`BaseVowel`]s
//! - 6 [`Tone`]s
//! - 2 [`Case`]s
//! - 144 precomposed Vietnamese vowel characters in total.
//!
//! The tests verify:
//!
//! - BaseVowel ID ordering and round-trip
//! - Tone ID round-trip and bounds
//! - exhaustive encode/decode bijection
//! - exact lowercase/uppercase surface forms
//! - decoder coverage over the Unicode scalar range
//! - rejection of non-vowels
//! - `is_vowel` consistency with the decoder
//! - shape replacement consistency

use vime_engine::{decode_vowel, encode_vowel, is_vowel, BaseVowel, Case, Shape, Tone};

const BASE_COUNT: usize = 12;
const TONE_COUNT: usize = 6;
const VOWEL_COUNT: usize = BASE_COUNT * TONE_COUNT * 2;

/// Expected lowercase surface forms.
///
/// Rows are ordered by [`BaseVowel`] priority ID.
/// Columns are ordered by [`Tone`] ID.
const ALL_LOWER: [[char; TONE_COUNT]; BASE_COUNT] = [
    // OHorn
    ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ'],
    // ECircumflex
    ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ'],
    // ABreve
    ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'],
    // OCircumflex
    ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ'],
    // ACircumflex
    ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'],
    // UHorn
    ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự'],
    // A
    ['a', 'á', 'à', 'ả', 'ã', 'ạ'],
    // O
    ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'],
    // E
    ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'],
    // I
    ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'],
    // U
    ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'],
    // Y
    ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'],
];

const BASES: [BaseVowel; BASE_COUNT] = [
    BaseVowel::OHorn,
    BaseVowel::ECircumflex,
    BaseVowel::ABreve,
    BaseVowel::OCircumflex,
    BaseVowel::ACircumflex,
    BaseVowel::UHorn,
    BaseVowel::A,
    BaseVowel::O,
    BaseVowel::E,
    BaseVowel::I,
    BaseVowel::U,
    BaseVowel::Y,
];

const TONES: [Tone; TONE_COUNT] = [
    Tone::Flat,
    Tone::Acute,
    Tone::Grave,
    Tone::Hook,
    Tone::Tilde,
    Tone::Dot,
];

const CASES: [Case; 2] = [Case::Lower, Case::Upper];

#[test]
fn test_base_vowel_is_no_shape_invariant() {
    // Verify `is_no_shape()` against the bit-extracted `shape()` for all 12 variants.
    // This guards against regression if `BaseVowel` discriminants are reordered.
    for vowel in BASES.iter() {
        let expected_is_no_shape = vowel.shape() == Shape::None;

        assert_eq!(
            vowel.is_no_shape(),
            expected_is_no_shape,
            "Invariant violated for variant {:?}: expected is_no_shape() to be {}, but got {}",
            vowel,
            expected_is_no_shape,
            vowel.is_no_shape()
        );
    }
}

#[test]
fn base_vowel_ids_are_valid_and_ordered() {
    for (expected_id, &base) in BASES.iter().enumerate() {
        assert_eq!(
            base.id(),
            expected_id,
            "{base:?} has unexpected priority ID"
        );

        assert_eq!(
            BaseVowel::from_id(expected_id),
            Ok(base),
            "BaseVowel::from_id({expected_id}) is inconsistent"
        );
    }

    assert!(BaseVowel::from_id(BASE_COUNT).is_err());
    assert!(BaseVowel::from_id(usize::MAX).is_err());

    assert!(BaseVowel::OHorn > BaseVowel::Y);
    assert!(BaseVowel::A > BaseVowel::Y);
}

#[test]
fn tone_ids_are_valid_and_ordered() {
    for (expected_id, &tone) in TONES.iter().enumerate() {
        assert_eq!(
            Tone::from_id(expected_id),
            tone,
            "Tone::from_id({expected_id}) is inconsistent"
        );

        assert_eq!(tone as usize, expected_id, "{tone:?} has unexpected ID");
    }

    assert!(Tone::from_id(TONE_COUNT) == Tone::Flat);
    assert!(Tone::from_id(usize::MAX) == Tone::Flat);
}

#[test]
fn encode_decode_is_bijective() {
    let mut seen = [None; VOWEL_COUNT];
    let mut count = 0;

    for (base_id, &base) in BASES.iter().enumerate() {
        assert_eq!(base.id(), base_id);

        for (tone_id, &tone) in TONES.iter().enumerate() {
            assert_eq!(tone as usize, tone_id);

            for &case in &CASES {
                let ch = encode_vowel(base, tone, case);

                assert_eq!(
                    decode_vowel(ch),
                    Some((base, tone, case)),
                    "decode mismatch for {base:?} {tone:?} {case:?}"
                );

                assert!(
                    !seen[..count].contains(&Some(ch)),
                    "duplicate encoded character {ch:?}"
                );

                seen[count] = Some(ch);
                count += 1;
            }
        }
    }

    assert_eq!(count, VOWEL_COUNT);
    assert!(seen.iter().all(Option::is_some));
}

#[test]
fn expected_lowercase_vowels_match_codec() {
    for (base_id, row) in ALL_LOWER.iter().enumerate() {
        let base = BASES[base_id];

        for (tone_id, &expected) in row.iter().enumerate() {
            let tone = TONES[tone_id];

            assert_eq!(
                encode_vowel(base, tone, Case::Lower),
                expected,
                "unexpected lowercase encoding for {base:?} {tone:?}"
            );

            assert_eq!(
                decode_vowel(expected),
                Some((base, tone, Case::Lower)),
                "unexpected lowercase decoding for {expected:?}"
            );
        }
    }
}

#[test]
fn expected_uppercase_vowels_match_codec() {
    for (base_id, row) in ALL_LOWER.iter().enumerate() {
        let base = BASES[base_id];

        for (tone_id, &lower) in row.iter().enumerate() {
            let tone = TONES[tone_id];
            let upper = lower.to_uppercase().next().unwrap();

            assert_eq!(
                encode_vowel(base, tone, Case::Upper),
                upper,
                "unexpected uppercase encoding for {base:?} {tone:?}"
            );

            assert_eq!(
                decode_vowel(upper),
                Some((base, tone, Case::Upper)),
                "unexpected uppercase decoding for {upper:?}"
            );
        }
    }
}

#[test]
fn decode_encode_round_trips_every_known_vowel() {
    let mut count = 0;

    for cp in 0..=0x10FFFF {
        let Some(ch) = char::from_u32(cp) else {
            continue;
        };

        let Some((base, tone, case)) = decode_vowel(ch) else {
            continue;
        };

        assert_eq!(
            encode_vowel(base, tone, case),
            ch,
            "decode/encode round-trip failed for U+{cp:04X} {ch:?}"
        );

        count += 1;
    }

    assert_eq!(
        count, VOWEL_COUNT,
        "decoder must recognize exactly {VOWEL_COUNT} characters"
    );
}

#[test]
fn is_vowel_matches_decoder() {
    for cp in 0..=0x10FFFF {
        let Some(ch) = char::from_u32(cp) else {
            continue;
        };

        assert_eq!(
            is_vowel(ch),
            decode_vowel(ch).is_some(),
            "is_vowel/decode_vowel mismatch at U+{cp:04X} {ch:?}"
        );
    }
}

#[test]
fn rejects_non_vowels() {
    const INVALID: [char; 31] = [
        // ASCII consonants
        'b', 'c', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'w',
        'x', 'z', // Vietnamese stroke
        'đ', 'Đ', // Digits / punctuation / whitespace
        '0', '9', '!', '@', '#', ' ', '\n', '\t', // Non-Vietnamese Latin letter
        'å',
    ];

    for ch in INVALID {
        assert_eq!(
            decode_vowel(ch),
            None,
            "{ch:?} must not decode as a Vietnamese vowel"
        );

        assert!(!is_vowel(ch), "{ch:?} must not be classified as a vowel");
    }
}

#[test]
fn ascii_vowels_decode_as_flat_lowercase() {
    for (base, ch) in [
        (BaseVowel::A, 'a'),
        (BaseVowel::E, 'e'),
        (BaseVowel::I, 'i'),
        (BaseVowel::O, 'o'),
        (BaseVowel::U, 'u'),
        (BaseVowel::Y, 'y'),
    ] {
        assert_eq!(
            decode_vowel(ch),
            Some((base, Tone::Flat, Case::Lower)),
            "unexpected decoding for {ch:?}"
        );

        assert!(is_vowel(ch));

        assert_eq!(encode_vowel(base, Tone::Flat, Case::Lower), ch);
    }

    for (base, ch) in [
        (BaseVowel::A, 'A'),
        (BaseVowel::E, 'E'),
        (BaseVowel::I, 'I'),
        (BaseVowel::O, 'O'),
        (BaseVowel::U, 'U'),
        (BaseVowel::Y, 'Y'),
    ] {
        assert_eq!(
            decode_vowel(ch),
            Some((base, Tone::Flat, Case::Upper)),
            "unexpected decoding for {ch:?}"
        );

        assert!(is_vowel(ch));

        assert_eq!(encode_vowel(base, Tone::Flat, Case::Upper), ch);
    }
}

#[test]
fn replace_shape_matches_from_parts() {
    const SHAPES: [Shape; 4] = [Shape::None, Shape::Circumflex, Shape::Breve, Shape::Horn];

    for &base in &BASES {
        for &shape in &SHAPES {
            assert_eq!(
                base.replace_shape(shape),
                BaseVowel::from_parts(base.root(), shape),
                "shape replacement mismatch for {base:?} + {shape:?}"
            );
        }
    }
}

#[test]
fn shaped_vowels_have_higher_priority_than_unshaped_vowels() {
    for &base in &BASES {
        if base.shape() == Shape::None {
            assert!(
                base.id() >= BaseVowel::A.id(),
                "{base:?} is unshaped but has a shaped-vowel priority ID"
            );
        } else {
            assert!(
                base.id() < BaseVowel::A.id(),
                "{base:?} is shaped but has an unshaped-vowel priority ID"
            );
        }
    }
}
