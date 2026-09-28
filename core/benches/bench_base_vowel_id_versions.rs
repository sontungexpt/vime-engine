//! A/B benchmark of the two versions of `BaseVowel::id()`.
//!
//! The tree carries an uncommitted refactor of the base-vowel ID API. The two
//! versions being compared are:
//!
//! - **HEAD** — `id(self) -> u8`, `from_id(usize) -> Option<Self>`,
//!   `encode_vowel(BaseVowel, Tone, bool) -> char`.
//! - **current** — `id(self) -> BaseVowelId` (a new `#[repr(u8)]` enum),
//!   `from_id(BaseVowelId) -> Option<Self>`, `BaseVowelId::from_u8`, and
//!   `encode_vowel(BaseVowelId, Tone, bool) -> char`.
//!
//! The HEAD bodies are copied verbatim into `mod head` below, for the same
//! reason `bench_nucleus_state_versions.rs` keeps its pre-macro copy: so both
//! versions live in one binary and are measured under identical conditions.
//! **No production code is touched by this bench.**
//!
//! Method follows `benches/BASELINES.md`: criterion with `sample_size(20)`,
//! `warm_up_time(1s)`, `measurement_time(2s)`, `black_box` on every input, every
//! call and the slice, per-element `Throughput`, inline function *items* (never
//! `fn` pointers in a slice — the indirect call swamps every effect measured
//! here), and a `loop_floor` candidate per group as the harness cost.
//!
//! `mod head_copy` is a second, byte-identical copy of the HEAD body. BASELINES.md
//! records that two copies of the same code in one binary can differ by up to
//! 27% from inlining context alone, so that gap is this file's noise floor and
//! every other delta must be read against it, not against zero.
//!
//!   cargo bench -p vime-engine --bench bench_base_vowel_id_versions

use std::hint::black_box;
use std::sync::OnceLock;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vime_engine::phonology::{encode_vowel, BaseVowel, BaseVowelId, RootVowel, Shape, Tone, Vowel};

const SINGLE_REPEATS: usize = 256;
const LARGE_LEN: usize = 1_000_000;
const INVALID: usize = usize::MAX;

