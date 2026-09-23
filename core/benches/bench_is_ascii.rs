//! Benchmarks ASCII guards for two `char`s.
//!
//! `a.is_ascii() && b.is_ascii()` (short-circuiting) vs a single combined test:
//! `(a | b) & 0xFF80 == 0` — equivalent because any non-ASCII `char` has a bit
//! at/above 0x80, which survives the OR.
//!   cargo bench --bench bench_is_ascii

use std::hint::black_box;
use std::time::Instant;

/// Two-`char` ASCII guard as in `onset.rs` / `coda.rs`.
#[inline(always)]
fn both_ascii_and(a: char, b: char) -> bool {
    a.is_ascii() && b.is_ascii()
}

/// Single-comparison variant computed as one integer test.
#[inline(always)]
fn both_ascii_or(a: char, b: char) -> bool {
    ((a as u32) | (b as u32)) < 0x80
}

#[inline(never)]
fn probe_and(pairs: &[(char, char)]) -> u64 {
    let mut acc = 0u64;
    for &(a, b) in pairs {
        acc = acc.wrapping_mul(31).wrapping_add(both_ascii_and(a, b) as u64);
    }
    black_box(acc)
}

#[inline(never)]
fn probe_or(pairs: &[(char, char)]) -> u64 {
    let mut acc = 0u64;
    for &(a, b) in pairs {
        acc = acc.wrapping_mul(31).wrapping_add(both_ascii_or(a, b) as u64);
    }
    black_box(acc)
}

fn time(f: impl Fn() -> u64, rounds: usize, iters: usize) -> std::time::Duration {
    let mut best = std::time::Duration::MAX;
    for _ in 0..rounds {
        let start = Instant::now();
        for _ in 0..iters {
            black_box(f());
        }
        best = best.min(start.elapsed());
    }
    best
}

fn main() {
    let vietnam = ['đ', 'ể', 'ộ', 'ợ', 'ạ'];

    // Mixtures of ASCII / non-ASCII first and second chars.
    let mix = |ascii: usize| -> Vec<(char, char)> {
        let mut v = Vec::new();
        for i in 0..ascii {
            v.push(((b'a' + (i as u8 % 26)) as char, (b'z' - (i as u8 % 26)) as char));
        }
        for i in 0..vietnam.len() {
            v.push((vietnam[i], vietnam[i]));
            v.push((vietnam[i], (b'a' + i as u8) as char));
            v.push(((b'a' + i as u8) as char, vietnam[i]));
        }
        v
    };

    let workloads: [(&str, Vec<(char, char)>); 3] = [
        ("98% ascii", mix(400)),
        ("82% ascii", mix(32)),
        ("0% ascii", mix(0)),
    ];

    let rounds = 200;
    let iters = 60_000;

    for (name, w) in workloads {
        // Symmetry: both routes must agree on every input.
        for &(a, b) in &w {
            assert_eq!(
                both_ascii_and(a, b),
                both_ascii_or(a, b),
                "mismatch ({:04x}:{:04x})",
                a as u32,
                b as u32
            );
        }

        let t1 = time(|| probe_and(&w), rounds, iters);
        let t2 = time(|| probe_or(&w), rounds, iters);
        let n1 = t1.as_nanos() as f64 / iters as f64;
        let n2 = t2.as_nanos() as f64 / iters as f64;
        println!(
            "{name:<10} ({} rows):  and {:6.2} ns/pass | or {:6.2} ns/pass | or/and {:.3}x",
            w.len(),
            n1,
            n2,
            n2 / n1
        );
    }
    println!("(best of {rounds} rounds; lower is better)");
}