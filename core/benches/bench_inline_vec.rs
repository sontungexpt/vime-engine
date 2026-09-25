//! Micro-benchmark: crate `InlineVec` vs `arrayvec::ArrayVec`.
//!
//! `InlineVec` replaces `ArrayVec` as the inline buffer backing
//! `BuildingSyllable` (onset/coda `char`, nucleus `ExtendedBaseVowel`). Both
//! types are measured on identical, capacity-bounded edit bursts — push-heavy
//! typing, caret insert/remove, backspace pops, and read-out — over the same
//! state population. Any regression from the swap shows up as ns/op.
//!
//!   cargo bench --bench bench_inline_vec

use std::time::Instant;

use arrayvec::ArrayVec;
use vime_engine::phonology::{BaseVowel, ExtendedBaseVowel};
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
        read_burst(s, acc);
        edit_burst(s, cap, v, &mut ops);
        pop_burst(s, &mut ops);
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
            let mut acc = 0usize;
            run_pass(&mut av, N, &mut acc, a, p, q, v);
            std::hint::black_box(acc);
        },
        rounds,
        iters,
    );
    let t_fa = time(
        || {
            let mut acc = 0usize;
            run_pass(&mut fa, N, &mut acc, a, p, q, v);
            std::hint::black_box(acc);
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
    println!("  checksum: ArrayVec {} / InlineVec {}", acc_av, acc_fa);
}

fn v(b: BaseVowel) -> ExtendedBaseVowel {
    ExtendedBaseVowel::with_case(b, false)
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

    // ─────────────────────── Nucleus · ExtendedBaseVowel<3> ───────────────────────
    let nucleus_specs: &[Vec<ExtendedBaseVowel>] = &[
        vec![v(BaseVowel::A)],
        vec![v(BaseVowel::A), v(BaseVowel::E)],
        vec![v(BaseVowel::U), v(BaseVowel::O)],
        vec![v(BaseVowel::U), v(BaseVowel::OHorn)],
        vec![v(BaseVowel::O), v(BaseVowel::A), v(BaseVowel::I)],
        vec![v(BaseVowel::U), v(BaseVowel::O), v(BaseVowel::Y)],
        vec![v(BaseVowel::I), v(BaseVowel::E), v(BaseVowel::U)],
    ];
    bench_pop::<ExtendedBaseVowel, 3>(
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

    // ─────────────────────── Fill strategy shootout (char<3>) ───────────────────────
    //
    // Fill a `char<3>` from a slice prefix of length `n` three public ways.
    // The input is `black_box`-pinned (no constant folding) and lengths are
    // accumulated into `acc`, printed afterwards, so every store is observed.
    let rounds = 1000;
    let iters = 500_000;

    for n in 1..=3usize {
        let mut acc = 0usize;
        let src: [char; 3] = ['t', 'r', 'a'];

        let t_push = time(
            || {
                let src = std::hint::black_box(&src);
                let mut b = InlineVec::<char, 3>::default();
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
                let mut b = InlineVec::<char, 3>::default();
                b.extend(src[..n].iter().copied());
                acc = acc.wrapping_add(b.len());
            },
            rounds,
            iters,
        );
        let t_slice = time(
            || {
                let src = std::hint::black_box(&src);
                let mut b = InlineVec::<char, 3>::default();
                b.extend_from_slice(&src[..n]);
                acc = acc.wrapping_add(b.len());
            },
            rounds,
            iters,
        );

        let ns = |t: std::time::Duration| t.as_nanos() as f64 / iters as f64;
        println!("── fill char<3> with {n} items (checksum {acc}) ──");
        println!("  push loop:          {:7.3} ns/op", ns(t_push));
        println!("  extend(iter):       {:7.3} ns/op", ns(t_extend));
        println!("  extend_from_slice:  {:7.3} ns/op", ns(t_slice));
    }

    bench_mechanism();
}

/// A raw byte column mirroring `InlineVec`'s backing store, with no borrow
/// or adapter indirection. Used to probe the *mechanism* ceiling of each
/// write strategy in isolation.
#[derive(Clone, Copy)]
struct Column {
    buf: [u8; 8],
    len: usize,
}

impl Default for Column {
    fn default() -> Self {
        Column {
            buf: [0; 8],
            len: 0,
        }
    }
}

impl Column {
    /// Last written byte, folded into the checksum so stores stay observable.
    fn check(&self) -> usize {
        self.buf[self.len.wrapping_sub(1) & 7] as usize
    }
}

/// Mechanism ceiling: same three write strategies against a raw column, so the
/// per-write cost is measured with zero trait/iterator overhead in the way.
fn bench_mechanism() {
    let rounds = 1000;
    let iters = 500_000;

    for n in 1..=3usize {
        let mut acc = 0usize;
        let src: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

        let t_item = time(
            || {
                let src = std::hint::black_box(&src);
                let mut col = Column::default();
                for &c in &src[..n] {
                    col.buf[col.len] = c;
                    col.len += 1;
                }
                acc = acc.wrapping_add(col.check());
            },
            rounds,
            iters,
        );
        let t_indexed = time(
            || {
                let src = std::hint::black_box(&src);
                let mut col = Column::default();
                let start = col.len;
                for i in 0..n {
                    col.buf[start + i] = src[i];
                }
                col.len = start + n;
                acc = acc.wrapping_add(col.check());
            },
            rounds,
            iters,
        );
        let t_memcpy = time(
            || {
                let src = std::hint::black_box(&src);
                let mut col = Column::default();
                col.buf[col.len..col.len + n].copy_from_slice(&src[..n]);
                col.len += n;
                acc = acc.wrapping_add(col.check());
            },
            rounds,
            iters,
        );

        let ns = |t: std::time::Duration| t.as_nanos() as f64 / iters as f64;
        println!("── mechanism, raw column {n}/8 (checksum {acc}) ──");
        println!("  per-item store:     {:7.3} ns/op", ns(t_item));
        println!("  indexed store:      {:7.3} ns/op", ns(t_indexed));
        println!("  memcpy (copy_slice):{:7.3} ns/op", ns(t_memcpy));
    }
}