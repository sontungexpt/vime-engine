//! Baseline for the production `BaseVowel::id()` priority-ID extraction.
//!
//! Unlike `bench_base_vowel_id.rs`, which measures bench-local candidate
//! representations, this bench calls the real
//! `BaseVowel::id()` (`core/src/phonology/vowel.rs`) so a refactor of that
//! method can be compared against the numbers recorded in
//! `benches/BASELINES.md`.
//!
//! Run with `cargo bench -p vime-engine --bench bench_base_vowel_id_production`.

use std::{hint::black_box, sync::OnceLock, time::Duration};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vime_engine::phonology::{BaseVowel, RootVowel, Shape};

const SINGLE_REPEATS: usize = 256;
const LARGE_LEN: usize = 1_000_000;
const INVALID: usize = u16::MAX as usize;

const ALL_VOWELS: [BaseVowel; 12] = [
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

// ─────────────────────────── Measured candidates ───────────────────────────
// `production` is the only one that tracks the crate. The others are here to
// show the headroom any refactor could chase, plus a loop that never calls
// `id()` so the recorded numbers can be read as "call cost + loop overhead".

#[inline(always)]
fn production(base: BaseVowel) -> usize {
    base.priority_id() as usize
}

/// The 12-arm `match` version: one comparison per arm against a `#[repr(u8)]`
/// discriminant. This is the alternative to the production LUT, and the same
/// body that builds the LUT through `id_fast`.
#[inline(always)]
fn match_id(base: BaseVowel) -> usize {
    let id = match base {
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
    };
    id as usize
}

/// Lower bound of this harness: same loop, no `id()` call.
#[inline(always)]
fn loop_floor(base: BaseVowel) -> usize {
    base as u16 as usize
}

// ───────────────────── Old git version (commit 8fd4a1b) ─────────────────────
// Before the `#[repr(u8)]` refactor `BaseVowel` was `#[repr(u16)]` with the
// tone-placement ID packed into bits 5..=8, so `id()` was a single shift and no
// table existed. This is a verbatim copy of that representation, kept
// bench-local so the old and new layouts can be measured in one run.

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u16)]
#[rustfmt::skip]
enum OldBaseVowel {
    Y           = (0  << 5) | ((RootVowel::Y as u16) << 2) | Shape::None as u16,
    U           = (1  << 5) | ((RootVowel::U as u16) << 2) | Shape::None as u16,
    I           = (2  << 5) | ((RootVowel::I as u16) << 2) | Shape::None as u16,
    E           = (3  << 5) | ((RootVowel::E as u16) << 2) | Shape::None as u16,
    O           = (4  << 5) | ((RootVowel::O as u16) << 2) | Shape::None as u16,
    A           = (5  << 5) | ((RootVowel::A as u16) << 2) | Shape::None as u16,
    UHorn       = (6  << 5) | ((RootVowel::U as u16) << 2) | Shape::Horn as u16,
    ACircumflex = (7  << 5) | ((RootVowel::A as u16) << 2) | Shape::Circumflex as u16,
    OCircumflex = (8  << 5) | ((RootVowel::O as u16) << 2) | Shape::Circumflex as u16,
    ABreve      = (9  << 5) | ((RootVowel::A as u16) << 2) | Shape::Breve as u16,
    ECircumflex = (10 << 5) | ((RootVowel::E as u16) << 2) | Shape::Circumflex as u16,
    OHorn       = (11 << 5) | ((RootVowel::O as u16) << 2) | Shape::Horn as u16,
}

const OLD_ALL_VOWELS: [OldBaseVowel; 12] = [
    OldBaseVowel::Y,
    OldBaseVowel::U,
    OldBaseVowel::I,
    OldBaseVowel::E,
    OldBaseVowel::O,
    OldBaseVowel::A,
    OldBaseVowel::UHorn,
    OldBaseVowel::ACircumflex,
    OldBaseVowel::OCircumflex,
    OldBaseVowel::ABreve,
    OldBaseVowel::ECircumflex,
    OldBaseVowel::OHorn,
];

/// The old `id()`: shift the ID out of the packed discriminant.
#[inline(always)]
fn old_production(base: OldBaseVowel) -> usize {
    ((base as u16) >> 5) as usize
}

#[inline(always)]
fn old_loop_floor(base: OldBaseVowel) -> usize {
    base as u16 as usize
}

struct LargeValues {
    vowels: Vec<BaseVowel>,
    old_vowels: Vec<OldBaseVowel>,
}

static LARGE_VALUES: OnceLock<LargeValues> = OnceLock::new();

fn large_values() -> &'static LargeValues {
    LARGE_VALUES.get_or_init(|| LargeValues {
        vowels: (0..LARGE_LEN)
            .map(|index| ALL_VOWELS[index % ALL_VOWELS.len()])
            .collect(),
        old_vowels: (0..LARGE_LEN)
            .map(|index| OLD_ALL_VOWELS[index % OLD_ALL_VOWELS.len()])
            .collect(),
    })
}

