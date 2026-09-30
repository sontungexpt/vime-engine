//! The C ABI over the VIME engine.
//!
//! `include/vime_engine.h` is the contract. Every symbol in it is defined here
//! with the same name and the same signature, and nothing else is exported.
//!
//! # The shape of the boundary
//!
//! ```text
//! C  ->  validate  ->  handle  ->  VIME core  ->  C-compatible result
//! ```
//!
//! Each exported function does the least that four steps need: check the pointer
//! and the arguments, take the handle's borrow, call the engine, and turn the
//! result into the `bool` / `NULL` / `0` the header documents. There is no
//! Vietnamese logic here at all — no parsing, no transformation, no validation,
//! no cursor arithmetic. The engine owns all of it, and this layer only moves
//! data across.
//!
//! # What the hot path costs
//!
//! A keystroke is
//!
//! ```text
//! validate -> insert -> mark the text stale -> return
//! ```
//!
//! and nothing else. No allocation, no rendering, no UTF-8 walking, no lock. The
//! text is only rendered when a frontend asks for it, and then only if it changed
//! since the last time somebody asked — which, for the usual
//! keystroke-then-repaint loop, means it is rendered once per keystroke and never
//! twice. Moving the caret is shorter still, because it changes neither string.
//! See [`handle`] for the buffers and why they live in the handle.
//!
//! # Failure values
//!
//! The header has no error channel beyond `bool`, `NULL`, `""` and `0`, so those
//! are the whole of it, and each is used where the header says it is:
//!
//! - a NULL handle is never dereferenced; every entry point checks first,
//! - an unknown enum discriminant, a NULL config where one is required, and a
//!   code point that is not a Unicode scalar all fail the documented way rather
//!   than being coerced into an enum or a `char`,
//! - a feature the engine does not have reports the header's neutral value and is
//!   named in its own doc comment, so a caller can tell "false because invalid"
//!   from "false because unimplemented" by reading the header, not by guessing.
//!
//! # Panics
//!
//! A Rust panic must not unwind into C: it would cross a frame the C side never
//! prepared for, and unwinding out of `extern "C"` aborts the process. Every
//! exported function is therefore wrapped in [`guard`], which catches a panic and
//! returns the same value the function returns for bad input. Nothing in the
//! layer is written to panic — the pointers are checked and the arguments are
//! validated — so the guard is there to contain a bug in the engine rather than
//! as a mechanism the code depends on. A handle that has panicked is left
//! unspecified; destroy it rather than reuse it.
//!
//! # Thread safety
//!
//! A handle is not synchronized and does not need to be. The ABI never says two
//! threads may use one handle, so no lock is added, and no handle is `Sync` by
//! accident. A `VimeSessionFactoryHandle` may be shared across threads only
//! through the engine's own shared configuration, which is where that support
//! already lives; sessions created from it are independent buffers and are as
//! single-threaded as the handle they came from.
//!
//! # Stability
//!
//! The exported names, signatures, struct layouts and enum values are fixed by
//! the header and locked by compile-time assertions in [`config`] and [`render`],
//! plus the C compile-and-link test in `tests/c_abi.rs`.

use std::ffi::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

mod config;
mod handle;
mod render;

pub use config::{
    VimeConfig, VimeInputMethod, VimeTonePlacement,
    VIME_INPUT_METHOD_TELEX, VIME_INPUT_METHOD_VNI, VIME_INPUT_METHOD_VIQR,
    VIME_TONE_PLACEMENT_MODERN, VIME_TONE_PLACEMENT_OLD,
};
pub use handle::{VimeSessionFactoryHandle, VimeSessionHandle};
pub use render::VimeRenderState;

/// Runs `body`, turning a panic into `fallback`.
///
/// The C ABI has no way to report a panic, so a panic has to become the same
/// answer the function gives for invalid input. `AssertUnwindSafe` is what lets a
/// panic through a function that holds a `&mut` handle; the handle is the only
/// thing that could be left inconsistent, and the caller is told to destroy it.
#[inline]
fn guard<T>(fallback: T, body: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(value) => value,
        Err(_) => fallback,
    }
}

// ─────────────────────────── factory lifecycle ───────────────────────────

/// Creates a new Session Factory with default configuration (Telex, Modern tone).
///
/// Returns NULL only if the allocation fails, which aborts rather than returning.
///
/// # Safety
///
/// The returned handle must be released with [`vime_session_factory_destroy`].
#[no_mangle]
pub extern "C" fn vime_session_factory_create() -> *mut VimeSessionFactoryHandle {
    guard(ptr::null_mut(), || {
        VimeSessionFactoryHandle::new(VimeConfig::init().to_engine_config()).into_raw()
    })
}

