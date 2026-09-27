//! Compare packed BaseVowel constructors with their former match-based forms.
//!
//! Run with `cargo bench -p vime-engine --bench bench_base_vowel_constructors`.

use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vime_engine::phonology::{BaseVowel, RootVowel, Shape};

const INVALID: usize = u16::MAX as usize;
const INVALID_ID: u8 = 0xF;
const REPEATS: usize = 256;

const ALL_ROOTS: [RootVowel; 6] = [
    RootVowel::A,
    RootVowel::E,
    RootVowel::I,
    RootVowel::O,
    RootVowel::U,
    RootVowel::Y,
];
const ROOT_VARIANTS: [BaseVowel; 6] = [
    BaseVowel::A,
    BaseVowel::E,
    BaseVowel::I,
    BaseVowel::O,
    BaseVowel::U,
    BaseVowel::Y,
];

const VALID_PARTS: [(RootVowel, Shape); 12] = [
    (RootVowel::Y, Shape::None),
    (RootVowel::U, Shape::None),
    (RootVowel::I, Shape::None),
    (RootVowel::E, Shape::None),
    (RootVowel::O, Shape::None),
    (RootVowel::A, Shape::None),
    (RootVowel::U, Shape::Horn),
    (RootVowel::A, Shape::Circumflex),
    (RootVowel::O, Shape::Circumflex),
    (RootVowel::A, Shape::Breve),
    (RootVowel::E, Shape::Circumflex),
    (RootVowel::O, Shape::Horn),
];

const ALL_PARTS: [(RootVowel, Shape); 24] = [
    (RootVowel::A, Shape::None),
    (RootVowel::A, Shape::Circumflex),
    (RootVowel::A, Shape::Breve),
    (RootVowel::A, Shape::Horn),
    (RootVowel::E, Shape::None),
    (RootVowel::E, Shape::Circumflex),
    (RootVowel::E, Shape::Breve),
    (RootVowel::E, Shape::Horn),
    (RootVowel::I, Shape::None),
    (RootVowel::I, Shape::Circumflex),
    (RootVowel::I, Shape::Breve),
    (RootVowel::I, Shape::Horn),
    (RootVowel::O, Shape::None),
    (RootVowel::O, Shape::Circumflex),
    (RootVowel::O, Shape::Breve),
    (RootVowel::O, Shape::Horn),
    (RootVowel::U, Shape::None),
    (RootVowel::U, Shape::Circumflex),
    (RootVowel::U, Shape::Breve),
    (RootVowel::U, Shape::Horn),
    (RootVowel::Y, Shape::None),
    (RootVowel::Y, Shape::Circumflex),
    (RootVowel::Y, Shape::Breve),
    (RootVowel::Y, Shape::Horn),
];

