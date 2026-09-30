//! The two opaque handles, and the text cache a session hands to C.
//!
//! # What a handle holds
//!
//! A session handle is the engine's own [`Session`] plus the buffers C reads
//! through. Nothing else: no copy of the composition, no copy of the settings,
//! no second opinion about the cursor. Everything a frontend can ask for is
//! either already in the [`Session`] or one of the two cached render buffers, so
//! there is no state here that can fall out of step with the engine.
//!
//! A factory handle is just the engine's [`SessionFactory`]. The shared
//! configuration is already an `Arc` inside the core, and sessions already pick
//! changes up through it, so wrapping it in another copy of the settings would
//! only be a second thing to keep correct.
//!
//! # Why the buffers live in the handle
//!
//! The header says a returned `const char *` "is managed by Session and remains
//! valid until next session call". That is a promise about a *stable* address
//! for as long as the session lives, which rules out handing out a pointer into
//! a `String` built for one call. So each session owns a buffer that is refilled
//! in place, and the pointer stays valid for the session's whole lifetime —
//! strictly longer than the header promises.
//!
//! Refilling reuses the allocation: `clear` keeps the capacity, so a session that
//! renders on every keystroke allocates once and then never again. The address
//! can still move if the text outgrows the capacity, which is harmless under this
//! contract but is why nothing in this file assumes a stable address *across* a
//! growth.
//!
//! # Why nothing is eagerly computed
//!
//! Insert, backspace, delete and the cursor moves only mark the buffers stale.
//! They never render, and never build a snapshot. A frontend that types a word
//! and only asks for the text at the end pays for one render, not one per
//! keystroke, and a frontend that never asks pays nothing. The stale flags are
//! the whole of the mutation path's cache work: two byte writes.
//!
//! # Why there is no `CText`
//!
//! The two buffers want the same three things — reuse the allocation, carry one
//! NUL, and say whether they are current — so they share [`TextCache`]. What
//! they do *not* share is when they are refilled, and that is the only thing the
//! old `fresh` flag was ever for.

use std::ffi::c_char;

use vime_engine::{Session, SessionFactory};

use crate::config::{EngineConfig, Keymap};
use crate::render::{measure, Measured, VimeRenderState};

/// The engine session a handle owns.
type EngineSession = Session<Keymap>;

/// A reusable, NUL-terminated UTF-8 buffer.
///
/// The invariant is one line: `text` is the current content followed by exactly
/// one NUL, and that NUL is its last byte. Keeping the terminator inside the
/// buffer is what makes the pointer a `const char *` with nothing to build
/// around it — no `CString`, no per-call scan for interior NULs, and no second
/// copy out of the engine's `String`.
///
/// Interior NULs are possible: U+0000 is a valid scalar value, so a caller may
/// insert one and the engine will hold it. That is not a memory-safety problem —
/// it truncates the C view of the text at that point, which is inherent to
/// handing out a C string, and the engine's own copy is unaffected.
struct TextCache {
    /// Content plus the terminator. Never handed out without the terminator.
    text: String,
    /// Whether `text` still describes the engine's current state.
    fresh: bool,
}

impl TextCache {
    const fn new() -> Self {
        Self {
            text: String::new(),
            fresh: false,
        }
    }

    /// Refills the buffer, measuring it in the same pass.
    ///
    /// `render` appends the engine's text to `out`; it is the engine's own
    /// allocation-free writer, called straight into the buffer that C will read,
    /// so the text is never materialised anywhere else first.
    ///
    /// The terminator goes on after the measurement, so every length this returns
    /// is a length of the content rather than of the C string around it.
    #[inline]
    fn refill(&mut self, render: impl FnOnce(&mut String), caret_chars: usize) -> Measured {
        self.text.clear();
        render(&mut self.text);
        self.fresh = true;
        let measured = measure(&self.text, caret_chars);
        self.text.reserve(1);
        self.text.push('\0');
        measured
    }

    /// The buffer as a C string.
    #[inline]
    fn as_ptr(&self) -> *const c_char {
        debug_assert!(self.text.ends_with('\0'), "TextCache lost its terminator");
        self.text.as_ptr().cast()
    }
}

