//! Compare implementations of removing a BaseVowel's structural shape.
//!
//! Run with `cargo bench -p vime-engine --bench bench_base_vowel_remove_shape`.

use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use vime_engine::phonology::BaseVowel;

const ALL: [BaseVowel; 12] = [
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

const SHAPED: [BaseVowel; 6] = [
    BaseVowel::UHorn,
    BaseVowel::ACircumflex,
    BaseVowel::OCircumflex,
    BaseVowel::ABreve,
    BaseVowel::ECircumflex,
    BaseVowel::OHorn,
];

#[inline(always)]
fn root_match(base: BaseVowel) -> BaseVowel {
    BaseVowel::from_root(base.root())
}

fn sum(values: &[BaseVowel], f: impl Fn(BaseVowel) -> BaseVowel) -> usize {
    let mut sum = 0usize;
    for &base in black_box(values) {
        sum = sum.wrapping_add(black_box(f(black_box(base))) as u16 as usize);
    }
    black_box(sum)
}

fn bench(c: &mut Criterion) {
    for (name, values) in [("all_12", &ALL[..]), ("shaped_6", &SHAPED[..])] {
        for &base in values {
            assert_eq!(base.remove_shape(), root_match(base));
        }

        let mut group = c.benchmark_group(format!("base_vowel_remove_shape/{name}"));
        group.throughput(Throughput::Elements(values.len() as u64));
        group.bench_function("match", |b| b.iter(|| sum(values, BaseVowel::remove_shape)));
        group.bench_function("root_extraction_then_match", |b| {
            b.iter(|| sum(values, root_match))
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
