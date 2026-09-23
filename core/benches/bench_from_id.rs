//! Standalone micro-benchmark comparing two `BaseVowel::from_id` variants:
//!
//! - `BaseVowel::from_id` — the current bounds-checked LUT implementation.
//! - `match_from_id` — the proposed 12-arm match implementation.
//!
//! Run with:
//!   cargo bench --bench bench_from_id

use std::hint::black_box;
use std::time::Instant;

use vime_engine::phonology::BaseVowel;

/// The match variant under test, transcribed verbatim from the proposal.
#[inline(always)]
pub const fn match_from_id(id: usize) -> Result<BaseVowel, ()> {
    match id {
        0 => Ok(BaseVowel::Y),
        1 => Ok(BaseVowel::U),
        2 => Ok(BaseVowel::I),
        3 => Ok(BaseVowel::E),
        4 => Ok(BaseVowel::O),
        5 => Ok(BaseVowel::A),
        6 => Ok(BaseVowel::UHorn),
        7 => Ok(BaseVowel::ACircumflex),
        8 => Ok(BaseVowel::OCircumflex),
        9 => Ok(BaseVowel::ABreve),
        10 => Ok(BaseVowel::ECircumflex),
        11 => Ok(BaseVowel::OHorn),
        _ => Err(()),
    }
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

fn main() {
    // Valid IDs (0..=11) dominate, plus out-of-range rejects.
    let mut workload: Vec<usize> = Vec::new();
    for _ in 0..8 {
        for id in 0..12 {
            workload.push(id);
        }
        for id in 12..16 {
            workload.push(id);
        }
    }

    // Sanity: both variants must agree on every input.
    for &id in &workload {
        assert_eq!(BaseVowel::from_id(id), match_from_id(id), "id {id}");
    }

    let rounds = 40;
    let iters = 200_000;

    let lut_time = time(
        || {
            for &id in &workload {
                black_box(BaseVowel::from_id(black_box(id)).ok());
            }
        },
        rounds,
        iters,
    );

    let match_time = time(
        || {
            for &id in &workload {
                black_box(match_from_id(black_box(id)).ok());
            }
        },
        rounds,
        iters,
    );

    let per_input = workload.len() as f64;
    let lut_ns = lut_time.as_nanos() as f64 / (iters as f64 * per_input);
    let match_ns = match_time.as_nanos() as f64 / (iters as f64 * per_input);

    println!("inputs per pass: {}", workload.len());
    println!("lut   impl: {:>8.2} ns/input  (best of {rounds})", lut_ns);
    println!("match impl: {:>8.2} ns/input  (best of {rounds})", match_ns);
    println!("ratio match/lut: {:.2}x", match_ns / lut_ns);
}
