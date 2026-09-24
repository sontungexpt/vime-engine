//! Micro-benchmark of `Composition` caret edits vs plain append.
//!
//! `Composition::insert` at the caret is the public string-append path; mid-word
//! edits reposition the caret (via `move_left`/`move_right`) before inserting, so
//! the benchmark also covers the per-position routing cost of the onset / vowel /
//! coda branches.
//!   cargo bench --bench bench_composition_insert

use std::hint::black_box;
use std::time::Instant;

use vime_engine::composition::syllable::SyllableBuilder;
use vime_engine::composition::Composition;
use vime_engine::phonology::rules::TonePlacement;
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

/// Append a keystroke sequence at the caret.
fn run_append(workload: &[&str], keymap: &DefaultKeymap) {
    for &word in workload {
        let mut composition =
            Composition::new(SyllableBuilder::new(*keymap, TonePlacement::Modern));
        for ch in word.chars() {
            black_box(composition.insert(ch));
        }
    }
}

/// Simulate cursor-navigation edits: after building a syllable, park the caret
/// at every position (start → `pos`), insert a 'w' (horn), then a trailing coda
/// 'n' at mid positions.
fn run_edit(workload: &[&str], keymap: &DefaultKeymap) {
    for &word in workload {
        let mut composition =
            Composition::new(SyllableBuilder::new(*keymap, TonePlacement::Modern));
        for ch in word.chars() {
            black_box(composition.insert(ch));
        }

        let len = word.chars().count();
        for pos in 0..=len {
            for _ in 0..=len {
                composition.move_left();
            }
            for _ in 0..pos {
                composition.move_right();
            }
            black_box(composition.insert('w'));
        }
        for pos in 1..=len {
            for _ in 0..=len {
                composition.move_left();
            }
            for _ in 0..pos {
                composition.move_right();
            }
            black_box(composition.insert('n'));
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

    let append_t = time(|| run_append(workload, &keymap), rounds, iters);
    let edit_t = time(|| run_edit(workload, &keymap), rounds, iters);

    let append_ns = append_t.as_nanos() as f64 / iters as f64;
    let edit_ns = edit_t.as_nanos() as f64 / iters as f64;

    println!("words per pass:      {}", workload.len());
    println!("append (caret end):  {:9.2} ns/pass", append_ns);
    println!(
        "mid caret edits:     {:9.2} ns/pass   ({:.3}x append)",
        edit_ns,
        edit_ns / append_ns
    );
    println!("(best of {rounds} rounds)");
}