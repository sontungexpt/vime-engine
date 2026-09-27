//! Compare priority-ID extraction by shifting against a root/shape LUT.
//!
//! Run with `cargo bench -p vime-engine --bench bench_base_vowel_id`.

#[path = "support/base_vowel_id.rs"]
mod lookup;

use std::{hint::black_box, sync::OnceLock, time::Duration};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lookup::{
    lut_id, match_id, old_id, packed_u128_id, packed_u64_id, StructuralBaseVowel, ALL_OLD_VOWELS,
    ALL_STRUCTURAL_VOWELS,
};
use vime_engine::phonology::BaseVowel;

const SINGLE_REPEATS: usize = 256;
const LARGE_LEN: usize = 1_000_000;

struct LargeValues {
    old: Vec<BaseVowel>,
    structural: Vec<StructuralBaseVowel>,
}

static LARGE_VALUES: OnceLock<LargeValues> = OnceLock::new();

fn large_values() -> &'static LargeValues {
    LARGE_VALUES.get_or_init(|| {
        let old = (0..LARGE_LEN)
            .map(|index| ALL_OLD_VOWELS[index % ALL_OLD_VOWELS.len()])
            .collect();
        let structural = (0..LARGE_LEN)
            .map(|index| ALL_STRUCTURAL_VOWELS[index % ALL_STRUCTURAL_VOWELS.len()])
            .collect();
        LargeValues { old, structural }
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

fn bench_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("base_vowel_id/single_repeated");
    group.throughput(Throughput::Elements(SINGLE_REPEATS as u64));

    for (name, old, structural) in [
        ("Y", BaseVowel::Y, StructuralBaseVowel::Y),
        ("A", BaseVowel::A, StructuralBaseVowel::A),
        ("UHorn", BaseVowel::UHorn, StructuralBaseVowel::UHorn),
        (
            "ECircumflex",
            BaseVowel::ECircumflex,
            StructuralBaseVowel::ECircumflex,
        ),
        ("OHorn", BaseVowel::OHorn, StructuralBaseVowel::OHorn),
    ] {
        group.bench_with_input(BenchmarkId::new("lut", name), &structural, |b, &base| {
            b.iter(|| sum_repeated(base, lut_id));
        });
        group.bench_with_input(BenchmarkId::new("match", name), &structural, |b, &base| {
            b.iter(|| sum_repeated(base, match_id));
        });
        group.bench_with_input(
            BenchmarkId::new("packed_u128", name),
            &structural,
            |b, &base| b.iter(|| sum_repeated(base, packed_u128_id)),
        );
        group.bench_with_input(
            BenchmarkId::new("packed_u64", name),
            &structural,
            |b, &base| b.iter(|| sum_repeated(base, packed_u64_id)),
        );
        group.bench_with_input(BenchmarkId::new("shift", name), &old, |b, &base| {
            b.iter(|| sum_repeated(base, old_id));
        });
    }

    group.finish();
}

fn bench_all_vowels(c: &mut Criterion) {
    let mut group = c.benchmark_group("base_vowel_id/all_12");
    group.throughput(Throughput::Elements(ALL_OLD_VOWELS.len() as u64));
    group.bench_function("lut", |b| {
        b.iter(|| sum_values(&ALL_STRUCTURAL_VOWELS, lut_id));
    });
    group.bench_function("match", |b| {
        b.iter(|| sum_values(&ALL_STRUCTURAL_VOWELS, match_id));
    });
    group.bench_function("packed_u128", |b| {
        b.iter(|| sum_values(&ALL_STRUCTURAL_VOWELS, packed_u128_id));
    });
    group.bench_function("packed_u64", |b| {
        b.iter(|| sum_values(&ALL_STRUCTURAL_VOWELS, packed_u64_id));
    });
    group.bench_function("shift", |b| {
        b.iter(|| sum_values(&ALL_OLD_VOWELS, old_id));
    });
    group.finish();
}

fn bench_large_workload(c: &mut Criterion) {
    let values = large_values();
    let mut group = c.benchmark_group("base_vowel_id/large_1m");
    group.throughput(Throughput::Elements(values.old.len() as u64));
    group.bench_function("lut", |b| {
        b.iter(|| sum_values(&values.structural, lut_id));
    });
    group.bench_function("match", |b| {
        b.iter(|| sum_values(&values.structural, match_id));
    });
    group.bench_function("packed_u128", |b| {
        b.iter(|| sum_values(&values.structural, packed_u128_id));
    });
    group.bench_function("packed_u64", |b| {
        b.iter(|| sum_values(&values.structural, packed_u64_id));
    });
    group.bench_function("shift", |b| {
        b.iter(|| sum_values(&values.old, old_id));
    });
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
    targets = bench_single, bench_all_vowels, bench_large_workload
}
criterion_main!(benches);