const ALL_BASES: [BaseVowel; 12] = [
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

const ALL_ROOTS: [RootVowel; 6] = [
    RootVowel::A,
    RootVowel::E,
    RootVowel::I,
    RootVowel::O,
    RootVowel::U,
    RootVowel::Y,
];

const ALL_SHAPES: [Shape; 4] = [Shape::None, Shape::Circumflex, Shape::Breve, Shape::Horn];

const ALL_TONES: [Tone; 6] = [
    Tone::Flat,
    Tone::Acute,
    Tone::Grave,
    Tone::Hook,
    Tone::Tilde,
    Tone::Dot,
];

// ───────────────────── Version 1 of 2: HEAD, verbatim ─────────────────────
// Bodies copied character-for-character from `git show
// HEAD:core/src/phonology/vowel.rs`, with `self` turned into an explicit
// parameter. The `#[repr(u8)]` layout of `BaseVowel` is identical in both
// versions, so these are the same functions the crate used to compile.
mod head {
    use vime_engine::phonology::{BaseVowel, Tone};

    /// HEAD: `pub const fn id(self) -> u8`.
    #[inline(always)]
    pub const fn id(base: BaseVowel) -> u8 {
        match base {
            BaseVowel::Y => 0,
            BaseVowel::U => 1,
            BaseVowel::I => 2,
            BaseVowel::E => 3,
            BaseVowel::O => 4,
            BaseVowel::A => 5,
            BaseVowel::UHorn => 6,
            BaseVowel::ACircumflex => 7,
            BaseVowel::OCircumflex => 8,
            BaseVowel::ABreve => 9,
            BaseVowel::ECircumflex => 10,
            BaseVowel::OHorn => 11,
        }
    }

    /// HEAD: `pub const fn from_id(vowel_id: usize) -> Option<Self>`.
    #[inline(always)]
    pub const fn from_id(vowel_id: usize) -> Option<BaseVowel> {
        match vowel_id {
            0 => Some(BaseVowel::Y),
            1 => Some(BaseVowel::U),
            2 => Some(BaseVowel::I),
            3 => Some(BaseVowel::E),
            4 => Some(BaseVowel::O),
            5 => Some(BaseVowel::A),
            6 => Some(BaseVowel::UHorn),
            7 => Some(BaseVowel::ACircumflex),
            8 => Some(BaseVowel::OCircumflex),
            9 => Some(BaseVowel::ABreve),
            10 => Some(BaseVowel::ECircumflex),
            11 => Some(BaseVowel::OHorn),
            _ => None,
        }
    }

    /// HEAD: `pub const fn encode_vowel(base: BaseVowel, tone: Tone, uppercase:
    /// bool) -> char`. The `ENCODED` table is unchanged in the working tree;
    /// only the index expression moved out of the function body.
    pub const fn encode_vowel(base: BaseVowel, tone: Tone, uppercase: bool) -> char {
        #[rustfmt::skip]
        const ENCODED: [char; 144] = [
            // Priority 0: Y
            'y', 'Y', 'ý', 'Ý', 'ỳ', 'Ỳ', 'ỷ', 'Ỷ', 'ỹ', 'Ỹ', 'ỵ', 'Ỵ',
            // Priority 1: U
            'u', 'U', 'ú', 'Ú', 'ù', 'Ù', 'ủ', 'Ủ', 'ũ', 'Ũ', 'ụ', 'Ụ',
            // Priority 2: I
            'i', 'I', 'í', 'Í', 'ì', 'Ì', 'ỉ', 'Ỉ', 'ĩ', 'Ĩ', 'ị', 'Ị',
            // Priority 3: E
            'e', 'E', 'é', 'É', 'è', 'È', 'ẻ', 'Ẻ', 'ẽ', 'Ẽ', 'ẹ', 'Ẹ',
            // Priority 4: O
            'o', 'O', 'ó', 'Ó', 'ò', 'Ò', 'ỏ', 'Ỏ', 'õ', 'Õ', 'ọ', 'Ọ',
            // Priority 5: A
            'a', 'A', 'á', 'Á', 'à', 'À', 'ả', 'Ả', 'ã', 'Ã', 'ạ', 'Ạ',
            // Priority 6: UHorn
            'ư', 'Ư', 'ứ', 'Ứ', 'ừ', 'Ừ', 'ử', 'Ử', 'ữ', 'Ữ', 'ự', 'Ự',
            // Priority 7: ACircumflex
            'â', 'Â', 'ấ', 'Ấ', 'ầ', 'Ầ', 'ẩ', 'Ẩ', 'ẫ', 'Ẫ', 'ậ', 'Ậ',
            // Priority 8: OCircumflex
            'ô', 'Ô', 'ố', 'Ố', 'ồ', 'Ồ', 'ổ', 'Ổ', 'ỗ', 'Ỗ', 'ộ', 'Ộ',
            // Priority 9: ABreve
            'ă', 'Ă', 'ắ', 'Ắ', 'ằ', 'Ằ', 'ẳ', 'Ẳ', 'ẵ', 'Ẵ', 'ặ', 'Ặ',
            // Priority 10: ECircumflex
            'ê', 'Ê', 'ế', 'Ế', 'ề', 'Ề', 'ể', 'Ể', 'ễ', 'Ễ', 'ệ', 'Ệ',
            // Priority 11: OHorn
            'ơ', 'Ơ', 'ớ', 'Ớ', 'ờ', 'Ờ', 'ở', 'Ở', 'ỡ', 'Ỡ', 'ợ', 'Ợ',
        ];

        let idx = ((base.id() as usize * 6 + tone as usize) << 1) | uppercase as usize;

        ENCODED[idx]
    }
}

// ─────────────────── Noise control: copy 2 of the HEAD body ───────────────────
// Deliberately a separate module with a byte-identical body. Whatever gap
// appears between `head` and `head_copy` is layout, not algorithm.
mod head_copy {
    use vime_engine::phonology::BaseVowel;

    #[inline(always)]
    pub const fn id(base: BaseVowel) -> u8 {
        match base {
            BaseVowel::Y => 0,
            BaseVowel::U => 1,
            BaseVowel::I => 2,
            BaseVowel::E => 3,
            BaseVowel::O => 4,
            BaseVowel::A => 5,
            BaseVowel::UHorn => 6,
            BaseVowel::ACircumflex => 7,
            BaseVowel::OCircumflex => 8,
            BaseVowel::ABreve => 9,
            BaseVowel::ECircumflex => 10,
            BaseVowel::OHorn => 11,
        }
    }
}

// ─────────────────────────────── Candidates ───────────────────────────────
// Each is an inline function *item* passed to a generic helper, so every
// candidate is monomorphised into the loop and called directly.

#[inline(always)]
fn head_id(base: BaseVowel) -> usize {
    head::id(base) as usize
}

#[inline(always)]
fn head_id_copy(base: BaseVowel) -> usize {
    head_copy::id(base) as usize
}

/// The shipped working-tree `id()`, widened for a common accumulator.
#[inline(always)]
fn current_id(base: BaseVowel) -> usize {
    base.id() as usize
}

/// Harness cost: same loop, no call.
#[inline(always)]
fn loop_floor(base: BaseVowel) -> usize {
    base as usize
}

// ───────────────────────────────── from_id ─────────────────────────────────

#[inline(always)]
fn head_from_id(id: u8) -> usize {
    match head::from_id(id as usize) {
        Some(base) => base as usize,
        None => INVALID,
    }
}

/// What a caller has to write now: parse the `u8` into a `BaseVowelId` (one
/// range check) and then resolve it (a second 12-arm match).
#[inline(always)]
fn current_from_id(id: u8) -> usize {
    match BaseVowelId::from_u8(id) {
        Some(base_id) => match BaseVowel::from_id(base_id) {
            Some(base) => base as usize,
            None => INVALID,
        },
        None => INVALID,
    }
}

/// The same resolution, but skipping the `from_u8` range check the way a caller
/// that already holds a `BaseVowelId` would. Isolates the cost of the added
/// `u8` -> enum step from the cost of the match itself.
#[inline(always)]
fn current_from_id_unchecked(id: u8) -> usize {
    // SAFETY: the large workload only ever feeds ids in 0..12, i.e. below
    // `BaseVowelId::COUNT`, and `bench_from_id_all_12` feeds 0..12 as well.
    let base_id = unsafe { BaseVowelId::from_u8_unchecked(id) };
    match BaseVowel::from_id(base_id) {
        Some(base) => base as usize,
        None => INVALID,
    }
}

// ─────────────────────────────── encode_vowel ───────────────────────────────

#[inline(always)]
fn head_encode(base: BaseVowel, tone: Tone, upper: bool) -> u32 {
    head::encode_vowel(base, tone, upper) as u32
}

#[inline(always)]
fn current_encode_id(base: BaseVowel, tone: Tone, upper: bool) -> u32 {
    encode_vowel(base.id(), tone, upper) as u32
}

#[inline(always)]
fn current_to_char(vowel: Vowel) -> u32 {
    vowel.to_char() as u32
}

#[inline(always)]
fn head_to_char(vowel: Vowel) -> u32 {
    head::encode_vowel(vowel.base(), vowel.tone(), vowel.is_upper()) as u32
}

// ───────────────────────────────── Workloads ─────────────────────────────────

struct Large {
    bases: Vec<BaseVowel>,
    ids: Vec<u8>,
    encodings: Vec<(BaseVowel, Tone, bool)>,
    vowels: Vec<Vowel>,
}

static LARGE: OnceLock<Large> = OnceLock::new();

fn large() -> &'static Large {
    LARGE.get_or_init(|| Large {
        bases: (0..LARGE_LEN)
            .map(|i| ALL_BASES[i % ALL_BASES.len()])
            .collect(),
        ids: (0..LARGE_LEN).map(|i| (i % 12) as u8).collect(),
        encodings: (0..LARGE_LEN)
            .map(|i| {
                (
                    ALL_BASES[i % ALL_BASES.len()],
                    ALL_TONES[i % ALL_TONES.len()],
                    i % 2 == 0,
                )
            })
            .collect(),
        vowels: (0..LARGE_LEN)
            .map(|i| {
                Vowel::new(
                    ALL_BASES[i % ALL_BASES.len()],
                    ALL_TONES[i % ALL_TONES.len()],
                    i % 2 == 0,
                )
            })
            .collect(),
    })
}