/// Creates a new Session Factory with custom initial configuration.
///
/// A NULL `config` is `VIME_CONFIG_INIT`, so "no configuration" and "the default
/// configuration" are the same request. Returns NULL if the configuration carries
/// an enum value the engine has no meaning for, and NULL if the allocation fails.
///
/// # Safety
///
/// `config` must be NULL or point to a readable `VimeConfig`, and the returned
/// handle must be released with [`vime_session_factory_destroy`].
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_create_with_config(
    config: *const VimeConfig,
) -> *mut VimeSessionFactoryHandle {
    guard(ptr::null_mut(), || {
        // SAFETY: the caller guarantees `config` is NULL or readable for the call.
        let Some(config) = (unsafe { VimeConfig::read(config) }) else {
            return ptr::null_mut();
        };
        VimeSessionFactoryHandle::new(config.to_engine_config()).into_raw()
    })
}

/// Destroys a Session Factory and frees associated memory.
///
/// Existing sessions created by the factory remain valid: they hold their own
/// reference to the shared configuration, and one taken privately is independent
/// of the factory entirely. A NULL factory is a no-op.
///
/// # Safety
///
/// `factory` must be NULL, or a handle from one of the factory constructors that
/// has not already been destroyed.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_destroy(factory: *mut VimeSessionFactoryHandle) {
    // SAFETY: the caller guarantees ownership of the handle passes to this call.
    drop(unsafe { VimeSessionFactoryHandle::into_box(factory) });
}

/// Updates the SharedConfig of the Factory.
///
/// All active sessions sharing this config (without private overrides) update on
/// their next operation, including a query that only reads the rendered text.
///
/// Returns false if the factory is NULL or the config carries an unknown enum
/// value; in the latter case the previous configuration is left in place.
///
/// # Safety
///
/// `factory` must be a live factory handle, and `config` must be NULL or point to
/// a readable `VimeConfig`.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_set_config(
    factory: *mut VimeSessionFactoryHandle,
    config: *const VimeConfig,
) -> bool {
    guard(false, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionFactoryHandle::from_raw(factory) }) else {
            return false;
        };
        if config.is_null() {
            return false;
        }
        // SAFETY: the caller guarantees `config` is readable for the call.
        let Some(config) = (unsafe { VimeConfig::read(config) }) else {
            return false;
        };
        handle.factory.set_config(config.to_engine_config());
        true
    })
}

// ─────────────────────────── session lifecycle ───────────────────────────

/// Creates a new Session bound to the Factory's SharedConfig.
///
/// The session follows later shared-configuration changes. Returns NULL if
/// `factory` is NULL.
///
/// # Safety
///
/// `factory` must be NULL or a live factory handle. The returned handle must be
/// released with [`vime_session_destroy`].
#[no_mangle]
pub unsafe extern "C" fn vime_session_create(
    factory: *mut VimeSessionFactoryHandle,
) -> *mut VimeSessionHandle {
    guard(ptr::null_mut(), || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionFactoryHandle::from_raw(factory) }) else {
            return ptr::null_mut();
        };
        VimeSessionHandle::new(handle.new_session()).into_raw()
    })
}

/// Creates a new Session with a private config override.
///
/// The session does not follow subsequent shared configuration changes until
/// [`vime_session_clear_config`] is called. A NULL `config` is `VIME_CONFIG_INIT`;
/// an unknown enum value fails the call and returns NULL rather than building a
/// session nobody asked for.
///
/// # Safety
///
/// `factory` must be NULL or a live factory handle, and `config` must be NULL or
/// point to a readable `VimeConfig`. The returned handle must be released with
/// [`vime_session_destroy`].
#[no_mangle]
pub unsafe extern "C" fn vime_session_create_with_config(
    factory: *mut VimeSessionFactoryHandle,
    config: *const VimeConfig,
) -> *mut VimeSessionHandle {
    guard(ptr::null_mut(), || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionFactoryHandle::from_raw(factory) }) else {
            return ptr::null_mut();
        };
        // SAFETY: the caller guarantees `config` is NULL or readable for the call.
        let Some(config) = (unsafe { VimeConfig::read(config) }) else {
            return ptr::null_mut();
        };
        VimeSessionHandle::new(handle.new_session_with(config.to_engine_config())).into_raw()
    })
}

