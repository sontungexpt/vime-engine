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
    decode_vowel, encode_vowel, is_vowel, BaseVowel, RootVowel, Shape, Tone, Vowel,
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
fn each_pair<A: Copy, B: Copy>(xs: &[A], ys: &[B], mut f: impl FnMut(A, B)) {
    for &x in xs {
        for &y in ys {
            f(x, y);
        }
    }
}

/// Runs `f` for every `(base, shape)` pair.
fn each_base_shape(f: impl FnMut(BaseVowel, Shape)) {
    each_pair(BASES, SHAPES, f);
}

/// Runs `f` for every `(root, shape)` pair.
fn each_root_shape(f: impl FnMut(RootVowel, Shape)) {
    each_pair(ROOTS, SHAPES, f);
}

/// Runs `f` for every `(base, tone, upper)` triple.
fn each_vowel(mut f: impl FnMut(BaseVowel, Tone, bool)) {
    for &base in BASES {
        for &tone in TONES {
            for &upper in &CASES {
                f(base, tone, upper);
            }
        }
    }
}

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

    // `RootVowel` discriminants run `Y, U, I, E, O, A`, not the order of the
    // `ROOTS` table, so check density rather than a specific assignment.
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
///
/// There used to be two ids: a build-dependent `id()` (a `popcnt`/`match` pair
/// selected by `cfg`) and a stable `priority_id()`, with `Ord` tracking the
/// latter. All three are now one function — `Ord` is `self.id().cmp(&other.id())`
/// — so there is no second source left to drift. The values still need pinning,
/// because the id order is a published contract that everything else keys off:
///
/// * it is **not** the packed-value order (`encode` is `(root << 2) | shape`,
///   so `UHorn` is 7 on the wire but 6 here),
/// * it is **not** the enum declaration order,
/// * `from_id` is a hand-written table that must be its exact inverse, and
/// * `tone_placement`'s `v > best` fallback picks its representative through
///   `Ord`, so `Ord` has to mean this order and not the packed one.
///
/// Pinning it literally means a future edit to `id`, `from_id`, `Ord`, or the
/// layout comment fails here rather than silently renumbering every lookup
/// table.
#[test]
fn base_vowel_id_table_is_pinned() {
    // (vowel, id) in tone-placement priority order.
    const EXPECTED: &[(BaseVowel, u8)] = &[
        (BaseVowel::Y, 0),
        (BaseVowel::U, 1),
        (BaseVowel::I, 2),
        (BaseVowel::E, 3),
        (BaseVowel::O, 4),
        (BaseVowel::A, 5),
        (BaseVowel::UHorn, 6),
        (BaseVowel::ACircumflex, 7),
        (BaseVowel::OCircumflex, 8),
        (BaseVowel::ABreve, 9),
        (BaseVowel::ECircumflex, 10),
        (BaseVowel::OHorn, 11),
    ];

    assert_eq!(
        EXPECTED.len(),
        BASE_COUNT,
        "every base vowel must be listed"
    );

    for &(vowel, want) in EXPECTED {
        assert_eq!(
            vowel.id(),
            want,
            "{vowel:?} has id {} but this test pins {want}",
            vowel.id()
        );
        assert_eq!(
            BaseVowel::from_id(want as usize),
            Some(vowel),
            "from_id({want}) must return {vowel:?}"
        );
    }

    // The id is not the packed value: closed vowels encode low but rank high.
    assert_ne!(
        BaseVowel::UHorn.id(),
        BaseVowel::UHorn as u8,
        "id must not be the packed discriminant"
    );
    assert!(
        (BaseVowel::UHorn as u8) < (BaseVowel::ECircumflex as u8),
        "packed order differs from id order: UHorn encodes below ECircumflex \
         but ranks above it"
    );

    // `id` must be a bijection onto 0..COUNT: the table above is only a real
    // contract if every id is claimed exactly once.
    let mut seen = [false; BASE_COUNT];
    for &vowel in BASES {
        let id = vowel.id() as usize;
        assert!(id < BASE_COUNT, "{vowel:?} has out-of-range id {id}");
        assert!(!seen[id], "id {id} is claimed by more than one vowel");
        seen[id] = true;
    }
    assert!(
        seen.iter().all(|&s| s),
        "some id in 0..{BASE_COUNT} is unused"
    );

    // Out of range on both ends.
    assert!(BaseVowel::from_id(BASE_COUNT).is_none());
    assert!(BaseVowel::from_id(usize::MAX).is_none());

    // `Ord` must agree with `id` for every pair, not just adjacent ones in
    // `BASES`: `tone_placement` compares vowels pairwise, so a disagreement
    // anywhere would change which vowel its fallback picks.
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
            let id = base.id() as usize;
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
///
/// The shifts are written out by hand instead of calling the private
/// `encode`, so the test pins the documented layout independently of the
/// implementation that produces it.
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

// ───────────────────────────────── Vowel ──────────────────────────────────

/// The tone ID is the discriminant, which `encode_vowel` relies on when it
/// indexes the 144-entry table by `tone as u8`.
///
/// This is the invariant that let `Tone::from_id` be deleted: with ID equal to
/// discriminant there is no separate lookup to get wrong. Named for what it
/// checks - `Tone` is deliberately unordered, so there is no order to verify.
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

/// The base field is 5 bits because that is all a `BaseVowel` needs.
///
/// This is what makes `Vowel::base` a lossless `u16 -> u8` narrowing. If a
/// thirteenth vowel were added with a higher encoding, this fails instead of
/// the field quietly needing a sixth bit.
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

    // 5 bits must be both necessary and sufficient: too few and the widest
    // vowel would not fit, too many and the field would admit encodings no
    // `BaseVowel` has. Asserted as a range rather than a literal so a
    // reordering of `RootVowel` cannot silently change the required width.
    assert!(
        widest > 15,
        "the widest base encoding ({widest}) now fits in 4 bits, so BASE_WIDTH is too wide"
    );
    assert!(
        widest <= BASE_FIELD_MAX,
        "the widest base encoding ({widest}) no longer fits the 5-bit base field"
    );
}

