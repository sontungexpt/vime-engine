//! Micro-benchmark: crate `InlineVec` vs `arrayvec::ArrayVec`.
//!
//! `InlineVec` replaces `ArrayVec` as the inline buffer backing
//! `BuildingSyllable` (onset/coda `char`, nucleus `CasedBaseVowel`). Both
//! types are measured on identical, capacity-bounded edit bursts — push-heavy
//! typing, caret insert/remove, backspace pops, and read-out — over the same
//! state population. Any regression from the swap shows up as ns/op.
//!
//!   cargo bench --bench bench_inline_vec

use std::time::Instant;

use arrayvec::ArrayVec;
use vime_engine::phonology::{BaseVowel, CasedBaseVowel};
use vime_engine::util::InlineVec;

/// The `BuildingSyllable` buffer surface: bounded push/insert/pop/remove,
/// cleared between words, read back as a slice.
trait Buf<T: Copy> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn fill(state: &[T]) -> Self;
    fn push(&mut self, v: T);
    fn pop(&mut self) -> Option<T>;
    fn insert(&mut self, i: usize, v: T);
    fn remove(&mut self, i: usize) -> T;
    fn clear(&mut self);
    fn count_iter(&self) -> usize;
}

macro_rules! impl_buf {
    ($t:ty) => {
        impl<T: Copy, const N: usize> Buf<T> for $t {
            #[inline(always)]
            fn len(&self) -> usize {
                self.len()
            }

            #[inline(always)]
            fn is_empty(&self) -> bool {
                self.is_empty()
            }

            fn fill(state: &[T]) -> Self {
                let mut b = Self::default();
                for &v in state {
                    b.push(v);
                }
                b
            }

            #[inline(always)]
            fn push(&mut self, v: T) {
                self.push(v)
            }

            #[inline(always)]
            fn pop(&mut self) -> Option<T> {
                self.pop()
            }

            #[inline(always)]
            fn insert(&mut self, i: usize, v: T) {
                self.insert(i, v)
            }

            #[inline(always)]
            fn remove(&mut self, i: usize) -> T {
                self.remove(i)
            }

            #[inline(always)]
            fn clear(&mut self) {
                self.clear()
            }

            #[inline(always)]
            fn count_iter(&self) -> usize {
                self.iter().count()
            }
        }
    };
}

impl_buf!(ArrayVec<T, N>);
impl_buf!(InlineVec<T, N>);

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

