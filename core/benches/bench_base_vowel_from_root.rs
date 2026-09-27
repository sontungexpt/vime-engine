//! Compare the two ways to turn a [`RootVowel`] into a [`BaseVowel`]:
//!
//! * `production` — the shipped body: `(root as u8) << 2` and a `transmute`,
//!   which is free because `BaseVowel` is `#[repr(u8)]` and that shift already
//!   produces the discriminant.
//! * `match_from_root` — the previous body, one comparison per root.
//!
//! `shift_transmute` is a bench-local copy of the shipped body, so the spread
//! between it and `production` is the noise floor for this file: two copies of
//! one algorithm that differ only in inlining context.
//!
//! Run with `cargo bench -p vime-engine --bench bench_base_vowel_from_root`.

use std::{hint::black_box, sync::OnceLock, time::Duration};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vime_engine::phonology::{BaseVowel, RootVowel};

const REPEATS: usize = 256;
const LARGE_LEN: usize = 1_000_000;
const INVALID: usize = u16::MAX as usize;

const ALL_ROOTS: [RootVowel; 6] = [
    RootVowel::Y,
    RootVowel::U,
    RootVowel::I,
    RootVowel::E,
    RootVowel::O,
    RootVowel::A,
];

/// Every `(root, base)` pair, in the order the engine's keymap would meet them.
const EXPECTED: [(RootVowel, BaseVowel); 6] = [
    (RootVowel::Y, BaseVowel::Y),
    (RootVowel::U, BaseVowel::U),
    (RootVowel::I, BaseVowel::I),
    (RootVowel::E, BaseVowel::E),
    (RootVowel::O, BaseVowel::O),
    (RootVowel::A, BaseVowel::A),
];

// ─────────────────────────── Measured candidates ───────────────────────────

#[inline(always)]
fn production(root: RootVowel) -> usize {
    BaseVowel::from_root(root) as u8 as usize
}

/// The previous implementation: one comparison per root.
#[inline(always)]
fn match_from_root(root: RootVowel) -> usize {
    let base = match root {
        RootVowel::Y => BaseVowel::Y,
        RootVowel::U => BaseVowel::U,
        RootVowel::I => BaseVowel::I,
        RootVowel::E => BaseVowel::E,
        RootVowel::O => BaseVowel::O,
        RootVowel::A => BaseVowel::A,
    };
    base as u8 as usize
}

/// Bench-local copy of the shipped body, for the noise control.
#[inline(always)]
fn shift_transmute(root: RootVowel) -> usize {
    // SAFETY: `(root as u8) << 2` is `encode_base_vowel(root, Shape::None)`,
    // which is the discriminant of that root's plain vowel.
    let base = unsafe { std::mem::transmute::<u8, BaseVowel>((root as u8) << 2) };
    base as u8 as usize
}

/// Lower bound of this harness: same loop, no conversion.
#[inline(always)]
fn loop_floor(root: RootVowel) -> usize {
    root as u8 as usize
}

fn sum_roots(values: &[RootVowel], f: impl Fn(RootVowel) -> usize) -> usize {
    let mut total = 0usize;
    for &root in black_box(values) {
        total = total.wrapping_add(black_box(f(black_box(root))));
    }
    black_box(total)
}

fn sum_repeated(root: RootVowel, f: impl Fn(RootVowel) -> usize) -> usize {
    let root = black_box(root);
    let mut total = 0usize;
    for _ in 0..REPEATS {
        total = total.wrapping_add(black_box(f(black_box(root))));
    }
    black_box(total)
}

struct LargeRoots {
    roots: Vec<RootVowel>,
}

static LARGE_ROOTS: OnceLock<LargeRoots> = OnceLock::new();

fn large_roots() -> &'static LargeRoots {
    LARGE_ROOTS.get_or_init(|| LargeRoots {
        roots: (0..LARGE_LEN)
            .map(|index| ALL_ROOTS[index % ALL_ROOTS.len()])
            .collect(),
    })
}

/// All four candidates must agree, or the measurement is meaningless.
fn assert_candidates_agree() {
    for &(root, base) in &EXPECTED {
        assert_eq!(BaseVowel::from_root(root), base, "from_root for {root:?}");
        assert_eq!(production(root), base as u8 as usize, "production {root:?}");
        assert_eq!(match_from_root(root), base as u8 as usize, "match {root:?}");
        assert_eq!(
            shift_transmute(root),
            base as u8 as usize,
            "shift+transmute {root:?}"
        );
    }
}

fn bench_all_6(c: &mut Criterion) {
    assert_candidates_agree();

    let mut group = c.benchmark_group("from_root/all_6");
    group.throughput(Throughput::Elements(ALL_ROOTS.len() as u64));
    for (name, f) in [
        ("production", production as fn(RootVowel) -> usize),
        ("match", match_from_root),
        ("shift_transmute", shift_transmute),
        ("loop_floor", loop_floor),
    ] {
        group.bench_function(name, |b| b.iter(|| sum_roots(&ALL_ROOTS, f)));
    }
    group.finish();
}

fn bench_repeated(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_root/repeated");
    group.throughput(Throughput::Elements(REPEATS as u64));
    for root in ALL_ROOTS {
        group.bench_with_input(
            BenchmarkId::new("production", format!("{root:?}")),
            &root,
            |b, &r| b.iter(|| sum_repeated(r, production)),
        );
        group.bench_with_input(
            BenchmarkId::new("match", format!("{root:?}")),
            &root,
            |b, &r| b.iter(|| sum_repeated(r, match_from_root)),
        );
        group.bench_with_input(
            BenchmarkId::new("shift_transmute", format!("{root:?}")),
            &root,
            |b, &r| b.iter(|| sum_repeated(r, shift_transmute)),
        );
        group.bench_with_input(
            BenchmarkId::new("loop_floor", format!("{root:?}")),
            &root,
            |b, &r| b.iter(|| sum_repeated(r, loop_floor)),
        );
    }
    group.finish();
}

fn bench_large(c: &mut Criterion) {
    assert_candidates_agree();
    let roots = large_roots();

    let mut group = c.benchmark_group("from_root/large_1m");
    group.throughput(Throughput::Elements(roots.roots.len() as u64));
    for (name, f) in [
        ("production", production as fn(RootVowel) -> usize),
        ("match", match_from_root),
        ("shift_transmute", shift_transmute),
        ("loop_floor", loop_floor),
    ] {
        group.bench_function(name, |b| b.iter(|| sum_roots(&roots.roots, f)));
    }
    group.finish();

    // Kept so the group is never empty if every case above is deleted.
    let mut group = c.benchmark_group("from_root/unreachable_after_refactor");
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
    targets = bench_all_6, bench_repeated, bench_large
}
criterion_main!(benches);
