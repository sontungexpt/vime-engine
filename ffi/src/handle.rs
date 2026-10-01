//! The two opaque handles, and the buffers a session hands to C.
//!
//! # What a session handle holds
//!
//! The engine's own [`Session`], the two NUL-terminated strings C reads, and the
//! last snapshot reported. That is all: no copy of the composition, no copy of
//! the settings, no second opinion about the caret. Every number a frontend can
//! ask for is either already in the [`Session`] or is derived from the two
//! buffers when it is asked for, so nothing here can fall out of step with the
//! engine.
//!
//! A factory handle is just the engine's [`SessionFactory`]. The shared
//! configuration is already an `Arc` inside the core and sessions already follow
//! it, so wrapping it in another copy of the settings would be a second thing to
//! keep correct.
//!
//! # Why the strings live in the handle
//!
//! The header promises a `const char *` that "remains valid until next session
//! call", which rules out handing out a pointer into a `String` built for one
//! call. So the session owns a buffer and refills it in place, and the pointer is
//! then good for the session's whole lifetime — strictly longer than the header
//! promises. `clear` keeps the allocation, so a session that renders on every
//! keystroke allocates once and never again. The address can still move if the text
//! outgrows the capacity, which that contract allows and which is why nothing here
//! assumes a stable address *across* a growth.
//!
//! # Why nothing is eagerly computed
//!
//! An edit only marks the two buffers stale. It never renders, never walks UTF-8
//! and never builds a snapshot, so a frontend that types a word and asks once pays
//! for one render rather than one per keystroke, and a frontend that never asks
//! pays nothing. A keystroke is `validate -> insert -> two stores -> return`.
//!
//! # The invalidation rule
//!
//! Only an operation that can change the *text* marks the buffers stale. Moving
//! the caret cannot: it moves two numbers the core owns and rewrites neither
//! string. Because the byte offsets are derived from the caret at the moment they
//! are asked for rather than stored, a caret move has nothing to invalidate, which
//! is why [`VimeSessionHandle::invalidate`] is not called on the cursor path.
//! Stated once: if the two strings could come out different, mark them stale; if
//! not, do nothing.

use std::ffi::c_char;

use vime_engine::{Session, SessionFactory};

use crate::config::{FfiKeymap, FfiSessionConfig};
use crate::render::{measure, VimeRenderState};

/// The engine session a handle owns.
type EngineSession = Session<FfiKeymap>;

/// The text inside a render buffer, without the terminator the fill added.
///
/// A buffer is either empty — a session that has not rendered yet — or the text
/// followed by exactly one NUL, so stripping that one NUL is the whole of it.
/// Stripping exactly one matters: U+0000 is a valid scalar value, so a caller may
/// insert one and the engine will hold it, and a buffer for the text `a\0` is
/// `a\0\0`. The C view still stops at the first NUL, which is what a C string is,
/// but the delivery lengths keep counting the full content.
#[inline]
fn content(buffer: &str) -> &str {
    buffer.strip_suffix('\0').unwrap_or(buffer)
}

/// Opaque handle to one independent typing session.
///
/// `#[repr(C)]` because it is a C type, though it is only ever passed behind a
/// pointer and its layout is therefore nobody's business but its own.
#[repr(C)]
pub struct VimeSessionHandle {
    /// The engine session, and the single source of truth for everything a
    /// frontend can observe.
    pub(crate) session: EngineSession,
    /// The rendered word, followed by one NUL.
    rendered: String,
    /// The raw keystrokes, followed by one NUL.
    raw: String,
    /// Whether `rendered` still describes the session.
    rendered_current: bool,
    /// Whether `raw` still describes the session. A separate flag from
    /// `rendered_current` because the two are filled by different calls: a
    /// frontend that only wants the keystrokes should not pay for a Vietnamese
    /// render, and one that only wants the word should not pay for the raw one.
    raw_current: bool,
    /// The length of the text most recently reported by `render_state`, in bytes
    /// and in characters — what a host has to erase before it can write the next
    /// one. This is the only state that has to survive an edit, because it
    /// describes text the engine no longer holds.
    delivered_bytes: usize,
    delivered_chars: usize,
    /// The snapshot handed to C. Kept in the handle so the returned pointer stays
    /// valid; `render_state` overwrites every field before handing it out, so
    /// this is only ever a placeholder.
    state: VimeRenderState,
}