/// Everything a session has already worked out, and would otherwise work out
/// again for the next caller that asks.
///
/// Two independent flags rather than one, because the two buffers are filled by
/// different calls: a frontend that only wants the raw keystrokes should not pay
/// for a Vietnamese render, and one that only wants the rendered word should not
/// pay for the raw one. The snapshot is *not* flagged, because assembling it is
/// a handful of field stores and no allocation, so there is nothing to save.
pub(crate) struct RenderCache {
    /// The rendered word, terminated.
    text: TextCache,
    /// The raw keystrokes, terminated.
    raw: TextCache,
    /// Where the rendered caret is, in bytes, and how long the word is.
    text_measured: Measured,
    /// Where the raw caret is, in bytes.
    raw_byte_idx: usize,
    /// Length of the text most recently reported by `render_state`, in bytes and
    /// in characters — what a host has to erase before it can write the new one.
    ///
    /// `render_state` is the only thing that moves these, so that a frontend which
    /// reads the text with `render_text` and then asks for a snapshot still gets
    /// the length of the text before the edit, which is the one it needs.
    delivered_bytes: usize,
    delivered_chars: usize,
    /// The snapshot handed to C, kept so the pointer stays valid.
    state: VimeRenderState,
}

impl RenderCache {
    const fn new() -> Self {
        Self {
            text: TextCache::new(),
            raw: TextCache::new(),
            text_measured: Measured {
                byte_idx: 0,
                len_bytes: 0,
                len_chars: 0,
            },
            raw_byte_idx: 0,
            delivered_bytes: 0,
            delivered_chars: 0,
            state: VimeRenderState {
                text: std::ptr::null(),
                raw_text: std::ptr::null(),
                cursor_byte_idx: 0,
                cursor_char_idx: 0,
                raw_cursor_byte_idx: 0,
                raw_cursor_char_idx: 0,
                bytes_to_delete: 0,
                chars_to_delete: 0,
                is_valid_vietnamese: false,
            },
        }
    }

    /// Marks both buffers stale without giving up their allocations.
    #[inline]
    fn invalidate(&mut self) {
        self.text.fresh = false;
        self.raw.fresh = false;
    }
}

/// Opaque handle to one independent typing session.
///
/// `#[repr(C)]` because it is a C type, though it is only ever passed behind a
/// pointer and its layout is therefore nobody's business but its own.
#[repr(C)]
pub struct VimeSessionHandle {
    /// The engine session. The single source of truth for everything a frontend
    /// can observe.
    pub(crate) session: EngineSession,
    /// The text this session hands to C, and the snapshot it reports.
    pub(crate) cache: RenderCache,
}

impl VimeSessionHandle {
    /// Wraps a session, and pre-sizes the two render buffers.
    ///
    /// A Vietnamese syllable is a handful of characters in a handful of bytes, so
    /// this covers the length every word reaches. Sizing both buffers once here
    /// means the first keystroke does not allocate either, and the steady state
    /// never reallocates. The raw buffer has no such bound — one keystroke is one
    /// character, however many are typed — so it grows if it has to, and keeps
    /// the larger allocation after that.
    pub(crate) fn new(session: EngineSession) -> Self {
        const TYPICAL_SYLLABLE_BYTES: usize = 64;
        let mut cache = RenderCache::new();
        cache.text.text.reserve(TYPICAL_SYLLABLE_BYTES);
        cache.raw.text.reserve(TYPICAL_SYLLABLE_BYTES);
        Self { session, cache }
    }

    pub(crate) fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    /// Adopts a shared configuration change, dropping the cache if it moved the
    /// word.
    ///
    /// The core resolves a shared-config change lazily and reports whether it
    /// re-rendered, which is exactly the condition under which the cached text is
    /// wrong. Doing it here is what lets a frontend that changes the settings
    /// see the new rendering on its next *query* and not only on its next
    /// keystroke.
    #[inline]
    pub(crate) fn adopt_shared_config(&mut self) {
        if self.session.refresh_config() {
            self.cache.invalidate();
        }
    }

    /// Forgets the cached text after an edit. Two byte writes; no rendering.
    #[inline]
    pub(crate) fn invalidate(&mut self) {
        self.cache.invalidate();
    }

    /// The rendered word as a C string, rendering it only if it is stale.
    #[inline]
    pub(crate) fn render_text(&mut self) -> *const c_char {
        self.ensure_text();
        self.cache.text.as_ptr()
    }

    /// The raw keystrokes as a C string, materialising them only if they are stale.
    #[inline]
    pub(crate) fn render_raw_text(&mut self) -> *const c_char {
        self.ensure_raw();
        self.cache.raw.as_ptr()
    }

    /// The rendered caret, in characters. No rendering: the core knows it.
    #[inline]
    pub(crate) fn cursor_char_idx(&mut self) -> usize {
        self.adopt_shared_config();
        self.session.cursor_pos()
    }

    /// The raw caret, in keystrokes. No rendering: the core knows it.
    #[inline]
    pub(crate) fn raw_cursor_char_idx(&mut self) -> usize {
        self.adopt_shared_config();
        self.session.raw_cursor_pos()
    }

