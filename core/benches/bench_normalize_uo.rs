//! Micro-benchmark of `normalize_uo_horn` (working tree) vs the old
//! `finalize_uo_shape` (committed `068edb6`).
//!
//! Both guard and body differ:
//! - old : `len < 2 -> return`, then indexed writes
//! - new : `coda empty && len < 3 -> return`, then `[v0, v1, ..]` slice-let writes
//!
//! So we also run a hybrid (old guard, new body) to separate the two effects.
//! Run with:
//!   cargo bench --bench bench_normalize_uo

use std::hint::black_box;
use std::time::Instant;

use vime_engine::phonology::{BaseVowel, ExtendedBaseVowel};

/// Faithful shrink of the `ArrayVec<ExtendedBaseVowel, 3>` nucleus + coda flag.
#[derive(Clone)]
struct Nucleus {
    vowels: [ExtendedBaseVowel; 3],
    len: usize,
    coda_empty: bool,
}

fn v(b: BaseVowel) -> ExtendedBaseVowel {
    ExtendedBaseVowel::with_case(b, false)
}

/// Old form, transcribed verbatim from `068edb6` `finalize_uo_shape`.
#[inline(always)]
fn old_normalize(n: &mut Nucleus) {
    let vowels = &mut n.vowels;
    if n.len < 2 {
        return;
    }

    match (vowels[0].get(), vowels[1].get()) {
        (BaseVowel::U, BaseVowel::OHorn) => {
            vowels[0].set(BaseVowel::UHorn);
        }
        (BaseVowel::UHorn, BaseVowel::O) => {
            vowels[1].set(BaseVowel::OHorn);
        }
        _ => {}
    }
}

/// New form, transcribed verbatim from the working tree `normalize_uo_horn`.
#[inline(always)]
fn new_normalize(n: &mut Nucleus) {
    if n.coda_empty && n.len < 3 {
        return;
    }

    if let [v0, v1, ..] = &mut n.vowels[..n.len] {
        match (v0.get(), v1.get()) {
            (BaseVowel::U, BaseVowel::OHorn) => v0.set(BaseVowel::UHorn),
            (BaseVowel::UHorn, BaseVowel::O) => v1.set(BaseVowel::OHorn),
            _ => {}
        }
    }
}

/// Hybrid: old guard, new body (isolates the body-style cost).
#[inline(always)]
fn hybrid_normalize(n: &mut Nucleus) {
    if n.len < 2 {
        return;
    }

    if let [v0, v1, ..] = &mut n.vowels[..n.len] {
        match (v0.get(), v1.get()) {
            (BaseVowel::U, BaseVowel::OHorn) => v0.set(BaseVowel::UHorn),
            (BaseVowel::UHorn, BaseVowel::O) => v1.set(BaseVowel::OHorn),
            _ => {}
        }
    }
}

fn time(f: impl FnMut(), rounds: usize, iters: usize) -> std::time::Duration {
    let mut f = f;
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
    // (vowels, coda_present, weight) — mirroring how the call sites reach it:
    // after a vowel push (usually len 2, no coda) and after a coda lands.
    let specs: &[(Vec<BaseVowel>, bool, usize)] = &[
        // len-1: instant early exit for both.
        (vec![BaseVowel::A], false, 8),
        (vec![BaseVowel::U], false, 6),
        (vec![BaseVowel::O], false, 6),
        // len-2, no coda: OLD folds, NEW early-returns (behavior difference!).
        (vec![BaseVowel::U, BaseVowel::OHorn], false, 2),
        (vec![BaseVowel::UHorn, BaseVowel::O], false, 2),
        // len-2, no coda, no-op body for both-when-reached.
        (vec![BaseVowel::U, BaseVowel::O], false, 5),
        (vec![BaseVowel::I, BaseVowel::E], false, 3),
        // len-2 with coda: both reach the body.
        (vec![BaseVowel::U, BaseVowel::OHorn], true, 4),
        (vec![BaseVowel::UHorn, BaseVowel::O], true, 4),
        (vec![BaseVowel::U, BaseVowel::O], true, 3),
        // len-3: both reach the body.
        (vec![BaseVowel::U, BaseVowel::OHorn, BaseVowel::I], false, 2),
        (vec![BaseVowel::UHorn, BaseVowel::O, BaseVowel::E], false, 2),
        (vec![BaseVowel::U, BaseVowel::O, BaseVowel::I], false, 3),
        (vec![BaseVowel::U, BaseVowel::OHorn, BaseVowel::I], true, 2),
    ];

    let mut states: Vec<Nucleus> = Vec::new();
    for (vows, coda, w) in specs {
        let n = Nucleus {
            len: vows.len(),
            coda_empty: !coda,
            vowels: [
                v(vows[0]),
                vows.get(1).map_or_else(|| v(BaseVowel::A), |b| v(*b)),
                vows.get(2).map_or_else(|| v(BaseVowel::A), |b| v(*b)),
            ],
        };
        for _ in 0..*w {
            states.push(Nucleus {
                vowels: n.vowels,
                len: n.len,
                coda_empty: n.coda_empty,
            });
        }
    }

    // Behavior audit: where do old and new disagree (the changed guard)?
    let mut diffs = 0;
    for s in &states {
        let (mut a, mut b) = (s.clone(), s.clone());
        old_normalize(&mut a);
        new_normalize(&mut b);
        if a.vowels != b.vowels {
            diffs += 1;
        }
    }
    println!("inputs per pass: {}", states.len());
    println!("states where old != new (guard change): {diffs}");
    assert_eq!(
        diffs, 4,
        "expected exactly the len-2/no-coda uo-horn states"
    );

    // Body-style equivalence: same `len < 2` guard, old indexed writes vs the
    // slice-let form. Outputs must agree on every state.
    for s in &states {
        let (mut a, mut b) = (s.clone(), s.clone());
        old_normalize(&mut a);
        hybrid_normalize(&mut b);
        assert!(
            a.vowels == b.vowels,
            "indexed and slice-let bodies must agree"
        );
    }

    let rounds = 300;
    let iters = 400_000;

    let old_t = time(
        || {
            for s in &mut states {
                old_normalize(black_box(s));
            }
        },
        rounds,
        iters,
    );
    let new_t = time(
        || {
            for s in &mut states {
                new_normalize(black_box(s));
            }
        },
        rounds,
        iters,
    );
    let hyb_t = time(
        || {
            for s in &mut states {
                hybrid_normalize(black_box(s));
            }
        },
        rounds,
        iters,
    );

    let n = states.len() as f64;
    let base = iters as f64 * n;
    println!(
        "old (guard+body 068edb6): {:7.3} ns/input",
        old_t.as_nanos() as f64 / base
    );
    println!(
        "new (all in one fn)   : {:7.3} ns/input",
        new_t.as_nanos() as f64 / base
    );
    println!(
        "hyb (old guard new bd): {:7.3} ns/input",
        hyb_t.as_nanos() as f64 / base
    );
    println!(
        "body-style effect (hyb/old): {:.3}x",
        hyb_t.as_nanos() as f64 / old_t.as_nanos() as f64
    );
    println!(
        "guard effect (new/hyb):      {:.3}x",
        new_t.as_nanos() as f64 / hyb_t.as_nanos() as f64
    );
}