/// Destroys a Session and frees its associated rendering caches.
///
/// Every pointer previously returned for this session becomes invalid. A NULL
/// session is a no-op.
///
/// # Safety
///
/// `session` must be NULL, or a handle from one of the session constructors that
/// has not already been destroyed.
#[no_mangle]
pub unsafe extern "C" fn vime_session_destroy(session: *mut VimeSessionHandle) {
    // SAFETY: the caller guarantees ownership of the handle passes to this call.
    drop(unsafe { VimeSessionHandle::into_box(session) });
}

// ────────────────────── session config isolation ──────────────────────

/// Sets a private config for the session, unlinking it from Factory SharedConfig.
///
/// Takes effect on the next operation. Returns false if the session or the config
/// is NULL, or if the config carries an unknown enum value — in which case the
/// session keeps the configuration it had.
///
/// # Safety
///
/// `session` must be a live session handle, and `config` must be NULL or point to
/// a readable `VimeConfig`.
#[no_mangle]
pub unsafe extern "C" fn vime_session_set_config(
    session: *mut VimeSessionHandle,
    config: *const VimeConfig,
) -> bool {
    guard(false, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        if config.is_null() {
            return false;
        }
        // SAFETY: the caller guarantees `config` is readable for the call.
        let Some(config) = (unsafe { VimeConfig::read(config) }) else {
            return false;
        };
        handle.session.set_private_config(config.to_engine_config());
        // A new configuration can re-render the word, so the cached text is now
        // stale even though no key was pressed.
        handle.invalidate();
        true
    })
}

/// Removes private config, reverting session to Factory SharedConfig.
///
/// Returns false if the session is NULL.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_clear_config(session: *mut VimeSessionHandle) -> bool {
    guard(false, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        handle.session.clear_private_config();
        handle.invalidate();
        true
    })
}

// ───────────────────────── input and editing ─────────────────────────

/// Inserts a single Unicode scalar value (UTF-32) at the current cursor position.
///
/// Returns false if the session is NULL or `character` is not a Unicode scalar
/// value, in which case nothing is inserted. A code point that is not a scalar
/// value — a surrogate, or anything above U+10FFFF — is rejected rather than
/// truncated into a different character.
///
/// U+0000 is a valid scalar value and is inserted like any other; it is the C
/// string returned by [`vime_session_render_text`] that stops at it, because that
/// is what a C string is.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_insert(
    session: *mut VimeSessionHandle,
    character: u32,
) -> bool {
    guard(false, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        let Some(character) = char::from_u32(character) else {
            return false;
        };
        handle.session.insert(character);
        handle.invalidate();
        true
    })
}

/// Performs a Backspace operation at the current cursor position.
///
/// Returns true if a character was deleted, false if the buffer is empty or the
/// session is NULL. True means a character left the rendered word, the raw
/// keystrokes, or both — the engine deletes from each buffer according to what it
/// holds, and the two are not always the same length.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_backspace(session: *mut VimeSessionHandle) -> bool {
    guard(false, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        let deleted = handle.session.backspace();
        handle.invalidate();
        *deleted.rendered() || *deleted.raw()
    })
}

/// Performs a Delete operation at the current cursor position.
///
/// Returns true if a character was deleted, false if there is nothing ahead of the
/// cursor or the session is NULL.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_delete(session: *mut VimeSessionHandle) -> bool {
    guard(false, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        let deleted = handle.session.delete();
        handle.invalidate();
        *deleted.rendered() || *deleted.raw()
    })
}

/// Moves cursor left by 1 character.
///
/// Returns true if the rendered caret moved, false if it is already at the
/// beginning of the word or the session is NULL. The reported caret is the
/// rendered one, matching [`vime_session_get_cursor_char_idx`]; the engine's raw
/// caret follows its own buffer, which can be somewhere else entirely after a
/// transform key consumed a keystroke.
///
/// The text is not re-rendered: a caret move rewrites neither string, and the
/// byte offset is derived from the caret when a caller asks for it.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_move_cursor_left(session: *mut VimeSessionHandle) -> bool {
    guard(false, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        let moved = handle.session.move_cursor_left();
        *moved.rendered()
    })
}

/// Moves cursor right by 1 character.
///
/// Returns true if the rendered caret moved, false if it is already at the end of
/// the word or the session is NULL. As with
/// [`vime_session_move_cursor_left`], the rendered caret is the one reported.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_move_cursor_right(session: *mut VimeSessionHandle) -> bool {
    guard(false, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        let moved = handle.session.move_cursor_right();
        *moved.rendered()
    })
}

// ────────────────────── render and state queries ──────────────────────