fn sum_bases(values: &[BaseVowel], f: impl Fn(BaseVowel) -> usize) -> usize {
    let mut sum = 0usize;
    for &base in black_box(values) {
        sum = sum.wrapping_add(black_box(f(black_box(base))));
    }
    black_box(sum)
}

fn sum_repeated(base: BaseVowel, f: impl Fn(BaseVowel) -> usize) -> usize {
    let base = black_box(base);
    let mut sum = 0usize;
    for _ in 0..SINGLE_REPEATS {
        sum = sum.wrapping_add(black_box(f(black_box(base))));
    }
    black_box(sum)
}

/// The working tree lets a caller keep the `BaseVowelId` itself instead of
/// casting to an integer, so measure that shape too.
fn sum_ids(values: &[BaseVowel], f: impl Fn(BaseVowel) -> BaseVowelId) -> usize {
    let mut sum = 0usize;
    for &base in black_box(values) {
        let id = black_box(f(black_box(base)));
        sum = sum.wrapping_add(id as usize);
    }
    black_box(sum)
}

fn sum_ids_u8(values: &[BaseVowel], f: impl Fn(BaseVowel) -> BaseVowelId) -> u8 {
    let mut sum = 0u8;
    for &base in black_box(values) {
        sum = sum.wrapping_add(black_box(f(black_box(base))) as u8);
    }
    black_box(sum)
}