    /// The rendered caret, in UTF-8 bytes.
    #[inline]
    pub(crate) fn cursor_byte_idx(&mut self) -> usize {
        self.ensure_text();
        self.cache.text_measured.byte_idx
    }

    /// The raw caret, in UTF-8 bytes.
    #[inline]
    pub(crate) fn raw_cursor_byte_idx(&mut self) -> usize {
        self.ensure_raw();
        self.cache.raw_byte_idx
    }

    /// Whether the buffer spells a complete, valid Vietnamese syllable.
    #[inline]
    pub(crate) fn is_valid_vietnamese(&mut self) -> bool {
        self.adopt_shared_config();
        self.session.is_valid()
    }

    /// A complete snapshot of everything a frontend needs for one repaint.
    ///
    /// Each buffer is materialised at most once and the four offsets are measured
    /// in the same pass that writes the text, so this is the cheapest way to get
    /// the whole picture: a caller that would otherwise call six getters pays for
    /// two renders and two walks instead.
    ///
    /// This is also the delivery point for the `*_to_delete` fields, which is why
    /// the previously-reported length is read out before it is replaced.
    pub(crate) fn render_state(&mut self) -> *const VimeRenderState {
        self.ensure_text();
        self.ensure_raw();

        // Split the borrow: the snapshot is written while the text is read.
        let Self { session, cache, .. } = self;
        let state = &mut cache.state;
        state.text = cache.text.as_ptr();
        state.raw_text = cache.raw.as_ptr();
        state.cursor_byte_idx = cache.text_measured.byte_idx;
        state.cursor_char_idx = session.cursor_pos();
        state.raw_cursor_byte_idx = cache.raw_byte_idx;
        state.raw_cursor_char_idx = session.raw_cursor_pos();
        state.bytes_to_delete = cache.delivered_bytes;
        state.chars_to_delete = cache.delivered_chars;
        state.is_valid_vietnamese = session.is_valid();

        // What the host is told to erase is the text it was last handed, so this
        // call's text becomes the next call's `*_to_delete`.
        cache.delivered_bytes = cache.text_measured.len_bytes;
        cache.delivered_chars = cache.text_measured.len_chars;

        &cache.state as *const VimeRenderState
    }

    /// Renders the word if the cache is stale.
    ///
    /// The config is adopted first, so a session notices a shared-config change on
    /// the query that asks for text rather than only on the next keystroke.
    #[inline]
    fn ensure_text(&mut self) {
        self.adopt_shared_config();
        if self.cache.text.fresh {
            return;
        }
        let caret = self.session.cursor_pos();
        let Self { session, cache, .. } = self;
        cache.text_measured = cache
            .text
            .refill(|out| session.write_rendered_to(out), caret);
    }

    /// Writes the raw keystrokes if the cache is stale.
    #[inline]
    fn ensure_raw(&mut self) {
        self.adopt_shared_config();
        if self.cache.raw.fresh {
            return;
        }
        let caret = self.session.raw_cursor_pos();
        let Self { session, cache, .. } = self;
        let measured = cache.raw.refill(|out| session.write_raw_to(out), caret);
        cache.raw_byte_idx = measured.byte_idx;
    }
}

/// Opaque handle to a session factory: one shared configuration, and the
/// sessions minted from it.
#[repr(C)]
pub struct VimeSessionFactoryHandle {
    pub(crate) factory: SessionFactory<Keymap>,
}

impl VimeSessionFactoryHandle {
    pub(crate) fn new(config: EngineConfig) -> Self {
        Self {
            factory: SessionFactory::new(config),
        }
    }

    pub(crate) fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    /// A new session that follows the shared configuration.
    pub(crate) fn new_session(&self) -> EngineSession {
        self.factory.new_session()
    }

    /// A new session with a private configuration, still able to fall back to
    /// the shared one.
    pub(crate) fn new_session_with(&self, config: EngineConfig) -> EngineSession {
        self.factory.new_session_with(config)
    }
}

// ──────────────────────────── pointer validation ────────────────────────────
//
// Every raw pointer that arrives from C passes through one of these before it is
// dereferenced, and nowhere else. Each returns a borrow with no lifetime
// attached to the pointer, which is the soundness condition: the handle outlives
// the call because the caller cannot destroy it mid-call, and the call cannot
// outlive the handle because destroying it is a separate call.
//
// The `'a` is the standard FFI bargain, the same one `Box::from_raw` makes. What
// keeps it honest here is that the exported functions never store a borrow and
// never call another exported function while holding one.

