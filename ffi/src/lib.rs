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

/// Runs an FFI operation across the Rust panic boundary.
///
/// A Rust panic must never unwind into C. If the operation panics, the caller
/// receives the supplied fallback value instead.
#[inline(always)]
fn catch_panic<T>(fallback: T, body: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(value) => value,
        Err(_) => fallback,
    }
}

/// Creates a new session factory using the default configuration.
#[no_mangle]
pub extern "C" fn vime_session_factory_create() -> *mut VimeSessionFactoryHandle {
    catch_panic(ptr::null_mut(), || {
        VimeSessionFactoryHandle::new(VimeConfig::default().to_ffi_session_config()).into_raw()
    })
}

/// Creates a new session factory using a custom configuration.
///
/// Returns NULL when `config` is NULL, invalid, or allocation fails.
///
/// # Safety
///
/// `config` must be NULL or point to a readable `VimeConfig` for the duration
/// of this call.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_create_with_config(
    config: *const VimeConfig,
) -> *mut VimeSessionFactoryHandle {
    catch_panic(ptr::null_mut(), || {
        if config.is_null() {
            return ptr::null_mut();
        }

        // SAFETY: The caller guarantees that `config` points to readable memory.
        let Some(config) = (unsafe { VimeConfig::read(config) }) else {
            return ptr::null_mut();
        };

        VimeSessionFactoryHandle::new(config.to_ffi_session_config()).into_raw()
    })
}

/// Destroys a session factory.
///
/// Passing NULL is a no-op.
///
/// Sessions created by the factory remain valid if the core's shared
/// configuration is reference-counted independently.
///
/// # Safety
///
/// `factory` must be NULL or a live factory handle whose ownership is
/// transferred to this call.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_destroy(factory: *mut VimeSessionFactoryHandle) {
    let _ = catch_panic((), || {
        // SAFETY: The caller transfers ownership of the factory to this call.
        drop(unsafe { VimeSessionFactoryHandle::into_box(factory) });
    });
}

/// Replaces the factory's shared configuration.
///
/// Sessions using the shared configuration observe the new configuration
/// according to the core engine's configuration propagation rules.
///
/// Returns false for NULL handles or invalid configurations.
///
/// # Safety
///
/// `factory` must be NULL or a valid live factory handle.
///
/// `config` must be NULL or point to a readable `VimeConfig`.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_set_config(
    factory: *mut VimeSessionFactoryHandle,
    config: *const VimeConfig,
) -> bool {
    catch_panic(false, || {
        let Some(factory) =
            // SAFETY: The caller guarantees that the handle is valid.
            (unsafe { VimeSessionFactoryHandle::from_raw(factory) })
        else {
            return false;
        };

        if config.is_null() {
            return false;
        }

        // SAFETY: The caller guarantees that `config` is readable.
        let Some(config) = (unsafe { VimeConfig::read(config) }) else {
            return false;
        };

        factory.set_config(config.to_ffi_session_config());
        true
    })
}

/// Returns the factory's current shared configuration.
///
/// Returns the default configuration when `factory` is NULL.
///
/// # Safety
///
/// `factory` must be NULL or a valid live factory handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_get_config(
    factory: *const VimeSessionFactoryHandle,
) -> VimeConfig {
    catch_panic(VimeConfig::default(), || {
        let Some(factory) =
            // SAFETY: The caller guarantees that the handle is valid.
            (unsafe { VimeSessionFactoryHandle::from_raw_const(factory) })
        else {
            return VimeConfig::default();
        };

        VimeConfig::from_ffi_session_config(factory.config())
    })
}

/// Creates a session using the factory's shared configuration.
///
/// Returns NULL when `factory` is NULL or allocation fails.
///
/// # Safety
///
/// `factory` must be NULL or a valid live factory handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_create(
    factory: *mut VimeSessionFactoryHandle,
) -> *mut VimeSessionHandle {
    catch_panic(ptr::null_mut(), || {
        let Some(factory) =
            // SAFETY: The caller guarantees that the handle is valid.
            (unsafe { VimeSessionFactoryHandle::from_raw(factory) })
        else {
            return ptr::null_mut();
        };

        VimeSessionHandle::new(factory.new_session()).into_raw()
    })
}

