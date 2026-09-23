//! Micro-benchmark comparing two `DefaultKeymap` key-classification strategies:
//!
//! - **linear** — the current slice scan (`tones.iter().any(...)`) as used by
//!   `is_tone_key` / `is_shape_key` / `is_stroke_key`.
//! - **mask** — a `u128` bitmask lookup via a shared `has_key`.
//! - **branchless** — a bitmask variant with a branchless lowercase (`| 0x20`)
//!   and shift guard. Kept ONLY as a documented caution: it does not
//!   generalize beyond `A-Z`. For punctuation keys such as VIQR's `'^'`
//!   (`0x5E`), `| 0x20` yields `'~'` (`0x7E`), so `'^'` collides with the `~`
//!   tone bit and breaks shape lookup. The correctness section below proves
//!   the collision against the real VIQR layout. Do not use — use `has_key`.
//!
//! Run with:
//!   cargo bench --bench bench_keylookup
//!
//! Verifies all strategies agree on every probed character, then reports the
//! best-of wall time per probe for each.

use std::hint::black_box;
use std::time::Instant;

use vime_engine::{DefaultKeymap, Rules};

/// Current implementation: linear scan over the matching slice.
#[inline(always)]
pub fn is_tone_key_linear(rules: &Rules, input: char) -> bool {
    let key = input.to_ascii_lowercase() as u8;
    rules.tones.iter().any(|map| map.key == key)
}

#[inline(always)]
pub fn is_shape_key_linear(rules: &Rules, input: char) -> bool {
    let key = input.to_ascii_lowercase() as u8;
    rules.shapes.iter().any(|map| map.key == key)
}

#[inline(always)]
pub fn is_stroke_key_linear(rules: &Rules, input: char) -> bool {
    let key = input.to_ascii_lowercase() as u8;
    rules.strokes.iter().any(|&c| c == key)
}

/// Proposed implementation: shared bitmask probe.
#[inline(always)]
pub fn has_key(mask: u128, input: char) -> bool {
    let key = input.to_ascii_lowercase() as u32;
    key < 128 && (mask & (1u128 << key)) != 0
}

#[inline(always)]
pub fn is_tone_key_mask(tone_mask: u128, input: char) -> bool {
    has_key(tone_mask, input)
}

#[inline(always)]
pub fn is_shape_key_mask(shape_mask: u128, input: char) -> bool {
    has_key(shape_mask, input)
}

#[inline(always)]
pub fn is_stroke_key_mask(stroke_mask: u128, input: char) -> bool {
    has_key(stroke_mask, input)
}

/// Proposed branchless variant: lowercase via `| 0x20` (no branch), mask with
/// `& 127` to keep the shift in-bounds, and multiply by `(code < 128)` to zero
/// non-ASCII inputs.
#[inline(always)]
pub fn has_key_branchless(mask: u128, input: char) -> bool {
    let code = (input as u32) | 0x20;
    let in_range = (code < 128) as u128;
    (mask & in_range.wrapping_neg() & (1u128 << (code & 127))) != 0
}

#[inline(always)]
pub fn is_tone_key_branchless(tone_mask: u128, input: char) -> bool {
    has_key_branchless(tone_mask, input)
}

#[inline(always)]
pub fn is_shape_key_branchless(shape_mask: u128, input: char) -> bool {
    has_key_branchless(shape_mask, input)
}

#[inline(always)]
pub fn is_stroke_key_branchless(stroke_mask: u128, input: char) -> bool {
    has_key_branchless(stroke_mask, input)
}

