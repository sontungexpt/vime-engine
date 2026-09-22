//! Micro-benchmark of `BuildingSyllableBuilder::insert` vs `push`.
//!
//! `insert(index == len, ...)` should be ~identical to `push` (it delegates).
//! Cursor-mid inserts are only reachable while editing, so we also measure the
//! per-position routing cost of the onset / vowel / coda branches.
//!   cargo run --release --example bench_insert

use std::hint::black_box;
use std::time::Instant;

use vime_engine::composition::BuildingSyllableBuilder;
use vime_engine::DefaultKeymap;

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

/// Append a keystroke sequence via push.
fn run_push(workload: &[&str], keymap: &DefaultKeymap<'_>) {
    for &word in workload {
        let mut b = BuildingSyllableBuilder::default();
        for ch in word.chars() {
            black_box(black_box(&mut b).push(keymap, ch));
        }
    }
}

/// Append the same workload via insert-at-end (should delegate to push).
fn run_insert_end(workload: &[&str], keymap: &DefaultKeymap<'_>) {
    for &word in workload {
        let mut b = BuildingSyllableBuilder::default();
        for ch in word.chars() {
            let idx = b.len();
            black_box(black_box(&mut b).insert(keymap, idx, ch));
        }
    }
}

/// Simulate cursor-navigation edits: after building a syllable, insert a 'w'
/// (horn) at every nucleus position, then a trailing coda 'n' at mid positions.
fn run_insert_mid(workload: &[&str], keymap: &DefaultKeymap<'_>) {
    for &word in workload {
        let mut b = BuildingSyllableBuilder::default();
        for ch in word.chars() {
            black_box(black_box(&mut b).push(keymap, ch));
        }
        let len = b.len();
        for pos in 0..=len {
            black_box(black_box(&mut b).insert(keymap, pos, 'w'));
        }
        for pos in 1..=len {
            black_box(black_box(&mut b).insert(keymap, pos, 'n'));
        }
    }
}

fn main() {
    let workload: &[&str] = &[
        "anh", "em", "o", "a", "u", "ai", "ao", "ay", "au", "ia", "iu", "ua", "uu", "ie", "cha",
        "cho", "chi", "nga", "ngh", "nhe", "khi", "pho", "qua", "que", "gio", "gia", "aw", "aa",
        "ow", "eekho", "aafamily", "som", "sinh", "hoc", "ban", "len", "how", "uow", "uo", "uoi",
        "uowng", "luowng", "nuowc", "thuowng", "nguoi", "tuoi", "cuoi", "cong", "long", "tan",
        "tam", "tap", "ach", "anhng", "vient", "hat", "bac", "khong", "toan", "vien", "nuoc",
        "vuon", "muot", "tham", "que", "thuy", "truong", "Viet", "Nam", "HA", "NOI", "DAN", "Tien",
    ];

    let keymap = DefaultKeymap::telex();
    let rounds = 200;
    let iters = 30_000;

    let push_t = time(|| run_push(workload, &keymap), rounds, iters);
    let ins_end_t = time(|| run_insert_end(workload, &keymap), rounds, iters);
    let ins_mid_t = time(|| run_insert_mid(workload, &keymap), rounds, iters);

    let push_ns = push_t.as_nanos() as f64 / iters as f64;
    let ins_end_ns = ins_end_t.as_nanos() as f64 / iters as f64;
    let ins_mid_ns = ins_mid_t.as_nanos() as f64 / iters as f64;

    println!("words per pass:      {}", workload.len());
    println!("push (append):       {:9.2} ns/pass", push_ns);
    println!(
        "insert@len (append): {:9.2} ns/pass   ({:.3}x push)",
        ins_end_ns,
        ins_end_ns / push_ns
    );
    println!(
        "insert mid (edits):  {:9.2} ns/pass   ({:.3}x push)",
        ins_mid_ns,
        ins_mid_ns / push_ns
    );
    println!("(best of {rounds} rounds)");
}
