//! Compare BaseVowel at HEAD, the current tree, and the proposed u8 layout.
//!
//! Run with `cargo bench -p vime-engine --bench bench_base_vowel_versions`.

use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use vime_engine::phonology::{BaseVowel as Current, RootVowel, Shape};

#[derive(Clone, Copy, Debug)]
#[repr(u16)]
enum Head {
    Y = (0 << 5) | ((Shape::None as u16) << 3) | RootVowel::Y as u16,
    U = (1 << 5) | ((Shape::None as u16) << 3) | RootVowel::U as u16,
    I = (2 << 5) | ((Shape::None as u16) << 3) | RootVowel::I as u16,
    E = (3 << 5) | ((Shape::None as u16) << 3) | RootVowel::E as u16,
    O = (4 << 5) | ((Shape::None as u16) << 3) | RootVowel::O as u16,
    A = (5 << 5) | ((Shape::None as u16) << 3) | RootVowel::A as u16,
    UHorn = (6 << 5) | ((Shape::Horn as u16) << 3) | RootVowel::U as u16,
    ACircumflex = (7 << 5) | ((Shape::Circumflex as u16) << 3) | RootVowel::A as u16,
    OCircumflex = (8 << 5) | ((Shape::Circumflex as u16) << 3) | RootVowel::O as u16,
    ABreve = (9 << 5) | ((Shape::Breve as u16) << 3) | RootVowel::A as u16,
    ECircumflex = (10 << 5) | ((Shape::Circumflex as u16) << 3) | RootVowel::E as u16,
    OHorn = (11 << 5) | ((Shape::Horn as u16) << 3) | RootVowel::O as u16,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum Proposed {
    Y = ((RootVowel::Y as u8) << 2) | Shape::None as u8,
    U = ((RootVowel::U as u8) << 2) | Shape::None as u8,
    I = ((RootVowel::I as u8) << 2) | Shape::None as u8,
    E = ((RootVowel::E as u8) << 2) | Shape::None as u8,
    O = ((RootVowel::O as u8) << 2) | Shape::None as u8,
    A = ((RootVowel::A as u8) << 2) | Shape::None as u8,
    UHorn = ((RootVowel::U as u8) << 2) | Shape::Horn as u8,
    ACircumflex = ((RootVowel::A as u8) << 2) | Shape::Circumflex as u8,
    OCircumflex = ((RootVowel::O as u8) << 2) | Shape::Circumflex as u8,
    ABreve = ((RootVowel::A as u8) << 2) | Shape::Breve as u8,
    ECircumflex = ((RootVowel::E as u8) << 2) | Shape::Circumflex as u8,
    OHorn = ((RootVowel::O as u8) << 2) | Shape::Horn as u8,
}

const HEAD_ALL: [Head; 12] = [
    Head::Y,
    Head::U,
    Head::I,
    Head::E,
    Head::O,
    Head::A,
    Head::UHorn,
    Head::ACircumflex,
    Head::OCircumflex,
    Head::ABreve,
    Head::ECircumflex,
    Head::OHorn,
];
const CURRENT_ALL: [Current; 12] = [
    Current::Y,
    Current::U,
    Current::I,
    Current::E,
    Current::O,
    Current::A,
    Current::UHorn,
    Current::ACircumflex,
    Current::OCircumflex,
    Current::ABreve,
    Current::ECircumflex,
    Current::OHorn,
];
const PROPOSED_ALL: [Proposed; 12] = [
    Proposed::Y,
    Proposed::U,
    Proposed::I,
    Proposed::E,
    Proposed::O,
    Proposed::A,
    Proposed::UHorn,
    Proposed::ACircumflex,
    Proposed::OCircumflex,
    Proposed::ABreve,
    Proposed::ECircumflex,
    Proposed::OHorn,
];

const ROOTS: [RootVowel; 6] = [
    RootVowel::A,
    RootVowel::E,
    RootVowel::I,
    RootVowel::O,
    RootVowel::U,
    RootVowel::Y,
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
const IDS: [usize; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const INVALID: usize = usize::MAX;
const ID_BY_ROOT_SHAPE: [u8; 24] = [
    5, 7, 9, 15, 3, 10, 15, 15, 2, 15, 15, 15, 4, 8, 15, 11, 1, 15, 15, 6, 0, 15, 15, 15,
];

#[inline(always)]
fn head_id(v: Head) -> usize {
    (v as u16 >> 5) as usize
}
#[inline(always)]
fn current_id(v: Current) -> usize {
    v.id() as usize
}
#[inline(always)]
fn proposed_id(v: Proposed) -> usize {
    ID_BY_ROOT_SHAPE[v as usize] as usize
}

#[inline(always)]
fn head_parts(v: Head) -> usize {
    let root = (v as u16 & 7) as usize;
    let shape = ((v as u16 >> 3) & 3) as usize;
    (root << 2) | shape
}
#[inline(always)]
fn current_parts(v: Current) -> usize {
    ((v.root() as usize) << 2) | v.shape() as usize
}
#[inline(always)]
fn proposed_parts(v: Proposed) -> usize {
    let raw = v as u8;
    ((((raw >> 2) & 7) as usize) << 2) | (raw & 3) as usize
}

#[inline(always)]
fn old_from_parts(root: RootVowel, shape: Shape) -> Result<Head, ()> {
    match (root, shape) {
        (RootVowel::O, Shape::None) => Ok(Head::O),
        (RootVowel::O, Shape::Horn) => Ok(Head::OHorn),
        (RootVowel::O, Shape::Circumflex) => Ok(Head::OCircumflex),
        (RootVowel::E, Shape::None) => Ok(Head::E),
        (RootVowel::E, Shape::Circumflex) => Ok(Head::ECircumflex),
        (RootVowel::A, Shape::None) => Ok(Head::A),
        (RootVowel::A, Shape::Breve) => Ok(Head::ABreve),
        (RootVowel::A, Shape::Circumflex) => Ok(Head::ACircumflex),
        (RootVowel::U, Shape::None) => Ok(Head::U),
        (RootVowel::U, Shape::Horn) => Ok(Head::UHorn),
        (RootVowel::I, Shape::None) => Ok(Head::I),
        (RootVowel::Y, Shape::None) => Ok(Head::Y),
        _ => Err(()),
    }
}
#[inline(always)]
fn current_from_parts(root: RootVowel, shape: Shape) -> Option<Current> {
    Current::from_parts(root, shape)
}
#[inline(always)]
fn proposed_from_parts(root: RootVowel, shape: Shape) -> Result<Proposed, ()> {
    let index = ((root as usize) << 2) | shape as usize;
    let id = ID_BY_ROOT_SHAPE[index];
    if id == 15 {
        Err(())
    } else {
        Ok(PROPOSED_ALL[id as usize])
    }
}

#[inline(always)]
fn head_from_id(id: usize) -> usize {
    if id < HEAD_ALL.len() {
        HEAD_ALL[id] as u16 as usize
    } else {
        INVALID
    }
}
#[inline(always)]
fn current_from_id(id: usize) -> usize {
    Current::from_id(id)
        .map(|v| v as u16 as usize)
        .unwrap_or(INVALID)
}
#[inline(always)]
fn proposed_from_id(id: usize) -> usize {
    if id < PROPOSED_ALL.len() {
        PROPOSED_ALL[id] as u8 as usize
    } else {
        INVALID
    }
}

#[inline(always)]
fn head_from_root(root: RootVowel) -> usize {
    match root {
        RootVowel::A => Head::A as u16 as usize,
        RootVowel::E => Head::E as u16 as usize,
        RootVowel::I => Head::I as u16 as usize,
        RootVowel::O => Head::O as u16 as usize,
        RootVowel::U => Head::U as u16 as usize,
        RootVowel::Y => Head::Y as u16 as usize,
    }
}
#[inline(always)]
fn current_from_root(root: RootVowel) -> usize {
    Current::from_root(root) as u16 as usize
}
#[inline(always)]
fn proposed_from_root(root: RootVowel) -> usize {
    match root {
        RootVowel::A => Proposed::A as u8 as usize,
        RootVowel::E => Proposed::E as u8 as usize,
        RootVowel::I => Proposed::I as u8 as usize,
        RootVowel::O => Proposed::O as u8 as usize,
        RootVowel::U => Proposed::U as u8 as usize,
        RootVowel::Y => Proposed::Y as u8 as usize,
    }
}

fn sum<T: Copy>(values: &[T], f: impl Fn(T) -> usize) -> usize {
    let mut total = 0usize;
    for &v in black_box(values) {
        total = total.wrapping_add(black_box(f(black_box(v))));
    }
    black_box(total)
}

fn bench_ids(c: &mut Criterion) {
    let mut g = c.benchmark_group("base_vowel_versions/id_all_12");
    g.throughput(Throughput::Elements(12));
    g.bench_function("HEAD", |b| b.iter(|| sum(&HEAD_ALL, head_id)));
    g.bench_function("current", |b| b.iter(|| sum(&CURRENT_ALL, current_id)));
    g.bench_function("proposed", |b| b.iter(|| sum(&PROPOSED_ALL, proposed_id)));
    g.finish();
}

fn bench_fields(c: &mut Criterion) {
    let mut g = c.benchmark_group("base_vowel_versions/root_shape_all_12");
    g.throughput(Throughput::Elements(12));
    g.bench_function("HEAD", |b| b.iter(|| sum(&HEAD_ALL, head_parts)));
    g.bench_function("current", |b| b.iter(|| sum(&CURRENT_ALL, current_parts)));
    g.bench_function("proposed", |b| {
        b.iter(|| sum(&PROPOSED_ALL, proposed_parts))
    });
    g.finish();
}

fn bench_from_parts(c: &mut Criterion) {
    for (name, values) in [("valid_12", &VALID_PARTS[..]), ("all_24", &ALL_PARTS[..])] {
        for &(root, shape) in values {
            let h = old_from_parts(root, shape).map(head_id).unwrap_or(INVALID);
            let cur = current_from_parts(root, shape)
                .map(current_id)
                .unwrap_or(INVALID);
            let p = proposed_from_parts(root, shape)
                .map(proposed_id)
                .unwrap_or(INVALID);
            assert_eq!(h, cur);
            assert_eq!(h, p);
        }
        let mut g = c.benchmark_group(format!("base_vowel_versions/from_parts/{name}"));
        g.throughput(Throughput::Elements(values.len() as u64));
        g.bench_function("HEAD", |b| {
            b.iter(|| {
                sum(values, |p| {
                    old_from_parts(p.0, p.1)
                        .map(|v| v as u16 as usize)
                        .unwrap_or(INVALID)
                })
            })
        });
        g.bench_function("current", |b| {
            b.iter(|| {
                sum(values, |p| {
                    current_from_parts(p.0, p.1)
                        .map(|v| v as u16 as usize)
                        .unwrap_or(INVALID)
                })
            })
        });
        g.bench_function("proposed", |b| {
            b.iter(|| {
                sum(values, |p| {
                    proposed_from_parts(p.0, p.1)
                        .map(|v| v as u8 as usize)
                        .unwrap_or(INVALID)
                })
            })
        });
        g.finish();
    }
}

fn bench_from_id(c: &mut Criterion) {
    let mut g = c.benchmark_group("base_vowel_versions/from_id_all_12");
    g.throughput(Throughput::Elements(12));
    g.bench_function("HEAD", |b| b.iter(|| sum(&IDS, head_from_id)));
    g.bench_function("current", |b| b.iter(|| sum(&IDS, current_from_id)));
    g.bench_function("proposed", |b| b.iter(|| sum(&IDS, proposed_from_id)));
    g.finish();
}

fn bench_from_root(c: &mut Criterion) {
    let mut g = c.benchmark_group("base_vowel_versions/from_root_all_6");
    g.throughput(Throughput::Elements(6));
    g.bench_function("HEAD", |b| b.iter(|| sum(&ROOTS, head_from_root)));
    g.bench_function("current", |b| b.iter(|| sum(&ROOTS, current_from_root)));
    g.bench_function("proposed", |b| b.iter(|| sum(&ROOTS, proposed_from_root)));
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
    targets = bench_ids, bench_fields, bench_from_parts, bench_from_id, bench_from_root
}
criterion_main!(benches);