fn build_masks(rules: &Rules) -> (u128, u128, u128) {
    let mut tone_mask = 0u128;
    let mut shape_mask = 0u128;
    let mut stroke_mask = 0u128;
    for map in rules.tones {
        tone_mask |= 1u128 << (map.key as u32);
    }
    for map in rules.shapes {
        shape_mask |= 1u128 << (map.key as u32);
    }
    for &c in rules.strokes {
        stroke_mask |= 1u128 << (c as u32);
    }
    (tone_mask, shape_mask, stroke_mask)
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
    // Representative single-character workload. Telex maps tones to s/f/r/x/j/z
    // and shapes to a/e/o/w, so plain letters, vowels, and typical keyboard
    // neighbors dominate; tone/shape/stroke keys are the frequent matches.
    let workload: Vec<char> = {
        let mut w = Vec::new();
        for (ch, weight) in [
            ('a', 20),
            ('e', 16),
            ('o', 18),
            ('i', 18),
            ('u', 16),
            ('s', 6),
            ('f', 4),
            ('r', 6),
            ('x', 4),
            ('j', 3),
            ('z', 2),
            ('w', 6),
            ('d', 6),
            ('n', 24),
            ('t', 20),
            ('c', 14),
            ('h', 12),
            ('g', 10),
            ('k', 10),
            ('l', 10),
            ('m', 8),
            ('p', 6),
            ('q', 4),
            ('b', 8),
            ('y', 4),
            ('v', 4),
            ('A', 4),
            ('O', 4),
            ('S', 2),
            ('F', 2),
            ('D', 2),
            ('W', 2),
            ('1', 2),
            ('3', 2),
            (' ', 2),
            // Non-matching keys that exercise the no-collision path
            ('!', 2),
            (';', 2),
            ('[', 2),
        ] {
            for _ in 0..weight {
                w.push(ch);
            }
        }
        w
    };

    let keymap = DefaultKeymap::telex();
    let rules: &Rules = keymap.config();
    let (tone_mask, shape_mask, stroke_mask) = build_masks(rules);

    // Correctness: all three strategies must agree for every probed character,
    // including non-ASCII characters and cases.
    let probe_set = [
        'a', 'A', 'e', 'E', 'o', 'O', 'i', 'u', 's', 'S', 'f', 'r', 'x', 'j', 'z', 'w', 'W', 'd',
        'D', 'n', 't', 'c', 'h', 'g', 'k', 'l', 'm', 'p', 'q', 'b', 'y', 'v', '1', '3', ' ', '!',
        ';', '[', 'Z', 'đ', 'Đ', '\u{0511}',
    ];
    for &ch in &probe_set {
        assert_eq!(
            is_tone_key_linear(rules, ch),
            is_tone_key_mask(tone_mask, ch),
            "tone disagreement on {ch:?}"
        );
        assert_eq!(
            is_tone_key_linear(rules, ch),
            is_tone_key_branchless(tone_mask, ch),
            "tone branchless disagreement on {ch:?}"
        );
        assert_eq!(
            is_shape_key_linear(rules, ch),
            is_shape_key_mask(shape_mask, ch),
            "shape disagreement on {ch:?}"
        );
        assert_eq!(
            is_shape_key_linear(rules, ch),
            is_shape_key_branchless(shape_mask, ch),
            "shape branchless disagreement on {ch:?}"
        );
        assert_eq!(
            is_stroke_key_linear(rules, ch),
            is_stroke_key_mask(stroke_mask, ch),
            "stroke disagreement on {ch:?}"
        );
        assert_eq!(
            is_stroke_key_linear(rules, ch),
            is_stroke_key_branchless(stroke_mask, ch),
            "stroke branchless disagreement on {ch:?}"
        );
    }

    // VIQR demonstration: `| 0x20` corrupts punctuation. `'^'` (0x5E) becomes
    // `'~'` (0x7E), which is exactly the VIQR Tilde tone key — so the branchless
    // variant reports `^` as a tone key and MISSES it as a shape key.
    let viqr = DefaultKeymap::viqr();
    let (viqr_tone, viqr_shape, _) = build_masks(viqr.config());
    assert!(is_tone_key_linear(viqr.config(), '^') == is_tone_key_mask(viqr_tone, '^'));
    assert_ne!(
        is_tone_key_linear(viqr.config(), '^'),
        is_tone_key_branchless(viqr_tone, '^'),
        "expected branchless collision: '^' misdetected as the '~' tone key"
    );
    assert!(
        is_shape_key_linear(viqr.config(), '^') != is_shape_key_branchless(viqr_shape, '^'),
        "expected branchless to miss '^' as a shape key"
    );

    let probes_per_pass = (workload.len() * 3) as f64;

    let rounds = 200;
    let iters = 200_000;

    let linear_time = time(
        || {
            for &ch in &workload {
                black_box(is_tone_key_linear(rules, black_box(ch)));
                black_box(is_shape_key_linear(rules, black_box(ch)));
                black_box(is_stroke_key_linear(rules, black_box(ch)));
            }
        },
        rounds,
        iters,
    );

    let mask_time = time(
        || {
            for &ch in &workload {
                black_box(is_tone_key_mask(tone_mask, black_box(ch)));
                black_box(is_shape_key_mask(shape_mask, black_box(ch)));
                black_box(is_stroke_key_mask(stroke_mask, black_box(ch)));
            }
        },
        rounds,
        iters,
    );

    let branchless_time = time(
        || {
            for &ch in &workload {
                black_box(is_tone_key_branchless(tone_mask, black_box(ch)));
                black_box(is_shape_key_branchless(shape_mask, black_box(ch)));
                black_box(is_stroke_key_branchless(stroke_mask, black_box(ch)));
            }
        },
        rounds,
        iters,
    );

    let linear_ns = linear_time.as_nanos() as f64 / (iters as f64 * probes_per_pass);
    let mask_ns = mask_time.as_nanos() as f64 / (iters as f64 * probes_per_pass);
    let branchless_ns =
        branchless_time.as_nanos() as f64 / (iters as f64 * probes_per_pass);

    println!("probes per pass: {}", probes_per_pass as usize);
    println!("linear impl:     {:>8.2} ns/probe  (best of {rounds})", linear_ns);
    println!("mask   impl:     {:>8.2} ns/probe  (best of {rounds})", mask_ns);
    println!("branchless impl: {:>8.2} ns/probe  (best of {rounds})", branchless_ns);
    println!("ratio linear/mask:        {:.2}x", linear_ns / mask_ns);
    println!("ratio linear/branchless:  {:.2}x", linear_ns / branchless_ns);
    println!("ratio branchless/mask:    {:.2}x", branchless_ns / mask_ns);
}