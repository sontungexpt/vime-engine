//! Micro-benchmark of the crate's [`SmallVec`], the spilling buffer behind
//! raw keystrokes and dead syllables.
//!
//!   cargo bench --bench bench_small_vec

use vime_engine::util::vec::SmallVec;

mod support;
use support::{ns_per_unit, rounds, time};

/// Push-heavy burst with spill behavior.
#[inline(always)]
fn typing_burst<T: Copy, const N: usize>(
    b: &mut SmallVec<T, N>,
    a: T,
    p: T,
    q: T,
    ops: &mut usize,
) {
    let cap = N;
    if b.len() < cap {
        b.push(a);
        *ops += 1;
    }
    if b.len() < cap {
        b.push(p);
        *ops += 1;
    }
    if !b.is_empty() {
        let _ = b.pop();
        *ops += 1;
    }
    if b.len() < cap {
        b.push(q);
        *ops += 1;
    }
}

/// Caret-edit burst: mid-cluster insert, first-slot insert, front removal.
#[inline(always)]
fn edit_burst<T: Copy, const N: usize>(b: &mut SmallVec<T, N>, v: T, ops: &mut usize) {
    if b.len() < N && !b.is_empty() {
        b.insert(b.len() / 2, v);
        *ops += 1;
    }
    if b.len() < N {
        b.insert(0, v);
        *ops += 1;
    }
    if !b.is_empty() {
        let _ = b.remove(0);
        *ops += 1;
    }
}

/// Backspace burst.
#[inline(always)]
fn pop_burst<T: Copy, const N: usize>(b: &mut SmallVec<T, N>, ops: &mut usize) {
    for _ in 0..2 {
        if !b.is_empty() {
            let _ = b.pop();
            *ops += 1;
        }
    }
}

/// Spill burst: force a spill and measure the cost.
#[inline(always)]
fn spill_burst<T: Copy, const N: usize>(b: &mut SmallVec<T, N>, v: T, ops: &mut usize, filler: T) {
    // Fill to capacity
    while b.len() < N {
        b.push(filler);
    }
    // One more push triggers spill
    b.push(v);
    *ops += 1;
}

/// One pass over a population of buffers, exercising every burst.
fn run_pass<T: Copy, const N: usize>(
    states: &mut [SmallVec<T, N>],
    a: T,
    p: T,
    q: T,
    v: T,
) -> (usize, usize) {
    let mut ops = 0usize;
    let mut reads = 0usize;
    for s in states {
        typing_burst(s, a, p, q, &mut ops);
        reads = reads.wrapping_add(s.len());
        edit_burst(s, v, &mut ops);
        pop_burst(s, &mut ops);
        s.clear();
        assert!(s.is_empty(), "clear must empty the buffer");
        s.push(a);
        ops += 1;
    }
    (ops, reads)
}

fn bench_pop<T: Copy, const N: usize>(label: &str, specs: &[Vec<T>], (a, p, q, v): (T, T, T, T)) {
    let mut states: Vec<SmallVec<T, N>> = specs
        .iter()
        .map(|s| {
            let mut b = SmallVec::<T, N>::default();
            b.extend_from_slice(s);
            b
        })
        .collect();

    let rounds = rounds();
    let iters = 300_000;

    // One untimed pass, to learn the real op count: that is the normalizer.
    let (ops_per_pass, _) = run_pass(&mut states, a, p, q, v);
    assert!(
        ops_per_pass > 0,
        "the burst must perform work, or the timing is meaningless"
    );

    let t = time(
        || {
            let (ops, reads) = run_pass(&mut states, a, p, q, v);
            std::hint::black_box(ops ^ reads);
        },
        rounds,
        iters,
    );

    println!("── {label} (SmallVec<T, {N}>, {} states) ──", states.len());
    println!(
        "  {:>7.3} ns/op   ({} ops/pass, best of {rounds})",
        ns_per_unit(t, iters, ops_per_pass as f64),
        ops_per_pass
    );
}

fn bench_spill<T: Copy, const N: usize>(label: &str, specs: &[Vec<T>], a: T, v: T) {
    let mut states: Vec<SmallVec<T, N>> = specs
        .iter()
        .map(|s| {
            let mut b = SmallVec::<T, N>::default();
            b.extend_from_slice(s);
            b
        })
        .collect();

    let rounds = rounds();
    let iters = 100_000; // Fewer iterations for spill tests

    let (ops_per_pass, _) = run_pass(&mut states, a, a, a, v);

    let t = time(
        || {
            let mut ops = 0usize;
            for s in &mut states {
                spill_burst(s, v, &mut ops, a);
            }
            std::hint::black_box(ops);
        },
        rounds,
        iters,
    );

    println!("── SPILL {label} (SmallVec<T, {N}>, {} states) ──", states.len());
    println!(
        "  {:>7.3} ns/op   ({} ops/pass, best of {rounds})",
        ns_per_unit(t, iters, ops_per_pass as f64),
        ops_per_pass
    );
}

/// Fill strategies.
fn bench_fill() {
    let rounds = rounds();
    let iters = 500_000;

    for n in 1..=3usize {
        let mut acc = 0usize;
        let src: [char; 8] = ['t', 'r', 'a', 'n', 's', 'p', 'o', 'r'];

        let t_push = time(
            || {
                let src = std::hint::black_box(&src);
                let mut b = SmallVec::<char, 4>::default();
                for &c in &src[..n] {
                    b.push(c);
                }
                acc = acc.wrapping_add(b.len());
            },
            rounds,
            iters,
        );
        let t_extend = time(
            || {
                let src = std::hint::black_box(&src);
                let mut b = SmallVec::<char, 4>::default();
                b.extend(src[..n].iter().copied());
                acc = acc.wrapping_add(b.len());
            },
            rounds,
            iters,
        );
        let t_slice = time(
            || {
                let src = std::hint::black_box(&src);
                let mut b = SmallVec::<char, 4>::default();
                b.extend_from_slice(&src[..n]);
                acc = acc.wrapping_add(b.len());
            },
            rounds,
            iters,
        );

        let rows = support::ranked(vec![
            ("push loop", ns_per_unit(t_push, iters, 1.0)),
            ("extend(iter)", ns_per_unit(t_extend, iters, 1.0)),
            ("extend_from_slice", ns_per_unit(t_slice, iters, 1.0)),
        ]);

        println!("── fill SmallVec<char, 4> with {n} items (checksum {acc}) ──");
        for (name, ns) in rows {
            println!("  {name:<18} {ns:>7.3} ns/op");
        }
    }
}

fn main() {
    // Onset: char<3>, the most-edited buffer in the builder.
    let onset: &[Vec<char>] = &[
        vec!['t'],
        vec!['t', 'h'],
        vec!['n', 'g', 'h'],
        vec!['q', 'u'],
        vec!['g', 'i'],
        vec!['t', 'r'],
    ];
    bench_pop::<char, 3>("onset", onset, ('n', 'g', 'h', 'x'));

    // Test spill behavior - inline capacity 4, push beyond
    println!();
    bench_spill::<char, 4>("spill onset", &[vec!['t', 'h']], 'a', 'z');

    // Test fill with larger inline capacity
    println!();
    bench_fill();

    // Test spill with larger buffers
    println!();
    bench_spill::<char, 8>("spill raw keystrokes", &[vec!['t', 'h', 'a', 'n', 'g']], 'a', 'z');
}