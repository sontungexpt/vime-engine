//! Micro-benchmark of `vowels_starts_with_uo`: the current `matches!` slice
//! pattern form vs the older `len() > 1 &&` indexing form. Both variants are
//! transcribed verbatim from the crate.
//!
//! Run with:
//!   cargo run --release --example bench_vowels_uo

use std::hint::black_box;
use std::time::Instant;

use vime_engine::phonology::{BaseVowel, CasedBaseVowel, RootVowel};

/// Old form (committed `068edb6`): length guard then indexed reads.
#[inline(always)]
fn old_starts_with_uo(vowels: &[CasedBaseVowel]) -> bool {
    let vowels = vowels;
    vowels.len() > 1
        && vowels[0].value.root() == RootVowel::U
        && vowels[1].value.root() == RootVowel::O
}

/// New form (working tree): `matches!` with a slice pattern and guard.
#[inline(always)]
fn new_starts_with_uo(vowels: &[CasedBaseVowel]) -> bool {
    matches!(
        vowels,
        [v0, v1, ..] if v0.value.root() == RootVowel::U && v1.value.root() == RootVowel::O
    )
}

fn time(f: impl Fn(), rounds: usize, iters: usize) -> std::time::Duration {
    let mut best = std::time::Duration::MAX;
    for _ in 0..rounds {
        let start = Instant::now();
        for _ in 0..iters {
            f();
        }
        best = best.min(start.elapsed());
    }
    best
}

fn v(b: BaseVowel) -> CasedBaseVowel {
    CasedBaseVowel::new(b, false)
}

fn main() {
    // Weighted workload matching how the check is exercised while typing:
    // mostly short nuclei, then pairs, a few triples, and the true `uo` pair
    // both plain and with the horn/shape already applied.
    let mut workload: Vec<(Vec<CasedBaseVowel>, usize)> = Vec::new();
    macro_rules! add {
        ($w:expr; $($b:expr),+) => {{
            let seq: Vec<CasedBaseVowel> = vec![$(v($b)),+];
            workload.push((seq, $w));
        }};
    }
    // len-1 (the overwhelming majority)
    add!(8; BaseVowel::A); add!(8; BaseVowel::O); add!(6; BaseVowel::U);
    add!(6; BaseVowel::I); add!(6; BaseVowel::E); add!(3; BaseVowel::OHorn);
    add!(3; BaseVowel::UHorn); add!(3; BaseVowel::OCircumflex); add!(2; BaseVowel::ACircumflex);
    add!(2; BaseVowel::ECircumflex); add!(2; BaseVowel::ABreve); add!(1; BaseVowel::Y);
    // len-2 pairs including the true `uo`
    add!(5; BaseVowel::U, BaseVowel::O);   // true
    add!(2; BaseVowel::O, BaseVowel::U);
    add!(2; BaseVowel::I, BaseVowel::E);
    add!(2; BaseVowel::A, BaseVowel::I);
    add!(2; BaseVowel::U, BaseVowel::A);
    add!(2; BaseVowel::O, BaseVowel::A);
    add!(1; BaseVowel::E, BaseVowel::U);
    add!(1; BaseVowel::U, BaseVowel::OHorn);
    add!(1; BaseVowel::UHorn, BaseVowel::O); // true
    // len-3
    add!(3; BaseVowel::U, BaseVowel::O, BaseVowel::I); // true
    add!(1; BaseVowel::I, BaseVowel::E, BaseVowel::U);
    add!(1; BaseVowel::O, BaseVowel::A, BaseVowel::I);
    add!(1; BaseVowel::U, BaseVowel::O, BaseVowel::E); // true

    // Expand by weight.
    let mut inputs: Vec<Vec<CasedBaseVowel>> = Vec::new();
    for (seq, w) in &workload {
        for _ in 0..*w {
            inputs.push(seq.clone());
        }
    }

    // Sanity: both variants must agree on every input.
    for seq in &inputs {
        assert_eq!(old_starts_with_uo(seq), new_starts_with_uo(seq), "{seq:?}");
    }

    let rounds = 300;
    let iters = 300_000;

    // Warm both.
    black_box(old_starts_with_uo(&inputs[0]));
    black_box(new_starts_with_uo(&inputs[0]));

    let old_t = time(
        || {
            for seq in &inputs {
                black_box(old_starts_with_uo(black_box(seq)));
            }
        },
        rounds,
        iters,
    );
    let new_t = time(
        || {
            for seq in &inputs {
                black_box(new_starts_with_uo(black_box(seq)));
            }
        },
        rounds,
        iters,
    );

    let n = inputs.len() as f64;
    let old_ns = old_t.as_nanos() as f64 / (iters as f64 * n);
    let new_ns = new_t.as_nanos() as f64 / (iters as f64 * n);

    println!("inputs per pass: {}", inputs.len());
    println!("true  uo cases / pass: {}", inputs.iter().filter(|s| old_starts_with_uo(s)).count());
    println!("old (len>1 &&)   : {:>7.3} ns/input  (best of {rounds})", old_ns);
    println!("new (matches![])  : {:>7.3} ns/input  (best of {rounds})", new_ns);
    println!("ratio new/old: {:.3}x", new_ns / old_ns);
}