//! Compare the two ways to resolve a `(RootVowel, Shape)` pair into a
//! `BaseVowel`:
//!
//! * `id_lut_then_variants_by_id` — what [`BaseVowel::from_parts`] does today:
//!   look the tone-placement ID up in `[Option<u8>; 24]`, then index
//!   `[BaseVowel; 12]` with it. Two dependent loads.
//! * `direct_variants_by_root_shape` — one lookup in `[Option<BaseVowel>; 24]`,
//!   returning the variant itself. One load.
//! * `packed_discriminant_transmute` — keep the ID lookup but rebuild the
//!   packed discriminant instead of indexing `VARIANTS_BY_ID`.
//!
//! The pair the hot path actually resolves is `replace_shape`, so the last
//! group measures that end to end.
//!
//! Run with `cargo bench -p vime-engine --bench bench_base_vowel_from_parts`.

use std::hint::black_box;
use std::sync::OnceLock;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vime_engine::phonology::{BaseVowel, RootVowel, Shape};

const INVALID: usize = u16::MAX as usize;
const REPEATS: usize = 256;
const SHAPE_WIDTH: usize = 2;
const ROOT_OFFSET: usize = 2;
const ID_OFFSET: usize = 5;

/// Every valid pair, in tone-placement ID order.
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

/// Every pair, valid or not: the 12 real combinations plus the 12 that
/// Vietnamese does not spell, which must resolve to `None`.
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

// Bench-local copies of the private production tables, so all three variants
// are measurable from Criterion without exposing them from the crate.

