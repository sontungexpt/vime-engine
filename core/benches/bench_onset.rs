//! Standalone micro-benchmark comparing two `Onset::from_chars` variants:
//!
//! - `Onset::from_chars` — the current slice-pattern-match implementation.
//! - `loop_from_chars` — the proposed loop + stack-buffer implementation.
//!
//! Run with:
//!   cargo bench --bench bench_onset

use std::hint::black_box;
use std::time::Instant;

use vime_engine::phonology::{Onset, OnsetParseError};

/// The loop variant under test, transcribed verbatim from the proposal.
///
/// Note: the proposal marks this `const fn`, but `&bytes[..chars.len()]`
/// requires const `Index`, which is still unstable — so this is benchmarked
/// as an equivalent `#[inline(always)]` runtime function.
#[inline(always)]
pub fn loop_from_chars(chars: &[char]) -> Result<Onset, OnsetParseError> {
    if chars.len() > Onset::MAX_LEN {
        return Err(OnsetParseError);
    }

    let mut bytes = [0u8; Onset::MAX_LEN];

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        if !ch.is_ascii() {
            if ch == 'đ' || ch == 'Đ' {
                return if chars.len() == 1 {
                    Ok(Onset::DStroke)
                } else {
                    Err(OnsetParseError)
                };
            }

            return Err(OnsetParseError);
        }

        bytes[i] = ch as u8;
        i += 1;
    }

    Onset::from_bytes(&bytes[..chars.len()])
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
    // Representative typing workload, weighted toward the frequent cases:
    // singles dominate, two-char clusters common, three-char rare, plus
    // invalid rejects and đ/Đ.
    let workload: Vec<&[char]> = {
        let mut w = Vec::new();
        for (chars, weight) in [
            (&[][..], 8),
            (&['t'][..], 24),
            (&['T'][..], 8),
            (&['p'][..], 14),
            (&['P'][..], 4),
            (&['b'][..], 20),
            (&['c'][..], 16),
            (&['d'][..], 12),
            (&['g'][..], 12),
            (&['h'][..], 12),
            (&['k'][..], 10),
            (&['l'][..], 16),
            (&['m'][..], 16),
            (&['n'][..], 16),
            (&['r'][..], 12),
            (&['s'][..], 12),
            (&['v'][..], 8),
            (&['x'][..], 6),
            (&['đ'][..], 8),
            (&['Đ'][..], 2),
            (&['c', 'h'][..], 14),
            (&['g', 'h'][..], 10),
            (&['g', 'i'][..], 10),
            (&['k', 'h'][..], 6),
            (&['n', 'h'][..], 8),
            (&['n', 'g'][..], 10),
            (&['p', 'h'][..], 6),
            (&['q', 'u'][..], 8),
            (&['t', 'h'][..], 14),
            (&['t', 'r'][..], 6),
            (&['n', 'g', 'h'][..], 4),
            (&['z'][..], 4),
            (&['q'][..], 4),
            (&['w'][..], 4),
            (&['f', 'f'][..], 2),
            (&['c', 'c', 'c'][..], 2),
        ] {
            for _ in 0..weight {
                w.push(chars);
            }
        }
        w
    };

    // Sanity: both variants must agree on every input.
    for chars in &workload {
        assert_eq!(
            Onset::from_chars(chars),
            loop_from_chars(chars),
            "{chars:?}"
        );
    }

    let rounds = 40;
    let iters = 200_000;

    let match_time = time(
        || {
            for chars in &workload {
                black_box(Onset::from_chars(black_box(chars)).ok());
            }
        },
        rounds,
        iters,
    );

    let loop_time = time(
        || {
            for chars in &workload {
                black_box(loop_from_chars(black_box(chars)).ok());
            }
        },
        rounds,
        iters,
    );

    let per_input = workload.len() as f64;
    let match_ns = match_time.as_nanos() as f64 / (iters as f64 * per_input);
    let loop_ns = loop_time.as_nanos() as f64 / (iters as f64 * per_input);

    println!("inputs per pass: {}", workload.len());
    println!("match impl: {:>8.2} ns/input  (best of {rounds})", match_ns);
    println!("loop  impl: {:>8.2} ns/input  (best of {rounds})", loop_ns);
    println!("ratio match/loop: {:.2}x", match_ns / loop_ns);
}
