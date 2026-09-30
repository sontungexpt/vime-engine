//! Micro-benchmark of the public `Composition` edit paths.
//!
//! Two shapes are measured, because they are the two ways a frontend drives a
//! buffer and they cost very differently:
//!
//! - **append** — keystrokes arrive at the end of the word, which is what plain
//!   typing does. Transforms keep the caret on the transformed character, so
//!   built-in Vietnamese words type naturally.
//! - **caret edits** — the caret is parked at every position and a key is
//!   inserted, which is the cost of a *fix*: the keystroke has to be routed
//!   against the onset / nucleus / coda split the caret is currently inside.
//!
//! The two used to be separate benches (`bench_composition_push` and
//! `bench_composition_insert`) carrying near-identical word lists — they
//! differed only by one duplicated entry. They are one workload measured two
//! ways, so they live here together.
//!
//!   cargo bench --bench bench_composition
//!
//! Measures wall time per pass, per input sequence and per push over a
//! deterministic telex workload covering onsets (single/cluster/`qu`), plain
//! and precomposed vowels, the `uo`/`ươ` family, shape/tone transforms, codas
//! and uppercase input.

use std::hint::black_box;

use vime_engine::composition::Composition;
use vime_engine::phonology::TonePlacement;
use vime_engine::DefaultKeymap;

/// The tone-placement scheme the benchmark parses under, as a value: the
/// composition takes it per operation rather than storing it.
const TONE: TonePlacement = TonePlacement::Modern;

mod support;
use support::{ns_per_unit, rounds, time};

/// Vietnamese words as raw telex keystrokes.
const WORKLOAD: &[&str] = &[
    // single onsets + plain vowels
    "anh", "em", "o", "a", "u", "ai", "ao", "ay", "au", "ia", "iu", "ua", "uu", "ie",
    // consonant clusters
    "cha", "cho", "chi", "nga", "ngh", "nhe", "khi", "pho", "qua", "que", "gio", "gia",
    // shapes and tones
    "aw", "aa", "ow", "eekho", "aafamily", "som", "sinh", "hoc", "ban", "len", "how",
    // uo / ươ family with and without the horn key
    "uow", "uo", "uoi", "uowng", "luowng", "nuowc", "thuowng", "nguoi", "tuoi", "cuoi",
    // codas
    "cong", "long", "tan", "tam", "tap", "ach", "vient", "hat", "bac", "khong", "toan", "vien",
    "nuoc", "vuon", "muot", "tham", "thuy", "truong", // uppercase
    "Viet", "Nam", "HA", "NOI", "DAN", "Tien",
];

/// A fresh engine-equivalent buffer: one `Composition` per word, so no state
/// leaks between words.
fn new_composition() -> Composition {
    Composition::new()
}

/// Appends every keystroke of every word, returning the number of pushes.
fn run_append(keymap: &DefaultKeymap<'static>) -> usize {
    let mut pushes = 0;
    for &word in WORKLOAD {
        let mut composition = new_composition();
        for ch in word.chars() {
            black_box(composition.insert(keymap, TONE, ch));
            pushes += 1;
        }
    }
    black_box(pushes)
}

/// Parks the caret at every position and inserts, the cost of fixing a word.
///
/// A horn (`w`) at every position from the start, then a coda letter (`n`)
/// walking back up. The caret is re-parked by counting `move_left` past the
/// start, which saturates, so the inner loops do not need to know the buffer
/// length.
fn run_caret_edits(keymap: &DefaultKeymap<'static>) -> usize {
    let mut edits = 0;
    for &word in WORKLOAD {
        let mut composition = new_composition();
        for ch in word.chars() {
            black_box(composition.insert(keymap, TONE, ch));
        }

        let len = word.chars().count();
        for key in ['w', 'n'] {
            let positions = if key == 'w' { 0..=len } else { 1..=len };
            for pos in positions {
                for _ in 0..=len {
                    composition.move_cursor_left();
                }
                for _ in 0..pos {
                    composition.move_cursor_right();
                }
                black_box(composition.insert(keymap, TONE, key));
                edits += 1;
            }
        }
    }
    black_box(edits)
}

fn main() {
    let keymap = DefaultKeymap::telex();
    let rounds = rounds();
    let iters = 30_000;

    // One untimed pass each, to learn how much work a pass really does. These
    // counts are the normalizers below, so they must come from the same code
    // path that is timed.
    let pushes = run_append(&keymap) as f64;
    let edits = run_caret_edits(&keymap) as f64;
    let sequences = WORKLOAD.len() as f64;

    println!("keystroke sequences per pass: {}", WORKLOAD.len());
    println!("pushes per pass:              {}", pushes as usize);
    println!("caret edits per pass:         {}", edits as usize);
    println!();

    let append = time(
        || {
            run_append(&keymap);
        },
        rounds,
        iters,
    );
    let caret = time(
        || {
            run_caret_edits(&keymap);
        },
        rounds,
        iters,
    );

    let per_push = ns_per_unit(append, iters, pushes);
    let per_sequence = ns_per_unit(append, iters, sequences);
    let per_edit = ns_per_unit(caret, iters, edits);

    println!("append:");
    println!(
        "  total:        {:>10.2} ns/pass",
        append.as_nanos() as f64 / iters as f64
    );
    println!("  per sequence: {:>10.2} ns", per_sequence);
    println!("  per push:     {:>10.2} ns", per_push);
    println!();

    println!("caret edits:");
    println!(
        "  total:        {:>10.2} ns/pass",
        caret.as_nanos() as f64 / iters as f64
    );
    println!("  per edit:     {:>10.2} ns", per_edit);
    println!(
        "  vs append:    {:>10.2}x  (per edit / per push)",
        per_edit / per_push
    );
    println!();
    println!("(best of {rounds} rounds)");
}