/// `#[repr(u8)]` is the reason the layout exists: if `BaseVowel` ever grows a
/// second byte, the 5-bit field in `Vowel` stops being a narrowing and every
/// size claim above it becomes wrong. Asserted here rather than trusted from
/// the attribute, since `repr(u8)` alone does not pin the *size* of an enum
/// whose widest variant is still one byte.
#[test]
fn base_vowel_uses_one_byte() {
    assert_eq!(std::mem::size_of::<BaseVowel>(), 1);
    assert_eq!(std::mem::align_of::<BaseVowel>(), 1);
}

#[test]
fn vowel_uses_two_bytes() {
    assert_eq!(std::mem::size_of::<Vowel>(), std::mem::size_of::<u16>());
}

/// Inserts the `Vowel` itself rather than its packed bits, so this exercises
/// `Hash` and `Eq` on `Vowel` - a derived impl that could diverge from the bit
/// layout - instead of only proving the bits are unique.
#[test]
fn vowel_equality_and_hash_distinguish_every_variant() {
    let mut seen = HashSet::with_capacity(VOWEL_COUNT);

    each_vowel(|base, tone, upper| {
        let vowel = Vowel::new(base, tone, upper);

        assert!(seen.insert(vowel), "duplicate value for {vowel:?}");
    });

    assert_eq!(seen.len(), VOWEL_COUNT);
}

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

        // Each setter starts from a known state, so a failure names the setter
        // that broke rather than reporting a value left over from the
        // previous loop.
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

    for plain in [
        'a', 'A', 'e', 'E', 'i', 'I', 'o', 'O', 'u', 'U', 'y', 'Y', 'ỵ',
    ] {
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

    // `seen.len() == VOWEL_COUNT` is the coverage proof: every one of the 144
    // triples produced a distinct character, so there is nothing left to
    // re-derive and re-check in a second pass.
    assert_eq!(seen.len(), VOWEL_COUNT);
}

#[test]
fn free_functions_agree_with_the_vowel_methods() {
    each_vowel(|base, tone, upper| {
        let vowel = Vowel::new(base, tone, upper);
        let ch = encode_vowel(base, tone, upper);

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

    // Walk every Unicode scalar value. Surrogates are not scalar values, so
    // split around that gap and avoid a `char::from_u32` check on each step.
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
        ('\u{00C0}', true), // À, Latin-1 lower boundary
        ('\u{00FD}', true), // ý
        ('\u{00FE}', false),
        ('\u{0101}', false),
        ('\u{0102}', true), // ă, Latin Extended lower boundary
        ('\u{01B0}', true), // ư, Latin Extended upper boundary
        ('\u{01B1}', false),
        ('\u{1E9F}', false),
        ('\u{1EA0}', true), // Vietnamese block lower boundary
        ('\u{1EF9}', true), // Vietnamese block upper boundary
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
    // `base()` / `is_upper()` must invert `new` for every value the codec can
    // produce: the packed form always decomposes back into a valid
    // BaseVowel × case pair, never a value outside `BaseVowel`'s range.
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

/// `tone_placement`'s >3-vowel fallback picks a representative with
/// `if v > best`, so it needs `Ord` to mean *tone-placement priority*.
///
/// Derived `Ord` compared packed values, which is not the documented
/// tone-placement priority. That silently sent the `tone_placement` fallback to
/// a lower-priority vowel.
#[test]
fn ordering_follows_priority_not_packed_value() {
    // The two orders must actually disagree somewhere, or this test would pass
    // even if `Ord` went back to comparing packed values. Counted rather than
    // hardcoded so a reordering of `RootVowel` cannot void the check.
    let inverted = BASES
        .windows(2)
        .filter(|pair| (pair[0] as u8) > (pair[1] as u8))
        .count();
    assert!(
        inverted > 0,
        "packed order now matches priority order, so this test proves nothing"
    );

    // Ord must follow the id order regardless.
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

    // Windows only covers neighbours, and transitivity is exactly the
    // assumption a regression would break. `tone_placement` compares arbitrary
    // pairs, so check every combination.
    for (i, &a) in BASES.iter().enumerate() {
        for (j, &b) in BASES.iter().enumerate() {
            if i < j {
                assert!(
                    a < b,
                    "{a:?} must sort before {b:?} (ids {} vs {})",
                    a.id(),
                    b.id()
                );
            } else if i > j {
                assert!(
                    b < a,
                    "{b:?} must sort before {a:?} (ids {} vs {})",
                    b.id(),
                    a.id()
                );
            }
        }
    }

    // The whole table, sorted, must equal declaration order.
    let mut sorted = BASES.to_vec();
    sorted.sort();
    assert_eq!(sorted, BASES);

    // `max` is what the fallback reaches for.
    assert_eq!(BASES.iter().copied().max(), Some(BaseVowel::OHorn));
    assert_eq!(BASES.iter().copied().min(), Some(BaseVowel::Y));
}
