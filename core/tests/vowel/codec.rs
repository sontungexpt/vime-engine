//! Character encoding/decoding and surface form tests.
//!
//! Covers: encode/decode bijection, free functions, surface forms, decoder
//! coverage, rejection of non-vowels, ASCII vowels, classifier boundaries,
//! const-context usability, and character method consistency.

use std::collections::HashSet;

use vime_engine::phonology::{
    decode_vowel, encode_vowel, is_vowel, BaseVowel, BaseVowelId, RootVowel, Shape, Tone, Vowel,
};

use super::*;

/// The tone ID is the discriminant, which `encode_vowel` relies on when it
/// indexes the 144-entry table by `tone as u8`.
#[test]
fn tone_ids_are_their_discriminants() {
    for (expected_id, &tone) in TONES.iter().enumerate() {
        assert_eq!(tone as usize, expected_id, "{tone:?} has unexpected ID");
    }
    assert_eq!(TONES.len(), TONE_COUNT);
}

#[test]
fn vowel_round_trips_every_base_and_case() {
    each_vowel(|base, tone, _| {
        let lower = Vowel::lower(base, tone);
        let upper = Vowel::upper(base, tone);

        assert_eq!(lower.base(), base, "lower value mismatch for {base:?}");
        assert!(!lower.is_upper(), "lower case flag set for {base:?}");
        assert_eq!(upper.base(), base, "upper value mismatch for {base:?}");
        assert!(upper.is_upper(), "upper case flag missing for {base:?}");

        assert_eq!(Vowel::new(base, tone, false), lower);
        assert_eq!(Vowel::new(base, tone, true), upper);
    });
}

#[test]
fn vowel_accessors_decompose_the_packed_value() {
    each_vowel(|base, tone, upper| {
        let vowel = Vowel::new(base, tone, upper);

        assert_eq!(vowel.base(), base, "base mismatch for {vowel:?}");
        assert_eq!(vowel.tone(), tone, "tone mismatch for {vowel:?}");
        assert_eq!(vowel.root(), base.root(), "root mismatch for {vowel:?}");
        assert_eq!(vowel.is_upper(), upper, "case mismatch for {vowel:?}");
        assert_eq!(
            vowel,
            Vowel::new(vowel.base(), vowel.tone(), vowel.is_upper())
        );
    });
}

// ─────────────────────────────── Character codec ───────────────────────────

#[test]
fn encode_decode_is_bijective() {
    let mut seen = HashSet::with_capacity(VOWEL_COUNT);

    each_vowel(|base, tone, upper| {
        let vowel = Vowel::new(base, tone, upper);
        let ch = vowel.to_char();

        assert_eq!(
            Vowel::from_char(ch),
            Some(vowel),
            "decode mismatch for {base:?} {tone:?} {upper:?}"
        );
        assert!(seen.insert(ch), "duplicate encoded character {ch:?}");
    });

    assert_eq!(seen.len(), VOWEL_COUNT);
}

#[test]
fn free_functions_agree_with_the_vowel_methods() {
    each_vowel(|base, tone, upper| {
        let vowel = Vowel::new(base, tone, upper);
        let ch = encode_vowel(base.id(), tone, upper);

        assert_eq!(ch, vowel.to_char());
        assert_eq!(decode_vowel(ch), Some(vowel));
        assert_eq!(decode_vowel(ch), Vowel::from_char(ch));
        assert!(is_vowel(ch));
    });

    assert_eq!(decode_vowel('b'), None);
    assert_eq!(decode_vowel('1'), None);
    assert_eq!(decode_vowel(' '), None);
    assert!(!is_vowel('b'));
}

#[test]
fn expected_surface_forms_match_codec() {
    for (base_id, row) in ALL_LOWER.iter().enumerate() {
        let base = BASES[base_id];

        for (tone_id, &lower) in row.iter().enumerate() {
            let tone = TONES[tone_id];
            let upper = lower.to_uppercase().next().unwrap();

            for (ch, is_upper) in [(lower, false), (upper, true)] {
                let vowel = Vowel::new(base, tone, is_upper);

                assert_eq!(
                    vowel.to_char(),
                    ch,
                    "unexpected encoding for {base:?} {tone:?} upper={is_upper}"
                );
                assert_eq!(
                    Vowel::from_char(ch),
                    Some(vowel),
                    "unexpected decoding for {ch:?}"
                );
            }
        }
    }
}

#[test]
fn decoder_coverage_round_trips_and_matches_is_vowel() {
    let mut count = 0;

    for cp in (0..=0xD7FF).chain(0xE000..=0x10FFFF) {
        let ch = char::from_u32(cp).expect("Unicode scalar value");
        let decoded = Vowel::from_char(ch);

        assert_eq!(
            is_vowel(ch),
            decoded.is_some(),
            "is_vowel/decode_vowel mismatch at U+{cp:04X} {ch:?}"
        );

        if let Some(decoded) = decoded {
            assert_eq!(
                Vowel::new(decoded.base(), decoded.tone(), decoded.is_upper()).to_char(),
                ch,
                "decode/encode round-trip failed for U+{cp:04X} {ch:?}"
            );
            count += 1;
        }
    }

    assert_eq!(
        count, VOWEL_COUNT,
        "decoder must recognize exactly {VOWEL_COUNT} characters"
    );
}