/// Creates a session with a private configuration.
///
/// The new session does not follow subsequent shared configuration changes
/// until its private configuration is cleared.
///
/// # Safety
///
/// `factory` must be NULL or a valid live factory handle.
///
/// `config` must be NULL or point to a readable `VimeConfig`.
#[no_mangle]
pub unsafe extern "C" fn vime_session_create_with_config(
    factory: *mut VimeSessionFactoryHandle,
    config: *const VimeConfig,
) -> *mut VimeSessionHandle {
    catch_panic(ptr::null_mut(), || {
        let Some(factory) =
            // SAFETY: The caller guarantees that the handle is valid.
            (unsafe { VimeSessionFactoryHandle::from_raw(factory) })
        else {
            return ptr::null_mut();
        };

        if config.is_null() {
            return ptr::null_mut();
        }

        // SAFETY: The caller guarantees that `config` is readable.
        let Some(config) = (unsafe { VimeConfig::read(config) }) else {
            return ptr::null_mut();
        };

        let session = factory.new_session_with(config.to_ffi_session_config());

        VimeSessionHandle::new(session).into_raw()
    })
}

/// Destroys a session.
///
/// Passing NULL is a no-op.
///
/// # Safety
///
/// `session` must be NULL or a live session handle whose ownership is
/// transferred to this call.
#[no_mangle]
pub unsafe extern "C" fn vime_session_destroy(session: *mut VimeSessionHandle) {
    let _ = catch_panic((), || {
        // SAFETY: The caller transfers ownership to this call.
        drop(unsafe { VimeSessionHandle::into_box(session) });
    });
}

/// Resets the session's composition and cursor state.
///
/// The effective configuration is preserved.
///
/// Passing NULL is a no-op.
///
/// # Safety
///
/// `session` must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_reset(session: *mut VimeSessionHandle) {
    let _ = catch_panic((), || {
        let Some(session) =
            // SAFETY: The caller guarantees that the handle is valid.
            (unsafe { VimeSessionHandle::from_raw(session) })
        else {
            return;
        };

        session.reset();
    });
}

/// Assigns a private configuration to a session.
///
/// Returns false when either pointer is NULL or the configuration is invalid.
///
/// # Safety
///
/// `session` must be NULL or a valid live session handle.
///
/// `config` must be NULL or point to a readable `VimeConfig`.
#[no_mangle]
pub unsafe extern "C" fn vime_session_set_config(
    session: *mut VimeSessionHandle,
    config: *const VimeConfig,
) -> bool {
    catch_panic(false, || {
        let Some(session) =
            // SAFETY: The caller guarantees that the handle is valid.
            (unsafe { VimeSessionHandle::from_raw(session) })
        else {
            return false;
        };

        if config.is_null() {
            return false;
        }

        // SAFETY: The caller guarantees that `config` is readable.
        let Some(config) = (unsafe { VimeConfig::read(config) }) else {
            return false;
        };

        session.set_private_config(config.to_ffi_session_config());
        true
    })
}

/// Removes the session's private configuration.
///
/// The session resumes following the factory's shared configuration.
///
/// Returns false when `session` is NULL.
///
/// # Safety
///
/// `session` must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_clear_config(session: *mut VimeSessionHandle) -> bool {
    catch_panic(false, || {
        let Some(session) =
            // SAFETY: The caller guarantees that the handle is valid.
            (unsafe { VimeSessionHandle::from_raw(session) })
        else {
            return false;
        };

        session.clear_private_config();
        true
    })
}

/// Moves the cursor left by up to `by` Unicode scalar positions.
///
/// Returns true when the cursor moved.
///
/// # Safety
///
/// `session` must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_move_cursor_left(
    session: *mut VimeSessionHandle,
    by: u32,
) -> bool {
    catch_panic(false, || {
        let Some(session) =
            // SAFETY: The caller guarantees that the handle is valid.
            (unsafe { VimeSessionHandle::from_raw(session) })
        else {
            return false;
        };

        session.move_cursor_left_by(by as usize)
    })
}

/// Moves the cursor right by up to `by` Unicode scalar positions.
///
/// Returns true when the cursor moved.
///
/// # Safety
///
/// `session` must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_move_cursor_right(
    session: *mut VimeSessionHandle,
    by: u32,
) -> bool {
    catch_panic(false, || {
        let Some(session) =
            // SAFETY: The caller guarantees that the handle is valid.
            (unsafe { VimeSessionHandle::from_raw(session) })
        else {
            return false;
        };

        session.move_cursor_right_by(by as usize)
    })
}

