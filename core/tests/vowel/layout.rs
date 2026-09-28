//! Vowel struct layout, equality, hash, and mutation tests.
//!
//! Covers: struct sizes, bit layout, equality/hash behavior,
//! mutating setters, copying `with_*` builders, `remove_tone` /
//! `remove_shape` and their `without_*` copies, and ordering by
//! priority vs packed value.

use vime_engine::phonology::{
    BaseVowel, BaseVowelId, RootVowel, Shape, Tone, Vowel,
};

use super::*;

/// Inserts the `Vowel` itself rather than its packed bits.
#[test]
fn vowel_equality_and_hash_distinguish_every_variant() {
    let mut seen = HashSet::with_capacity(VOWEL_COUNT);

    each_vowel(|base, tone, upper| {
        let vowel = Vowel::new(base, tone, upper);
        assert!(seen.insert(vowel), "duplicate value for {vowel:?}");
    });

    assert_eq!(seen.len(), VOWEL_COUNT);
}

/// base().id() must track the base vowel through every mutation, and
/// equality must ignore it.
#[test]
fn base_id_tracks_the_base_and_is_ignored_by_equality() {
    for &base in BASES {
        for &other in BASES {
            if base == other {
                continue;
            }

            let v = Vowel::lower(base, Tone::Acute);
            assert_eq!(v.base().id(), base.id(), "new() for {base:?}");

            let mut w = v;
            w.set_base(other);
            assert_eq!(w.base().id(), other.id(), "set_base for {other:?}");
            assert_eq!(w.base(), other, "set_base did not change the base");

            assert_eq!(
                v.with_base(other).base().id(),
                other.id(),
                "with_base for {other:?}"
            );
            assert_eq!(
                v.without_shape().base().id(),
                base.remove_shape().id(),
                "without_shape for {base:?}"
            );
            assert_eq!(
                v.without_tone().base().id(),
                base.id(),
                "tone/case changes must not change the base id"
            );
            assert_eq!(
                v.with_upper(true).base().id(),
                base.id(),
                "case changes must not change the base id"
            );
        }
    }

    for &base in BASES {
        for &tone in TONES {
            for upper in CASES {
                let v = Vowel::new(base, tone, upper);
                assert_eq!(v.to_char(), encode_vowel(base.id(), tone, upper));
                let mut w = v;
                w.set_base(base);
                assert_eq!(w.to_char(), encode_vowel(base.id(), tone, upper));
            }
        }
    }
}

/// The base field is 5 bits because that is all a `BaseVowel` needs.
#[test]
fn base_field_is_five_bits_wide() {
    const BASE_FIELD_MAX: u16 = (1 << 5) - 1;

    for &base in BASES {
        assert!(
            (base as u16) <= BASE_FIELD_MAX,
            "{base:?} (encoding {}) does not fit the 5-bit base field",
            base as u16
        );
    }

    let widest = BASES.iter().map(|&base| base as u16).max().unwrap();

    assert!(
        widest > 15,
        "the widest base encoding ({widest}) now fits in 4 bits, so BASE_WIDTH is too wide"
    );
    assert!(
        widest <= BASE_FIELD_MAX,
        "the widest base encoding ({widest}) no longer fits the 5-bit base field"
    );
}

/// `#[repr(u8)]` is the reason the layout exists.
#[test]
fn base_vowel_uses_one_byte() {
    assert_eq!(std::mem::size_of::<BaseVowel>(), 1);
    assert_eq!(std::mem::align_of::<BaseVowel>(), 1);
}

#[test]
fn vowel_uses_two_bytes() {
    assert_eq!(std::mem::size_of::<Vowel>(), std::mem::size_of::<u16>());
}

#[test]
fn vowel_uses_the_documented_bit_layout() {
    each_vowel(|base, tone, upper| {
        let bits = Vowel::new(base, tone, upper).bits();

        assert_eq!((bits >> 4) & 0b1_1111, base as u16, "base field mismatch");
        assert_eq!((bits >> 1) & 0b0111, tone as u16, "tone field mismatch");
        assert_eq!(bits & 1, upper as u16, "case field mismatch");
        assert_eq!(bits >> 9, 0, "reserved bits must stay zero for {base:?}");
    });
}

/// Mutating setters preserve fields other than the one being changed.
#[test]
fn vowel_setters_preserve_the_other_field() {
    each_vowel(|base, tone, initial_case| {
        let flipped = !initial_case;

        let mut cased = Vowel::new(base, tone, initial_case);
        cased.set_upper(flipped);

        assert_eq!(cased.base(), base, "set_upper changed base");
        assert_eq!(cased.tone(), tone, "set_upper changed tone");
        assert_eq!(cased.is_upper(), flipped);
        assert_eq!(cased, Vowel::new(base, tone, flipped));

        for &replacement in BASES {
            let mut cased = Vowel::new(base, tone, flipped);
            cased.set_base(replacement);

            assert_eq!(cased.base(), replacement, "set_base failed");
            assert_eq!(cased.tone(), tone, "set_base changed tone");
            assert_eq!(cased.is_upper(), flipped, "set_base changed case");
        }

        for &replacement in TONES {
            let mut cased = Vowel::new(base, tone, flipped);
            cased.set_tone(replacement);

            assert_eq!(cased.tone(), replacement, "set_tone failed");
            assert_eq!(cased.base(), base, "set_tone changed base");
            assert_eq!(cased.is_upper(), flipped, "set_tone changed case");
        }
    });
}