fn sum_values<T: Copy>(values: &[T], id: impl Fn(T) -> usize) -> usize {
    let mut sum = 0usize;
    for &base in black_box(values) {
        sum = sum.wrapping_add(black_box(id(black_box(base))));
    }
    black_box(sum)
}

fn sum_repeated<T: Copy>(base: T, id: impl Fn(T) -> usize) -> usize {
    let base = black_box(base);
    let mut sum = 0usize;
    for _ in 0..SINGLE_REPEATS {
        sum = sum.wrapping_add(black_box(id(black_box(base))));
    }
    black_box(sum)
}

/// Tone placement ranks candidates by ID, so a compare-and-keep-maximum loop
/// is the shape production code actually performs.
fn max_id<T: Copy>(values: &[T], id: impl Fn(T) -> usize) -> usize {
    let mut best = 0usize;
    for &base in black_box(values) {
        let candidate = id(black_box(base));
        if candidate > best {
            best = candidate;
        }
    }
    black_box(best)
}

/// The old representation must produce the same IDs as both new versions, and
/// the 12 variants must stay in the same order.
fn assert_old_matches_new() {
    for (new, old) in ALL_VOWELS.iter().zip(OLD_ALL_VOWELS) {
        let new = *new;
        assert_eq!(
            production(new),
            old_production(old),
            "old vs LUT for {new:?}"
        );
        assert_eq!(
            match_id(new),
            old_production(old),
            "old vs match for {new:?}"
        );
    }
}

/// Every version under test must agree with each other and with `from_parts`
/// for every root/shape pair, valid or not.
fn assert_candidates_agree() {
    const ROOTS: [RootVowel; 6] = [
        RootVowel::A,
        RootVowel::E,
        RootVowel::I,
        RootVowel::O,
        RootVowel::U,
        RootVowel::Y,
    ];
    const SHAPES: [Shape; 4] = [Shape::None, Shape::Circumflex, Shape::Breve, Shape::Horn];

    for root in ROOTS {
        for shape in SHAPES {
            match BaseVowel::from_parts(root, shape) {
                Some(vowel) => {
                    let id = vowel.priority_id() as usize;
                    assert_eq!(production(vowel), id, "LUT for {vowel:?}");
                    assert_eq!(match_id(vowel), id, "match for {vowel:?}");
                    assert_eq!(BaseVowel::from_priority_id(id), Some(vowel), "id round-trip");
                }
                None => assert!(
                    ALL_VOWELS
                        .iter()
                        .all(|&v| v.root() != root || v.shape() != shape),
                    "{root:?} + {shape:?} is valid but from_parts says otherwise",
                ),
            }
        }
    }
}

fn bench_all_12(c: &mut Criterion) {
    assert_candidates_agree();
    for &base in &ALL_VOWELS {
        assert_eq!(production(base), base.priority_id() as usize);
        assert_eq!(production(base), match_id(base));
    }

    let mut group = c.benchmark_group("base_vowel_id_production/all_12");
    group.throughput(Throughput::Elements(ALL_VOWELS.len() as u64));
    for (name, f) in [
        ("production", production as fn(BaseVowel) -> usize),
        ("match_id", match_id),
        ("loop_floor", loop_floor),
    ] {
        group.bench_function(name, |b| b.iter(|| sum_values(&ALL_VOWELS, f)));
    }
    group.finish();
}

fn bench_single_repeated(c: &mut Criterion) {
    let mut group = c.benchmark_group("base_vowel_id_production/single_repeated");
    group.throughput(Throughput::Elements(SINGLE_REPEATS as u64));
    for (name, base) in [
        ("Y", BaseVowel::Y),
        ("A", BaseVowel::A),
        ("UHorn", BaseVowel::UHorn),
        ("ECircumflex", BaseVowel::ECircumflex),
        ("OHorn", BaseVowel::OHorn),
    ] {
        group.bench_with_input(BenchmarkId::new("production", name), &base, |b, &base| {
            b.iter(|| sum_repeated(base, production));
        });
        group.bench_with_input(BenchmarkId::new("match_id", name), &base, |b, &base| {
            b.iter(|| sum_repeated(base, match_id));
        });
        group.bench_with_input(BenchmarkId::new("loop_floor", name), &base, |b, &base| {
            b.iter(|| sum_repeated(base, loop_floor));
        });
    }
    group.finish();
}

fn bench_large_workload(c: &mut Criterion) {
    let values = large_values();
    let mut group = c.benchmark_group("base_vowel_id_production/large_1m");
    group.throughput(Throughput::Elements(values.vowels.len() as u64));
    for (name, f) in [
        ("production", production as fn(BaseVowel) -> usize),
        ("match_id", match_id),
        ("loop_floor", loop_floor),
    ] {
        group.bench_function(name, |b| b.iter(|| sum_values(&values.vowels, f)));
    }
    group.finish();
}

