//! Benchmarks `NucleusState::from_vowels` (slice match) against a packed-u16
//! const-match variant.
//!
//! The packed variant packs up to three [`BaseVowel`]s into a `u16` (length in
//! the low nibble, then one 4-bit id slot per vowel) and dispatches with a
//! `match` over the packed key. Ids come from [`BaseVowel::id`] (0..=11), so the
//! real bit-packed discriminant layout is left untouched.
//!   cargo bench --bench bench_nucleus_state

use std::hint::black_box;
use std::time::Instant;

use vime_engine::phonology::rules::NucleusState;
use vime_engine::phonology::BaseVowel;

/// One probe row: a slice view over a fixed 3-vowel buffer.
#[derive(Clone, Copy)]
struct Seq {
    len: u8,
    buf: [BaseVowel; 3],
}

impl Seq {
    fn new(v: &[BaseVowel]) -> Self {
        let mut buf = [BaseVowel::Y; 3];
        buf[..v.len()].copy_from_slice(v);
        Self {
            len: v.len() as u8,
            buf,
        }
    }

    fn slice(&self) -> &[BaseVowel] {
        &self.buf[..self.len as usize]
    }
}

/// Packs a nucleus into a `u16`; `0` for empty or over-long nuclei.
#[inline(always)]
fn pack_key(v: &[BaseVowel]) -> u16 {
    let len = v.len();
    if len == 0 || len > 3 {
        return 0;
    }

    let mut key = len as u16;
    for (i, vowel) in v.iter().enumerate() {
        key |= (vowel.id() as u16) << (4 * (i + 1));
    }
    key
}

/// Const-match dispatch over the packed key. Table generated from the same
/// Valid / InComplete families as the slice-match implementation.
fn from_packed(packed: u16) -> NucleusState {
    use NucleusState::*;
    match packed {
        0x0051 => Valid,
        0x0091 => Valid,
        0x0071 => Valid,
        0x0031 => Valid,
        0x00A1 => Valid,
        0x0021 => Valid,
        0x0001 => Valid,
        0x0041 => Valid,
        0x0081 => Valid,
        0x00B1 => Valid,
        0x0011 => Valid,
        0x0061 => Valid,
        0x0252 => Valid,
        0x0452 => Valid,
        0x0152 => Valid,
        0x0052 => Valid,
        0x0172 => Valid,
        0x0072 => Valid,
        0x0522 => Valid,
        0x0A22 => Valid,
        0x1A23 => Valid,
        0x0122 => Valid,
        0x0A02 => Valid,
        0x1A03 => Valid,
        0x0432 => Valid,
        0x01A2 => Valid,
        0x0542 => Valid,
        0x0942 => Valid,
        0x2543 => Valid,
        0x4543 => Valid,
        0x1543 => Valid,
        0x0543 => Valid,
        0x0342 => Valid,
        0x4343 => Valid,
        0x0242 => Valid,
        0x0282 => Valid,
        0x02B2 => Valid,
        0x0012 => Valid,
        0x1013 => Valid,
        0x5013 => Valid,
        0xA013 => Valid,
        0x0512 => Valid,
        0x4513 => Valid,
        0x0712 => Valid,
        0x0713 => Valid,
        0x0B12 => Valid,
        0x0812 => Valid,
        0x2813 => Valid,
        0x0A12 => Valid,
        0x0212 => Valid,
        0x0B62 => Valid,
        0x2B63 => Valid,
        0x1B63 => Valid,
        0x0562 => Valid,
        0x0262 => Valid,
        0x0162 => Valid,

        0x0322 => InComplete,
        0x1323 => InComplete,
        0x0302 => InComplete,
        0x1303 => InComplete,
        0x0132 => InComplete,
        0x0442 => InComplete,
        0x3013 => InComplete,
        0x0412 => InComplete,
        0x2413 => InComplete,
        0x2B13 => InComplete,
        0x1413 => InComplete,
        0x1B13 => InComplete,
        0x0312 => InComplete,
        0x0462 => InComplete,
        0x2463 => InComplete,
        0x1463 => InComplete,
        0x0112 => InComplete,

        _ => Dead,
    }
}

/// Sentinel nibble marking an "absent" vowel slot in the 3-nibble key.
const SENTINEL: usize = 0xF;

