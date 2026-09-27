//! Compare a sentinel-valued ID LUT with an `Option<u8>` LUT.
//!
//! Run with `cargo bench -p vime-engine --bench bench_base_vowel_option_lut`.

use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use vime_engine::phonology::{BaseVowel, RootVowel, Shape};

const INVALID_ID: u8 = 0xF;
const ROOT_OFFSET: usize = 2;
const ID_OFFSET: usize = 5;

// Index: root * 4 + shape; Shape discriminants are None, Circumflex, Breve, Horn.
const SENTINEL_LUT: [u8; 24] = [
    BaseVowel::A.priority_id(),
    BaseVowel::ACircumflex.priority_id(),
    BaseVowel::ABreve.priority_id(),
    INVALID_ID,
    BaseVowel::E.priority_id(),
    BaseVowel::ECircumflex.priority_id(),
    INVALID_ID,
    INVALID_ID,
    BaseVowel::I.priority_id(),
    INVALID_ID,
    INVALID_ID,
    INVALID_ID,
    BaseVowel::O.priority_id(),
    BaseVowel::OCircumflex.priority_id(),
    INVALID_ID,
    BaseVowel::OHorn.priority_id(),
    BaseVowel::U.priority_id(),
    INVALID_ID,
    INVALID_ID,
    BaseVowel::UHorn.priority_id(),
    BaseVowel::Y.priority_id(),
    INVALID_ID,
    INVALID_ID,
    INVALID_ID,
];

const OPTION_LUT: [Option<u8>; 24] = [
    Some(BaseVowel::A.priority_id()),
    Some(BaseVowel::ACircumflex.priority_id()),
    Some(BaseVowel::ABreve.priority_id()),
    None,
    Some(BaseVowel::E.priority_id()),
    Some(BaseVowel::ECircumflex.priority_id()),
    None,
    None,
    Some(BaseVowel::I.priority_id()),
    None,
    None,
    None,
    Some(BaseVowel::O.priority_id()),
    Some(BaseVowel::OCircumflex.priority_id()),
    None,
    Some(BaseVowel::OHorn.priority_id()),
    Some(BaseVowel::U.priority_id()),
    None,
    None,
    Some(BaseVowel::UHorn.priority_id()),
    Some(BaseVowel::Y.priority_id()),
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

#[inline(always)]
fn sentinel_id(root: RootVowel, shape: Shape) -> Option<u8> {
    let index = ((root as usize) << 2) | shape as usize;
    let id = SENTINEL_LUT[index];
    if id == INVALID_ID {
        None
    } else {
        Some(id)
    }
}

#[inline(always)]
fn option_id(root: RootVowel, shape: Shape) -> Option<u8> {
    let index = ((root as usize) << 2) | shape as usize;
    OPTION_LUT[index]
}

#[inline(always)]
fn option_from_parts(root: RootVowel, shape: Shape) -> Option<BaseVowel> {
    let index = ((root as usize) << 2) | shape as usize;
    match OPTION_LUT[index] {
        Some(id) => Some(VARIANTS_BY_ID[id as usize]),
        None => None,
    }
}

#[inline(always)]
fn transmute_from_parts(root: RootVowel, shape: Shape) -> Option<BaseVowel> {
    let id = BaseVowel::id_from_parts(root, shape)?;
    let raw = ((id as u16) << ID_OFFSET) | ((root as u16) << ROOT_OFFSET) | shape as u16;

    // SAFETY: `id_from_parts` only returns Some for valid root/shape pairs,
    // and its ID plus those components form a declared BaseVowel discriminant.
    Some(unsafe { std::mem::transmute::<u16, BaseVowel>(raw) })
}

fn sum_parts(parts: &[(RootVowel, Shape)], id: impl Fn(RootVowel, Shape) -> Option<u8>) -> usize {
    let mut sum = 0usize;
    for &(root, shape) in black_box(parts) {
        let value = black_box(id(black_box(root), black_box(shape)).unwrap_or(INVALID_ID));
        sum = sum.wrapping_add(value as usize);
    }
    black_box(sum)
}

fn sum_constructors(
    parts: &[(RootVowel, Shape)],
    from_parts: impl Fn(RootVowel, Shape) -> Option<BaseVowel>,
) -> usize {
    let mut sum = 0usize;
    for &(root, shape) in black_box(parts) {
        let value = match from_parts(black_box(root), black_box(shape)) {
            Some(base) => base as u16 as usize,
            None => INVALID_ID as usize,
        };
        sum = sum.wrapping_add(black_box(value));
    }
    black_box(sum)
}

fn bench(c: &mut Criterion) {
    for (name, parts) in [("valid_12", &VALID_PARTS[..]), ("all_24", &ALL_PARTS[..])] {
        for &(root, shape) in parts {
            assert_eq!(sentinel_id(root, shape), option_id(root, shape));
            assert_eq!(
                BaseVowel::from_parts(root, shape),
                option_from_parts(root, shape)
            );
            assert_eq!(
                BaseVowel::from_parts(root, shape),
                transmute_from_parts(root, shape)
            );
        }
        let mut group = c.benchmark_group(format!("base_vowel_option_lut/{name}"));
        group.throughput(Throughput::Elements(parts.len() as u64));
        group.bench_function("sentinel_u8", |b| b.iter(|| sum_parts(parts, sentinel_id)));
        group.bench_function("option_u8", |b| b.iter(|| sum_parts(parts, option_id)));
        group.finish();

        let mut group = c.benchmark_group(format!("base_vowel_option_from_parts/{name}"));
        group.throughput(Throughput::Elements(parts.len() as u64));
        group.bench_function("sentinel_u8", |b| {
            b.iter(|| sum_constructors(parts, BaseVowel::from_parts))
        });
        group.bench_function("option_u8", |b| {
            b.iter(|| sum_constructors(parts, option_from_parts))
        });
        group.finish();

        let mut group = c.benchmark_group(format!("base_vowel_option_transmute/{name}"));
        group.throughput(Throughput::Elements(parts.len() as u64));
        group.bench_function("variant_table", |b| {
            b.iter(|| sum_constructors(parts, BaseVowel::from_parts))
        });
        group.bench_function("packed_transmute", |b| {
            b.iter(|| sum_constructors(parts, transmute_from_parts))
        });
        group.finish();
    }
}

fn configure() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2))
        .sample_size(30)
}

criterion_group! {
    name = benches;
    config = configure();
    targets = bench
}
criterion_main!(benches);