/// [RENDER TEXT] Gets the transformed Vietnamese UTF-8 display string.
///
/// Returns "" for an empty buffer, and NULL if the session is NULL — a NULL
/// return means "no session", which an empty word does not, so the two are
/// distinguishable.
///
/// The pointer is owned by the session, must not be freed, and stays valid until
/// the next call on this session. It is rendered on demand and cached, so calling
/// this twice with no edit in between costs one render in total.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_render_text(
    session: *mut VimeSessionHandle,
) -> *const c_char {
    guard(ptr::null(), || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return ptr::null();
        };
        handle.render_text()
    })
}

/// [RAW TEXT] Gets the raw UTF-8 sequence typed by user.
///
/// For `a` then `w` the rendered text is `ă` while this is `aw`, because a
/// transform key is consumed rather than becoming a character of its own. The
/// same lifetime rules as [`vime_session_render_text`] apply.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_render_raw_text(
    session: *mut VimeSessionHandle,
) -> *const c_char {
    guard(ptr::null(), || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return ptr::null();
        };
        handle.render_raw_text()
    })
}

/// [VALIDATION] Checks if current buffer conforms to Vietnamese orthography rules.
///
/// True when the buffer spells a complete Vietnamese syllable that the engine's
/// phonotactic rules accept — `ba`, `toan`, `ương` — and false while it is still a
/// fragment (`b`), while the parse has failed (`qwerty`), and for an empty buffer.
/// The verdict is the engine's own, so a word the engine's spelling rules reject
/// is reported invalid rather than being second-guessed here.
///
/// Returns false if the session is NULL.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_is_valid_vietnamese(session: *mut VimeSessionHandle) -> bool {
    guard(false, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        handle.is_valid_vietnamese()
    })
}

/// [RENDER CURSOR] Gets rendered cursor position in CodePoints (Unicode characters).
///
/// A character index into [`vime_session_render_text`], never a byte offset, and
/// never more than that text's length. Reported for a NULL session as 0.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_cursor_char_idx(
    session: *mut VimeSessionHandle,
) -> usize {
    guard(0, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return 0;
        };
        handle.cursor_char_idx()
    })
}

/// [RENDER CURSOR] Gets rendered cursor position in UTF-8 Byte offset.
///
/// A byte offset into [`vime_session_render_text`], always on a character
/// boundary, so a frontend may slice with it directly. Rendering the word is
/// what makes it available, and the result is cached, so this costs a render only
/// when the word has changed. Reported for a NULL session as 0.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_cursor_byte_idx(
    session: *mut VimeSessionHandle,
) -> usize {
    guard(0, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return 0;
        };
        handle.cursor_byte_idx()
    })
}

/// [RAW CURSOR] Gets raw cursor position in CodePoints (Unicode characters).
///
/// A keystroke index into [`vime_session_render_raw_text`]. It differs from
/// [`vime_session_get_cursor_char_idx`] whenever a transform key has been
/// consumed. Reported for a NULL session as 0.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_raw_cursor_char_idx(
    session: *mut VimeSessionHandle,
) -> usize {
    guard(0, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return 0;
        };
        handle.raw_cursor_char_idx()
    })
}

/// [RAW CURSOR] Gets raw cursor position in UTF-8 Byte offset.
///
/// A byte offset into [`vime_session_render_raw_text`], always on a character
/// boundary. Reported for a NULL session as 0.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_raw_cursor_byte_idx(
    session: *mut VimeSessionHandle,
) -> usize {
    guard(0, || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return 0;
        };
        handle.raw_cursor_byte_idx()
    })
}

/// [ADVANCED STATE] Retrieves a complete snapshot of current session render state.
///
/// Cheaper than the six individual getters, because each string is rendered at
/// most once and each byte offset is one short walk over text already in hand.
///
/// # The delete counts
///
/// `bytes_to_delete` and `chars_to_delete` are the length of the text this
/// session reported through *this function* the previous time it was called, in
/// bytes and in characters. A host integrating at the text level deletes that
/// much and then writes `text`, which is the whole point of a uinput integration.
/// Both are 0 on the first call, and the text this call reports becomes the next
/// call's delete count — so one repaint per keystroke is the cycle they describe.
/// [`vime_session_render_text`] deliberately does not move them, leaving that to
/// a caller that reads the text on its own schedule.
///
/// The lengths cover the whole rendered word, including any U+0000 a caller
/// inserted, even though `text` as a C string stops at the first one.
///
/// The pointer is owned by the session, must not be freed, and stays valid until
/// the next call on this session. Returns NULL if the session is NULL.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_render_state(
    session: *mut VimeSessionHandle,
) -> *const VimeRenderState {
    guard(ptr::null(), || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return ptr::null();
        };
        handle.render_state()
    })
}