#[test]
fn rejects_non_vowels() {
    for ch in INVALID {
        assert_eq!(
            Vowel::from_char(ch),
            None,
            "{ch:?} must not decode as a Vietnamese vowel"
        );
        assert!(!is_vowel(ch), "{ch:?} must not be classified as a vowel");
    }
}

#[test]
fn ascii_vowels_decode_as_flat() {
    for (base, lower, upper) in [
        (BaseVowel::A, 'a', 'A'),
        (BaseVowel::E, 'e', 'E'),
        (BaseVowel::I, 'i', 'I'),
        (BaseVowel::O, 'o', 'O'),
        (BaseVowel::U, 'u', 'U'),
        (BaseVowel::Y, 'y', 'Y'),
    ] {
        for (ch, is_upper) in [(lower, false), (upper, true)] {
            assert_eq!(
                Vowel::from_char(ch),
                Some(Vowel::new(base, Tone::Flat, is_upper)),
                "unexpected decoding for {ch:?}"
            );
            assert!(is_vowel(ch));
            assert_eq!(Vowel::new(base, Tone::Flat, is_upper).to_char(), ch);
        }
    }
}

#[test]
fn vowel_classifier_covers_range_boundaries() {
    let cases = [
        ('@', false),
        ('A', true),
        ('Z', false),
        ('[', false),
        ('`', false),
        ('a', true),
        ('z', false),
        ('{', false),
        ('\u{00C0}', true),
        ('\u{00FD}', true),
        ('\u{00FE}', false),
        ('\u{0101}', false),
        ('\u{0102}', true),
        ('\u{01B0}', true),
        ('\u{01B1}', false),
        ('\u{1E9F}', false),
        ('\u{1EA0}', true),
        ('\u{1EF9}', true),
        ('\u{1EFA}', false),
    ];

    for (ch, expected) in cases {
        assert_eq!(
            is_vowel(ch),
            expected,
            "classifier mismatch for U+{:04X}",
            ch as u32
        );
        assert_eq!(
            decode_vowel(ch).is_some(),
            expected,
            "decoder mismatch for U+{:04X}",
            ch as u32
        );
    }
}

#[test]
fn vowel_character_methods_match_encoder() {
    each_vowel(|base, tone, is_upper| {
        let cased = Vowel::new(base, Tone::Flat, is_upper);

        assert_eq!(
            Vowel::new(cased.base(), cased.tone(), cased.is_upper()).to_char(),
            Vowel::new(base, Tone::Flat, is_upper).to_char(),
            "flat character mismatch for {base:?}, upper={is_upper}"
        );
        assert_eq!(
            Vowel::new(cased.base(), tone, cased.is_upper()).to_char(),
            Vowel::new(base, tone, is_upper).to_char(),
            "toned character mismatch for {base:?} {tone:?}, upper={is_upper}"
        );
    });
}

#[test]
fn vowel_decoded_value_stays_in_bounds() {
    each_vowel(|base, tone, case| {
        let vowel = Vowel::new(base, tone, case);
        let decoded =
            Vowel::from_char(vowel.to_char()).expect("codec must round-trip its own output");

        assert_eq!(decoded.tone(), tone);
        assert_eq!(decoded.base(), base);
        assert_eq!(decoded.is_upper(), case);
        assert_eq!(decoded.root(), base.root());
        assert_eq!(decoded.bits(), vowel.bits());
    });
}

const CONST_VOWEL: Vowel = Vowel::new(BaseVowel::ACircumflex, Tone::Dot, true);
const CONST_CHAR: char = CONST_VOWEL.to_char();
const CONST_ROOT: RootVowel = CONST_VOWEL.root();

#[test]
fn codec_is_usable_in_const_context() {
    assert_eq!(CONST_CHAR, 'Ậ');
    assert_eq!(CONST_ROOT, RootVowel::A);
    assert_eq!(CONST_VOWEL.base(), BaseVowel::ACircumflex);
    assert_eq!(CONST_VOWEL.tone(), Tone::Dot);
    assert!(CONST_VOWEL.is_upper());
    assert_eq!(Vowel::from_char(CONST_CHAR), Some(CONST_VOWEL));
    assert_eq!(decode_vowel(CONST_CHAR), Some(CONST_VOWEL));
    assert!(is_vowel(CONST_CHAR));
    assert_eq!(Vowel::lower(BaseVowel::E, Tone::Acute).to_char(), 'é');
    assert_eq!(
        BaseVowel::UHorn.replace_shape(Shape::None),
        Some(BaseVowel::U)
    );
    assert!(!is_vowel('q'));
}