fn sum_ids_from_bytes(values: &[u8], f: impl Fn(u8) -> usize) -> usize {
    let mut sum = 0usize;
    for &id in black_box(values) {
        sum = sum.wrapping_add(black_box(f(black_box(id))));
    }
    black_box(sum)
}

fn sum_encodings(
    values: &[(BaseVowel, Tone, bool)],
    f: impl Fn(BaseVowel, Tone, bool) -> u32,
) -> u32 {
    let mut sum = 0u32;
    for &(base, tone, upper) in black_box(values) {
        let (base, tone, upper) = (black_box(base), black_box(tone), black_box(upper));
        sum = sum.wrapping_add(black_box(f(base, tone, upper)));
    }
    black_box(sum)
}

fn sum_vowels(values: &[Vowel], f: impl Fn(Vowel) -> u32) -> u32 {
    let mut sum = 0u32;
    for &vowel in black_box(values) {
        sum = sum.wrapping_add(black_box(f(black_box(vowel))));
    }
    black_box(sum)
}

// ──────────────────────── Correctness before timing ────────────────────────
// A "win" here would be worthless if the two versions disagreed, so every
// candidate is checked against HEAD before any measurement starts.

fn assert_versions_agree() {
    for &base in &ALL_BASES {
        let head_value = head::id(base);
        assert_eq!(base.id() as u8, head_value, "id() drift for {base:?}");
        assert_eq!(
            head_copy::id(base),
            head_value,
            "noise control drifted for {base:?}"
        );

        // Round trip through both directions of the ID API.
        assert_eq!(
            BaseVowel::from_id(base.id()),
            head::from_id(head_value as usize),
            "from_id drift for {base:?}"
        );
        assert_eq!(
            BaseVowelId::from_u8(head_value),
            Some(base.id()),
            "BaseVowelId::from_u8 drift for {base:?}"
        );

        // The only production consumer of `id()`.
        for tone in ALL_TONES {
            for upper in [false, true] {
                assert_eq!(
                    encode_vowel(base.id(), tone, upper),
                    head::encode_vowel(base, tone, upper),
                    "encode_vowel drift for {base:?} {tone:?} upper={upper}"
                );
                assert_eq!(
                    Vowel::new(base, tone, upper).to_char(),
                    head::encode_vowel(base, tone, upper),
                    "to_char drift for {base:?} {tone:?} upper={upper}"
                );
            }
        }
    }

    // Every root/shape pair that resolves must agree with HEAD through the
    // whole ID round trip, checked across all 24 combinations rather than only
    // the 12 that happen to be vowels.
    for root in ALL_ROOTS {
        for shape in ALL_SHAPES {
            if let Some(base) = BaseVowel::from_parts(root, shape) {
                assert_eq!(
                    base.id() as u8,
                    head::id(base),
                    "id() disagrees for {root:?} + {shape:?}"
                );
                assert_eq!(
                    BaseVowel::from_id(base.id()),
                    head::from_id(head::id(base) as usize),
                    "from_id disagrees for {root:?} + {shape:?}"
                );
            }
        }
    }

    // Out-of-range IDs must still be rejected on both sides.
    assert_eq!(head::from_id(BaseVowel::COUNT), None);
    assert_eq!(head::from_id(usize::MAX), None);
    assert_eq!(
        BaseVowelId::from_u8(BaseVowel::COUNT as u8),
        None,
        "from_u8 must reject the first out-of-range id"
    );
    assert_eq!(BaseVowelId::from_u8(u8::MAX), None);
    assert_eq!(BaseVowelId::COUNT, BaseVowel::COUNT);
}