/// Builds `[NucleusState; 4096]` indexed by `a | (b << 4) | (c << 8)`.
///
/// Each present vowel contributes its 4-bit [`BaseVowel::id`] (0..=11); each
/// absent slot contributes `SENTINEL` (0xF). With ids 0..=11 this cannot
/// collide between different lengths (a length-2 key always has bit 4..7 set,
/// a length-1 key never does), and the max key is `0xFFF = 4095`.
fn build_lut() -> Box<[NucleusState; 4096]> {
    use BaseVowel::*;
    let vs = [Y, U, I, E, O, A, UHorn, ACircumflex, OCircumflex, ABreve, ECircumflex, OHorn];

    let mut table = Box::new([NucleusState::Dead; 4096]);
    for (i, &a) in vs.iter().enumerate() {
        table[i | (SENTINEL << 4) | (SENTINEL << 8)] = NucleusState::check(&[a]);
        for (j, &b) in vs.iter().enumerate() {
            table[i | (j << 4) | (SENTINEL << 8)] = NucleusState::check(&[a, b]);
            for (k, &c) in vs.iter().enumerate() {
                table[i | (j << 4) | (k << 8)] = NucleusState::check(&[a, b, c]);
            }
        }
    }
    table
}

/// Packs a nucleus into the 3-nibble LUT key.
#[inline(always)]
fn lut_key(s: &Seq) -> usize {
    let a = s.buf[0].id() as usize;
    let b = if s.len > 1 { s.buf[1].id() as usize } else { SENTINEL };
    let c = if s.len > 2 { s.buf[2].id() as usize } else { SENTINEL };
    a | (b << 4) | (c << 8)
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

/// Every sequence of 1..=3 vowels: 12 + 144 + 1728 = 1884 probes.
fn exhaustive() -> Vec<Seq> {
    let vs = [
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

    let mut out = Vec::new();
    for &v in &vs {
        out.push(Seq::new(&[v]));
    }
    for &a in &vs {
        for &b in &vs {
            out.push(Seq::new(&[a, b]));
        }
    }
    for &a in &vs {
        for &b in &vs {
            for &c in &vs {
                out.push(Seq::new(&[a, b, c]));
            }
        }
    }
    out
}

/// The known Valid / InComplete nuclei only: the hot path.
fn realistic() -> Vec<Seq> {
    let x: &[&[BaseVowel]] = &[
        &[BaseVowel::A],
        &[BaseVowel::ABreve],
        &[BaseVowel::A, BaseVowel::I],
        &[BaseVowel::A, BaseVowel::U],
        &[BaseVowel::I, BaseVowel::E],
        &[BaseVowel::I, BaseVowel::ECircumflex],
        &[BaseVowel::I, BaseVowel::ECircumflex, BaseVowel::U],
        &[BaseVowel::O, BaseVowel::A],
        &[BaseVowel::O, BaseVowel::A, BaseVowel::I],
        &[BaseVowel::U, BaseVowel::O],
        &[BaseVowel::U, BaseVowel::OCircumflex],
        &[BaseVowel::U, BaseVowel::OCircumflex, BaseVowel::I],
        &[BaseVowel::U, BaseVowel::Y],
        &[BaseVowel::U, BaseVowel::Y, BaseVowel::ECircumflex],
        &[BaseVowel::UHorn, BaseVowel::OHorn],
        &[BaseVowel::UHorn, BaseVowel::OHorn, BaseVowel::I],
        &[BaseVowel::U, BaseVowel::A],
        &[BaseVowel::U, BaseVowel::E],
        &[BaseVowel::E, BaseVowel::O],
        &[BaseVowel::O, BaseVowel::I],
        &[BaseVowel::O, BaseVowel::O],
        &[BaseVowel::U, BaseVowel::I],
        &[BaseVowel::UHorn, BaseVowel::U],
        &[BaseVowel::UHorn, BaseVowel::A],
        &[BaseVowel::Y, BaseVowel::ECircumflex],
        &[BaseVowel::Y, BaseVowel::E],
    ];
    x.iter().map(|s| Seq::new(s)).collect()
}

#[inline(never)]
fn probe_v1(workload: &[Seq]) -> u32 {
    let mut acc = 0u32;
    for s in workload {
        acc = acc
            .wrapping_mul(31)
            .wrapping_add(match NucleusState::check(s.slice()) {
                NucleusState::Dead => 1,
                NucleusState::Valid => 2,
                NucleusState::InComplete => 3,
            });
    }
    black_box(acc)
}

#[inline(never)]
fn probe_v2(workload: &[Seq]) -> u32 {
    let mut acc = 0u32;
    for s in workload {
        acc = acc
            .wrapping_mul(31)
            .wrapping_add(match from_packed(pack_key(s.slice())) {
                NucleusState::Dead => 1,
                NucleusState::Valid => 2,
                NucleusState::InComplete => 3,
            });
    }
    black_box(acc)
}

#[inline(never)]
fn probe_v3(workload: &[Seq], lut: &[NucleusState; 4096]) -> u32 {
    let mut acc = 0u32;
    for s in workload {
        acc = acc
            .wrapping_mul(31)
            .wrapping_add(match lut[lut_key(s)] {
                NucleusState::Dead => 1,
                NucleusState::Valid => 2,
                NucleusState::InComplete => 3,
            });
    }
    black_box(acc)
}

fn main() {
    let exhaustive = exhaustive();
    let realistic = realistic();
    let lut = build_lut();

    // Symmetry: every route must agree everywhere, and the known sets must
    // appear exactly.
    let mut valid = 0;
    let mut incomplete = 0;
    for s in &exhaustive {
        let v1 = NucleusState::check(s.slice());
        let v2 = from_packed(pack_key(s.slice()));
        let v3 = lut[lut_key(s)];
        assert_eq!(v1, v2, "packed mismatch for {:?}", s.slice());
        assert_eq!(v1, v3, "lut mismatch for {:?} (index {})", s.slice(), lut_key(s));
        match v1 {
            NucleusState::Valid => valid += 1,
            NucleusState::InComplete => incomplete += 1,
            NucleusState::Dead => {}
        }
    }
    assert_eq!(
        (valid, incomplete),
        (56, 17),
        "table drifted from the source families"
    );

    let rounds = 150;
    let iters = 30_000;

    let t1 = time(
        || {
            probe_v1(&exhaustive);
        },
        rounds,
        iters,
    );
    let t2 = time(
        || {
            probe_v2(&exhaustive);
        },
        rounds,
        iters,
    );

    let t3 = time(
        || {
            probe_v3(&exhaustive, &lut);
        },
        rounds,
        iters,
    );

    let n1 = t1.as_nanos() as f64 / iters as f64;
    let n2 = t2.as_nanos() as f64 / iters as f64;
    let n3 = t3.as_nanos() as f64 / iters as f64;

    println!("exhaustive ({} rows):", exhaustive.len());
    println!(
        "  slice match:  {:8.2} ns/pass   ({:6.2} ns/probe)",
        n1,
        n1 / exhaustive.len() as f64
    );
    println!(
        "  packed match: {:8.2} ns/pass   ({:6.2} ns/probe)   ({:.3}x slice)",
        n2,
        n2 / exhaustive.len() as f64,
        n2 / n1
    );
    println!(
        "  lut [4096]:   {:8.2} ns/pass   ({:6.2} ns/probe)   ({:.3}x slice)",
        n3,
        n3 / exhaustive.len() as f64,
        n3 / n1
    );

    let r1 = time(
        || {
            probe_v1(&realistic);
        },
        rounds,
        iters,
    );
    let r2 = time(
        || {
            probe_v2(&realistic);
        },
        rounds,
        iters,
    );
    let r3 = time(
        || {
            probe_v3(&realistic, &lut);
        },
        rounds,
        iters,
    );

    let m1 = r1.as_nanos() as f64 / iters as f64;
    let m2 = r2.as_nanos() as f64 / iters as f64;
    let m3 = r3.as_nanos() as f64 / iters as f64;

    println!("realistic ({} rows):", realistic.len());
    println!(
        "  slice match:  {:8.2} ns/pass   ({:6.2} ns/probe)",
        m1,
        m1 / realistic.len() as f64
    );
    println!(
        "  packed match: {:8.2} ns/pass   ({:6.2} ns/probe)   ({:.3}x slice)",
        m2,
        m2 / realistic.len() as f64,
        m2 / m1
    );
    println!(
        "  lut [4096]:   {:8.2} ns/pass   ({:6.2} ns/probe)   ({:.3}x slice)",
        m3,
        m3 / realistic.len() as f64,
        m3 / m1
    );
    println!("(best of {rounds} rounds)");
}