impl VimeSessionHandle {
    /// Borrows a session handle, or `None` for NULL.
    ///
    /// # Safety
    ///
    /// `ptr` must be NULL or a pointer returned by [`Self::into_raw`] that has not
    /// been destroyed, and no other borrow of it may be live.
    #[inline]
    pub(crate) unsafe fn from_raw<'a>(ptr: *mut Self) -> Option<&'a mut Self> {
        if ptr.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees `ptr` came from `Box::into_raw` in
        // `into_raw` and has not been dropped since, so it points at a live
        // `VimeSessionHandle` with unique ownership. The returned lifetime is not
        // tied to the pointer, which is why the contract above has to hold.
        Some(unsafe { &mut *ptr })
    }

    /// Takes ownership of a session handle back, for `vime_session_destroy`.
    ///
    /// # Safety
    ///
    /// `ptr` must be NULL or a pointer returned by [`Self::into_raw`] that has not
    /// been destroyed, and ownership of it must pass to this call.
    #[inline]
    pub(crate) unsafe fn into_box(ptr: *mut Self) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the pointer came from `Box::into_raw` and
        // hands over ownership, which is exactly what `Box::from_raw` assumes.
        Some(unsafe { Box::from_raw(ptr) })
    }
}

impl VimeSessionFactoryHandle {
    /// Borrows a factory handle, or `None` for NULL.
    ///
    /// # Safety
    ///
    /// `ptr` must be NULL or a pointer returned by [`Self::into_raw`] that has not
    /// been destroyed, and no other borrow of it may be live.
    #[inline]
    pub(crate) unsafe fn from_raw<'a>(ptr: *mut Self) -> Option<&'a mut Self> {
        if ptr.is_null() {
            return None;
        }
        // SAFETY: as for `VimeSessionHandle::from_raw`.
        Some(unsafe { &mut *ptr })
    }

    /// Takes ownership of a factory handle back, for `vime_session_factory_destroy`.
    ///
    /// # Safety
    ///
    /// `ptr` must be NULL or a pointer returned by [`Self::into_raw`] that has not
    /// been destroyed, and ownership of it must pass to this call.
    #[inline]
    pub(crate) unsafe fn into_box(ptr: *mut Self) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }
        // SAFETY: as for `VimeSessionHandle::into_box`.
        Some(unsafe { Box::from_raw(ptr) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vime_engine::{DefaultKeymap, Settings};

    fn session() -> VimeSessionHandle {
        VimeSessionHandle::new(
            SessionFactory::<Keymap>::from_keymap(Settings::default(), DefaultKeymap::telex())
                .new_session(),
        )
    }

    fn text_of(ptr: *const c_char) -> String {
        assert!(!ptr.is_null());
        // SAFETY: every pointer this test reads came from a live handle, which
        // owns the buffer behind it.
        unsafe { std::ffi::CStr::from_ptr(ptr) }
            .to_str()
            .unwrap()
            .to_owned()
    }

    #[test]
    fn an_empty_session_renders_an_empty_string_not_null() {
        let mut handle = session();
        let ptr = handle.render_text();
        assert_eq!(text_of(ptr), "");
        assert_eq!(text_of(handle.render_raw_text()), "");
    }

    #[test]
    fn a_terminator_is_appended_exactly_once() {
        let mut handle = session();
        for ch in "hoas".chars() {
            handle.session.insert(ch);
            handle.invalidate();
        }
        let ptr = handle.render_text();
        // SAFETY: the pointer is owned by the live handle, and the render is four
        // characters plus the terminator, so five bytes are in bounds.
        let bytes = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), 5) };
        assert_eq!(bytes, "hoá\0".as_bytes());
    }

    /// The whole point of the cached buffers: a second render is free, and a
    /// refill after an edit reuses the same allocation.
    #[test]
    fn refilling_reuses_the_allocation() {
        let mut handle = session();
        let first = handle.render_text();
        for ch in "toan".chars() {
            handle.session.insert(ch);
            handle.invalidate();
        }
        let second = handle.render_text();
        assert_eq!(first, second, "the allocation was reused in place");
        assert_eq!(text_of(second), "toan");
    }

    #[test]
    fn stale_text_is_replaced_not_appended() {
        let mut handle = session();
        for ch in "toan".chars() {
            handle.session.insert(ch);
            handle.invalidate();
        }
        assert_eq!(text_of(handle.render_text()), "toan");
        for ch in "hoa".chars() {
            handle.session.insert(ch);
            handle.invalidate();
        }
        assert_eq!(text_of(handle.render_text()), "toanhoa");
        handle.session.reset();
        handle.invalidate();
        assert_eq!(text_of(handle.render_text()), "");
    }
}