// ─────────────────────────────────── Groups ───────────────────────────────────

fn bench_id_all_12(c: &mut Criterion) {
    assert_versions_agree();

    let mut g = c.benchmark_group("id_versions/all_12");
    g.throughput(Throughput::Elements(ALL_BASES.len() as u64));
    g.bench_function("head", |b| b.iter(|| sum_bases(&ALL_BASES, head_id)));
    g.bench_function("head_copy", |b| {
        b.iter(|| sum_bases(&ALL_BASES, head_id_copy))
    });
    g.bench_function("current", |b| b.iter(|| sum_bases(&ALL_BASES, current_id)));
    g.bench_function("loop_floor", |b| {
        b.iter(|| sum_bases(&ALL_BASES, loop_floor))
    });
    g.finish();
}

fn bench_id_single_repeated(c: &mut Criterion) {
    for (name, base) in [
        ("Y", BaseVowel::Y),
        ("A", BaseVowel::A),
        ("UHorn", BaseVowel::UHorn),
        ("ECircumflex", BaseVowel::ECircumflex),
        ("OHorn", BaseVowel::OHorn),
    ] {
        let mut g = c.benchmark_group(format!("id_versions/single_repeated/{name}"));
        g.throughput(Throughput::Elements(SINGLE_REPEATS as u64));
        g.bench_with_input(BenchmarkId::new("head", name), &base, |b, &base| {
            b.iter(|| sum_repeated(base, head_id));
        });
        g.bench_with_input(BenchmarkId::new("head_copy", name), &base, |b, &base| {
            b.iter(|| sum_repeated(base, head_id_copy));
        });
        g.bench_with_input(BenchmarkId::new("current", name), &base, |b, &base| {
            b.iter(|| sum_repeated(base, current_id));
        });
        g.bench_with_input(BenchmarkId::new("loop_floor", name), &base, |b, &base| {
            b.iter(|| sum_repeated(base, loop_floor));
        });
        g.finish();
    }
}

/// The group to quote: 1,000,000 values cycling the 12-entry source, so the
/// input stream misses L1 while any table always hits it.
fn bench_id_large_1m(c: &mut Criterion) {
    let values = large();
    let mut g = c.benchmark_group("id_versions/large_1m");
    g.throughput(Throughput::Elements(LARGE_LEN as u64));
    g.bench_function("head", |b| b.iter(|| sum_bases(&values.bases, head_id)));
    g.bench_function("head_copy", |b| {
        b.iter(|| sum_bases(&values.bases, head_id_copy))
    });
    g.bench_function("current", |b| {
        b.iter(|| sum_bases(&values.bases, current_id))
    });
    g.bench_function("loop_floor", |b| {
        b.iter(|| sum_bases(&values.bases, loop_floor))
    });
    g.finish();
}

