use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

mod config;
mod factory;
mod session;

pub use config::{
    VimeConfig, VimeInputMethod, VimeTonePlacement, VIME_INPUT_METHOD_TELEX,
    VIME_INPUT_METHOD_VIQR, VIME_INPUT_METHOD_VNI, VIME_TONE_PLACEMENT_MODERN,
    VIME_TONE_PLACEMENT_OLD,
};

pub use factory::VimeSessionFactoryHandle;

pub use session::{VimeInsertResult, VimeSessionHandle};

/// Runs an FFI operation across the Rust panic boundary, returning fallback on panic.
#[inline(always)]
fn ffi_guard<T>(fallback: T, body: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(value) => value,
        Err(_) => fallback,
    }
}

/// Creates a session factory with default config (Telex, Modern tone).
#[no_mangle]
pub extern "C" fn vime_session_factory_create() -> *mut VimeSessionFactoryHandle {
    ffi_guard(ptr::null_mut(), || {
        VimeSessionFactoryHandle::new(VimeConfig::default().to_ffi_session_config()).into_raw()
    })
}

/// Creates a session factory with custom config.
///
/// Returns NULL if config is NULL, invalid, or allocation fails.
///
/// # Safety
/// `config` must be NULL or point to a readable `VimeConfig`.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_create_with_config(
    config: *const VimeConfig,
) -> *mut VimeSessionFactoryHandle {
    ffi_guard(ptr::null_mut(), || {
        if config.is_null() {
            return ptr::null_mut();
        }
        let Some(config) = (unsafe { VimeConfig::read(config) }) else {
            return ptr::null_mut();
        };
        VimeSessionFactoryHandle::new(config.to_ffi_session_config()).into_raw()
    })
}

/// Destroys a session factory. NULL is a no-op.
///
/// Sessions remain valid if core's shared config is reference-counted independently.
///
/// # Safety
/// `factory` must be NULL or a live handle whose ownership is transferred.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_destroy(factory: *mut VimeSessionFactoryHandle) {
    ffi_guard((), || {
        drop(unsafe { VimeSessionFactoryHandle::into_box(factory) })
    });
}

/// Replaces the factory's shared configuration.
///
/// Sessions using shared config observe changes per core engine's propagation rules.
/// Returns false for NULL handles or invalid configs.
///
/// # Safety
/// `factory` must be NULL or a valid live handle.
/// `config` must be NULL or point to a readable `VimeConfig`.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_set_config(
    factory: *mut VimeSessionFactoryHandle,
    config: *const VimeConfig,
) -> bool {
    ffi_guard(false, || {
        let Some(factory) = (unsafe { VimeSessionFactoryHandle::from_raw(factory) }) else {
            return false;
        };
        if config.is_null() {
            return false;
        }
        let Some(config) = (unsafe { VimeConfig::read(config) }) else {
            return false;
        };
        factory.set_config(config.to_ffi_session_config());
        true
    })
}

/// Returns the factory's current shared configuration (default if NULL).
///
/// # Safety
/// `factory` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_get_config(
    factory: *const VimeSessionFactoryHandle,
) -> VimeConfig {
    ffi_guard(VimeConfig::default(), || {
        let Some(factory) = (unsafe { VimeSessionFactoryHandle::from_raw_const(factory) }) else {
            return VimeConfig::default();
        };
        VimeConfig::from_ffi_session_config(factory.config())
    })
}

/// Creates a session using the factory's shared configuration.
///
/// Returns NULL if factory is NULL or allocation fails.
///
/// # Safety
/// `factory` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_create(
    factory: *mut VimeSessionFactoryHandle,
) -> *mut VimeSessionHandle {
    ffi_guard(ptr::null_mut(), || {
        let Some(factory) = (unsafe { VimeSessionFactoryHandle::from_raw(factory) }) else {
            return ptr::null_mut();
        };
        VimeSessionHandle::new(factory.new_session()).into_raw()
    })
}

/// Creates a session with a private config (does not follow shared changes until cleared).
///
/// # Safety
/// `factory` must be NULL or a valid live handle.
/// `config` must be NULL or point to a readable `VimeConfig`.
#[no_mangle]
pub unsafe extern "C" fn vime_session_create_with_config(
    factory: *mut VimeSessionFactoryHandle,
    config: *const VimeConfig,
) -> *mut VimeSessionHandle {
    ffi_guard(ptr::null_mut(), || {
        let Some(factory) = (unsafe { VimeSessionFactoryHandle::from_raw(factory) }) else {
            return ptr::null_mut();
        };
        if config.is_null() {
            return ptr::null_mut();
        }
        let Some(config) = (unsafe { VimeConfig::read(config) }) else {
            return ptr::null_mut();
        };
        let session = factory.new_session_with(config.to_ffi_session_config());
        VimeSessionHandle::new(session).into_raw()
    })
}

/// Destroys a session. NULL is a no-op.
///
/// # Safety
/// `session` must be NULL or a live handle whose ownership is transferred.
#[no_mangle]
pub unsafe extern "C" fn vime_session_destroy(session: *mut VimeSessionHandle) {
    ffi_guard((), || drop(unsafe { VimeSessionHandle::into_box(session) }));
}

/// Resets the session's composition and cursor state. Config is preserved.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_reset(session: *mut VimeSessionHandle) {
    ffi_guard((), || {
        if let Some(session) = unsafe { VimeSessionHandle::from_raw(session) } {
            session.reset();
        }
    });
}