impl VimeSessionHandle {
    /// Wraps a session, and pre-sizes the two render buffers.
    ///
    /// A Vietnamese syllable is a handful of characters in a handful of bytes, so
    /// this covers the length every word reaches. Sizing both buffers once here
    /// means the first keystroke does not allocate either, and the steady state
    /// never reallocates. The raw buffer has no such bound — one keystroke is one
    /// character, however many are typed — so it grows if it has to, and keeps the
    /// larger allocation after that.
    pub(crate) fn new(session: EngineSession) -> Self {
        const TYPICAL_SYLLABLE_BYTES: usize = 64;
        let mut rendered = String::new();
        rendered.reserve(TYPICAL_SYLLABLE_BYTES);
        let mut raw = String::new();
        raw.reserve(TYPICAL_SYLLABLE_BYTES);
        Self {
            session,
            rendered,
            raw,
            rendered_current: false,
            raw_current: false,
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

    pub(crate) fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    /// Marks the cached text stale, keeping both allocations.
    ///
    /// Called only by operations that can change the text, and only after the core
    /// has applied them.
    #[inline]
    pub(crate) fn invalidate(&mut self) {
        self.rendered_current = false;
        self.raw_current = false;
    }

    /// Drops the cached text if the core reports it adopted a newer config.
    ///
    /// The core owns config resolution: every config-dependent `Session`
    /// operation calls `pull_config` for itself. This is not that. It exists
    /// only because a *shared* config change arrives through the factory, which
    /// holds no reference to this handle, so there is no other path by which
    /// this cache can learn that the text it holds has stopped describing the
    /// session. Without it, `fill_rendered` would return the cached string
    /// without ever asking the session, and the word on screen would keep the
    /// old tone placement.
    ///
    /// A private-config change does not need this: that runs through
    /// `set_private_config` on this handle, which invalidates directly.
    #[inline]
    fn sync(&mut self) {
        if self.session.pull_config() {
            self.invalidate();
        }
    }

    /// The rendered word as a C string, rendering it only if it is stale.
    #[inline]
    pub(crate) fn render_text(&mut self) -> *const c_char {
        self.sync();
        self.fill_rendered();
        self.rendered.as_ptr().cast()
    }

    /// The raw keystrokes as a C string, materialising them only if they are stale.
    #[inline]
    pub(crate) fn render_raw_text(&mut self) -> *const c_char {
        self.sync();
        self.fill_raw();
        self.raw.as_ptr().cast()
    }

    /// The rendered caret, in characters. No rendering: the core knows it.
    ///
    /// No `sync` either: the caret counts positions in an already-parsed word,
    /// and adopting a config does not re-parse the buffer, so a config change
    /// cannot move it.
    #[inline]
    pub(crate) fn cursor_char_idx(&mut self) -> usize {
        self.session.rendered_cursor()
    }

    /// The rendered caret, in UTF-8 bytes, derived from the rendered word.
    #[inline]
    pub(crate) fn cursor_byte_idx(&mut self) -> usize {
        self.sync();
        self.fill_rendered();
        let (byte_idx, _) = measure(content(&self.rendered), self.session.rendered_cursor());
        byte_idx
    }

    /// The raw caret, in keystrokes. No rendering: the core knows it.
    #[inline]
    pub(crate) fn raw_cursor_char_idx(&mut self) -> usize {
        self.session.raw_cursor()
    }

    /// The raw caret, in UTF-8 bytes, derived from the raw keystrokes.
    #[inline]
    pub(crate) fn raw_cursor_byte_idx(&mut self) -> usize {
        self.sync();
        self.fill_raw();
        let (byte_idx, _) = measure(content(&self.raw), self.session.raw_cursor());
        byte_idx
    }

    /// Whether the buffer spells a complete, valid Vietnamese syllable.
    ///
    /// No `sync`: validity is a phonotactic question about the parsed syllable,
    /// which neither the keymap nor the tone-placement scheme takes part in.
    #[inline]
    pub(crate) fn is_valid_vietnamese(&mut self) -> bool {
        self.session.is_phonotactically_valid()
    }

    /// A complete snapshot of everything a frontend needs for one repaint.
    ///
    /// Cheaper than the six individual getters, because each buffer is materialised
    /// at most once and each byte offset is one short walk over text that is already
    /// in hand. This is also the delivery point for the `*_to_delete` fields, which
    /// is why the previously-reported lengths are read out before they are replaced.
    pub(crate) fn render_state(&mut self) -> *const VimeRenderState {
        self.sync();
        self.fill_rendered();
        self.fill_raw();

        // One walk per buffer: the rendered text gives the caret byte offset and the
        // character count the next call will need to delete; the raw text gives its
        // own caret offset and nothing else. The core positions are character
        // indices, and each is what its own walk converts to a byte offset.
        let cursor_chars = self.session.rendered_cursor();
        let raw_cursor_chars = self.session.raw_cursor();
        let rendered = content(&self.rendered);
        let (cursor_byte_idx, len_chars) = measure(rendered, cursor_chars);
        let len_bytes = rendered.len();
        let (raw_cursor_byte_idx, _) = measure(content(&self.raw), raw_cursor_chars);

        self.state = VimeRenderState {
            text: self.rendered.as_ptr().cast(),
            raw_text: self.raw.as_ptr().cast(),
            cursor_byte_idx,
            cursor_char_idx: cursor_chars,
            raw_cursor_byte_idx,
            raw_cursor_char_idx: raw_cursor_chars,
            bytes_to_delete: self.delivered_bytes,
            chars_to_delete: self.delivered_chars,
            is_valid_vietnamese: self.session.is_phonotactically_valid(),
        };

        // What the host is told to erase is the text it was last handed, so this
        // call's text becomes the next call's `*_to_delete`.
        self.delivered_bytes = len_bytes;
        self.delivered_chars = len_chars;

        &self.state as *const VimeRenderState
    }

    /// Writes the rendered word into the buffer if it is stale.
    ///
    /// The core's writer appends straight into the buffer C will read, so the text
    /// is never materialised anywhere else first. It appends rather than replaces,
    /// which is what the `clear` is for; `clear` keeps the capacity, so this costs
    /// no allocation in the steady state.
    fn fill_rendered(&mut self) {
        if self.rendered_current {
            return;
        }
        self.rendered.clear();
        self.session.write_rendered_to(&mut self.rendered);
        self.rendered.reserve(1);
        self.rendered.push('\0');
        self.rendered_current = true;
    }

    /// Writes the raw keystrokes into the buffer if they are stale.
    fn fill_raw(&mut self) {
        if self.raw_current {
            return;
        }
        self.raw.clear();
        self.session.write_raw_to(&mut self.raw);
        self.raw.reserve(1);
        self.raw.push('\0');
        self.raw_current = true;
    }
}

/// Opaque handle to a session factory: one shared configuration, and the
/// sessions minted from it.
#[repr(C)]
pub struct VimeSessionFactoryHandle {
    pub(crate) factory: SessionFactory<FfiKeymap>,
}

impl VimeSessionFactoryHandle {
    pub(crate) fn new(config: FfiSessionConfig) -> Self {
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
    pub(crate) fn new_session_with(&self, config: FfiSessionConfig) -> EngineSession {
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
            SessionFactory::<FfiKeymap>::from_keymap(Settings::default(), DefaultKeymap::telex())
                .new_session(),
        )
    }

    fn type_text(handle: &mut VimeSessionHandle, text: &str) {
        for ch in text.chars() {
            handle.session.insert(ch);
            handle.invalidate();
        }
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
        type_text(&mut handle, "hoas");
        let ptr = handle.render_text();
        // SAFETY: the pointer is owned by the live handle, and the render is four
        // characters plus the terminator, so five bytes are in bounds.
        let bytes = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), 5) };
        assert_eq!(bytes, "hoá\0".as_bytes());
    }

    /// A buffer that is refilled in place: the address C was given last time is the
    /// address it gets this time, and the text behind it is the new one.
    #[test]
    fn refilling_reuses_the_allocation() {
        let mut handle = session();
        let first = handle.render_text();
        type_text(&mut handle, "toan");
        let second = handle.render_text();
        assert_eq!(first, second, "the allocation was reused in place");
        assert_eq!(text_of(second), "toan");
    }

    #[test]
    fn stale_text_is_replaced_not_appended() {
        let mut handle = session();
        type_text(&mut handle, "toan");
        assert_eq!(text_of(handle.render_text()), "toan");
        type_text(&mut handle, "hoa");
        assert_eq!(text_of(handle.render_text()), "toanhoa");
        handle.session.reset();
        handle.invalidate();
        assert_eq!(text_of(handle.render_text()), "");
    }

    /// A buffer's contents are its text without the terminator, and a word may end
    /// in U+0000 because that is a valid scalar value a caller is allowed to
    /// insert. Exactly one NUL comes off, so `a\0` is two characters and two bytes
    /// even though the C view of it is just `a`.
    #[test]
    fn the_terminator_is_stripped_exactly_once() {
        assert_eq!(content("a\0"), "a");
        assert_eq!(content("a\0\0"), "a\0");
        assert_eq!(content(""), "", "a buffer that has not been filled yet");
    }

    /// The two buffers are filled independently: asking for one does not pay for
    /// the other.
    #[test]
    fn each_buffer_is_filled_on_its_own_schedule() {
        let mut handle = session();
        type_text(&mut handle, "hoas");
        assert_eq!(text_of(handle.render_text()), "hoá");
        assert!(!handle.raw_current, "the raw buffer is still stale");
        assert_eq!(text_of(handle.render_raw_text()), "hoas");
        assert!(handle.raw_current);

        type_text(&mut handle, "c");
        assert!(!handle.rendered_current);
        assert!(!handle.raw_current);
    }

    /// Moving the caret changes the text's *position*, not the text, so the
    /// invalidation rule says there is nothing to drop. The offsets are derived
    /// from the core's caret, so they still follow it.
    #[test]
    fn moving_the_caret_leaves_the_text_alone() {
        let mut handle = session();
        type_text(&mut handle, "dduongf");
        assert_eq!(text_of(handle.render_text()), "đùong");
        assert!(*handle.session.move_cursor_left_by(1).rendered());
        assert_eq!(text_of(handle.render_text()), "đùong");
        assert_eq!(handle.cursor_char_idx(), 4);
        // 'đ' and 'ù' are two bytes each, so four characters in is six bytes.
        assert_eq!(handle.cursor_byte_idx(), 6);
    }
}