/// Clears input buffers and resets session state to initial conditions.
///
/// The configuration in force is kept; only the typed input is discarded. A NULL
/// session is a no-op.
///
/// # Safety
///
/// `session` must be a live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_reset(session: *mut VimeSessionHandle) {
    guard((), || {
        // SAFETY: the caller guarantees a live handle; NULL is checked inside.
        let Some(handle) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return;
        };
        handle.session.reset();
        handle.invalidate();
    })
}

// ───────────────────────────── system info ─────────────────────────────

/// Gets the semver string of the VIME Engine C-FFI library.
///
/// Statically allocated at compile time, so this never allocates and the pointer
/// is valid for the life of the process. It is the FFI crate's own version.
///
/// # Safety
///
/// The returned pointer must not be freed.
#[no_mangle]
pub extern "C" fn vime_version() -> *const c_char {
    // A `&'static str` built at compile time, NUL-terminated by the literal.
    const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr().cast()
}

#[cfg(test)]
mod tests {
    use super::*;
    use vime_engine::{Session, SessionFactory};

    /// A session driven through the raw pointers, with no RAII wrapper, so the
    /// tests exercise the same path a C caller takes.
    fn raw_session() -> *mut VimeSessionHandle {
        // SAFETY: a live factory is created, used, and released here; the session
        // outlives the factory on purpose, which is the documented behaviour.
        let factory = vime_session_factory_create();
        assert!(!factory.is_null());
        let session = unsafe { vime_session_create(factory) };
        unsafe { vime_session_factory_destroy(factory) };
        assert!(!session.is_null());
        session
    }

    fn render_text(session: *mut VimeSessionHandle) -> String {
        // SAFETY: `session` is live for the duration of the call.
        let ptr = unsafe { vime_session_render_text(session) };
        assert!(!ptr.is_null());
        // SAFETY: the pointer is owned by the live session.
        unsafe { std::ffi::CStr::from_ptr(ptr) }
            .to_str()
            .unwrap()
            .to_owned()
    }

    fn type_text(session: *mut VimeSessionHandle, text: &str) -> String {
        // SAFETY: the handle is live.
        unsafe { vime_session_reset(session) };
        for ch in text.chars() {
            // SAFETY: the handle is live.
            assert!(unsafe { vime_session_insert(session, ch as u32) });
        }
        render_text(session)
    }

    #[test]
    fn a_session_outlives_its_factory() {
        // SAFETY: both handles are live and destroyed exactly once.
        unsafe {
            let session = raw_session();
            assert!(vime_session_insert(session, 'a' as u32));
            assert_eq!(render_text(session), "a");
            vime_session_destroy(session);
        }
    }

    #[test]
    fn the_engine_renders_what_the_core_renders() {
        // SAFETY: the handle is live and destroyed exactly once.
        unsafe {
            let session = raw_session();
            assert_eq!(type_text(session, "hoas"), "hoá");
            assert_eq!(type_text(session, "uowng"), "ương");
            // Input the parser cannot read comes back verbatim.
            assert_eq!(type_text(session, "qwerty"), "qwerty");
            vime_session_destroy(session);
        }
    }

    #[test]
    fn the_version_is_static_and_nul_terminated() {
        // SAFETY: the returned pointer is static and must not be freed.
        let ptr = unsafe_free_version();
        // SAFETY: `vime_version` documents a static NUL-terminated string.
        let text = unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap();
        assert_eq!(text, env!("CARGO_PKG_VERSION"));
        // Stable across calls, because it is the same static.
        assert_eq!(ptr, unsafe_free_version());
    }

    fn unsafe_free_version() -> *const c_char {
        vime_version()
    }

    #[test]
    fn a_core_render_and_an_ffi_render_agree() {
        // The layer must not be able to invent text the engine did not produce.
        let mut core: Session<vime_engine::DefaultKeymap<'static>> =
            SessionFactory::telex(vime_engine::Settings::default()).new_session();
        // SAFETY: the handle is live and destroyed exactly once.
        let session = raw_session();
        for word in ["hoas", "uowng", "thuowng", "nguoi", "toan", "qwerty"] {
            core.reset();
            // SAFETY: the handle is live.
            unsafe { vime_session_reset(session) };
            for ch in word.chars() {
                core.insert(ch);
                // SAFETY: the handle is live.
                assert!(unsafe { vime_session_insert(session, ch as u32) });
            }
            let from_core: String = core.rendered().into_iter().collect();
            assert_eq!(render_text(session), from_core, "{word:?}");
        }
        // SAFETY: the handle is live.
        unsafe { vime_session_destroy(session) };
    }
}
