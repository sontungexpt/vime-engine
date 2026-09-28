//! BaseVowel, Shape, Tone, and RootVowel property tests.
//!
//! Covers: marker enums, BaseVowel construction, ID table, root/shape mapping,
//! bit layout, shape queries, replace_shape, remove_shape, and ordering.

use std::collections::HashSet;

use vime_engine::phonology::{
    BaseVowel, BaseVowelId, RootVowel, Shape, Tone,
};

use super::*;

// ───────────────────────────── Shape and Tone ─────────────────────────────

#[test]
fn shape_is_some_excludes_none() {
    for (expected_id, &shape) in SHAPES.iter().enumerate() {
        assert_eq!(
            shape as u8, expected_id as u8,
            "{shape:?} has unexpected ID"
        );
        assert_eq!(
            shape.is_some(),
            expected_id != 0,
            "{shape:?} is_some mismatch"
        );
    }
}

#[test]
fn tone_is_some_excludes_flat() {
    for (expected_id, &tone) in TONES.iter().enumerate() {
        assert_eq!(
            tone.is_some(),
            expected_id != 0,
            "{tone:?} is_some mismatch"
        );
    }
}

#[test]
fn marker_enums_default_to_the_unmarked_variant() {
    assert_eq!(Shape::default(), Shape::None);
    assert_eq!(Tone::default(), Tone::Flat);
    assert!(!Shape::default().is_some());
    assert!(!Tone::default().is_some());
    assert_eq!(Shape::default() as u8, 0);
    assert_eq!(Tone::default() as u8, 0);
}

// ─────────────────────────────── BaseVowel ───────────────────────────────

#[test]
fn root_ids_are_dense_and_from_root_builds_plain_vowels() {
    assert_eq!(ROOTS.len(), 6);

    let mut ids: Vec<u8> = ROOTS.iter().map(|root| *root as u8).collect();
    ids.sort_unstable();
    assert_eq!(ids, [0, 1, 2, 3, 4, 5], "root IDs must be dense");

    for &root in ROOTS {
        let base = BaseVowel::from_root(root);

        assert_eq!(base.root(), root, "from_root round-trip failed");
        assert_eq!(
            base.shape(),
            Shape::None,
            "{root:?} produced a shaped vowel"
        );
        assert!(base.is_plain());
        assert!(!base.is_shaped());
        assert_eq!(base, base.remove_shape(), "{root:?} from_root is not plain");
    }
}

#[test]
fn base_vowel_count_covers_every_variant() {
    assert_eq!(BaseVowel::COUNT, 12);
    assert_eq!(BaseVowel::COUNT, BASE_COUNT);
    assert_eq!(
        BaseVowel::COUNT,
        EXPECTED_PARTS.iter().filter(|p| p.2.is_some()).count()
    );
}

#[test]
fn every_root_is_represented_by_at_least_one_base_vowel() {
    let mut total = 0;

    for &root in ROOTS {
        let count = BASES.iter().filter(|base| base.root() == root).count();

        assert!(count > 0, "{root:?} has no base vowel");
        total += count;
    }

    assert_eq!(total, BaseVowel::COUNT, "every base vowel must have a root");
}

/// Pins the exact `id()` values, which the rest of this file only derives
/// positionally from `BASES`.
#[test]
fn base_vowel_id_table_is_pinned() {
    const EXPECTED: &[(BaseVowel, BaseVowelId)] = &[
        (BaseVowel::Y, BaseVowelId::Y),
        (BaseVowel::U, BaseVowelId::U),
        (BaseVowel::I, BaseVowelId::I),
        (BaseVowel::E, BaseVowelId::E),
        (BaseVowel::O, BaseVowelId::O),
        (BaseVowel::A, BaseVowelId::A),
        (BaseVowel::UHorn, BaseVowelId::UHorn),
        (BaseVowel::ACircumflex, BaseVowelId::ACircumflex),
        (BaseVowel::OCircumflex, BaseVowelId::OCircumflex),
        (BaseVowel::ABreve, BaseVowelId::ABreve),
        (BaseVowel::ECircumflex, BaseVowelId::ECircumflex),
        (BaseVowel::OHorn, BaseVowelId::OHorn),
    ];

    assert_eq!(EXPECTED.len(), BASE_COUNT, "every base vowel must be listed");

    for &(vowel, want) in EXPECTED {
        assert_eq!(
            vowel.id(),
            want,
            "{vowel:?} has id {:?} but this test pins {want:?}",
            vowel.id()
        );
        assert_eq!(
            BaseVowel::from_id(want),
            Some(vowel),
            "from_id({want:?}) must return {vowel:?}"
        );
    }

    assert_ne!(
        BaseVowel::UHorn.id() as u8,
        BaseVowel::UHorn as u8,
        "id must not be the packed discriminant"
    );
    assert!(
        (BaseVowel::UHorn as u8) < (BaseVowel::ECircumflex as u8),
        "packed order differs from id order: UHorn encodes below ECircumflex \
         but ranks above it"
    );

    let mut seen = [false; BASE_COUNT];
    for &vowel in BASES {
        let id = vowel.id() as u8 as usize;
        assert!(id < BASE_COUNT, "{vowel:?} has out-of-range id {id}");
        assert!(!seen[id], "id {id} is claimed by more than one vowel");
        seen[id] = true;
    }
    assert!(
        seen.iter().all(|&s| s),
        "some id in 0..{BASE_COUNT} is unused"
    );

    assert!(BaseVowelId::from_u8(BASE_COUNT as u8).is_none());
    assert!(BaseVowelId::from_u8(u8::MAX).is_none());

    for (i, &a) in EXPECTED.iter().enumerate() {
        for (j, &b) in EXPECTED.iter().enumerate() {
            if i < j {
                assert!(a.0 < b.0, "{:?} must sort before {:?}", a.0, b.0);
            } else if i > j {
                assert!(b.0 < a.0, "{:?} must sort before {:?}", b.0, a.0);
            } else {
                assert_eq!(a.0, b.0);
            }
        }
    }
    assert!(BaseVowel::OHorn > BaseVowel::Y);
    assert!(BaseVowel::A > BaseVowel::Y);
}