// Benchmark copy of BaseVowel::ID_BY_ROOT_SHAPE. The production helper is
// private, so this keeps its exact lookup algorithm measurable from Criterion.
const ID_BY_ROOT_SHAPE: [u8; 24] = [
    5, 7, 9, INVALID_ID, 3, 10, INVALID_ID, INVALID_ID, 2, INVALID_ID, INVALID_ID, INVALID_ID, 4,
    8, INVALID_ID, 11, 1, INVALID_ID, INVALID_ID, 6, 0, INVALID_ID, INVALID_ID, INVALID_ID,
];
const VARIANTS_BY_ID: [BaseVowel; 12] = [
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

#[inline(always)]
fn table_id_from_parts(root: RootVowel, shape: Shape) -> u8 {
    let index = ((root as usize) << 2) | shape as usize;
    ID_BY_ROOT_SHAPE[index]
}

#[inline(always)]
fn match_id_from_parts(root: RootVowel, shape: Shape) -> u8 {
    match (root, shape) {
        (RootVowel::A, Shape::None) => 5,
        (RootVowel::A, Shape::Circumflex) => 7,
        (RootVowel::A, Shape::Breve) => 9,
        (RootVowel::E, Shape::None) => 3,
        (RootVowel::E, Shape::Circumflex) => 10,
        (RootVowel::I, Shape::None) => 2,
        (RootVowel::O, Shape::None) => 4,
        (RootVowel::O, Shape::Circumflex) => 8,
        (RootVowel::O, Shape::Horn) => 11,
        (RootVowel::U, Shape::None) => 1,
        (RootVowel::U, Shape::Horn) => 6,
        (RootVowel::Y, Shape::None) => 0,
        _ => INVALID_ID,
    }
}

#[inline(always)]
fn table_parts_id(parts: (RootVowel, Shape)) -> usize {
    table_id_from_parts(parts.0, parts.1) as usize
}

#[inline(always)]
fn match_parts_id(parts: (RootVowel, Shape)) -> usize {
    match_id_from_parts(parts.0, parts.1) as usize
}

/// Previous implementation: one match arm for every valid pair.
#[inline(always)]
fn old_from_parts(root: RootVowel, shape: Shape) -> Result<BaseVowel, ()> {
    match (root, shape) {
        (RootVowel::O, Shape::None) => Ok(BaseVowel::O),
        (RootVowel::O, Shape::Horn) => Ok(BaseVowel::OHorn),
        (RootVowel::O, Shape::Circumflex) => Ok(BaseVowel::OCircumflex),
        (RootVowel::E, Shape::None) => Ok(BaseVowel::E),
        (RootVowel::E, Shape::Circumflex) => Ok(BaseVowel::ECircumflex),
        (RootVowel::A, Shape::None) => Ok(BaseVowel::A),
        (RootVowel::A, Shape::Breve) => Ok(BaseVowel::ABreve),
        (RootVowel::A, Shape::Circumflex) => Ok(BaseVowel::ACircumflex),
        (RootVowel::U, Shape::None) => Ok(BaseVowel::U),
        (RootVowel::U, Shape::Horn) => Ok(BaseVowel::UHorn),
        (RootVowel::I, Shape::None) => Ok(BaseVowel::I),
        (RootVowel::Y, Shape::None) => Ok(BaseVowel::Y),
        _ => Err(()),
    }
}

/// Previous implementation: direct match on the root.
#[inline(always)]
fn old_from_root(root: RootVowel) -> BaseVowel {
    match root {
        RootVowel::A => BaseVowel::A,
        RootVowel::E => BaseVowel::E,
        RootVowel::I => BaseVowel::I,
        RootVowel::O => BaseVowel::O,
        RootVowel::U => BaseVowel::U,
        RootVowel::Y => BaseVowel::Y,
    }
}

#[inline(always)]
fn from_id_lookup(root: RootVowel, shape: Shape) -> Result<BaseVowel, ()> {
    let id = table_id_from_parts(root, shape);
    if id == INVALID_ID {
        Err(())
    } else {
        BaseVowel::from_priority_id(id as usize).ok_or(())
    }
}

#[inline(always)]
fn variant_table_from_parts(root: RootVowel, shape: Shape) -> Result<BaseVowel, ()> {
    let id = table_id_from_parts(root, shape);
    if id == INVALID_ID {
        Err(())
    } else {
        Ok(VARIANTS_BY_ID[id as usize])
    }
}

#[inline(always)]
fn id_lookup_parts_id(parts: (RootVowel, Shape)) -> usize {
    from_id_lookup(parts.0, parts.1)
        .map(|base| base as u16 as usize)
        .unwrap_or(INVALID)
}

#[inline(always)]
fn old_parts_id(parts: (RootVowel, Shape)) -> usize {
    old_from_parts(parts.0, parts.1)
        .map(|base| base as u16 as usize)
        .unwrap_or(INVALID)
}

#[inline(always)]
fn current_root_id(root: RootVowel) -> usize {
    BaseVowel::from_root(root) as u16 as usize
}

#[inline(always)]
fn old_root_id(root: RootVowel) -> usize {
    old_from_root(root) as u16 as usize
}

#[inline(always)]
fn array_root_id(root: RootVowel) -> usize {
    ROOT_VARIANTS[root as usize] as u16 as usize
}

fn sum<T: Copy>(values: &[T], f: impl Fn(T) -> usize) -> usize {
    let mut total = 0usize;
    for &value in black_box(values) {
        total = total.wrapping_add(black_box(f(black_box(value))));
    }
    black_box(total)
}

fn sum_repeated<T: Copy>(value: T, f: impl Fn(T) -> usize) -> usize {
    let value = black_box(value);
    let mut total = 0usize;
    for _ in 0..REPEATS {
        total = total.wrapping_add(black_box(f(black_box(value))));
    }
    black_box(total)
}

fn bench_from_parts(c: &mut Criterion) {
    for (group_name, values) in [
        ("valid_12", &VALID_PARTS[..]),
        ("all_24_pairs", &ALL_PARTS[..]),
    ] {
        for &parts in values {
            assert_eq!(id_lookup_parts_id(parts), old_parts_id(parts));
        }

        let mut group =
            c.benchmark_group(format!("base_vowel_constructor/from_parts/{group_name}"));
        group.throughput(Throughput::Elements(values.len() as u64));
        group.bench_function("id_table_then_from_id", |b| {
            b.iter(|| sum(values, id_lookup_parts_id));
        });
        group.bench_function("match", |b| {
            b.iter(|| sum(values, old_parts_id));
        });
        group.finish();
    }

    let mut group = c.benchmark_group("base_vowel_constructor/from_parts/repeated");
    group.throughput(Throughput::Elements(REPEATS as u64));
    for (name, parts) in [
        ("A_circumflex", (RootVowel::A, Shape::Circumflex)),
        ("E_circumflex", (RootVowel::E, Shape::Circumflex)),
        ("O_horn", (RootVowel::O, Shape::Horn)),
    ] {
        assert_eq!(id_lookup_parts_id(parts), old_parts_id(parts));
        group.bench_with_input(
            BenchmarkId::new("id_table_then_from_id", name),
            &parts,
            |b, &value| {
                b.iter(|| sum_repeated(value, id_lookup_parts_id));
            },
        );
        group.bench_with_input(BenchmarkId::new("match", name), &parts, |b, &value| {
            b.iter(|| sum_repeated(value, old_parts_id));
        });
    }
    group.finish();
}

/// The former `transmute` candidate cannot exist under `#[repr(u8)]`: a `u16`
/// discriminant no longer fits `BaseVowel`. The surviving candidate is the
/// variant table; the mask-based `from_parts` body is measured in
/// `bench_base_vowel_from_parts.rs`.
fn bench_variant_table_vs_transmute(c: &mut Criterion) {
    for (group_name, values) in [
        ("valid_12", &VALID_PARTS[..]),
        ("all_24_pairs", &ALL_PARTS[..]),
    ] {
        for &(root, shape) in values {
            assert_eq!(
                variant_table_from_parts(root, shape),
                BaseVowel::from_parts(root, shape).ok_or(()),
                "constructor mismatch for {root:?} + {shape:?}"
            );
        }

        let mut group =
            c.benchmark_group(format!("base_vowel_constructor/variant_table/{group_name}"));
        group.throughput(Throughput::Elements(values.len() as u64));
        group.bench_function("variant_table", |b| {
            b.iter(|| {
                sum(values, |(root, shape)| {
                    variant_table_from_parts(root, shape)
                        .map(|vowel| vowel as u16 as usize)
                        .unwrap_or(INVALID)
                })
            });
        });
        group.bench_function("production", |b| {
            b.iter(|| {
                sum(values, |(root, shape)| {
                    BaseVowel::from_parts(root, shape)
                        .map(|vowel| vowel as u16 as usize)
                        .unwrap_or(INVALID)
                })
            });
        });
        group.finish();
    }
}

fn bench_from_root(c: &mut Criterion) {
    for root in ALL_ROOTS {
        assert_eq!(current_root_id(root), old_root_id(root));
        assert_eq!(current_root_id(root), array_root_id(root));
    }

    let mut group = c.benchmark_group("base_vowel_constructor/from_root/all_6");
    group.throughput(Throughput::Elements(ALL_ROOTS.len() as u64));
    group.bench_function("priority_packed", |b| {
        b.iter(|| sum(&ALL_ROOTS, current_root_id));
    });
    group.bench_function("match", |b| {
        b.iter(|| sum(&ALL_ROOTS, old_root_id));
    });
    group.bench_function("array_lookup", |b| {
        b.iter(|| sum(&ALL_ROOTS, array_root_id));
    });
    group.finish();

    let mut group = c.benchmark_group("base_vowel_constructor/from_root/repeated");
    group.throughput(Throughput::Elements(REPEATS as u64));
    for root in [RootVowel::A, RootVowel::U, RootVowel::Y] {
        assert_eq!(current_root_id(root), old_root_id(root));
        assert_eq!(current_root_id(root), array_root_id(root));
        group.bench_with_input(
            BenchmarkId::new("priority_packed", format!("{root:?}")),
            &root,
            |b, &value| b.iter(|| sum_repeated(value, current_root_id)),
        );
        group.bench_with_input(
            BenchmarkId::new("match", format!("{root:?}")),
            &root,
            |b, &value| b.iter(|| sum_repeated(value, old_root_id)),
        );
        group.bench_with_input(
            BenchmarkId::new("array_lookup", format!("{root:?}")),
            &root,
            |b, &value| b.iter(|| sum_repeated(value, array_root_id)),
        );
    }
    group.finish();
}

fn bench_id_from_parts(c: &mut Criterion) {
    for (group_name, values) in [
        ("valid_12", &VALID_PARTS[..]),
        ("all_24_pairs", &ALL_PARTS[..]),
    ] {
        for &(root, shape) in values {
            assert_eq!(
                table_id_from_parts(root, shape),
                match_id_from_parts(root, shape),
                "ID mismatch for {root:?} {shape:?}"
            );
        }

        let mut group = c.benchmark_group(format!("base_vowel_id_from_parts/{group_name}"));
        group.throughput(Throughput::Elements(values.len() as u64));
        group.bench_function("table", |b| {
            b.iter(|| sum(values, table_parts_id));
        });
        group.bench_function("match", |b| {
            b.iter(|| sum(values, match_parts_id));
        });
        group.finish();
    }

    let mut group = c.benchmark_group("base_vowel_id_from_parts/repeated");
    group.throughput(Throughput::Elements(REPEATS as u64));
    for (name, parts) in [
        ("A_circumflex", (RootVowel::A, Shape::Circumflex)),
        ("E_circumflex", (RootVowel::E, Shape::Circumflex)),
        ("O_horn", (RootVowel::O, Shape::Horn)),
    ] {
        assert_eq!(table_parts_id(parts), match_parts_id(parts));
        group.bench_with_input(BenchmarkId::new("table", name), &parts, |b, &value| {
            b.iter(|| sum_repeated(value, table_parts_id));
        });
        group.bench_with_input(BenchmarkId::new("match", name), &parts, |b, &value| {
            b.iter(|| sum_repeated(value, match_parts_id));
        });
    }
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
    targets = bench_from_parts, bench_variant_table_vs_transmute, bench_from_root, bench_id_from_parts
}
criterion_main!(benches);
