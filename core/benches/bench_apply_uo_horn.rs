//! Micro-benchmark of `apply_uo_horn` body style.
//!
//! Current working tree (slice-let) vs the indexed form we restored:
//! - new (working tree):   `let [v0, v1, ..] = &mut vowels[..]`, then `match (*v0.value(), *v1.value())`
//! - old (restored here):  `let vowels = &vowels`, then `match (*vowels[0].value(), *vowels[1].value())`
//!
//! The shape-toggle helper is identical on both sides (real `apply_vowel_shape`
//! is shared and orthogonal to the body style). Run with:
//!   cargo bench --bench bench_apply_uo_horn

use std::hint::black_box;
use std::time::Instant;

use vime_engine::phonology::{BaseVowel, ExtendedBaseVowel, Shape};

/// Faithful shrink of the `ArrayVec<ExtendedBaseVowel, 3>` nucleus.
#[derive(Clone)]
struct Nucleus {
    vowels: [ExtendedBaseVowel; 3],
    len: usize,
}

fn v(b: BaseVowel) -> ExtendedBaseVowel {
    ExtendedBaseVowel::with_case(b, false)
}

/// Shared shape-toggle: Horn on index `i` (reverts when already Horn).
#[inline(always)]
fn apply_horn(n: &mut Nucleus, i: usize) -> bool {
    let old = n.vowels[i].get();
    if old.has_shape(Shape::Horn) {
        n.vowels[i].set(old.remove_shape());
        return true;
    }
    let Ok(new) = old.replace_shape(Shape::Horn) else {
        return false;
    };
    n.vowels[i].set(new);
    true
}

/// Working-tree form (slice-let borrow pair).
#[inline(always)]
fn slicelet_uo_horn(n: &mut Nucleus) -> bool {
    let [v0, v1, ..] = &mut n.vowels[..n.len] else {
        return false;
    };

    match (v0.get(), v1.get()) {
        // ươ -> uo (Revert)
        (BaseVowel::UHorn, BaseVowel::OHorn) => {
            v0.set(BaseVowel::U);
            v1.set(BaseVowel::O);
            true
        }
        // ưô, ưo, uo, uô -> Horn on index 1
        (BaseVowel::UHorn, BaseVowel::OCircumflex | BaseVowel::O)
        | (BaseVowel::U, BaseVowel::O | BaseVowel::OCircumflex) => apply_horn(n, 1),
        // uơ -> Horn on index 0 (thành ươ)
        (BaseVowel::U, BaseVowel::OHorn) => apply_horn(n, 0),
        _ => false,
    }
}

/// Indexed form (restored; NLL ends the shared borrow before the writes).
#[inline(always)]
fn indexed_uo_horn(n: &mut Nucleus) -> bool {
    let vowels = &n.vowels;
    match (vowels[0].get(), vowels[1].get()) {
        // ươ -> uo (Revert)
        (BaseVowel::UHorn, BaseVowel::OHorn) => {
            n.vowels[0].set(BaseVowel::U);
            n.vowels[1].set(BaseVowel::O);
            true
        }
        // ưô, ưo, uo, uô -> Horn on index 1
        (BaseVowel::UHorn, BaseVowel::OCircumflex | BaseVowel::O)
        | (BaseVowel::U, BaseVowel::O | BaseVowel::OCircumflex) => apply_horn(n, 1),
        // uơ -> Horn on index 0 (thành ươ)
        (BaseVowel::U, BaseVowel::OHorn) => apply_horn(n, 0),
        _ => false,
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
    // Reachable `u o`-family nuclei, weighted by relative push frequency.
    let specs: &[(Vec<BaseVowel>, usize)] = &[
        // len-2 uo/uơ/ươ families (the hot Horn-key targets).
        (vec![BaseVowel::U, BaseVowel::O], 8),
        (vec![BaseVowel::U, BaseVowel::OHorn], 4),
        (vec![BaseVowel::UHorn, BaseVowel::O], 4),
        (vec![BaseVowel::U, BaseVowel::OCircumflex], 3),
        (vec![BaseVowel::UHorn, BaseVowel::OCircumflex], 2),
        (vec![BaseVowel::UHorn, BaseVowel::OHorn], 2),
        // len-3 uo with a trailing vowel (body still may hit index 0/1).
        (vec![BaseVowel::U, BaseVowel::O, BaseVowel::I], 2),
        (vec![BaseVowel::U, BaseVowel::OHorn, BaseVowel::I], 1),
        // non-uo states (early `_ => Ignored`).
        (vec![BaseVowel::A, BaseVowel::E], 4),
        (vec![BaseVowel::I, BaseVowel::A], 3),
        (vec![BaseVowel::O, BaseVowel::A], 2),
    ];

    let mut states: Vec<Nucleus> = Vec::new();
    for (vows, w) in specs {
        let n = Nucleus {
            len: vows.len(),
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
            });
        }
    }

    // Behavior audit: both must agree on every state.
    for s in &states {
        let (mut a, mut b) = (s.clone(), s.clone());
        assert_eq!(slicelet_uo_horn(&mut a), indexed_uo_horn(&mut b));
        assert!(a.vowels == b.vowels, "body styles must agree");
    }

    let rounds = 300;
    let iters = 400_000;

    let slice_t = time(
        || {
            for s in &mut states {
                black_box(slicelet_uo_horn(black_box(s)));
            }
        },
        rounds,
        iters,
    );
    let index_t = time(
        || {
            for s in &mut states {
                black_box(indexed_uo_horn(black_box(s)));
            }
        },
        rounds,
        iters,
    );

    let n = states.len() as f64;
    let base = iters as f64 * n;
    println!("inputs per pass: {}", states.len());
    println!(
        "slice-let body:  {:7.3} ns/input",
        slice_t.as_nanos() as f64 / base
    );
    println!(
        "indexed body:    {:7.3} ns/input",
        index_t.as_nanos() as f64 / base
    );
    println!(
        "indexed/slicelet effect: {:.3}x",
        index_t.as_nanos() as f64 / slice_t.as_nanos() as f64
    );
}