/// The enum can now be accumulated without a cast. This group exists because a
/// caller-side change to the accumulator width shows up here only.
fn bench_id_enum_accumulator(c: &mut Criterion) {
    let values = large();

    let mut g = c.benchmark_group("id_versions/enum_accumulator_1m");
    g.throughput(Throughput::Elements(LARGE_LEN as u64));
    g.bench_function("current_usize", |b| {
        b.iter(|| sum_ids(&values.bases, |v| v.id()))
    });
    g.bench_function("current_u8", |b| {
        b.iter(|| sum_ids_u8(&values.bases, |v| v.id()))
    });
    g.bench_function("head_as_u8", |b| {
        b.iter(|| {
            let mut sum = 0u8;
            for &base in black_box(&values.bases) {
                sum = sum.wrapping_add(black_box(head::id(black_box(base))));
            }
            black_box(sum)
        })
    });
    g.finish();
}

fn bench_from_id_all_12(c: &mut Criterion) {
    fn sum_12(f: impl Fn(u8) -> usize) -> usize {
        let mut sum = 0usize;
        for id in 0..12u8 {
            sum = sum.wrapping_add(black_box(f(black_box(id))));
        }
        black_box(sum)
    }

    let mut g = c.benchmark_group("id_versions/from_id_all_12");
    g.throughput(Throughput::Elements(12));
    g.bench_function("head", |b| b.iter(|| sum_12(head_from_id)));
    g.bench_function("current", |b| b.iter(|| sum_12(current_from_id)));
    g.bench_function("current_unchecked", |b| {
        b.iter(|| sum_12(current_from_id_unchecked))
    });
    g.finish();
}

fn bench_from_id_large_1m(c: &mut Criterion) {
    let values = large();
    let mut g = c.benchmark_group("id_versions/from_id_large_1m");
    g.throughput(Throughput::Elements(LARGE_LEN as u64));
    g.bench_function("head", |b| {
        b.iter(|| sum_ids_from_bytes(&values.ids, head_from_id))
    });
    g.bench_function("current", |b| {
        b.iter(|| sum_ids_from_bytes(&values.ids, current_from_id))
    });
    g.bench_function("current_unchecked", |b| {
        b.iter(|| sum_ids_from_bytes(&values.ids, current_from_id_unchecked))
    });
    g.finish();
}

/// `encode_vowel` is the one production caller of `id()`, so this is the group
/// that decides whether the refactor costs anything a user can feel.
fn bench_encode_vowel_1m(c: &mut Criterion) {
    let values = large();
    let mut g = c.benchmark_group("id_versions/encode_vowel_1m");
    g.throughput(Throughput::Elements(LARGE_LEN as u64));
    g.bench_function("head", |b| {
        b.iter(|| sum_encodings(&values.encodings, head_encode))
    });
    g.bench_function("current_id", |b| {
        b.iter(|| sum_encodings(&values.encodings, current_encode_id))
    });
    g.finish();
}

fn bench_to_char_1m(c: &mut Criterion) {
    let values = large();
    let mut g = c.benchmark_group("id_versions/to_char_1m");
    g.throughput(Throughput::Elements(LARGE_LEN as u64));
    g.bench_function("head", |b| {
        b.iter(|| sum_vowels(&values.vowels, head_to_char))
    });
    g.bench_function("current", |b| {
        b.iter(|| sum_vowels(&values.vowels, current_to_char))
    });
    g.finish();
}

fn configure() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2))
        .sample_size(20)
}

criterion_group! {
    name = benches;
    config = configure();
    targets = bench_id_all_12, bench_id_single_repeated, bench_id_large_1m,
              bench_id_enum_accumulator, bench_from_id_all_12, bench_from_id_large_1m,
              bench_encode_vowel_1m, bench_to_char_1m
}
criterion_main!(benches);
