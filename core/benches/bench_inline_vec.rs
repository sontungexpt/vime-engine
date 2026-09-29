//! Micro-benchmark of the crate's own [`InlineVec`], the buffer behind nuclei,
//! onsets and codas.
//!
//! `InlineVec` replaced `arrayvec::ArrayVec` as that backing store. The A/B
//! between the two settled that question and was removed along with the
//! `arrayvec` dev-dependency; what remains is the thing worth watching over
//! time, which is the container's own cost on the edit bursts the syllable
//! builder actually issues: push-heavy typing, caret insert/remove, backspace
//! pops, and the read-out that render walks.
//!
//!   cargo bench --bench bench_inline_vec
//!
//! A second section measures the three public ways to fill a buffer from a
//! slice, plus the mechanism ceiling behind them, so a future `push`-loop
//! change can be judged against the best the hardware offers.

use vime_engine::phonology::{BaseVowel, Tone, Vowel};
use vime_engine::util::InlineVec;

mod support;
use support::{ns_per_unit, rounds, time};

/// Push-heavy burst: typing a syllable under the real capacity guard, then one
/// candidate fix (pop + push).
#[inline(always)]
fn typing_burst<T: Copy, const N: usize>(
    b: &mut InlineVec<T, N>,
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
fn edit_burst<T: Copy, const N: usize>(b: &mut InlineVec<T, N>, v: T, ops: &mut usize) {
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
fn pop_burst<T: Copy, const N: usize>(b: &mut InlineVec<T, N>, ops: &mut usize) {
    for _ in 0..2 {
        if !b.is_empty() {
            let _ = b.pop();
            *ops += 1;
        }
    }
}

/// One pass over a population of buffers, exercising every burst.
fn run_pass<T: Copy, const N: usize>(
    states: &mut [InlineVec<T, N>],
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

/// Benchmarks one population (`N` capacity, `T` element).
fn bench_pop<T: Copy, const N: usize>(label: &str, specs: &[Vec<T>], (a, p, q, v): (T, T, T, T)) {
    let mut states: Vec<InlineVec<T, N>> = specs
        .iter()
        .map(|s| {
            let mut b = InlineVec::<T, N>::default();
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

    println!("── {label} (InlineVec<T, {N}>, {} states) ──", states.len());
    println!(
        "  {:>7.3} ns/op   ({} ops/pass, best of {rounds})",
        ns_per_unit(t, iters, ops_per_pass as f64),
        ops_per_pass
    );
}

fn v(b: BaseVowel) -> Vowel {
    Vowel::lower(b, Tone::Flat)
}

/// Fill strategies, plus a raw-column ceiling to compare them against.
fn bench_fill() {
    let rounds = rounds();
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

        let rows = support::ranked(vec![
            ("push loop", ns_per_unit(t_push, iters, 1.0)),
            ("extend(iter)", ns_per_unit(t_extend, iters, 1.0)),
            ("extend_from_slice", ns_per_unit(t_slice, iters, 1.0)),
        ]);

        println!("── fill InlineVec<char, 3> with {n} items (checksum {acc}) ──");
        for (name, ns) in rows {
            println!("  {name:<18} {ns:>7.3} ns/op");
        }
    }
}

/// A raw byte column with no borrow or adapter indirection, to probe the
/// mechanism ceiling of each write strategy in isolation.
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

fn bench_mechanism() {
    let rounds = rounds();
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

        let rows = support::ranked(vec![
            ("per-item store", ns_per_unit(t_item, iters, 1.0)),
            ("indexed store", ns_per_unit(t_indexed, iters, 1.0)),
            ("memcpy (copy_slice)", ns_per_unit(t_memcpy, iters, 1.0)),
        ]);

        println!("── mechanism, raw column {n}/8 (checksum {acc}) ──");
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

    // Nucleus: Vowel<3>, the largest element in the crate.
    let nucleus: &[Vec<Vowel>] = &[
        vec![v(BaseVowel::A)],
        vec![v(BaseVowel::A), v(BaseVowel::E)],
        vec![v(BaseVowel::U), v(BaseVowel::O)],
        vec![v(BaseVowel::U), v(BaseVowel::OHorn)],
        vec![v(BaseVowel::O), v(BaseVowel::A), v(BaseVowel::I)],
        vec![v(BaseVowel::U), v(BaseVowel::O), v(BaseVowel::Y)],
        vec![v(BaseVowel::I), v(BaseVowel::E), v(BaseVowel::U)],
    ];
    bench_pop::<Vowel, 3>(
        "nucleus",
        nucleus,
        (
            v(BaseVowel::A),
            v(BaseVowel::E),
            v(BaseVowel::U),
            v(BaseVowel::O),
        ),
    );

    // Coda: char<2>, the smallest capacity.
    let coda: &[Vec<char>] = &[vec!['n'], vec!['t'], vec!['n', 'g'], vec!['n', 'h']];
    bench_pop::<char, 2>("coda", coda, ('n', 'h', 'c', 'g'));

    println!();
    bench_fill();
    println!();
    bench_mechanism();
}