const ID_BY_ROOT_SHAPE: [Option<u8>; 24] = [
    Some(BaseVowel::A.id()),
    Some(BaseVowel::ACircumflex.id()),
    Some(BaseVowel::ABreve.id()),
    None,
    Some(BaseVowel::E.id()),
    Some(BaseVowel::ECircumflex.id()),
    None,
    None,
    Some(BaseVowel::I.id()),
    None,
    None,
    None,
    Some(BaseVowel::O.id()),
    Some(BaseVowel::OCircumflex.id()),
    None,
    Some(BaseVowel::OHorn.id()),
    Some(BaseVowel::U.id()),
    None,
    None,
    Some(BaseVowel::UHorn.id()),
    Some(BaseVowel::Y.id()),
    None,
    None,
    None,
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

const VARIANTS_BY_ROOT_SHAPE: [Option<BaseVowel>; 24] = [
    Some(BaseVowel::A),
    Some(BaseVowel::ACircumflex),
    Some(BaseVowel::ABreve),
    None,
    Some(BaseVowel::E),
    Some(BaseVowel::ECircumflex),
    None,
    None,
    Some(BaseVowel::I),
    None,
    None,
    None,
    Some(BaseVowel::O),
    Some(BaseVowel::OCircumflex),
    None,
    Some(BaseVowel::OHorn),
    Some(BaseVowel::U),
    None,
    None,
    Some(BaseVowel::UHorn),
    Some(BaseVowel::Y),
    None,
    None,
    None,
];

/// Candidate 1: the shipped implementation, one ID lookup plus one index.
///
/// The `match` is kept verbatim instead of `Option::map` so this stays a
/// byte-for-byte copy of the body it replaced.
#[allow(clippy::manual_map)]
#[inline(always)]
fn id_lut_then_variants_by_id(root: RootVowel, shape: Shape) -> Option<BaseVowel> {
    let index = ((root as usize) << SHAPE_WIDTH) | shape as usize;
    match ID_BY_ROOT_SHAPE[index] {
        Some(vowel_id) => Some(VARIANTS_BY_ID[vowel_id as usize]),
        None => None,
    }
}

/// Candidate 2: a single lookup that yields the variant directly.
#[inline(always)]
fn direct_variants_by_root_shape(root: RootVowel, shape: Shape) -> Option<BaseVowel> {
    let index = ((root as usize) << SHAPE_WIDTH) | shape as usize;
    VARIANTS_BY_ROOT_SHAPE[index]
}

/// Candidate 3: keep the ID lookup, rebuild the packed discriminant.
#[allow(clippy::question_mark)]
#[inline(always)]
fn packed_discriminant_transmute(root: RootVowel, shape: Shape) -> Option<BaseVowel> {
    let index = ((root as usize) << SHAPE_WIDTH) | shape as usize;
    let Some(vowel_id) = ID_BY_ROOT_SHAPE[index] else {
        return None;
    };

    let raw = ((vowel_id as u16) << ID_OFFSET) | ((root as u16) << ROOT_OFFSET) | shape as u16;

    // SAFETY: the ID LUT only holds `Some` for the 12 declared root/shape
    // pairs, and ID + components of a declared pair form its discriminant.
    Some(unsafe { std::mem::transmute::<u16, BaseVowel>(raw) })
}

#[inline(always)]
fn production(root: RootVowel, shape: Shape) -> usize {
    BaseVowel::from_parts(root, shape)
        .map(|base| base as u16 as usize)
        .unwrap_or(INVALID)
}

#[inline(always)]
fn id_lut(root: RootVowel, shape: Shape) -> usize {
    id_lut_then_variants_by_id(root, shape)
        .map(|base| base as u16 as usize)
        .unwrap_or(INVALID)
}

#[inline(always)]
fn direct(root: RootVowel, shape: Shape) -> usize {
    direct_variants_by_root_shape(root, shape)
        .map(|base| base as u16 as usize)
        .unwrap_or(INVALID)
}

#[inline(always)]
fn transmute(root: RootVowel, shape: Shape) -> usize {
    packed_discriminant_transmute(root, shape)
        .map(|base| base as u16 as usize)
        .unwrap_or(INVALID)
}

fn sum_pairs(values: &[(RootVowel, Shape)], f: impl Fn(RootVowel, Shape) -> usize) -> usize {
    let mut total = 0usize;
    for &(root, shape) in black_box(values) {
        total = total.wrapping_add(black_box(f(black_box(root), black_box(shape))));
    }
    black_box(total)
}

fn sum_repeated(parts: (RootVowel, Shape), f: impl Fn(RootVowel, Shape) -> usize) -> usize {
    let (root, shape) = black_box(parts);
    let mut total = 0usize;
    for _ in 0..REPEATS {
        total = total.wrapping_add(black_box(f(black_box(root), black_box(shape))));
    }
    black_box(total)
}

/// `replace_shape` is the only production caller of `from_parts`, so measure
/// the whole call rather than the lookup alone.
#[inline(always)]
fn replace_shape_id_lut(base: BaseVowel, shape: Shape) -> usize {
    let index = ((base.root() as usize) << SHAPE_WIDTH) | shape as usize;
    match ID_BY_ROOT_SHAPE[index] {
        Some(vowel_id) => {
            let new = VARIANTS_BY_ID[vowel_id as usize];
            (base.id() as usize) + (new as u16 as usize)
        }
        None => base.id() as usize,
    }
}

#[inline(always)]
fn replace_shape_direct(base: BaseVowel, shape: Shape) -> usize {
    let index = ((base.root() as usize) << SHAPE_WIDTH) | shape as usize;
    match VARIANTS_BY_ROOT_SHAPE[index] {
        Some(new) => (base.id() as usize) + (new as u16 as usize),
        None => base.id() as usize,
    }
}

/// All 12 valid bases, shaped with the key that changes each of them.
const SHAPE_TURNS: [(BaseVowel, Shape); 12] = [
    (BaseVowel::A, Shape::Circumflex),
    (BaseVowel::A, Shape::Breve),
    (BaseVowel::E, Shape::Circumflex),
    (BaseVowel::I, Shape::Horn),
    (BaseVowel::O, Shape::Circumflex),
    (BaseVowel::O, Shape::Horn),
    (BaseVowel::U, Shape::Horn),
    (BaseVowel::Y, Shape::Horn),
    (BaseVowel::ACircumflex, Shape::Breve),
    (BaseVowel::ECircumflex, Shape::Breve),
    (BaseVowel::OHorn, Shape::Breve),
    (BaseVowel::UHorn, Shape::Breve),
];

fn bench_from_parts(c: &mut Criterion) {
    for (group_name, values) in [
        ("valid_12", &VALID_PARTS[..]),
        ("all_24_pairs", &ALL_PARTS[..]),
    ] {
        for &(root, shape) in values {
            assert_eq!(
                BaseVowel::from_parts(root, shape),
                id_lut_then_variants_by_id(root, shape),
                "id-lut candidate diverges for {root:?} + {shape:?}"
            );
            assert_eq!(
                BaseVowel::from_parts(root, shape),
                direct_variants_by_root_shape(root, shape),
                "direct candidate diverges for {root:?} + {shape:?}"
            );
            assert_eq!(
                BaseVowel::from_parts(root, shape),
                packed_discriminant_transmute(root, shape),
                "transmute candidate diverges for {root:?} + {shape:?}"
            );
        }

        let mut group = c.benchmark_group(format!("from_parts/{group_name}"));
        group.throughput(Throughput::Elements(values.len() as u64));
        group.bench_function("production_id_lut_then_variants_by_id", |b| {
            b.iter(|| sum_pairs(values, production));
        });
        group.bench_function("id_lut_then_variants_by_id", |b| {
            b.iter(|| sum_pairs(values, id_lut));
        });
        group.bench_function("direct_variants_by_root_shape", |b| {
            b.iter(|| sum_pairs(values, direct));
        });
        group.bench_function("packed_discriminant_transmute", |b| {
            b.iter(|| sum_pairs(values, transmute));
        });
        group.finish();
    }
}

/// One pair, hammered: the shape key of a single keystroke.
fn bench_from_parts_repeated(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_parts/repeated");
    group.throughput(Throughput::Elements(REPEATS as u64));
    for (name, parts) in [
        ("A_circumflex", (RootVowel::A, Shape::Circumflex)),
        ("E_circumflex", (RootVowel::E, Shape::Circumflex)),
        ("O_horn", (RootVowel::O, Shape::Horn)),
        ("A_horn_invalid", (RootVowel::A, Shape::Horn)),
    ] {
        group.bench_with_input(
            BenchmarkId::new("production_id_lut_then_variants_by_id", name),
            &parts,
            |b, &value| b.iter(|| sum_repeated(value, production)),
        );
        group.bench_with_input(
            BenchmarkId::new("id_lut_then_variants_by_id", name),
            &parts,
            |b, &value| b.iter(|| sum_repeated(value, id_lut)),
        );
        group.bench_with_input(
            BenchmarkId::new("direct_variants_by_root_shape", name),
            &parts,
            |b, &value| b.iter(|| sum_repeated(value, direct)),
        );
        group.bench_with_input(
            BenchmarkId::new("packed_discriminant_transmute", name),
            &parts,
            |b, &value| b.iter(|| sum_repeated(value, transmute)),
        );
    }
    group.finish();
}

fn bench_replace_shape(c: &mut Criterion) {
    for &turn in &SHAPE_TURNS {
        let (base, shape) = turn;
        let expected = base.replace_shape(shape);
        let id_lut_result = replace_shape_id_lut(base, shape);
        let direct_result = replace_shape_direct(base, shape);
        let expected_id = expected.map_or(base.id() as usize, |v| {
            (base.id() as usize) + (v as u16 as usize)
        });
        assert_eq!(
            id_lut_result, expected_id,
            "id-lut turn mismatch for {base:?}"
        );
        assert_eq!(
            direct_result, expected_id,
            "direct turn mismatch for {base:?}"
        );
    }

    let mut group = c.benchmark_group("replace_shape/one_key_per_base");
    group.throughput(Throughput::Elements(SHAPE_TURNS.len() as u64));
    group.bench_function("id_lut_then_variants_by_id", |b| {
        b.iter(|| {
            let mut total = 0usize;
            for &(base, shape) in black_box(&SHAPE_TURNS) {
                total = total.wrapping_add(black_box(replace_shape_id_lut(
                    black_box(base),
                    black_box(shape),
                )));
            }
            black_box(total)
        });
    });
    group.bench_function("direct_variants_by_root_shape", |b| {
        b.iter(|| {
            let mut total = 0usize;
            for &(base, shape) in black_box(&SHAPE_TURNS) {
                total = total.wrapping_add(black_box(replace_shape_direct(
                    black_box(base),
                    black_box(shape),
                )));
            }
            black_box(total)
        });
    });
    group.finish();
}

/// One million pairs, cycling through a 24-entry source array so the input is
/// too large to stay in L1 while both LUTs always are. This is the only group
/// in this file whose signal exceeds the code-alignment noise the 12/24-element
/// loops show, so judge "is the second load free?" here.
const LARGE_LEN: usize = 1_000_000;

struct LargePairs {
    valid: Vec<(RootVowel, Shape)>,
    all: Vec<(RootVowel, Shape)>,
}

static LARGE_PAIRS: OnceLock<LargePairs> = OnceLock::new();

fn large_pairs() -> &'static LargePairs {
    LARGE_PAIRS.get_or_init(|| LargePairs {
        valid: (0..LARGE_LEN)
            .map(|index| VALID_PARTS[index % VALID_PARTS.len()])
            .collect(),
        all: (0..LARGE_LEN)
            .map(|index| ALL_PARTS[index % ALL_PARTS.len()])
            .collect(),
    })
}