/// Push-heavy burst: typing a syllable with the real capacity guard, then
/// one candidate fix (pop + push).
#[inline(always)]
fn typing_burst<T: Copy, B: Buf<T>>(b: &mut B, cap: usize, a: T, p: T, q: T, ops: &mut usize) {
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
fn edit_burst<T: Copy, B: Buf<T>>(b: &mut B, cap: usize, v: T, ops: &mut usize) {
    if b.len() < cap && !b.is_empty() {
        b.insert(b.len() / 2, v);
        *ops += 1;
    }
    if b.len() < cap {
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
fn pop_burst<T: Copy, B: Buf<T>>(b: &mut B, ops: &mut usize) {
    if !b.is_empty() {
        let _ = b.pop();
        *ops += 1;
    }
    if !b.is_empty() {
        let _ = b.pop();
        *ops += 1;
    }
}

/// Read-out burst (the `to_chars` render path walks the live slice).
#[inline(always)]
fn read_burst<T: Copy, B: Buf<T>>(b: &mut B, acc: &mut usize) {
    let n = b.count_iter();
    *acc = acc.wrapping_add(n);
}

#[inline(always)]
fn run_pass<T: Copy, B: Buf<T>>(
    states: &mut [B],
    cap: usize,
    acc: &mut usize,
    a: T,
    p: T,
    q: T,
    v: T,
) -> usize {
    let mut ops = 0usize;
    for s in states {
        typing_burst(s, cap, a, p, q, &mut ops);
        edit_burst(s, cap, v, &mut ops);
        pop_burst(s, &mut ops);
        read_burst(s, acc);
        s.clear();
        if !s.is_empty() {
            ops += 1;
        }
        s.push(a);
        ops += 1;
    }
    ops
}

/// Benchmarks one population (`N` capacity, `T` element) on both buffer types.
fn bench_pop<T: Copy, const N: usize>(
    label: &str,
    specs: &[Vec<T>],
    (a, p, q, v): (T, T, T, T),
) where
    ArrayVec<T, N>: Buf<T>,
    InlineVec<T, N>: Buf<T>,
{
    let mut av: Vec<ArrayVec<T, N>> = specs.iter().map(|s| ArrayVec::fill(s)).collect();
    let mut fa: Vec<InlineVec<T, N>> = specs.iter().map(|s| InlineVec::fill(s)).collect();

    let mut acc_av = 0usize;
    let mut acc_fa = 0usize;

    let rounds = 2000;
    let iters = 300_000;

    let ops_per_pass = run_pass(&mut av, N, &mut acc_av, a, p, q, v);
    assert_eq!(
        ops_per_pass,
        run_pass(&mut fa, N, &mut acc_fa, a, p, q, v),
        "both sides must execute the same edit burst"
    );
    assert_eq!(acc_av, acc_fa, "both sides must produce the same reads");

    let t_av = time(
        || {
            run_pass(&mut av, N, &mut acc_av, a, p, q, v);
        },
        rounds,
        iters,
    );
    let t_fa = time(
        || {
            run_pass(&mut fa, N, &mut acc_fa, a, p, q, v);
        },
        rounds,
        iters,
    );

    let base = iters as f64 * ops_per_pass as f64;
    let av_ns = t_av.as_nanos() as f64 / base;
    let fa_ns = t_fa.as_nanos() as f64 / base;

    println!("── {label} (capacity {N}, states {}) ──", specs.len());
    println!("  ArrayVec:   {:7.3} ns/op", av_ns);
    println!("  InlineVec: {:7.3} ns/op", fa_ns);
    println!("  InlineVec/ArrayVec: {:.3}x", fa_ns / av_ns);
}

fn v(b: BaseVowel) -> CasedBaseVowel {
    CasedBaseVowel::new(b, false)
}

fn main() {
    // ─────────────────────── Onset · char<3> ───────────────────────
    let onset_specs: &[Vec<char>] = &[
        vec!['t'],
        vec!['t', 'h'],
        vec!['n', 'g', 'h'],
        vec!['q', 'u'],
        vec!['g', 'i'],
        vec!['t', 'r'],
    ];
    println!("Onset/coda buffers: char, capacity {}.", "3/2");

    bench_pop::<char, 3>("onset", onset_specs, ('n', 'g', 'h', 'x'));

    // ─────────────────────── Nucleus · CasedBaseVowel<3> ───────────────────────
    let nucleus_specs: &[Vec<CasedBaseVowel>] = &[
        vec![v(BaseVowel::A)],
        vec![v(BaseVowel::A), v(BaseVowel::E)],
        vec![v(BaseVowel::U), v(BaseVowel::O)],
        vec![v(BaseVowel::U), v(BaseVowel::OHorn)],
        vec![v(BaseVowel::O), v(BaseVowel::A), v(BaseVowel::I)],
        vec![v(BaseVowel::U), v(BaseVowel::O), v(BaseVowel::Y)],
        vec![v(BaseVowel::I), v(BaseVowel::E), v(BaseVowel::U)],
    ];
    bench_pop::<CasedBaseVowel, 3>(
        "nucleus",
        nucleus_specs,
        (
            v(BaseVowel::A),
            v(BaseVowel::E),
            v(BaseVowel::U),
            v(BaseVowel::O),
        ),
    );

    // ─────────────────────── Coda · char<2> ───────────────────────
    let coda_specs: &[Vec<char>] = &[vec!['n'], vec!['t'], vec!['n', 'g'], vec!['n', 'h']];
    bench_pop::<char, 2>("coda", coda_specs, ('n', 'h', 'c', 'g'));
}