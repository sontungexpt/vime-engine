//! Micro-benchmark choosing the fastest correct `has_key` bitmask probe.
//!
//! All variants must agree with `has_key_guard` (the canonical
//! `to_ascii_lowercase` + `key < 128` form) for every probed character,
//! including non-ASCII input where a raw low-byte `| 0x20` trick would
//! collide (e.g. `'Ŧ'` truncates to `0x66` = `'f'`).
//!
//! Run with:
//!   cargo bench --bench bench_has_key

use std::hint::black_box;
use std::time::Instant;

/// V0 — canonical guard + shift.
#[inline(always)]
pub fn has_key_guard(mask: u128, input: char) -> bool {
    let lower = input.to_ascii_lowercase() as u32;
    lower < 128 && (mask & (1u128 << lower)) != 0
}

/// V1 — early `is_ascii` return, u8 shift path.
#[inline(always)]
pub fn has_key_ascii(mask: u128, input: char) -> bool {
    if !input.is_ascii() {
        return false;
    }

    let lower = (input as u8).to_ascii_lowercase();
    (mask & (1u128 << lower)) != 0
}

/// V2 — branchless: arithmetic lowercase, clamped shift, setcc gate.
#[inline(always)]
pub fn has_key_clamp(mask: u128, input: char) -> bool {
    let c = input as u32;

    // Add 32 only for 'A'..='Z' (bit 5 is clear exactly there).
    let is_uppercase = (c.wrapping_sub(b'A' as u32) < 26) as u32;
    let lower = c ^ (is_uppercase << 5);

    // Probe the bit; clamp the shift to 0..127 and AND away the rest.
    let bit = (mask >> (lower & 127)) & 1;
    (bit as u8 & (lower < 128) as u8) != 0
}

