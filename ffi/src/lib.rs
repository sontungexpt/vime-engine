use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

mod config;
mod handle;
mod render;

pub use config::{
    VimeConfig, VimeInputMethod, VimeTonePlacement, VIME_INPUT_METHOD_TELEX,
    VIME_INPUT_METHOD_VIQR, VIME_INPUT_METHOD_VNI, VIME_TONE_PLACEMENT_MODERN,
    VIME_TONE_PLACEMENT_OLD,
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
    VimeSessionFactoryHandle::new(VimeConfig::default().to_ffi_session_config()).into_raw()
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
    if config.is_null() {
        return ptr::null_mut();
    }

    // SAFETY: the caller guarantees `config` is NULL or readable for the call.
    let Some(config) = (unsafe { VimeConfig::read(config) }) else {
        return ptr::null_mut();
    };

    VimeSessionFactoryHandle::new(config.to_ffi_session_config()).into_raw()
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

/// Updates the SharedSessionConfig of the Factory.
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
    let handle = match unsafe { VimeSessionFactoryHandle::from_raw(factory) } {
        Some(handle) => handle,
        None => return false,
    };

    if config.is_null() {
        return false;
    }

    // SAFETY: the caller guarantees `config` is readable for the call.
    let Some(config) = (unsafe { VimeConfig::read(config) }) else {
        return false;
    };

    handle.set_config(config.to_ffi_session_config());
    true
}
