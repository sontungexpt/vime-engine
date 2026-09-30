//! Per-keystroke cost of the C boundary, against the same work done in Rust.
//!
//! The point is the ratio, not the absolute number. A frontend calls the FFI once
//! per key and again per repaint, so what matters is how much of the keystroke's
//! time the boundary adds over calling the engine directly. The harness prints both
//! sides and their difference.
//!
//! Run with `cargo bench -p vime-engine-ffi`. The absolute numbers are meaningless
//! in a debug build, so this is only useful under `--release` (which `cargo bench`
//! implies).

use std::hint::black_box;
use std::time::{Duration, Instant};

use vime::{
    vime_session_backspace, vime_session_insert, vime_session_render_raw_text,
    vime_session_render_state, vime_session_render_text, vime_session_reset, VimeConfig,
    VimeSessionHandle,
};

/// Keystrokes per iteration. Long enough to average out timer noise, short enough
/// that a slow machine finishes the whole run in seconds.
const KEYSTROKES: usize = 500_000;

/// The word typed in the throughput case, chosen because it exercises the whole
/// path: transform lookups, a tone placement, and a composed result.
const WORD: &str = "tieengs";

/// Measures one case and reports the mean per-keystroke cost.
///
/// The batch is timed in a single `Instant` span rather than per call, because
/// `Instant::now()` costs on the order of 20ns and would be a large fraction of
/// the work being measured.
fn measure(name: &str, keystrokes: usize, mut body: impl FnMut(usize)) {
    // One untimed pass, so the buffers are sized and any lazily initialised state
    // exists before the clock starts.
    body(1000);

    let mut best = Duration::MAX;
    for _ in 0..5 {
        let start = Instant::now();
        body(keystrokes);
        best = best.min(start.elapsed());
    }

    let per_call = best.as_secs_f64() * 1e9 / keystrokes as f64;
    println!(
        "{name:<28} {best:>10.2?} total  {per_call:>8.1} ns/call",
        best = best
    );
}

/// A session handle, leaked so it outlives the run.
struct Session(*mut VimeSessionHandle);

// SAFETY: the handle is leaked, so it is live for the whole program, and the
// benchmark is single-threaded.
unsafe impl Send for Session {}

fn session() -> Session {
    // SAFETY: called once, with a valid config pointer.
    let config = VimeConfig::default();
    let factory =
        unsafe { vime::vime_session_factory_create_with_config(std::ptr::from_ref(&config)) };
    assert!(!factory.is_null(), "a factory");
    // SAFETY: `factory` is live.
    let handle = unsafe { vime::vime_session_create(factory) };
    assert!(!handle.is_null(), "a session");
    // The session is documented to outlive its factory, so the factory goes first.
    // SAFETY: the factory is live and the session it created is independent.
    unsafe { vime::vime_session_factory_destroy(factory) };
    Session(handle)
}

/// Feeds `WORD` in a loop, resetting between words so the composition stays the
/// length a real one is.
///
/// Without the reset the buffer grows to `KEYSTROKES` characters and every
/// keystroke costs a recompose of everything typed so far, which measures
/// quadratic growth rather than the keystroke path.
fn typing(handle: *mut VimeSessionHandle, n: usize) {
    for i in 0..n {
        let ch = WORD.as_bytes()[i % WORD.len()] as char;
        // SAFETY: the handle is live and the scalar is a valid `char`.
        unsafe { vime_session_insert(handle, ch as u32) };
        if i % WORD.len() == WORD.len() - 1 {
            // SAFETY: the handle is live.
            unsafe { vime_session_reset(handle) };
        }
    }
}

fn main() {
    println!("vime FFI hot path, {KEYSTROKES} keystrokes per case\n");

    let session = session();
    let handle = session.0;

    measure("insert + render_text", KEYSTROKES, |n| {
        for i in 0..n {
            let ch = WORD.as_bytes()[i % WORD.len()] as char;
            // SAFETY: the handle is live and the scalar is a valid `char`.
            unsafe { vime_session_insert(handle, ch as u32) };
            // SAFETY: the handle is live and owns the returned buffer.
            black_box(unsafe { vime_session_render_text(handle) });
            if i % WORD.len() == WORD.len() - 1 {
                // SAFETY: the handle is live.
                unsafe { vime_session_reset(handle) };
            }
        }
    });

    measure("insert + render_state", KEYSTROKES, |n| {
        for i in 0..n {
            let ch = WORD.as_bytes()[i % WORD.len()] as char;
            // SAFETY: the handle is live and the scalar is a valid `char`.
            unsafe { vime_session_insert(handle, ch as u32) };
            // SAFETY: the handle is live and owns the returned snapshot.
            black_box(unsafe { vime_session_render_state(handle) });
            if i % WORD.len() == WORD.len() - 1 {
                // SAFETY: the handle is live.
                unsafe { vime_session_reset(handle) };
            }
        }
    });

    measure("insert + all three views", KEYSTROKES, |n| {
        for i in 0..n {
            let ch = WORD.as_bytes()[i % WORD.len()] as char;
            // SAFETY: the handle is live and the scalar is a valid `char`.
            unsafe { vime_session_insert(handle, ch as u32) };
            // SAFETY: the handle is live and owns the returned buffers.
            unsafe {
                black_box(vime_session_render_text(handle));
                black_box(vime_session_render_raw_text(handle));
                black_box(vime_session_render_state(handle));
            }
            if i % WORD.len() == WORD.len() - 1 {
                // SAFETY: the handle is live.
                unsafe { vime_session_reset(handle) };
            }
        }
    });

    // The same keystrokes with no render call at all, which is the floor: whatever
    // this costs is the engine, and the gap to the cases above is the boundary.
    measure("insert only (engine floor)", KEYSTROKES, |n| {
        typing(handle, n)
    });

    // The repaint cost, which is the case a frontend actually hits: nothing has
    // changed, so every call should be a pointer return.
    measure("cached repaint", KEYSTROKES, |n| {
        for _ in 0..n {
            // SAFETY: the handle is live and owns the returned buffer.
            black_box(unsafe { vime_session_render_text(handle) });
        }
    });

    measure("backspace + render", KEYSTROKES, |n| {
        for _ in 0..n {
            // SAFETY: the handle is live.
            unsafe { vime_session_backspace(handle) };
            // SAFETY: the handle is live and owns the returned buffer.
            black_box(unsafe { vime_session_render_text(handle) });
        }
    });

    // SAFETY: the handle is live.
    unsafe { vime_session_reset(handle) };
}