/// Inserts one Unicode scalar value at the current cursor.
///
/// `character` is UTF-32, not UTF-8.
///
/// Returns true when the session state changed.
///
/// # Safety
///
/// `session` must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_insert(
    session: *mut VimeSessionHandle,
    character: u32,
) -> VimeInsertResult {
    catch_panic(VimeInsertResult::invalid(), || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return VimeInsertResult::invalid();
        };

        session.insert(character)
    })
}

#[no_mangle]
pub unsafe extern "C" fn vime_session_backspace(session: *mut VimeSessionHandle) -> bool {
    catch_panic(false, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };

        session.backspace()
    })
}

#[no_mangle]
pub unsafe extern "C" fn vime_session_delete(session: *mut VimeSessionHandle) -> bool {
    catch_panic(false, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };

        session.delete()
    })
}

/// Returns the caret position in the rendered buffer, in characters from the
/// start.
///
/// Returns 0 when `session` is NULL.
///
/// # Safety
///
/// `session` must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_rendered_cursor(
    session: *mut VimeSessionHandle,
) -> usize {
    catch_panic(0, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw_const(session) }) else {
            return 0;
        };

        session.rendered_cursor()
    })
}

/// Returns the caret position in the raw keystroke buffer, in keystrokes from
/// the start.
///
/// Not interchangeable with [`vime_session_get_rendered_cursor`]: a transform
/// consumes a keystroke without lengthening the rendered word, so the two
/// positions need not agree.
///
/// Returns 0 when `session` is NULL.
///
/// # Safety
///
/// `session` must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_raw_cursor(session: *mut VimeSessionHandle) -> usize {
    catch_panic(0, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw_const(session) }) else {
            return 0;
        };

        session.raw_cursor()
    })
}

/// Returns the length of the rendered buffer, in characters.
///
/// Returns 0 when `session` is NULL or the buffer is empty.
///
/// # Safety
///
/// `session` must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_rendered_len(session: *mut VimeSessionHandle) -> usize {
    catch_panic(0, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw_const(session) }) else {
            return 0;
        };

        session.rendered_len()
    })
}

/// Returns the length of the rendered buffer, in UTF-8 bytes.
///
/// Returns 0 when `session` is NULL or the buffer is empty.
///
/// # Safety
///
/// `session` must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_rendered_len_utf8(
    session: *mut VimeSessionHandle,
) -> usize {
    catch_panic(0, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw_const(session) }) else {
            return 0;
        };

        session.rendered_len_utf8()
    })
}

/// Returns the length of the raw keystroke buffer, in keystrokes.
///
/// Returns 0 when `session` is NULL or the buffer is empty.
///
/// # Safety
///
/// `session` must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_raw_len(session: *mut VimeSessionHandle) -> usize {
    catch_panic(0, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw_const(session) }) else {
            return 0;
        };

        session.raw_len()
    })
}

/// Returns the length of the raw keystroke buffer, in UTF-8 bytes.
///
/// Returns 0 when `session` is NULL or the buffer is empty.
///
/// # Safety
///
/// `session` must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_raw_len_utf8(session: *mut VimeSessionHandle) -> usize {
    catch_panic(0, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw_const(session) }) else {
            return 0;
        };

        session.raw_len_utf8()
    })
}

/// Reports whether the cursor can move one character left.
/// Reports whether the cursor can move one character left.
///
/// Returns false when  is NULL or the cursor is already at the start.
///
/// # Safety
///
///  must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_can_move_cursor_left(
    session: *mut VimeSessionHandle,
) -> bool {
    catch_panic(false, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw_const(session) }) else {
            return false;
        };

        session.can_move_cursor_left()
    })
}

/// Reports whether the cursor can move one character right.
///
/// Returns false when  is NULL or the cursor is already at the end.
///
/// # Safety
///
///  must be NULL or a valid live session handle.
#[no_mangle]
pub unsafe extern "C" fn vime_session_can_move_cursor_right(
    session: *mut VimeSessionHandle,
) -> bool {
    catch_panic(false, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw_const(session) }) else {
            return false;
        };

        session.can_move_cursor_right()
    })
}

#[no_mangle]
pub unsafe extern "C" fn vime_session_is_valid_vietnamese(session: *mut VimeSessionHandle) -> bool {
    catch_panic(false, || {
        let Some(session) = (unsafe { VimeSessionHandle::from_raw(session) }) else {
            return false;
        };

        session.is_valid_vietnamese()
    })
}

/// Returns the VIME engine semantic version.
///
/// The returned pointer refers to static storage and must not be freed.
#[no_mangle]
pub extern "C" fn vime_version() -> *const std::ffi::c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}