#[test]
fn root_shape_table_matches_the_allowed_vowels() {
    for &(root, shape, expected) in &EXPECTED_PARTS {
        assert_eq!(
            BaseVowel::from_parts(root, shape),
            expected,
            "from_parts mismatch for {root:?} + {shape:?}"
        );

        if let Some(base) = expected {
            let id = base.id();
            assert_eq!(
                BaseVowel::from_id(id),
                Some(base),
                "id round-trip for {root:?} + {shape:?}"
            );
            assert_eq!(base.root(), root);
            assert_eq!(base.shape(), shape);
        }
    }
}

#[test]
fn every_root_shape_pair_is_covered_exactly_once() {
    each_root_shape(|root, shape| {
        let listed = EXPECTED_PARTS
            .iter()
            .filter(|(r, s, _)| *r == root && *s == shape)
            .count();

        assert_eq!(listed, 1, "table must list {root:?} + {shape:?} once");
    });
}

/// Checks the layout documented on `BaseVowel`: shape in bits 0-1, root in
/// bits 2-4, bits 5-7 unused.
#[test]
fn base_vowel_uses_the_documented_bit_layout() {
    let mut bits = HashSet::new();

    for &base in BASES {
        let raw = base as u8;

        assert!(
            bits.insert(raw),
            "{base:?} reuses the packed bits of another variant"
        );
        assert_eq!(raw & 0b0000_0011, base.shape() as u8, "shape bits mismatch");
        assert_eq!(
            (raw >> 2) & 0b0000_0111,
            base.root() as u8,
            "root bits mismatch"
        );
        assert_eq!(
            raw >> 5,
            0,
            "bits 5-7 are unused and must stay zero for {base:?}"
        );
        assert_eq!(
            raw,
            (base.root() as u8) << 2 | base.shape() as u8,
            "{base:?} is not (root << 2) | shape"
        );
    }

    assert_eq!(bits.len(), BaseVowel::COUNT);
}

#[test]
fn shape_queries_match_shape_extraction() {
    for &base in BASES {
        let shaped = base.shape() != Shape::None;

        assert_eq!(base.is_plain(), !shaped, "is_plain mismatch for {base:?}");
        assert_eq!(base.is_shaped(), shaped, "is_shaped mismatch for {base:?}");
    }
}

#[test]
fn has_shape_matches_only_its_own_shape() {
    each_base_shape(|base, shape| {
        assert_eq!(
            base.is_shape(shape),
            base.shape() == shape,
            "has_shape mismatch for {base:?} + {shape:?}"
        );
    });
}

#[test]
fn replace_shape_matches_from_parts() {
    each_base_shape(|base, shape| {
        assert_eq!(
            base.replace_shape(shape),
            BaseVowel::from_parts(base.root(), shape),
            "shape replacement mismatch for {base:?} + {shape:?}"
        );
    });

    assert_eq!(
        BaseVowel::A.replace_shape(Shape::Circumflex),
        Some(BaseVowel::ACircumflex)
    );
    assert_eq!(BaseVowel::A.replace_shape(Shape::Horn), None);
    assert_eq!(
        BaseVowel::OCircumflex.replace_shape(Shape::Horn),
        Some(BaseVowel::OHorn)
    );
    assert_eq!(BaseVowel::I.replace_shape(Shape::Horn), None);
}

#[test]
fn remove_shape_returns_the_plain_vowel_with_the_same_root() {
    for &base in BASES {
        let plain = base.remove_shape();

        assert_eq!(plain, BaseVowel::from_root(base.root()), "{base:?}");
        assert!(plain.is_plain());
        assert_eq!(plain.root(), base.root());
        assert_eq!(plain.shape(), Shape::None);
        assert_eq!(
            plain.remove_shape(),
            plain,
            "remove_shape must be idempotent"
        );
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
