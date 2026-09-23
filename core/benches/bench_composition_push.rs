//! Micro-benchmark of `Composition` append (the public string build path).
//!
//! `Composition::insert` appends at the caret; transforms keep the caret on the
//! transformed character, so built-in Vietnamese words type naturally.
//!   cargo bench --bench bench_composition_push
//!
//! Measures wall time per pass, per input sequence and per push over a
//! deterministic telex workload covering onsets (single/cluster/`qu`), plain
//! and precomposed vowels, the `uo`/`ươ` family, shape/tone transforms, codas
//! and uppercase input.

use std::hint::black_box;
use std::time::Instant;

use vime_engine::composition::Composition;
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

/// Appends every keystroke of every word through a fresh `Composition`, returning
/// the number of pushes performed (to normalize times).
fn run_workload(workload: &[&str], keymap: &DefaultKeymap) -> usize {
    let mut pushes = 0;
    for &word in workload {
        let mut composition = Composition::new(*keymap);
        for ch in word.chars() {
            black_box(composition.insert(ch));
            pushes += 1;
        }
    }
    black_box(pushes)
}

fn main() {
    // Vietnamese words as raw telex keystrokes.
    let workload: &[&str] = &[
        // single onsets + plain vowels
        "anh", "em", "o", "a", "u", "ai", "ao", "ay", "au", "ia", "iu", "ua", "uu", "ie",
        // consonant clusters
        "cha", "cho", "chi", "nga", "ngh", "nhe", "khi", "pho", "qua", "que", "gio", "gia",
        // shapes and tones
        "aw", "aa", "ow", "eekho", "aafamily", "som", "sinh", "hoc", "ban", "len", "how",
        // uo / uo family with and without the horn key
        "uow", "uo", "uoi", "uowng", "luowng", "nuowc", "thuowng", "nguoi", "tuoi", "cuoi",
        // codas
        "cong", "long", "tan", "tam", "tap", "anh", "ach", "anhng", "vient", "hat", "bac",
        // full words with tones
        "khong", "toan", "vien", "nuoc", "vuon", "muot", "tham", "que", "thuy", "truong",
        // uppercase
        "Viet", "Nam", "HA", "NOI", "DAN", "Tien",
    ];

    let keymap = DefaultKeymap::telex();
    let pushes_per_pass = run_workload(workload, &keymap) as f64;

    println!("keystroke sequences per pass: {}", workload.len());
    println!("pushes per pass:              {}", pushes_per_pass as usize);

    let rounds = 200;
    let iters = 40_000;

    let best = time(
        || {
            run_workload(workload, &keymap);
        },
        rounds,
        iters,
    );

    let ns = best.as_nanos() as f64;
    println!("total:        {:>10.2} ns/pass", ns / iters as f64);
    println!(
        "per sequence: {:>10.2} ns/input",
        ns / (iters as f64 * workload.len() as f64)
    );
    println!(
        "per push:     {:>10.2} ns/push",
        ns / (iters as f64 * pushes_per_pass)
    );
    println!("(best of {rounds} rounds)");
}