//! Benchmarks the two forms of `decode_vowel`'s ASCII fast path.
//!
//! Variant A is the current 12-arm per-case `match` (one arm per vowel, both
//! cases). Variant B is the bit-trick form: uppercase detected from ASCII's
//! 0x20 case bit, lowercase derived via `| 0x20`, then a 6-arm match on the
//! lowercased letter. Both must agree with the crate's `decode_vowel` over the
//! whole ASCII range; a per-item folding checksum keeps results observable.
//!   cargo bench --bench bench_decode_ascii

use std::hint::black_box;
use std::time::Instant;

use vime_engine::phonology::{decode_vowel, BaseVowel, BaseVowel::*, ExtendedBaseVowel, Tone, Tone::*};

/// Current fast path (as in `decode_vowel`).
#[inline(always)]
fn ascii_match(c: char) -> Option<(ExtendedBaseVowel, Tone)> {
    match c {
        'a' => Some((ExtendedBaseVowel::lower(A), Flat)),
        'A' => Some((ExtendedBaseVowel::upper(A), Flat)),
        'o' => Some((ExtendedBaseVowel::lower(O), Flat)),
        'O' => Some((ExtendedBaseVowel::upper(O), Flat)),
        'e' => Some((ExtendedBaseVowel::lower(E), Flat)),
        'E' => Some((ExtendedBaseVowel::upper(E), Flat)),
        'i' => Some((ExtendedBaseVowel::lower(I), Flat)),
        'I' => Some((ExtendedBaseVowel::upper(I), Flat)),
        'u' => Some((ExtendedBaseVowel::lower(U), Flat)),
        'U' => Some((ExtendedBaseVowel::upper(U), Flat)),
        'y' => Some((ExtendedBaseVowel::lower(Y), Flat)),
        'Y' => Some((ExtendedBaseVowel::upper(Y), Flat)),
        _ => None,
    }
}

/// Bit-trick candidate: check the 0x20 case bit, force lowercase, match 6.
#[inline(always)]
fn ascii_bittrick(code: u32) -> Option<(ExtendedBaseVowel, Tone)> {
    let base = match (code | 0x20) as u8 as char {
        'a' => A,
        'o' => O,
        'e' => E,
        'i' => I,
        'u' => U,
        'y' => Y,
        _ => return None,
    };
    Some((ExtendedBaseVowel::with_case(base, (code & 0x20) == 0), Flat))
}

/// ASCII→priority-ID table: index by the raw `code & 0x7F` (both cases land in
/// their own slots); `0xFF` marks a non-vowel ASCII byte.
const ASCII_LUT: [u8; 128] = {
    let mut t = [0xFFu8; 128];
    let mut i = 0;
    while i < 128 {
        t[i] = ascii_id_of(i as u32);
        i += 1;
    }
    t
};

/// Priority ID (`0..=11`) of the plain vowel for the ASCII byte, else `0xFF`.
#[inline(always)]
const fn ascii_id_of(code: u32) -> u8 {
    match (code | 0x20) & 0x7F {
        0x61 => 5, // a/A
        0x65 => 3, // e/E
        0x69 => 2, // i/I
        0x6F => 4, // o/O
        0x75 => 1, // u/U
        0x79 => 0, // y/Y
        _ => 0xFF,
    }
}

/// LUT candidate: one masked load, one sentinel test, no match tree.
#[inline(always)]
fn ascii_lut(code: u32) -> Option<(ExtendedBaseVowel, Tone)> {
    let id = ASCII_LUT[(code & 0x7F) as usize];
    if id == 0xFF {
        return None;
    }
    // SAFETY: `id` came from the LUT, which only ever stores IDs `< 12`.
    let base = unsafe { BaseVowel::from_id_unchecked(id as usize) };
Some((ExtendedBaseVowel::with_case(base, (code & 0x20) == 0), Flat))
}

/// Folds a decode result into the checksum.
#[inline(always)]
fn fold(acc: u64, r: Option<(ExtendedBaseVowel, Tone)>) -> u64 {
    match r {
        Some((cased, tone)) => acc
            .wrapping_mul(31)
            .wrapping_add(cased.get() as u64 * 7 + tone as u64 + 1),
        None => acc ^ 0x9E37_79B9_7F4A_7C15,
    }
}

#[inline(never)]
fn probe(rx: impl Fn(char) -> u64, chars: &[char]) -> u64 {
    let mut acc = 0u64;
    for &c in chars {
        acc = acc.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(rx(c));
    }
    black_box(acc)
}

#[inline(always)]
fn use_match(c: char) -> u64 {
    fold(0, ascii_match(c))
}

#[inline(always)]
fn use_bittrick(c: char) -> u64 {
    fold(0, ascii_bittrick(c as u32))
}

#[inline(always)]
fn use_lut(c: char) -> u64 {
    fold(0, ascii_lut(c as u32))
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
    // Faithfulness: every ASCII char decodes identically in all variants and
    // in the crate's `decode_vowel`.
    for code in 0x00..=0x7Fu32 {
        let c = char::from_u32(code).unwrap();
        assert_eq!(
            ascii_match(c),
            ascii_bittrick(code),
            "bittrick mismatch at 0x{code:02x}"
        );
        assert_eq!(
            decode_vowel(c),
            ascii_lut(code),
            "lut mismatch at 0x{code:02x}"
        );
    }

    // Works in ASCII only: the bit-trick relies on the 0x20 case bit and on
    // `| 0x20` not escaping u8, so it is only meaningful under the
    // `0x00..=0x7F` branch that gates it in `decode_vowel`.

    // Mimic `bench_is_ascii`: three workloads over a shared letters pool.
    let letters: Vec<char> = "AaOoEeIiUuYy".chars().collect();
    let nonvowels: Vec<char> = "bcdfghjklmnpqrstvwxz 0123456789-_.".chars().collect();

    // 1: realistic typing (mostly vowels). 2: prose/url (mostly non-vowels).
    // 3: uniform printable ASCII.
    let mix = |p_vowel: usize, rows: usize| -> Vec<char> {
        let mut v = Vec::new();
        for i in 0..rows {
            let source = if i % 100 < p_vowel {
                &letters
            } else {
                &nonvowels
            };
            v.push(source[i % source.len()]);
        }
        v
    };

    let workloads: [(&str, Vec<char>); 3] = [
        ("typing (70% vowels)", mix(70, 1000)),
        ("prose/url (20% vowels)", mix(20, 1000)),
        ("uniform printable", {
            let mut v = Vec::new();
            for _ in 0..40 {
                v.extend((0x20..=0x7Eu32).map(|code| char::from_u32(code).unwrap()));
            }
            v
        }),
    ];

    let rounds = 200;
    let iters = 30_000;

    for (name, w) in workloads {
        let t1 = time(|| probe(use_match, &w), rounds, iters);
        let t2 = time(|| probe(use_bittrick, &w), rounds, iters);
        let t3 = time(|| probe(use_lut, &w), rounds, iters);
        let n1 = t1.as_nanos() as f64 / iters as f64;
        let n2 = t2.as_nanos() as f64 / iters as f64;
        let n3 = t3.as_nanos() as f64 / iters as f64;
        println!(
            "{name:<18} ({} chars): match {:6.2} | bittrick {:6.2} | LUT {:6.2} ns/pass  (bit/match {:.3}x, lut/match {:.3}x)",
            w.len(),
            n1,
            n2,
            n3,
            n2 / n1,
            n3 / n1
        );
    }
    println!("(best of {rounds} rounds; checksums agree by construction; lower is better)");
}