fn bench_max_id(c: &mut Criterion) {
    let values = large_values();
    assert_eq!(max_id(&ALL_VOWELS, production), 11);
    assert_eq!(max_id(&ALL_VOWELS, match_id), 11);

    let mut group = c.benchmark_group("base_vowel_id_production/max_id");
    group.throughput(Throughput::Elements(values.vowels.len() as u64));
    for (name, f) in [
        ("production", production as fn(BaseVowel) -> usize),
        ("match_id", match_id),
    ] {
        group.bench_function(name, |b| b.iter(|| max_id(&values.vowels, f)));
    }
    group.finish();
}

/// Sum with a `u8` accumulator: the cheapest way to observe the `id()` return
/// type, so a `u8 -> usize` widening change cannot hide behind a `usize` add.
fn bench_u8_accumulator(c: &mut Criterion) {
    fn sum_u8(values: &[BaseVowel]) -> u8 {
        let mut sum = 0u8;
        for &base in black_box(values) {
            sum = sum.wrapping_add(black_box(base.priority_id()));
        }
        black_box(sum)
    }

    let values = large_values();
    let mut group = c.benchmark_group("base_vowel_id_production/u8_accumulator");
    group.throughput(Throughput::Elements(values.vowels.len() as u64));
    group.bench_function("production", |b| b.iter(|| sum_u8(&values.vowels)));
    group.finish();

    let mut group = c.benchmark_group("base_vowel_id_production/unreachable_after_refactor");
    // Kept so the group is never empty if every case above is deleted.
    group.throughput(Throughput::Elements(1));
    group.bench_function("no_op", |b| b.iter(|| black_box(INVALID)));
    group.finish();
}

fn bench_old_all_12(c: &mut Criterion) {
    assert_old_matches_new();

    let mut group = c.benchmark_group("base_vowel_id_old/all_12");
    group.throughput(Throughput::Elements(OLD_ALL_VOWELS.len() as u64));
    for (name, f) in [
        ("old_shift", old_production as fn(OldBaseVowel) -> usize),
        ("old_loop_floor", old_loop_floor),
    ] {
        group.bench_function(name, |b| b.iter(|| sum_values(&OLD_ALL_VOWELS, f)));
    }
    group.finish();
}

fn bench_old_single_repeated(c: &mut Criterion) {
    let mut group = c.benchmark_group("base_vowel_id_old/single_repeated");
    group.throughput(Throughput::Elements(SINGLE_REPEATS as u64));
    for (name, old) in [
        ("Y", OldBaseVowel::Y),
        ("A", OldBaseVowel::A),
        ("UHorn", OldBaseVowel::UHorn),
        ("ECircumflex", OldBaseVowel::ECircumflex),
        ("OHorn", OldBaseVowel::OHorn),
    ] {
        group.bench_with_input(BenchmarkId::new("old_shift", name), &old, |b, &old| {
            b.iter(|| sum_repeated(old, old_production));
        });
        group.bench_with_input(BenchmarkId::new("old_loop_floor", name), &old, |b, &old| {
            b.iter(|| sum_repeated(old, old_loop_floor));
        });
    }
    group.finish();
}

fn bench_old_large_workload(c: &mut Criterion) {
    let values = large_values();
    let mut group = c.benchmark_group("base_vowel_id_old/large_1m");
    group.throughput(Throughput::Elements(values.old_vowels.len() as u64));
    for (name, f) in [
        ("old_shift", old_production as fn(OldBaseVowel) -> usize),
        ("old_loop_floor", old_loop_floor),
    ] {
        group.bench_function(name, |b| b.iter(|| sum_values(&values.old_vowels, f)));
    }
    group.finish();
}

fn bench_old_max_id(c: &mut Criterion) {
    let values = large_values();
    assert_eq!(max_id(&OLD_ALL_VOWELS, old_production), 11);

    let mut group = c.benchmark_group("base_vowel_id_old/max_id");
    group.throughput(Throughput::Elements(values.old_vowels.len() as u64));
    group.bench_function("old_shift", |b| {
        b.iter(|| max_id(&values.old_vowels, old_production))
    });
    group.finish();
}

fn bench_old_u8_accumulator(c: &mut Criterion) {
    fn sum_u8(values: &[OldBaseVowel]) -> u8 {
        let mut sum = 0u8;
        for &base in black_box(values) {
            sum = sum.wrapping_add(black_box(old_production(base) as u8));
        }
        black_box(sum)
    }

    let values = large_values();
    let mut group = c.benchmark_group("base_vowel_id_old/u8_accumulator");
    group.throughput(Throughput::Elements(values.old_vowels.len() as u64));
    group.bench_function("old_shift", |b| b.iter(|| sum_u8(&values.old_vowels)));
    group.finish();
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
    targets = bench_all_12, bench_single_repeated, bench_large_workload, bench_max_id,
              bench_u8_accumulator, bench_old_all_12, bench_old_single_repeated,
              bench_old_large_workload, bench_old_max_id, bench_old_u8_accumulator
}
criterion_main!(benches);
