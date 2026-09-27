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
use vime_engine::phonology::BaseVowel;

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

/// Highest `#[repr(u16)]` discriminant, i.e. table length - 1.
const DISCRIMINANT_MAX: usize = BaseVowel::OHorn as usize;

const fn build_id_lut<const N: usize>(vowels: [BaseVowel; N]) -> [u8; DISCRIMINANT_MAX + 1] {
    let mut table = [0u8; DISCRIMINANT_MAX + 1];
    let mut index = 0;
    while index < N {
        let base = vowels[index];
        table[base as usize] = base.id();
        index += 1;
    }
    table
}

const ID_BY_DISCRIMINANT: [u8; DISCRIMINANT_MAX + 1] = build_id_lut(ALL_VOWELS);

// ─────────────────────────── Measured candidates ───────────────────────────
// `production` is the only one that tracks the crate. The others are here to
// show the headroom any refactor could chase, plus a loop that never calls
// `id()` so the recorded numbers can be read as "call cost + loop overhead".

#[inline(always)]
fn production(base: BaseVowel) -> usize {
    base.id() as usize
}

/// Bench-local copy of the shipped body, kept so a change to the crate shows
/// up as a difference between `production` and `shift_u16`.
#[inline(always)]
fn shift_u16(base: BaseVowel) -> usize {
    (base as u16 >> 5) as usize
}

/// Candidate that trades the shift for a table load keyed by discriminant.
///
/// `#[repr(u16)]` leaves a 372-entry table, which is the honest cost of a LUT
/// here. A `#[repr(u8)]` refactor would shrink it to 24 usable bytes, but that
/// cannot be A/B-tested before the refactor lands: on the current repr
/// `(base as u8)` truncates away ID bits above 7, so the "narrower shift" idea
/// is not expressible until the discriminants actually move.
#[inline(always)]
fn lut_id(base: BaseVowel) -> usize {
    ID_BY_DISCRIMINANT[base as usize] as usize
}

/// Lower bound of this harness: same loop, no `id()` call.
#[inline(always)]
fn loop_floor(base: BaseVowel) -> usize {
    base as u16 as usize
}

struct LargeValues {
    vowels: Vec<BaseVowel>,
}

static LARGE_VALUES: OnceLock<LargeValues> = OnceLock::new();

fn large_values() -> &'static LargeValues {
    LARGE_VALUES.get_or_init(|| LargeValues {
        vowels: (0..LARGE_LEN)
            .map(|index| ALL_VOWELS[index % ALL_VOWELS.len()])
            .collect(),
    })
}

fn sum_values(values: &[BaseVowel], id: impl Fn(BaseVowel) -> usize) -> usize {
    let mut sum = 0usize;
    for &base in black_box(values) {
        sum = sum.wrapping_add(black_box(id(black_box(base))));
    }
    black_box(sum)
}

fn sum_repeated(base: BaseVowel, id: impl Fn(BaseVowel) -> usize) -> usize {
    let base = black_box(base);
    let mut sum = 0usize;
    for _ in 0..SINGLE_REPEATS {
        sum = sum.wrapping_add(black_box(id(black_box(base))));
    }
    black_box(sum)
}

/// Tone placement ranks candidates by ID, so a compare-and-keep-maximum loop
/// is the shape production code actually performs.
fn max_id(values: &[BaseVowel], id: impl Fn(BaseVowel) -> usize) -> usize {
    let mut best = 0usize;
    for &base in black_box(values) {
        let candidate = id(black_box(base));
        if candidate > best {
            best = candidate;
        }
    }
    black_box(best)
}

fn bench_all_12(c: &mut Criterion) {
    for &base in &ALL_VOWELS {
        assert_eq!(production(base), base.id() as usize);
        assert_eq!(production(base), shift_u16(base));
        assert_eq!(production(base), lut_id(base));
    }

    let mut group = c.benchmark_group("base_vowel_id_production/all_12");
    group.throughput(Throughput::Elements(ALL_VOWELS.len() as u64));
    for (name, f) in [
        ("production", production as fn(BaseVowel) -> usize),
        ("shift_u16", shift_u16),
        ("lut_id", lut_id),
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
        group.bench_with_input(BenchmarkId::new("shift_u16", name), &base, |b, &base| {
            b.iter(|| sum_repeated(base, shift_u16));
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
        ("shift_u16", shift_u16),
        ("lut_id", lut_id),
        ("loop_floor", loop_floor),
    ] {
        group.bench_function(name, |b| b.iter(|| sum_values(&values.vowels, f)));
    }
    group.finish();
}

fn bench_max_id(c: &mut Criterion) {
    let values = large_values();
    assert_eq!(max_id(&ALL_VOWELS, production), 11);
    assert_eq!(max_id(&ALL_VOWELS, lut_id), 11);

    let mut group = c.benchmark_group("base_vowel_id_production/max_id");
    group.throughput(Throughput::Elements(values.vowels.len() as u64));
    group.bench_function("production", |b| {
        b.iter(|| max_id(&values.vowels, production))
    });
    group.finish();
}

/// Sum with a `u8` accumulator: the cheapest way to observe the `id()` return
/// type, so a `u8 -> usize` widening change cannot hide behind a `usize` add.
fn bench_u8_accumulator(c: &mut Criterion) {
    fn sum_u8(values: &[BaseVowel]) -> u8 {
        let mut sum = 0u8;
        for &base in black_box(values) {
            sum = sum.wrapping_add(black_box(base.id()));
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
              bench_u8_accumulator
}
criterion_main!(benches);