/// V3 — branchless: clamped shift, multiply gate.
#[inline(always)]
pub fn has_key_multiply(mask: u128, input: char) -> bool {
    let c = input as u32;
    let is_uppercase = (c.wrapping_sub(b'A' as u32) < 26) as u32;
    let lower = c ^ (is_uppercase << 5);

    let masked = mask & (1u128 << (lower & 127));
    (masked * (lower < 128) as u128) != 0
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
    // Correctness: every variant must agree with the canonical form over a
    // wide probe set, including non-ASCII chars and the `| 0x20` traps.
    let probes: Vec<char> = (0u32..0x80).map(char::from_u32).flatten().collect();

    let mut probes = probes;
    probes.extend([
        'đ', 'Đ', 'ă', 'â', 'ệ', 'ẹ', 'ơ', 'ư', 'ç', 'é', 'Ŧ', 'Ć', 'Ō', 'Ā', 'Å', 'Æ', 'Ⓐ',
        '\u{0511}', '\u{01A0}', '\u{1EC7}', '^', '~', '.', '`',
    ]);

    // Telex masks contain tone keys (s/f/r/x/j/z) and shape keys (a/e/o/w).
    let mut tone_mask = 0u128;
    for &b in b"sfrxjz" {
        tone_mask |= 1u128 << b;
    }
    let mut shape_mask = 0u128;
    for &b in b"aewo" {
        shape_mask |= 1u128 << b;
    }

    for &ch in &probes {
        assert_eq!(
            has_key_guard(tone_mask, ch),
            has_key_ascii(tone_mask, ch),
            "ascii disagreement on {ch:?}"
        );
        assert_eq!(
            has_key_guard(tone_mask, ch),
            has_key_clamp(tone_mask, ch),
            "clamp disagreement on {ch:?}"
        );
        assert_eq!(
            has_key_guard(tone_mask, ch),
            has_key_multiply(tone_mask, ch),
            "multiply disagreement on {ch:?}"
        );
        assert_eq!(
            has_key_guard(shape_mask, ch),
            has_key_ascii(shape_mask, ch),
            "ascii shape disagreement on {ch:?}"
        );
        assert_eq!(
            has_key_guard(shape_mask, ch),
            has_key_clamp(shape_mask, ch),
            "clamp shape disagreement on {ch:?}"
        );
        assert_eq!(
            has_key_guard(shape_mask, ch),
            has_key_multiply(shape_mask, ch),
            "multiply shape disagreement on {ch:?}"
        );
        // The `| 0x20` trap: 'Ŧ' (0x166) truncates to 'f' (0x66).
        if ch == '\u{0166}' {
            assert!(!has_key_guard(tone_mask, ch), "'Ŧ' must NOT be a tone key");
        }
    }
    println!("correctness: ok across {} probes", probes.len());

    // Workload: realistic mix — letters, tone/shape keys, digits, punctuation,
    // and occasional non-ASCII (dead-buffer input).
    let workload: Vec<char> = {
        let mut w = Vec::new();
        for (ch, weight) in [
            ('a', 20), ('e', 16), ('o', 18), ('i', 18), ('u', 16),
            ('s', 6), ('f', 4), ('r', 6), ('x', 4), ('j', 3), ('z', 2),
            ('w', 6), ('d', 6), ('n', 24), ('t', 20), ('c', 14), ('h', 12),
            ('g', 10), ('k', 10), ('l', 10), ('m', 8), ('p', 6), ('q', 4),
            ('b', 8), ('y', 4), ('v', 4), ('A', 4), ('O', 4), ('S', 2),
            ('F', 2), ('D', 2), ('W', 2), ('1', 2), ('3', 2), (' ', 2),
            ('!', 2), (';', 2), ('[', 2), ('đ', 1), ('ệ', 1),
        ] {
            for _ in 0..weight {
                w.push(ch);
            }
        }
        w
    };

    // Unicode-heavy mix: non-ASCII (dead-buffer) input dominates.
    let workload_unicode: Vec<char> = {
        let mut w = Vec::new();
        for (ch, weight) in [
            ('a', 8), ('s', 2), ('w', 2), ('A', 2),
            ('đ', 14), ('ệ', 14), ('ơ', 12), ('ư', 12), ('ă', 10), ('â', 10),
            ('Ŧ', 4), ('é', 6), ('Ā', 6), ('Ⓐ', 2), ('\u{0511}', 2), ('ç', 4),
        ] {
            for _ in 0..weight {
                w.push(ch);
            }
        }
        w
    };

    let rounds = 200;
    let iters = 200_000;
    let probes_per_pass = workload.len() as f64;

    let guard_t = time(
        || {
            for &ch in &workload {
                black_box(has_key_guard(tone_mask, black_box(ch)));
                black_box(has_key_guard(shape_mask, black_box(ch)));
            }
        },
        rounds,
        iters,
    );
    let ascii_t = time(
        || {
            for &ch in &workload {
                black_box(has_key_ascii(tone_mask, black_box(ch)));
                black_box(has_key_ascii(shape_mask, black_box(ch)));
            }
        },
        rounds,
        iters,
    );
    let clamp_t = time(
        || {
            for &ch in &workload {
                black_box(has_key_clamp(tone_mask, black_box(ch)));
                black_box(has_key_clamp(shape_mask, black_box(ch)));
            }
        },
        rounds,
        iters,
    );
    let multiply_t = time(
        || {
            for &ch in &workload {
                black_box(has_key_multiply(tone_mask, black_box(ch)));
                black_box(has_key_multiply(shape_mask, black_box(ch)));
            }
        },
        rounds,
        iters,
    );

    let ns = |t: std::time::Duration| t.as_nanos() as f64 / (iters as f64 * probes_per_pass);

    let guard_ns = ns(guard_t);
    let ascii_ns = ns(ascii_t);
    let clamp_ns = ns(clamp_t);
    let multiply_ns = ns(multiply_t);

    println!("probes per pass: {}", probes_per_pass as usize);
    println!(
        "guard    (V0):   {:>8.2} ns/probe  (best of {rounds})",
        guard_ns
    );
    println!(
        "ascii    (V1):   {:>8.2} ns/probe  (best of {rounds})",
        ascii_ns
    );
    println!(
        "clamp    (V2):   {:>8.2} ns/probe  (best of {rounds})",
        clamp_ns
    );
    println!(
        "multiply (V3):   {:>8.2} ns/probe  (best of {rounds})",
        multiply_ns
    );

    let best = [
        ("guard", guard_ns),
        ("ascii", ascii_ns),
        ("clamp", clamp_ns),
        ("multiply", multiply_ns),
    ]
    .into_iter()
    .min_by(|a, b| a.1.total_cmp(&b.1))
    .unwrap();
    println!("winner (ascii-heavy): {} ({:.2} ns/probe)", best.0, best.1);

    // Second pass over the Unicode-heavy workload.
    let probes_unicode = workload_unicode.len() as f64;
    let ug = ns(time(
        || {
            for &ch in &workload_unicode {
                black_box(has_key_guard(tone_mask, black_box(ch)));
                black_box(has_key_guard(shape_mask, black_box(ch)));
            }
        },
        rounds,
        iters,
    ));
    let ua = ns(time(
        || {
            for &ch in &workload_unicode {
                black_box(has_key_ascii(tone_mask, black_box(ch)));
                black_box(has_key_ascii(shape_mask, black_box(ch)));
            }
        },
        rounds,
        iters,
    ));
    let uc = ns(time(
        || {
            for &ch in &workload_unicode {
                black_box(has_key_clamp(tone_mask, black_box(ch)));
                black_box(has_key_clamp(shape_mask, black_box(ch)));
            }
        },
        rounds,
        iters,
    ));
    let um = ns(time(
        || {
            for &ch in &workload_unicode {
                black_box(has_key_multiply(tone_mask, black_box(ch)));
                black_box(has_key_multiply(shape_mask, black_box(ch)));
            }
        },
        rounds,
        iters,
    ));
    println!("unicode-heavy probes per pass: {}", probes_unicode as usize);
    println!(
        "guard    (V0):   {:>8.2} ns/probe  (best of {rounds})",
        ug
    );
    println!(
        "ascii    (V1):   {:>8.2} ns/probe  (best of {rounds})",
        ua
    );
    println!(
        "clamp    (V2):   {:>8.2} ns/probe  (best of {rounds})",
        uc
    );
    println!(
        "multiply (V3):   {:>8.2} ns/probe  (best of {rounds})",
        um
    );
    let ubest = [("guard", ug), ("ascii", ua), ("clamp", uc), ("multiply", um)]
        .into_iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap();
    println!("winner (unicode-heavy): {} ({:.2} ns/probe)", ubest.0, ubest.1);
}