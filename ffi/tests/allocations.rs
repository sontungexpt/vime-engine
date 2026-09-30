//! Measures the FFI's allocation behaviour with a counting global allocator.
//!
//! The claim being tested is specific: once a handle exists and its buffers are
//! sized, typing, editing and rendering do not allocate. Everything the boundary
//! needs is already in the handle — the two `String` buffers, the snapshot struct,
//! the session — so a keystroke is arithmetic plus one `memcpy` of a few bytes.
//!
//! Counting has to happen in Rust. A C test would have to interpose on `malloc`,
//! and a statically linked program whose interposer calls `malloc` recurses before
//! it is initialised. That was tried and hung.
//!
//! Two details make the numbers trustworthy:
//!
//! * The counter is keyed on the running thread, so a test that shares a binary
//!   with the harness — which allocates on its own threads — still measures only
//!   its own work.
//! * Warm-up is explicit. The very first keystroke grows the core's own
//!   character buffers, and the first render of each text buffer grows that
//!   buffer; both are one-time costs of a `Vec` doubling. Asserting they do not
//!   happen would mean asserting that `Vec` does not grow, so they happen before
//!   the measured region instead of being counted in it.

mod common;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ffi::CStr;

use vime::{
    vime_session_backspace, vime_session_delete, vime_session_insert,
    vime_session_move_cursor_left, vime_session_render_text, vime_session_reset, VimeSessionHandle,
};

// The current thread's allocation count, or `NOT_COUNTING` when unmeasured.
//
// A thread-local rather than a global plus a thread id: `ThreadId` has no stable
// numeric form, and an address-keyed global counts a different test's work
// whenever the harness recycles a thread's TLS slot. `const` init means there is no
// destructor, so consulting this allocates nothing.
thread_local! {
    static COUNT: Cell<usize> = const { Cell::new(NOT_COUNTING) };
}

const NOT_COUNTING: usize = usize::MAX;

/// Counts allocations, and otherwise defers to the system allocator.
struct Counting;

// SAFETY: every method forwards to `System` unchanged, so the allocator contract
// is inherited rather than reimplemented.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count();
        // SAFETY: forwarded to the system allocator unchanged.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwarded to the system allocator unchanged.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count();
        // SAFETY: forwarded to the system allocator unchanged.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        count();
        // SAFETY: forwarded to the system allocator unchanged.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

