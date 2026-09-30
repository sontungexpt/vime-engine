//! Shared timing harness for the benches in this directory.
//!
//! Every bench here is `harness = false` and drives its own loop, so the timer
//! is deliberately tiny: a best-of-N wall clock.
//!
//! # Only compare within a single run
//!
//! Best-of-N is still not enough to make a number portable. Measured on the
//! development machine (i7-1195G7, `scaling_governor = powersave`, so turbo
//! stays on and no frequency is pinned), **two byte-identical runs of the same
//! binary disagreed by 1.15x-1.41x** depending on the figure:
//!
//! | Figure | Run 1 | Run 2 | Slower / faster |
//! | --- | --- | --- | --- |
//! | `bench_composition` append, per push | 20.69 ns | 15.82 ns | 1.31x |
//! | `bench_composition` caret edits, per edit | 49.68 ns | 57.14 ns | 1.15x |
//! | `bench_array_vec` onset, ns/op | 0.734 | 1.032 | 1.41x |
//!
//! That spread is code layout and turbo behaviour, not the algorithm, so a
//! delta smaller than roughly 30% between two separate runs means nothing. Two
//! consequences:
//!
//! - **Compare candidates in one run.** The benches that compare several
//!   implementations (`bench_array_vec`) print them together for exactly this
//!   reason: same binary, same conditions, one timer.
//! - **Never quote a ratio taken from two different runs**, and do not commit
//!   recorded figures as a regression baseline. They will drift with the
//!   hardware, the governor and the allocator, and will not detect a
//!   regression that is smaller than that drift.
//!
//! Before a run whose numbers are meant to be compared, pin the governor:
//!
//! ```sh
//! sudo cpupower frequency-set -g performance   # restore with -g powersave after
//! ```

#![allow(dead_code)]

use std::time::{Duration, Instant};

/// The number of timed repetitions each bench takes. Overridable so a slow
/// machine can trade resolution for runtime via `VIME_BENCH_ROUNDS`.
pub fn rounds() -> usize {
    std::env::var("VIME_BENCH_ROUNDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200)
}

/// Runs `f` `iters` times, `rounds()` times over, and returns the fastest pass.
///
/// Returns the whole pass, not a per-iteration duration, so each caller divides
/// by whatever its own normalization is (per push, per sequence, per element).
pub fn time(f: impl FnMut(), rounds: usize, iters: usize) -> Duration {
    let mut f = f;
    let mut best = Duration::MAX;
    for _ in 0..rounds {
        let start = Instant::now();
        for _ in 0..iters {
            f();
        }
        best = best.min(start.elapsed());
    }
    best
}

/// Nanoseconds for one `time` pass, normalized by `per_pass` units of work.
///
/// `iters` must match the value handed to [`time`]: the returned figure is
/// "ns per unit", already divided by both the iteration count and `per_pass`.
pub fn ns_per_unit(t: Duration, iters: usize, per_pass: f64) -> f64 {
    t.as_nanos() as f64 / (iters as f64 * per_pass)
}

/// A `(name, value)` list, reported fastest-first.
pub fn ranked(mut rows: Vec<(&str, f64)>) -> Vec<(&str, f64)> {
    rows.sort_by(|a, b| a.1.total_cmp(&b.1));
    rows
}
