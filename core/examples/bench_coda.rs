//! Standalone micro-benchmark comparing two `Coda::from_bytes` variants:
//!
//! - `Coda::from_bytes` — the current fully-inlined match implementation.
//! - `delegating_from_bytes` — the proposed variant that forwards the
//!   single- and two-byte arms to `from_byte` / `from_two_bytes`.
//!
//! Run with:
//!   cargo run --release --example bench_coda

use std::hint::black_box;
use std::time::Instant;

use vime_engine::phonology::{Coda, CodaParseError};

/// The current fully-inlined variant, transcribed verbatim into the example so
/// both candidates share the same compilation context.
#[inline(always)]
pub const fn inline_from_bytes(bytes: &[u8]) -> Result<Coda, CodaParseError> {
    match bytes {
        [] => Ok(Coda::None),

        &[byte] => match byte | 0x20 {
            b'c' => Ok(Coda::C),
            b'm' => Ok(Coda::M),
            b'n' => Ok(Coda::N),
            b'p' => Ok(Coda::P),
            b't' => Ok(Coda::T),
            _ => Err(CodaParseError),
        },

        &[first, second] => match [first | 0x20, second | 0x20] {
            [b'c', b'h'] => Ok(Coda::Ch),
            [b'n', b'g'] => Ok(Coda::Ng),
            [b'n', b'h'] => Ok(Coda::Nh),
            _ => Err(CodaParseError),
        },

        _ => Err(CodaParseError),
    }
}

/// The delegating variant under test, transcribed verbatim from the proposal.
#[inline(always)]
pub const fn delegating_from_bytes(bytes: &[u8]) -> Result<Coda, CodaParseError> {
    match bytes {
        [] => Ok(Coda::None),

        // Single-byte codas ("c", "m", "n", "p", "t")
        &[byte] => delegating_from_byte(byte),

        // Two-byte codas ("ch", "ng", "nh")
        &[first, second] => delegating_from_two_bytes(first, second),

        _ => Err(CodaParseError),
    }
}

#[inline(always)]
const fn delegating_from_byte(byte: u8) -> Result<Coda, CodaParseError> {
    match byte | 0x20 {
        b'c' => Ok(Coda::C),
        b'm' => Ok(Coda::M),
        b'n' => Ok(Coda::N),
        b'p' => Ok(Coda::P),
        b't' => Ok(Coda::T),
        _ => Err(CodaParseError),
    }
}

#[inline(always)]
const fn delegating_from_two_bytes(first: u8, second: u8) -> Result<Coda, CodaParseError> {
    match (first | 0x20, second | 0x20) {
        (b'c', b'h') => Ok(Coda::Ch),
        (b'n', b'g') => Ok(Coda::Ng),
        (b'n', b'h') => Ok(Coda::Nh),
        _ => Err(CodaParseError),
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
    // Representative workload, weighted toward the frequent cases: empty,
    // single letters dominate, two-char clusters common, plus invalid
    // rejects and over-long inputs.
    let workload: Vec<&[u8]> = {
        let mut w = Vec::new();
        for (bytes, weight) in [
            (&b""[..], 8),
            (&b"c"[..], 16),
            (&b"C"[..], 4),
            (&b"m"[..], 20),
            (&b"M"[..], 4),
            (&b"n"[..], 24),
            (&b"N"[..], 4),
            (&b"p"[..], 14),
            (&b"P"[..], 4),
            (&b"t"[..], 16),
            (&b"T"[..], 4),
            (&b"k"[..], 4),
            (&b"x"[..], 4),
            (&b"ch"[..], 14),
            (&b"CH"[..], 3),
            (&b"ng"[..], 16),
            (&b"NG"[..], 3),
            (&b"nh"[..], 14),
            (&b"NH"[..], 3),
            (&b"gh"[..], 3),
            (&b"ph"[..], 3),
            (&b"ccc"[..], 2),
        ] {
            for _ in 0..weight {
                w.push(bytes);
            }
        }
        w
    };

    // Sanity: all three variants must agree on every input.
    for bytes in &workload {
        assert_eq!(
            Coda::from_bytes(bytes),
            inline_from_bytes(bytes),
            "{bytes:?}"
        );
        assert_eq!(
            inline_from_bytes(bytes),
            delegating_from_bytes(bytes),
            "{bytes:?}"
        );
    }

    let rounds = 200;
    let iters = 200_000;

    let inline_time = time(
        || {
            for bytes in &workload {
                black_box(inline_from_bytes(black_box(bytes)).ok());
            }
        },
        rounds,
        iters,
    );

    let delegating_time = time(
        || {
            for bytes in &workload {
                black_box(delegating_from_bytes(black_box(bytes)).ok());
            }
        },
        rounds,
        iters,
    );

    let per_input = workload.len() as f64;
    let inline_ns = inline_time.as_nanos() as f64 / (iters as f64 * per_input);
    let delegating_ns = delegating_time.as_nanos() as f64 / (iters as f64 * per_input);

    println!("inputs per pass: {}", workload.len());
    println!("inline     impl: {:>8.2} ns/input  (best of {rounds})", inline_ns);
    println!("delegating impl: {:>8.2} ns/input  (best of {rounds})", delegating_ns);
    println!("ratio delegating/inline: {:.2}x", delegating_ns / inline_ns);
}