fn bench_large(c: &mut Criterion) {
    let pairs = large_pairs();

    for (group_name, values) in [
        ("valid_1m", &pairs.valid[..]),
        ("all_24_1m", &pairs.all[..]),
    ] {
        for &(root, shape) in values.iter().take(24) {
            assert_eq!(
                BaseVowel::from_parts(root, shape),
                id_lut_then_variants_by_id(root, shape)
            );
            assert_eq!(
                BaseVowel::from_parts(root, shape),
                direct_variants_by_root_shape(root, shape)
            );
        }

        let mut group = c.benchmark_group(format!("from_parts/{group_name}"));
        group.throughput(Throughput::Elements(values.len() as u64));
        group.bench_function("production_id_lut_then_variants_by_id", |b| {
            b.iter(|| sum_pairs(values, production));
        });
        group.bench_function("id_lut_then_variants_by_id", |b| {
            b.iter(|| sum_pairs(values, id_lut));
        });
        group.bench_function("direct_variants_by_root_shape", |b| {
            b.iter(|| sum_pairs(values, direct));
        });
        group.bench_function("packed_discriminant_transmute", |b| {
            b.iter(|| sum_pairs(values, transmute));
        });
        group.finish();
    }
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
    targets = bench_from_parts, bench_from_parts_repeated, bench_replace_shape, bench_large
}
criterion_main!(benches);
