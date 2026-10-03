//! Vowel codec and classification tests for 144 Vietnamese vowels (12 base × 6 tones × 2 cases).
//!
//! Tests cover: marker enum density/ordering, BaseVowel ID table, encode/decode bijection,
//! Unicode scan, field layout, setters/getters, Eq/Hash, const-eval, shape queries.
//! Assertions are deduplicated: density via macro, round-trip once via whole-Vowel equality,
//! ordering pinned in one test.

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
///
/// `q` and `w` are here because the *English* letter set is not the Vietnamese
/// one; `đ` is the Vietnamese stroke letter; `å` is a real Latin letter that
/// just is not a Vietnamese vowel.
const INVALID: [char; 31] = [
    // ASCII consonants that are not Vietnamese vowel letters
    'b', 'c', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x',
    'z', // The Vietnamese stroke letter, in both cases
    'đ', 'Đ', // Digits, punctuation, whitespace
    '0', '9', '!', '@', '#', ' ', '\n', '\t',
    // A Latin letter that is not a Vietnamese vowel
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

// ──────────────────────── RootVowel / Shape / Tone ────────────────────────

/// Generates the density test for one `#[repr(u8)]` marker enum.
///
/// All three declare `id()` as `self as u8`, so every claim about their ids is
/// really a claim about *declaration order*, and two properties are worth
/// separating:
///
/// 1. `id()` stays the identity. If it ever becomes a remap, the lookup tables
///    indexed by `id()` silently point at the wrong entry.
/// 2. Ids run `0..N` with no gaps, in declaration order. `encode_vowel` indexes
///    its 144-entry table by `tone as u8`, so a gap or a reorder mislabels
///    every vowel past it.
///
/// The generated test walks the variants and asserts `id() == <position>`, which
/// establishes density and order together, plus the identity check above. The
/// optional `is_some =` argument names that enum's own predicate, so the
/// per-enum semantics cannot drift apart from the density claim either.
macro_rules! marker_enum_dense {
    (
        $name:ident, $ty:ident, [$($variant:ident),+ $(,)?]
        $(, is_some = $is_some:path)?
    ) => {
        #[test]
        fn $name() {
            const DECLARED: &[$ty] = &[$($ty::$variant),+];

            for (expected, variant) in DECLARED.iter().enumerate() {
                assert_eq!(
                    variant.id(),
                    *variant as u8,
                    "{variant:?}: id() must stay the raw discriminant, not a remap",
                );
                assert_eq!(
                    variant.id(),
                    expected as u8,
                    "{variant:?}: ids must run 0..{} with no gaps, in declaration order",
                    DECLARED.len(),
                );
                $(assert_eq!(
                    $is_some(*variant),
                    expected != 0,
                    "{variant:?}: is_some() must mean \"is not the unmarked variant\"",
                );)?
            }
        }
    };
}

marker_enum_dense!(
    root_vowels_are_dense_in_declared_order,
    RootVowel,
    [Y, U, I, E, O, A]
);

marker_enum_dense!(
    shapes_are_dense_in_declared_order,
    Shape,
    [None, Circumflex, Breve, Horn],
    is_some = Shape::is_some
);

marker_enum_dense!(
    tones_are_dense_in_declared_order,
    Tone,
    [Flat, Acute, Grave, Hook, Tilde, Dot],
    is_some = Tone::is_some
);

#[test]
fn marker_enums_default_to_the_unmarked_variant() {
    assert_eq!(Shape::default(), Shape::None);
    assert_eq!(Tone::default(), Tone::Flat);
    assert!(!Shape::default().is_some());
    assert!(!Tone::default().is_some());
}

// ─────────────────────────────── BaseVowel ───────────────────────────────

/// `from_root` must build the plain vowel for every root, and that vowel must
/// satisfy the `is_plain` / `remove_shape` invariants.
#[test]
fn from_root_builds_a_plain_vowel() {
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

/// Pins the exact `id()` values.
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

    assert_eq!(
        EXPECTED.len(),
        BASE_COUNT,
        "every base vowel must be listed"
    );
    assert_eq!(BaseVowelId::COUNT, 12);
    assert_eq!(
        BaseVowelId::COUNT,
        BASE_COUNT,
        "id count and variant count must agree"
    );
    for &(vowel, want) in EXPECTED {
        assert_eq!(
            vowel.id(),
            want,
            "{vowel:?} has id {:?} but this test pins {want:?}",
            vowel.id()
        );
        assert_eq!(
            BaseVowel::from_id(want),
            vowel,
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
        "packed order differs from id order"
    );

    // The ids are dense: every slot in `0..COUNT` is used exactly once, which
    // is what makes `from_id` a total function over that range.
    let mut seen = [false; BASE_COUNT];
    for &vowel in BASES {
        let id = vowel.id() as u8 as usize;
        assert!(id < BASE_COUNT, "{vowel:?} has out-of-range id {id}");
        assert!(!seen[id], "id {id} is duplicated");
        seen[id] = true;
    }
    assert!(
        seen.iter().all(|&s| s),
        "some id in 0..{BASE_COUNT} is unused"
    );
    assert!(BaseVowelId::from_u8(BASE_COUNT as u8).is_none());
    assert!(BaseVowelId::from_u8(u8::MAX).is_none());
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
                base,
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

/// Checks the layout documented on `BaseVowel`: shape in bits 0-1, root in bits 2-4.
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

// ───────────────────────────────── BaseVowelId ──────────────────────────

/// `from_u8` and `from_u8_unchecked` must accept every discriminant in
/// `0..COUNT` and agree on it, must decode every id a `BaseVowel` produces,
/// and must reject everything at or past `COUNT`.
#[test]
fn base_vowel_id_from_u8_covers_every_discriminant() {
    for raw in 0..BaseVowelId::COUNT as u8 {
        // SAFETY: `raw < BaseVowelId::COUNT` is this loop's condition, which is
        // exactly the safety contract of `from_u8_unchecked`.
        let unchecked = unsafe { BaseVowelId::from_u8_unchecked(raw) };
        assert_eq!(unchecked as u8, raw, "raw {raw} must decode to itself");
        assert_eq!(
            BaseVowelId::from_u8(raw),
            Some(unchecked),
            "raw {raw} mismatch"
        );
    }

    for &vowel in BASES {
        assert_eq!(BaseVowelId::from_u8(vowel.id() as u8), Some(vowel.id()));
    }

    assert!(BaseVowelId::from_u8(BaseVowelId::COUNT as u8).is_none());
    assert!(BaseVowelId::from_u8(u8::MAX).is_none());
}

// ─────────────────────────────────── Vowel ──────────────────────────────

/// The three constructors must agree: `lower` / `upper` differ only in the case
/// bit, and both must equal the equivalent explicit `Vowel::new`.
#[test]
fn lower_and_upper_constructors_agree_with_new() {
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

/// The `base().id()` accessor must track the base through every mutation, and
/// must be unaffected by the tone and case fields.
#[test]
fn base_id_tracks_the_base_through_every_mutation() {
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
                assert_eq!(v.to_char(), encode_vowel(base, tone, upper));
                let mut w = v;
                w.set_base(base);
                assert_eq!(w.to_char(), encode_vowel(base, tone, upper));
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

/// Specific bit values for known vowels.
#[test]
fn vowel_bits_match_expected_packed_values() {
    let vowel = Vowel::new(BaseVowel::A, Tone::Acute, false);
    assert_eq!(vowel.bits(), 0b0000_0001_0100_0010);
    let vowel = Vowel::new(BaseVowel::A, Tone::Acute, true);
    assert_eq!(vowel.bits(), 0b0000_0001_0100_0011);
    let vowel = Vowel::new(BaseVowel::Y, Tone::Flat, false);
    assert_eq!(vowel.bits(), 0b0000_0000_0000_0000);
    let vowel = Vowel::new(BaseVowel::OHorn, Tone::Dot, true);
    assert_eq!(vowel.bits(), 0b0000_0001_0011_1011);
}

/// The three setters each mutate in place and compose, so setting every field in
/// sequence lands on the same value as constructing it directly.
///
/// Note the setters return `()`, not `&mut Self` — unlike `remove_tone` /
/// `remove_shape`, they cannot be chained. That difference is deliberate: the
/// setters always write a known field, so there is nothing to read back from
/// the result.
#[test]
fn vowel_setters_mutate_in_place_and_compose() {
    let mut v = Vowel::new(BaseVowel::A, Tone::Acute, false);
    v.set_base(BaseVowel::O);
    v.set_tone(Tone::Grave);
    v.set_upper(true);
    assert_eq!(v, Vowel::new(BaseVowel::O, Tone::Grave, true));
    let mut v = Vowel::new(BaseVowel::E, Tone::Flat, false);
    v.set_base(BaseVowel::U);
    assert_eq!(v, Vowel::new(BaseVowel::U, Tone::Flat, false));
}

/// All 144 `(base, tone, case)` combinations must be distinct under `Eq` and
/// must land in distinct `HashSet` slots — otherwise the codec could map two
/// spellings onto one vowel without any round-trip noticing.
#[test]
fn vowel_equality_and_hash_distinguish_every_variant() {
    let mut seen = HashSet::with_capacity(VOWEL_COUNT);
    each_vowel(|base, tone, upper| {
        let vowel = Vowel::new(base, tone, upper);
        assert!(seen.insert(vowel), "duplicate value for {vowel:?}");
    });
    assert_eq!(seen.len(), VOWEL_COUNT);
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

/// remove_tone returns `&mut Self` for chaining.
#[test]
fn remove_tone_returns_self_for_chaining() {
    let mut v = Vowel::new(BaseVowel::A, Tone::Acute, false);
    v.remove_tone().remove_tone();
    assert_eq!(v, Vowel::new(BaseVowel::A, Tone::Flat, false));
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

/// remove_shape returns `&mut Self` for chaining.
#[test]
fn remove_shape_returns_self_for_chaining() {
    let mut v = Vowel::new(BaseVowel::ACircumflex, Tone::Acute, false);
    v.remove_shape().remove_shape();
    assert_eq!(v, Vowel::new(BaseVowel::A, Tone::Acute, false));
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

/// `tone_placement`'s >3-vowel fallback needs `Ord` to mean
/// *tone-placement priority*, not packed value — so [`BASES`] must be listed in
/// ascending id order while the packed discriminants run the other way.
///
/// Adjacent pairs are enough for the ordering proof: `Ord` is total and
/// transitive, so an ascending chain down the whole list establishes every
/// pairwise relation without the quadratic loop.
#[test]
fn ordering_follows_priority_not_packed_value() {
    // Guard: if the two orders ever agreed, this test would prove nothing.
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

    let mut sorted = BASES.to_vec();
    sorted.sort();
    assert_eq!(sorted, BASES);
    assert_eq!(BASES.iter().copied().max(), Some(BaseVowel::OHorn));
    assert_eq!(BASES.iter().copied().min(), Some(BaseVowel::Y));

    // The ids form the same ascending chain, so the two extrema must agree.
    let ids: Vec<BaseVowelId> = BASES.iter().map(|b| b.id()).collect();
    let mut sorted_ids = ids.clone();
    sorted_ids.sort();
    assert_eq!(sorted_ids, ids);
    assert_eq!(ids.iter().copied().max(), Some(BaseVowelId::OHorn));
    assert_eq!(ids.iter().copied().min(), Some(BaseVowelId::Y));
}

// ─────────────────────────────── Character codec ───────────────────────────

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

/// `encode_vowel` produces exactly 144 unique characters, one per
/// `(base, tone, case)`, and every one decodes back to the vowel it came from.
///
/// This is also the suite's round-trip proof. `Vowel::to_char` *is*
/// `encode_vowel(base.id(), tone, upper)` and `Vowel::from_char` *is*
/// `decode_vowel`, so combined with `free_functions_agree_with_the_vowel_methods`
/// this establishes `from_char(to_char(v)) == v` for all 144 — and since
/// `Vowel` is a `#[repr(transparent)] u16` with a derived `PartialEq`, that one
/// equality pins every field and every bit at once.
#[test]
fn encode_vowel_covers_all_144_combinations() {
    let mut seen = HashSet::with_capacity(VOWEL_COUNT);
    for &base in BASES {
        for &tone in TONES {
            for &upper in &CASES {
                let ch = encode_vowel(base, tone, upper);
                assert!(
                    seen.insert(ch),
                    "duplicate encode_vowel output for {base:?} {tone:?} {upper:?}"
                );
                assert_eq!(
                    Vowel::from_char(ch),
                    Some(Vowel::new(base, tone, upper)),
                    "decode mismatch for {base:?} {tone:?} {upper:?}"
                );
            }
        }
    }
    assert_eq!(
        seen.len(),
        VOWEL_COUNT,
        "must produce exactly {VOWEL_COUNT} unique characters"
    );
}

/// One exhaustive pass over every Unicode scalar value, checking everything
/// that can go wrong between the classifier, the decoder and the encoder:
/// they must accept exactly the same characters, a decoded vowel must re-encode
/// to itself, and the accepted set must be exactly [`VOWEL_COUNT`] characters.
///
/// This replaces three separate ~1.1M-iteration scans (classifier⇒decoder
/// agreement, set equality, and decode/encode round-trip) that were each a
/// strict subset of this one.
#[test]
fn decoder_and_classifier_agree_across_the_whole_unicode_range() {
    let mut accepted = HashSet::with_capacity(VOWEL_COUNT);

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
            assert!(accepted.insert(ch), "{ch:?} was accepted more than once");
        }
    }

    assert_eq!(
        accepted.len(),
        VOWEL_COUNT,
        "the codec must recognize exactly {VOWEL_COUNT} characters"
    );
}

/// `is_vowel` + `decode_vowel` agree on every Latin Extended-A vowel.
#[test]
fn latin_extended_a_is_vowel_and_decodes_consistently() {
    let latin_extended_a: Vec<char> = (0x0100..=0x017F)
        .filter_map(char::from_u32)
        .filter(|ch| is_vowel(*ch))
        .collect();
    assert!(
        !latin_extended_a.is_empty(),
        "must find at least one Latin Extended-A vowel"
    );
    for ch in &latin_extended_a {
        assert!(
            decode_vowel(*ch).is_some(),
            "{ch:?} is_vowel but decode_vowel failed"
        );
        assert_eq!(
            is_vowel(*ch),
            decode_vowel(*ch).is_some(),
            "{ch:?} is_vowel/decode_vowel mismatch"
        );
    }
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

const CONST_VOWEL: Vowel = Vowel::new(BaseVowel::ACircumflex, Tone::Dot, true);
const CONST_CHAR: char = CONST_VOWEL.to_char();
const CONST_ROOT: RootVowel = CONST_VOWEL.root();
const CONST_BASE: BaseVowel = CONST_VOWEL.base();
const CONST_TONE: Tone = CONST_VOWEL.tone();
const CONST_BITS: u16 = CONST_VOWEL.bits();
const CONST_ENCODED: char = encode_vowel(BaseVowel::ACircumflex, Tone::Dot, true);
const CONST_DECODED: Option<Vowel> = decode_vowel('Ậ');

#[test]
fn codec_is_usable_in_const_context() {
    assert_eq!(CONST_CHAR, 'Ậ');
    assert_eq!(CONST_ROOT, RootVowel::A);
    assert_eq!(CONST_BASE, BaseVowel::ACircumflex);
    assert_eq!(CONST_TONE, Tone::Dot);
    assert!(CONST_VOWEL.is_upper());
    assert_eq!(CONST_VOWEL.root(), RootVowel::A);
    assert_eq!(CONST_BITS, 347);
    assert_eq!(CONST_ENCODED, CONST_CHAR);
    assert_eq!(CONST_DECODED, Some(CONST_VOWEL));
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