/// Assigns a private config to a session (stops following shared config).
/// Returns false if either pointer is NULL or config is invalid.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
/// `config` must be NULL or point to a readable `VimeConfig`.
#[no_mangle]
pub unsafe extern "C" fn vime_session_set_config(
    session: *mut VimeSessionHandle,
    config: *const VimeConfig,
) -> bool {
    ffi_guard(false, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        if config.is_null() {
            return false;
        }
        let Some(config) = (unsafe { VimeConfig::read(config) }) else {
            return false;
        };
        session.set_private_config(config.to_ffi_session_config());
        true
    })
}

/// Removes private config; session resumes following factory's shared config.
/// Returns false if session is NULL.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_clear_config(session: *mut VimeSessionHandle) -> bool {
    ffi_guard(false, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        session.clear_private_config();
        true
    })
}

/// Moves cursor left by up to `by` Unicode scalar positions. Returns true if moved.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_move_cursor_left(
    session: *mut VimeSessionHandle,
    by: u32,
) -> bool {
    ffi_guard(false, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        session.move_cursor_left_by(by as usize)
    })
}

/// Moves cursor right by up to `by` Unicode scalar positions. Returns true if moved.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_move_cursor_right(
    session: *mut VimeSessionHandle,
    by: u32,
) -> bool {
    ffi_guard(false, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        session.move_cursor_right_by(by as usize)
    })
}

/// Inserts one Unicode scalar (UTF-32) at the current cursor.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_insert(
    session: *mut VimeSessionHandle,
    character: u32,
) -> VimeInsertResult {
    ffi_guard(VimeInsertResult::invalid(), || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return VimeInsertResult::invalid();
        };
        session.insert(character)
    })
}

/// Performs Backspace at the current cursor.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_backspace(session: *mut VimeSessionHandle) -> bool {
    ffi_guard(false, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        session.backspace()
    })
}

/// Performs Delete at the current cursor.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_delete(session: *mut VimeSessionHandle) -> bool {
    ffi_guard(false, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };
        session.delete()
    })
}

/// Returns caret position in rendered buffer (chars from start). Returns 0 if NULL.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_rendered_cursor(
    session: *mut VimeSessionHandle,
) -> usize {
    ffi_guard(0, || {
        unsafe { VimeSessionHandle::from_raw_const(session) }.map_or(0, |s| s.rendered_cursor())
    })
}

/// Returns caret position in raw keystroke buffer (keystrokes from start).
/// Not interchangeable with rendered cursor: a transform consumes a keystroke
/// without lengthening the rendered word.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_raw_cursor(session: *mut VimeSessionHandle) -> usize {
    ffi_guard(0, || {
        unsafe { VimeSessionHandle::from_raw_const(session) }.map_or(0, |s| s.raw_cursor())
    })
}

/// Returns rendered buffer length in chars. Returns 0 if NULL or empty.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_rendered_len(session: *mut VimeSessionHandle) -> usize {
    ffi_guard(0, || {
        unsafe { VimeSessionHandle::from_raw_const(session) }.map_or(0, |s| s.rendered_len())
    })
}

/// Returns rendered buffer length in UTF-8 bytes. Returns 0 if NULL or empty.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_rendered_len_utf8(
    session: *mut VimeSessionHandle,
) -> usize {
    ffi_guard(0, || {
        unsafe { VimeSessionHandle::from_raw_const(session) }.map_or(0, |s| s.rendered_len_utf8())
    })
}

/// Returns raw keystroke buffer length in keystrokes. Returns 0 if NULL or empty.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_raw_len(session: *mut VimeSessionHandle) -> usize {
    ffi_guard(0, || {
        unsafe { VimeSessionHandle::from_raw_const(session) }.map_or(0, |s| s.raw_len())
    })
}

/// Returns raw keystroke buffer length in UTF-8 bytes. Returns 0 if NULL or empty.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_raw_len_utf8(session: *mut VimeSessionHandle) -> usize {
    ffi_guard(0, || {
        unsafe { VimeSessionHandle::from_raw_const(session) }.map_or(0, |s| s.raw_len_utf8())
    })
}

/// Reports whether cursor can move one char left. False if NULL or at start.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_can_move_cursor_left(
    session: *mut VimeSessionHandle,
) -> bool {
    ffi_guard(false, || {
        unsafe { VimeSessionHandle::from_raw_const(session) }
            .map_or(false, |s| s.can_move_cursor_left())
    })
}

/// Reports whether cursor can move one char right. False if NULL or at end.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_can_move_cursor_right(
    session: *mut VimeSessionHandle,
) -> bool {
    ffi_guard(false, || {
        unsafe { VimeSessionHandle::from_raw_const(session) }
            .map_or(false, |s| s.can_move_cursor_right())
    })
}

/// Checks if buffer spells a valid Vietnamese syllable.
///
/// # Safety
/// `session` must be NULL or a valid live handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_is_valid_vietnamese(session: *mut VimeSessionHandle) -> bool {
    ffi_guard(false, || {
        unsafe { VimeSessionHandle::from_raw(session) }.map_or(false, |s| s.is_valid_vietnamese())
    })
}

/// Returns the VIME engine semantic version (static storage, do not free).
#[no_mangle]
pub extern "C" fn vime_version() -> *const std::ffi::c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}
