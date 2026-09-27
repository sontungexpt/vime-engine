//! Benchmarks the previous hand-written `NucleusState::check` (a single flat
//! `match vowels { [...] => ..., }` over slice patterns) against the current
//! implementation, which expands the same rule table through the
//! `nucleus_rules!` macro into `match vowels.len() { 2 => match vowels { .. },
//! 3 => match vowels { .. } }`.
//!
//! The pre-macro version is kept verbatim in `mod old` below so the two stay
//! comparable; this bench does not change or "fix" either one, it only
//! measures them. A correctness pass compares the two on every 1..=3 vowel
//! sequence before timing starts, so a behavior drift between them is
//! reported loudly here instead of silently skewing the numbers.
//!
//!   cargo bench --bench bench_nucleus_state_versions

use std::hint::black_box;
use std::time::Instant;

use vime_engine::phonology::BaseVowel;
use vime_engine::phonology::NucleusState;

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

/// Faithful copy of the pre-macro implementation (see `nucleus copy.rs`).
/// Kept local to the bench so the production module only has to carry one
/// (the current) implementation.
mod old {
    use vime_engine::phonology::{BaseVowel, NucleusState};

    pub fn check(vowels: &[BaseVowel]) -> NucleusState {
        use BaseVowel::*;
        use NucleusState::*;

        match vowels {
            [] => InComplete,

            // ─────────────────── Single vowels ───────────────────
            [A] => Valid,           // a
            [ABreve] => Valid,      // ă
            [ACircumflex] => Valid, // â

            [E] => Valid,           // e
            [ECircumflex] => Valid, // ê

            [I] => Valid, // i
            [Y] => Valid, // y

            [O] => Valid,           // o
            [OCircumflex] => Valid, // ô
            [OHorn] => Valid,       // ơ

            [U] => Valid,     // u
            [UHorn] => Valid, // ư

            // ─────────────────── a family ───────────────────
            [A, I] => Valid,           // ai
            [A, O] => Valid,           // ao
            [A, U] => Valid,           // au
            [A, Y] => Valid,           // ay
            [ACircumflex, U] => Valid, // âu
            [ACircumflex, Y] => Valid, // ây

            // ─────────────────── i / y family ───────────────────
            [I, A] => Valid,              // ia
            [I, E] => InComplete,         // ie
            [I, ECircumflex] => Valid,    // iê
            [I, E, U] => InComplete,      // ieu
            [I, ECircumflex, U] => Valid, // iêu
            [I, U] => Valid,              // iu

            [Y, E] => InComplete,         // ye
            [Y, ECircumflex] => Valid,    // yê
            [Y, E, U] => InComplete,      // yeu
            [Y, ECircumflex, U] => Valid, // yêu

            // ─────────────────── e family ───────────────────
            [E, O] => Valid,           // eo
            [E, U] => InComplete,      // eu
            [ECircumflex, U] => Valid, // êu

            // ─────────────────── o family ───────────────────
            [O, A] => Valid,           // oa
            [O, ABreve] => Valid,      // oă
            [O, A, I] => Valid,        // oai
            [O, A, O] => Valid,        // oao
            [O, A, U] => Valid,        // oau
            [O, A, Y] => Valid,        // oay
            [O, E] => Valid,           // oe
            [O, E, O] => Valid,        // oeo
            [O, I] => Valid,           // oi
            [OCircumflex, I] => Valid, // ôi
            [OHorn, I] => Valid,       // ơi
            [O, O] => InComplete,      // oo

            // ─────────────────── u + y family ───────────────────
            [U, Y] => Valid,              // uy
            [U, Y, U] => Valid,           // uyu
            [U, Y, A] => Valid,           // uya
            [U, Y, E] => InComplete,      // uye
            [U, Y, ECircumflex] => Valid, // uyê

            // ─────────────────── u + a family ───────────────────
            [U, A] => Valid,              // ua
            [U, A, O] => Valid,           // uao
            [U, ACircumflex] => Valid,    // uâ
            [U, ACircumflex, Y] => Valid, // uây

            // ─────────────────── u + o transactional family ───────────────────
            [U, O] => InComplete,      // uo
            [U, OHorn] => Valid,       // uơ
            [U, OCircumflex] => Valid, // uô

            [U, O, I] => InComplete,      // uoi
            [U, OCircumflex, I] => Valid, // uôi
            [U, OHorn, I] => InComplete,  // uơi

            [U, O, U] => InComplete,     // uou
            [U, OHorn, U] => InComplete, // uơu

            // ─────────────────── u + e family ───────────────────
            [U, E] => InComplete,      // ue
            [U, ECircumflex] => Valid, // uê
            [U, I] => Valid,           // ui

            // ─────────────────── ư + o transactional family ───────────────────
            [UHorn, O] => InComplete, // ưo
            [UHorn, OHorn] => Valid,  // ươ

            [UHorn, O, I] => InComplete, // ưoi
            [UHorn, OHorn, I] => Valid,  // ươi

            [UHorn, O, U] => InComplete, // ưou
            [UHorn, OHorn, U] => Valid,  // ươu

            // ─────────────────── ư family ───────────────────
            [UHorn, A] => Valid,  // ưa
            [UHorn, I] => Valid,  // ưi
            [U, U] => InComplete, // uu
            [UHorn, U] => Valid,  // ưu

            // ─────────────────── Invalid ───────────────────
            _ => Dead,
        }
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
fn probe_old(workload: &[Seq]) -> u32 {
    let mut acc = 0u32;
    for s in workload {
        acc = acc
            .wrapping_mul(31)
            .wrapping_add(match old::check(s.slice()) {
                NucleusState::Dead => 1,
                NucleusState::Valid => 2,
                NucleusState::InComplete => 3,
            });
    }
    black_box(acc)
}

#[inline(never)]
fn probe_new(workload: &[Seq]) -> u32 {
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

fn main() {
    let exhaustive = exhaustive();
    let realistic = realistic();

    // Symmetry check: report drift between old and new, never fix it here.
    let mut valid = 0;
    let mut incomplete = 0;
    let mut mismatches = 0;
    for s in &exhaustive {
        let v_old = old::check(s.slice());
        let v_new = NucleusState::check(s.slice());
        if v_old != v_new {
            mismatches += 1;
            eprintln!(
                "mismatch for {:?}: old={:?} new={:?}",
                s.slice(),
                v_old,
                v_new
            );
        }
        match v_new {
            NucleusState::Valid => valid += 1,
            NucleusState::InComplete => incomplete += 1,
            NucleusState::Dead => {}
        }
    }
    if mismatches > 0 {
        println!(
            "WARNING: {mismatches} sequence(s) differ between old and new \
             `NucleusState::check` (see stderr) -- benchmarking both as-is, \
             not fixing them."
        );
    }
    println!(
        "new table: {valid} valid, {incomplete} incomplete (out of {} rows)",
        exhaustive.len()
    );

    let rounds = 150;
    let iters = 30_000;

    let t_old = time(
        || {
            probe_old(&exhaustive);
        },
        rounds,
        iters,
    );
    let t_new = time(
        || {
            probe_new(&exhaustive);
        },
        rounds,
        iters,
    );

    let n_old = t_old.as_nanos() as f64 / iters as f64;
    let n_new = t_new.as_nanos() as f64 / iters as f64;

    println!("exhaustive ({} rows):", exhaustive.len());
    println!(
        "  old (flat match):   {:8.2} ns/pass   ({:6.2} ns/probe)",
        n_old,
        n_old / exhaustive.len() as f64
    );
    println!(
        "  new (macro match):   {:8.2} ns/pass   ({:6.2} ns/probe)   ({:.3}x old)",
        n_new,
        n_new / exhaustive.len() as f64,
        n_new / n_old
    );

    let r_old = time(
        || {
            probe_old(&realistic);
        },
        rounds,
        iters,
    );
    let r_new = time(
        || {
            probe_new(&realistic);
        },
        rounds,
        iters,
    );

    let m_old = r_old.as_nanos() as f64 / iters as f64;
    let m_new = r_new.as_nanos() as f64 / iters as f64;

    println!("realistic ({} rows):", realistic.len());
    println!(
        "  old (flat match):   {:8.2} ns/pass   ({:6.2} ns/probe)",
        m_old,
        m_old / realistic.len() as f64
    );
    println!(
        "  new (macro match):   {:8.2} ns/pass   ({:6.2} ns/probe)   ({:.3}x old)",
        m_new,
        m_new / realistic.len() as f64,
        m_new / m_old
    );
    println!("(best of {rounds} rounds)");
}
