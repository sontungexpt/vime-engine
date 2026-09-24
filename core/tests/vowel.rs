//! Vowel codec and classification tests.
//!
//! The public vowel codec covers:
//!
//! - 12 [`BaseVowel`]s
//! - 6 [`Tone`]s
//! - 2 cases
//! - 144 precomposed Vietnamese vowel characters in total.
//!
//! The tests verify:
//!
//! - BaseVowel ID ordering and round-trip
//! - Tone ID round-trip and bounds
//! - exhaustive encode/decode bijection
//! - exact lowercase/uppercase surface forms
//! - one exhaustive scan over the Unicode scalar range checking decoder
//!   coverage, round-trips and `is_vowel` consistency
//! - rejection of non-vowels
//! - shape replacement and `is_plain` / `is_shaped` consistency

use vime_engine::phonology::{
    decode_vowel, encode_vowel, is_vowel, BaseVowel, CasedBaseVowel, Shape, Tone,
};

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
const BASE_COUNT: usize = BASES.len();
const TONE_COUNT: usize = TONES.len();
const VOWEL_COUNT: usize = BASE_COUNT * TONE_COUNT * 2;

/// Expected lowercase surface forms.
///
/// Rows are ordered by [`BaseVowel`] priority ID.
/// Columns are ordered by [`Tone`] ID.
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

    for &base in BASES {
        for &tone in TONES {
            for &case in &[false, true] {
                let ch = encode_vowel(base, tone, case);

                assert_eq!(
                    decode_vowel(ch),
                    Some((CasedBaseVowel::new(base, case), tone)),
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
fn expected_surface_forms_match_codec() {
    for (base_id, row) in ALL_LOWER.iter().enumerate() {
        let base = BASES[base_id];

        for (tone_id, &lower) in row.iter().enumerate() {
            let tone = TONES[tone_id];

            assert_eq!(
                encode_vowel(base, tone, false),
                lower,
                "unexpected lowercase encoding for {base:?} {tone:?}"
            );

            assert_eq!(
                decode_vowel(lower),
                Some((CasedBaseVowel::new(base, false), tone)),
                "unexpected lowercase decoding for {lower:?}"
            );

            let upper = lower.to_uppercase().next().unwrap();

            assert_eq!(
                encode_vowel(base, tone, true),
                upper,
                "unexpected uppercase encoding for {base:?} {tone:?}"
            );

            assert_eq!(
                decode_vowel(upper),
                Some((CasedBaseVowel::new(base, true), tone)),
                "unexpected uppercase decoding for {upper:?}"
            );
        }
    }
}

#[test]
fn decoder_coverage_round_trips_and_matches_is_vowel() {
    let mut count = 0;

    for cp in 0..=0x10FFFF {
        let Some(ch) = char::from_u32(cp) else {
            continue;
        };

        let Some((cased, tone)) = decode_vowel(ch) else {
            assert!(
                !is_vowel(ch),
                "is_vowel/decode_vowel mismatch at U+{cp:04X} {ch:?}"
            );
            continue;
        };

        assert!(
            is_vowel(ch),
            "is_vowel/decode_vowel mismatch at U+{cp:04X} {ch:?}"
        );

        assert_eq!(
            encode_vowel(cased.get(), tone, cased.is_upper()),
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
fn ascii_vowels_decode_as_flat() {
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
            Some((CasedBaseVowel::new(base, false), Tone::Flat)),
            "unexpected decoding for {ch:?}"
        );

        assert!(is_vowel(ch));

        assert_eq!(encode_vowel(base, Tone::Flat, false), ch);
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
            Some((CasedBaseVowel::new(base, true), Tone::Flat)),
            "unexpected decoding for {ch:?}"
        );

        assert!(is_vowel(ch));

        assert_eq!(encode_vowel(base, Tone::Flat, true), ch);
    }
}

#[test]
fn replace_shape_matches_from_parts() {
    const SHAPES: &[Shape] = &[Shape::None, Shape::Circumflex, Shape::Breve, Shape::Horn];

    for &base in BASES {
        for &shape in SHAPES {
            assert_eq!(
                base.replace_shape(shape),
                BaseVowel::from_parts(base.root(), shape),
                "shape replacement mismatch for {base:?} + {shape:?}"
            );
        }
    }
}

#[test]
fn shape_queries_match_shape_extraction() {
    // `is_plain` / `is_shaped` must agree with the bit-extracted `shape()` for
    // all 12 variants; guards regression if `BaseVowel` discriminants are
    // reordered.
    for &base in BASES {
        let shaped = base.shape() != Shape::None;

        assert_eq!(base.is_plain(), !shaped, "is_plain mismatch for {base:?}");

        assert_eq!(base.is_shaped(), shaped, "is_shaped mismatch for {base:?}");
    }
}

#[test]
fn shaped_vowels_have_higher_priority_than_unshaped_vowels() {
    for &base in BASES {
        if base.shape() == Shape::None {
            assert!(
                base.id() <= BaseVowel::A.id(),
                "{base:?} is unshaped but has a shaped-vowel priority ID"
            );
        } else {
            assert!(
                base.id() > BaseVowel::A.id(),
                "{base:?} is shaped but has an unshaped-vowel priority ID"
            );
        }
    }
}

#[test]
fn cased_base_vowel_round_trips_every_base_and_case() {
    for &base in BASES {
        let lower = CasedBaseVowel::lower(base);
        let upper = CasedBaseVowel::upper(base);

        assert_eq!(lower.get(), base, "lower value mismatch for {base:?}");
        assert!(!lower.is_upper(), "lower case flag set for {base:?}");
        assert_eq!(upper.get(), base, "upper value mismatch for {base:?}");
        assert!(upper.is_upper(), "upper case flag missing for {base:?}");

        assert_eq!(CasedBaseVowel::new(base, false), lower);
        assert_eq!(CasedBaseVowel::new(base, true), upper);
    }
}

#[test]
fn cased_base_vowel_setters_preserve_the_other_field() {
    for &base in BASES {
        for initial_case in [false, true] {
            let mut cased = CasedBaseVowel::new(base, initial_case);

            cased.set_upper(!initial_case);
            assert_eq!(cased.get(), base, "set_upper changed {base:?}");
            assert_eq!(cased.is_upper(), !initial_case);

            for &replacement in BASES {
                cased.set_value(replacement);

                assert_eq!(cased.get(), replacement, "set_value failed");
                assert_eq!(cased.is_upper(), !initial_case, "set_value changed case");
            }
        }
    }
}

#[test]
fn cased_base_vowel_character_methods_match_encoder() {
    for &base in BASES {
        for &tone in TONES {
            for is_upper in [false, true] {
                let cased = CasedBaseVowel::new(base, is_upper);

                assert_eq!(
                    cased.to_char(),
                    encode_vowel(base, Tone::Flat, is_upper),
                    "flat character mismatch for {base:?}, upper={is_upper}"
                );
                assert_eq!(
                    cased.to_char_tone(tone),
                    encode_vowel(base, tone, is_upper),
                    "toned character mismatch for {base:?} {tone:?}, upper={is_upper}"
                );
            }
        }
    }
}

#[test]
fn cased_base_vowel_uses_two_bytes() {
    assert_eq!(
        std::mem::size_of::<CasedBaseVowel>(),
        std::mem::size_of::<u16>()
    );
}