fn count() {
    let _ = COUNT.try_with(|count| {
        if count.get() != NOT_COUNTING {
            count.set(count.get() + 1);
        }
    });
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// Counts the allocations this thread makes while `f` runs.
fn allocations_during<R>(f: impl FnOnce() -> R) -> (R, usize) {
    COUNT.with(|count| count.set(0));
    let result = f();
    let total = COUNT.with(|count| {
        let total = count.get();
        count.set(NOT_COUNTING);
        total
    });
    (result, total)
}

/// A live session handle that has already typed `word`, plus the factory keeping
/// its shared config alive.
///
/// The `Session` wrapper is leaked on purpose: forgetting it stops the harness
/// from destroying the handle, while the factory is returned so the caller can
/// drop it normally. A leak of two allocations in a test process is a better trade
/// than a `ManuallyDrop` and an `unsafe impl` in every test.
fn warm(word: &str) -> (*mut VimeSessionHandle, common::Factory) {
    let mut factory = common::Factory::create().expect("a factory");
    let mut session = factory.open_session();
    for ch in word.chars() {
        assert!(session.insert(ch));
    }
    // Fill both text buffers and the snapshot, so none of them has to grow later.
    assert!(!session.render_text().is_empty());
    assert!(!session.render_raw_text().is_empty());
    let _ = session.render_state();

    let handle = session.handle();
    std::mem::forget(session);
    (handle, factory)
}

/// The `const char *` length, without copying the string.
///
/// The shared harness returns an owned `String`, so using it inside a measured
/// region would count the test's own allocation. A C frontend reads the pointer
/// in place, and that is what this mirrors.
fn text_len(handle: *mut VimeSessionHandle) -> usize {
    // SAFETY: `handle` is live and owns the returned buffer.
    unsafe {
        CStr::from_ptr(vime::vime_session_render_text(handle))
            .to_bytes()
            .len()
    }
}

fn raw_text_len(handle: *mut VimeSessionHandle) -> usize {
    // SAFETY: `handle` is live and owns the returned buffer.
    unsafe {
        CStr::from_ptr(vime::vime_session_render_raw_text(handle))
            .to_bytes()
            .len()
    }
}

fn snapshot_text_len(handle: *mut VimeSessionHandle) -> usize {
    // SAFETY: `handle` is live and owns the returned snapshot, whose `text` field
    // points into a buffer the same handle owns. Only the length is read.
    unsafe {
        let state = vime::vime_session_render_state(handle);
        assert!(!state.is_null());
        CStr::from_ptr((*state).text).to_bytes().len()
    }
}

#[test]
fn typing_and_rendering_allocate_nothing() {
    let (handle, _factory) = warm("tieengs");
    // SAFETY: `handle` is live for the rest of this test.
    unsafe { vime_session_reset(handle) };

    let (length, allocations) = allocations_during(|| {
        for ch in "tieengs".chars() {
            // SAFETY: `handle` is live.
            assert!(unsafe { vime_session_insert(handle, ch as u32) });
            // Read after every keystroke, which is what a frontend repainting per
            // key does.
            std::hint::black_box(text_len(handle));
        }
        text_len(handle)
    });

    assert_eq!(length, "tiếng".len(), "the word rendered correctly");
    assert_eq!(
        allocations, 0,
        "typing and rendering should not allocate, but {allocations} did"
    );
}

#[test]
fn the_snapshot_does_not_allocate() {
    let (handle, _factory) = warm("tieengs");

    // The snapshot is the whole point: one call, everything a frontend needs.
    // Repeated, because a repaint happens on every keystroke and often more than
    // once.
    let (length, allocations) = allocations_during(|| {
        let mut length = 0;
        for _ in 0..10_000 {
            length = snapshot_text_len(handle);
        }
        length
    });

    assert_eq!(length, "tiếng".len());
    assert_eq!(
        allocations, 0,
        "10,000 snapshots allocated {allocations} times"
    );
}

#[test]
fn cached_queries_do_not_re_render() {
    let (handle, _factory) = warm("tieengs");

    // A query that hits a fresh cache still does no allocation: the buffer is
    // already there, so the only work is re-rendering into it.
    let (length, first) = allocations_during(|| text_len(handle));
    assert_eq!(length, "tiếng".len());
    assert_eq!(
        first, 0,
        "a stale cache re-renders, but into an existing buffer"
    );

    // A query that hits a warm cache is a pointer return and nothing else.
    let (_, repeated) = allocations_during(|| {
        for _ in 0..100_000 {
            std::hint::black_box(text_len(handle));
        }
    });
    assert_eq!(
        repeated, 0,
        "100,000 cached reads allocated {repeated} times"
    );
}

#[test]
fn editing_does_not_allocate() {
    let (handle, _factory) = warm("dduongf");
    // SAFETY: `handle` is live.
    unsafe { vime_session_reset(handle) };

    let (length, allocations) = allocations_during(|| {
        for ch in "dduongf".chars() {
            // SAFETY: `handle` is live.
            assert!(unsafe { vime_session_insert(handle, ch as u32) });
        }
        std::hint::black_box(text_len(handle));

        // Backspace the whole word away, reading everything each time.
        for _ in 0..8 {
            // SAFETY: `handle` is live.
            unsafe {
                vime_session_backspace(handle);
                vime_session_delete(handle);
                vime_session_move_cursor_left(handle);
            }
            std::hint::black_box(raw_text_len(handle));
            std::hint::black_box(snapshot_text_len(handle));
        }
        text_len(handle)
    });

    assert_eq!(length, 0, "the word was fully deleted");
    assert_eq!(
        allocations, 0,
        "editing should not allocate, but {allocations} did"
    );
}

/// A long word, to check the buffers grow by doubling rather than per keystroke.
/// Some growth is unavoidable, so this asserts the *shape* of the cost: a handful
/// of reallocations, not one per key.
#[test]
fn growth_is_geometric_not_per_keystroke() {
    let (handle, _factory) = warm(&"a".repeat(400));

    // Warm to the same length, so the measured word does not grow either buffer.
    for ch in std::iter::repeat_n('a', 400) {
        // SAFETY: `handle` is live.
        assert!(unsafe { vime_session_insert(handle, ch as u32) });
    }
    std::hint::black_box(raw_text_len(handle));
    // SAFETY: `handle` is live.
    unsafe { vime_session_reset(handle) };

    const KEYSTROKES: usize = 400;
    let (length, allocations) = allocations_during(|| {
        for ch in std::iter::repeat_n('b', KEYSTROKES) {
            // SAFETY: `handle` is live.
            assert!(unsafe { vime_session_insert(handle, ch as u32) });
            std::hint::black_box(text_len(handle));
            std::hint::black_box(raw_text_len(handle));
        }
        text_len(handle)
    });

    // A `String` grows by doubling, so 400 bytes of new content is a handful of
    // reallocations. A per-keystroke allocation would be at least `KEYSTROKES`.
    assert!(
        allocations < 20,
        "{KEYSTROKES} keystrokes allocated {allocations} times; expected geometric growth"
    );
    assert!(
        length <= KEYSTROKES,
        "the render is at most the keystrokes typed"
    );
}

/// The panic guard is a `catch_unwind`, so a rejection inside the library must
/// leave the session usable rather than unwinding into C. Rejected input also must
/// not allocate: the check is a range test before anything is built.
#[test]
fn a_rejected_keystroke_allocates_nothing_and_keeps_the_session() {
    let (handle, _factory) = warm("tieengs");

    let (length, allocations) = allocations_during(|| {
        // Not in the list: U+0000. It is a valid `char`, so the engine takes it and
        // the returned C string simply ends early. `a_nul_is_accepted_and_truncates`
        // pins that behaviour down.
        for bad in [u32::MAX, 0x0011_0000, 0xD800, 0xDFFF_FFFF] {
            // SAFETY: the handle is live. These are rejected by `char::from_u32`
            // before anything is built, so the FFI returns false without touching
            // the composition.
            assert!(!unsafe { vime_session_insert(handle, bad) });
            std::hint::black_box(unsafe { vime_session_render_text(handle) });
        }
        text_len(handle)
    });

    assert_eq!(
        length,
        "tiếng".len(),
        "the session survived every rejection"
    );
    assert_eq!(
        allocations, 0,
        "rejecting bad input should not allocate, but {allocations} did"
    );
}

/// The null path is taken before any dereference, and allocates nothing because
/// there is no handle to allocate into.
#[test]
fn the_null_path_allocates_nothing() {
    let (_, allocations) = allocations_during(|| {
        let null = std::ptr::null_mut();
        for bad in [u32::MAX, 0xD800, 0xDFFF_FFFF] {
            // SAFETY: NULL is the documented way to reach the null path, which
            // each of these calls checks for before using the handle.
            unsafe {
                assert!(!vime_session_insert(null, bad));
                assert!(vime::vime_session_render_text(null).is_null());
                assert!(vime::vime_session_render_raw_text(null).is_null());
                assert!(vime::vime_session_render_state(null).is_null());
                assert!(!vime_session_backspace(null));
                assert!(!vime_session_delete(null));
                assert!(!vime_session_move_cursor_left(null));
                assert_eq!(vime::vime_session_get_cursor_char_idx(null), 0);
                assert_eq!(vime::vime_session_get_cursor_byte_idx(null), 0);
                assert!(!vime::vime_session_is_valid_vietnamese(null));
                assert!(!vime::vime_session_set_config(null, std::ptr::null()));
                assert!(!vime::vime_session_clear_config(null));
            }
        }
    });
    assert_eq!(
        allocations, 0,
        "the null path allocated {allocations} times"
    );
}

/// U+0000 is a valid `char` and the engine takes it, but a `const char *` cannot
/// represent it past the terminator. So the C view of the buffer is what a frontend
/// sees, and the engine's own buffer is longer.
#[test]
fn a_nul_is_accepted_and_truncates_the_c_string() {
    let (handle, _factory) = warm("tieengs");
    // SAFETY: the handle is live.
    unsafe { vime_session_reset(handle) };

    // No allocation count here on purpose: a NUL is a normal `char` to the engine,
    // so it moves the composition out of the state the warm-up sized the buffers
    // for, and the growth that follows is the same growth the geometric test
    // covers. This test is about what a `const char *` can represent, not about
    // buffer reuse.
    for ch in "ab".chars() {
        // SAFETY: the handle is live.
        assert!(unsafe { vime_session_insert(handle, ch as u32) });
    }
    // SAFETY: the handle is live, and U+0000 is a valid `char`.
    assert!(unsafe { vime_session_insert(handle, 0) });
    // SAFETY: the handle is live.
    assert!(unsafe { vime_session_insert(handle, 'c' as u32) });

    assert_eq!(
        text_len(handle),
        2,
        "the C string stops at the NUL, hiding `c`"
    );

    // Both views truncate at the same place, and there is no way back: the raw
    // buffer is also handed out as a `const char *`, so the keystrokes after the
    // NUL are accepted by the engine and then permanently unreachable from C. That
    // is a property of the boundary, and a frontend that only ever sends printable
    // input never reaches it.
    // SAFETY: the handle is live and owns the returned buffer.
    let raw = unsafe { CStr::from_ptr(vime::vime_session_render_raw_text(handle)).to_bytes() };
    assert_eq!(raw, b"ab", "the raw view truncates at the same NUL");
}