/// Copying with_* methods return modified copies without mutating the original.
#[test]
fn with_methods_return_modified_copies() {
    each_vowel(|base, tone, upper| {
        let vowel = Vowel::new(base, tone, upper);

        for &replacement in BASES {
            assert_eq!(
                vowel.with_base(replacement),
                Vowel::new(replacement, tone, upper),
                "with_base mismatch"
            );
        }

        for &replacement in TONES {
            assert_eq!(
                vowel.with_tone(replacement),
                Vowel::new(base, replacement, upper),
                "with_tone mismatch"
            );
        }

        for flag in CASES {
            assert_eq!(
                vowel.with_upper(flag),
                Vowel::new(base, tone, flag),
                "with_upper mismatch"
            );
        }

        assert_eq!(
            vowel,
            Vowel::new(base, tone, upper),
            "with_* mutated the original"
        );
    });
}

/// remove_tone clears the tone and mutates in place.
#[test]
fn remove_tone_clears_the_tone_in_place() {
    each_vowel(|base, tone, upper| {
        let mut vowel = Vowel::new(base, tone, upper);
        let removed = vowel.remove_tone();

        assert_eq!(
            removed.tone(),
            Tone::Flat,
            "remove_tone did not clear the tone"
        );
        assert_eq!(removed.base(), base, "remove_tone changed base");
        assert_eq!(removed.is_upper(), upper, "remove_tone changed case");
        assert_eq!(*removed, Vowel::new(base, Tone::Flat, upper));
        assert_eq!(
            vowel,
            Vowel::new(base, Tone::Flat, upper),
            "remove_tone must mutate in place"
        );
    });
}

/// without_tone leaves the original untouched.
#[test]
fn without_tone_leaves_the_original_untouched() {
    each_vowel(|base, tone, upper| {
        let vowel = Vowel::new(base, tone, upper);

        assert_eq!(vowel.without_tone(), vowel.with_tone(Tone::Flat));
        assert_eq!(vowel.without_tone(), Vowel::new(base, Tone::Flat, upper));
        assert_eq!(
            vowel,
            Vowel::new(base, tone, upper),
            "without_tone mutated the original"
        );
    });

    assert_eq!(
        Vowel::new(BaseVowel::UHorn, Tone::Hook, true)
            .without_tone()
            .to_char(),
        'Ư'
    );
    assert_eq!(
        Vowel::new(BaseVowel::A, Tone::Tilde, false)
            .without_tone()
            .to_char(),
        'a'
    );
}

/// remove_shape strips the shape and mutates in place.
#[test]
fn remove_shape_strips_the_shape_and_keeps_tone_and_case() {
    each_vowel(|base, tone, upper| {
        let mut vowel = Vowel::new(base, tone, upper);
        let expected = Vowel::new(base.remove_shape(), tone, upper);
        let removed = vowel.remove_shape();

        assert_eq!(
            removed.base().shape(),
            Shape::None,
            "{base:?} kept its shape"
        );
        assert_eq!(removed.tone(), tone, "remove_shape changed tone");
        assert_eq!(removed.is_upper(), upper, "remove_shape changed case");
        assert_eq!(*removed, expected);
        assert_eq!(vowel, expected, "remove_shape must mutate in place");
        assert_eq!(Vowel::new(base, tone, upper).without_shape(), expected);
    });
}

/// remove_shape on decoded characters matches the stripped spelling.
#[test]
fn remove_shape_on_decoded_characters_matches_the_stripped_spelling() {
    for (shaped, plain) in STRIPPED {
        let vowel = Vowel::from_char(shaped).expect("shaped character must decode");

        assert_eq!(
            vowel.without_shape().to_char(),
            plain,
            "stripping {shaped:?} failed"
        );
        assert!(
            vowel.base().is_shaped(),
            "{shaped:?} should decode to a shaped vowel"
        );
    }

    for plain in ['a', 'A', 'e', 'E', 'i', 'I', 'o', 'O', 'u', 'U', 'y', 'Y', 'ỵ'] {
        let vowel = Vowel::from_char(plain).expect("plain character must decode");

        assert!(
            vowel.base().is_plain(),
            "{plain:?} should decode to a plain vowel"
        );
        assert_eq!(
            vowel.without_shape(),
            vowel,
            "stripping {plain:?} changed the value"
        );
    }
}

/// `tone_placement`'s >3-vowel fallback needs `Ord` to mean
/// *tone-placement priority*, not packed value.
#[test]
fn ordering_follows_priority_not_packed_value() {
    let inverted = BASES
        .windows(2)
        .filter(|pair| (pair[0] as u8) > (pair[1] as u8))
        .count();
    assert!(
        inverted > 0,
        "packed order now matches priority order, so this test proves nothing"
    );

    for pair in BASES.windows(2) {
        assert!(
            pair[0] < pair[1],
            "{:?} must sort before {:?}",
            pair[0],
            pair[1]
        );
        assert!(
            pair[0].id() < pair[1].id(),
            "{:?} must have the lower ID",
            pair[0]
        );
    }

    for (i, &a) in BASES.iter().enumerate() {
        for (j, &b) in BASES.iter().enumerate() {
            if i < j {
                assert!(
                    a < b,
                    "{a:?} must sort before {b:?} (ids {:?} vs {:?})",
                    a.id(),
                    b.id()
                );
            } else if i > j {
                assert!(
                    b < a,
                    "{b:?} must sort before {a:?} (ids {:?} vs {:?})",
                    b.id(),
                    a.id()
                );
            }
        }
    }

    let mut sorted = BASES.to_vec();
    sorted.sort();
    assert_eq!(sorted, BASES);

    assert_eq!(BASES.iter().copied().max(), Some(BaseVowel::OHorn));
    assert_eq!(BASES.iter().copied().min(), Some(BaseVowel::Y));
}
