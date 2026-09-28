//! The C ABI: every `vime_*` function here is `extern "C"` and safe to call
//! from a frontend that speaks the header in `include/vime_engine.h`.
//!
//! Two rules hold throughout, and both are what the handle exists to provide:
//!
//! - **A NULL handle is never a crash.** Every entry point checks and returns a
//!   neutral value: an empty [`VimeOutput`], a NULL string, or `false`.
//! - **Text is read separately from the action that announces it.** A call that
//!   changes state returns only an action; the frontend then asks
//!   [`vime_parsed`] or [`vime_committed`] for the text it actually wants. The
//!   word is rendered on that first read, so a frontend that never displays
//!   it never pays for it.

use std::ffi::c_char;
use std::ptr;

use vime_engine::composition::syllable::SyllableContext;
use vime_engine::phonology::TonePlacement;
use vime_engine::{Config, DefaultKeymap, Engine, KeyEvent};

pub mod convert;
pub mod types;

pub use types::{
    VimeAction, VimeEngineHandle, VimeInputMethod, VimeKey, VimeKeyEvent, VimeOutput,
    VimeTonePlacement,
};

/// Binds a possibly-NULL handle, or returns `default` from the caller.
///
/// Every entry point needs this, and spelling it out each time is both noisy
/// and easy to forget on a new one.
macro_rules! handle_or {
    ($ptr:expr, $default:expr) => {
        match $ptr.as_mut() {
            Some(handle) => handle,
            None => return $default,
        }
    };
}

/// The built-in keymap for `method`, or `None` if the discriminant is unknown.
///
/// A `repr(u32)` enum can hold any value a C caller passes, so the trailing
/// match arm is reachable from C even though it is not from Rust.
fn keymap_for(method: VimeInputMethod) -> Option<DefaultKeymap<'static>> {
    #[allow(unreachable_patterns)]
    match method {
        VimeInputMethod::Telex => Some(DefaultKeymap::telex()),
        VimeInputMethod::Vni => Some(DefaultKeymap::vni()),
        VimeInputMethod::Viqr => Some(DefaultKeymap::viqr()),
        _ => None,
    }
}

/// Creates a Telex engine with the default configuration.
#[no_mangle]
pub extern "C" fn vime_create() -> *mut VimeEngineHandle {
    VimeEngineHandle::new(Engine::telex(Config::default())).into_raw()
}

/// Creates an engine for any built-in input method with the given
/// tone-placement scheme. Returns NULL for an unknown input method.
#[no_mangle]
pub extern "C" fn vime_create_with(
    method: VimeInputMethod,
    tone_placement: VimeTonePlacement,
) -> *mut VimeEngineHandle {
    let (Some(keymap), Ok(tone_placement)) =
        (keymap_for(method), TonePlacement::try_from(tone_placement))
    else {
        return ptr::null_mut();
    };
    let engine = Engine::with_context(
        Config::default(),
        SyllableContext::new(keymap, tone_placement),
    );
    VimeEngineHandle::new(engine).into_raw()
}

/// Destroys an engine instance, invalidating every pointer it handed out.
///
/// # Safety
///
/// `engine` must be NULL or a live pointer from [`vime_create`] /
/// [`vime_create_with`]. Passing NULL is allowed; passing an already-destroyed
/// pointer is not, and will double-free.
#[no_mangle]
pub unsafe extern "C" fn vime_destroy(engine: *mut VimeEngineHandle) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

/// Clears the buffer, returning the action for the resulting empty word.
///
/// # Safety
///
/// `engine` must be NULL or a live pointer from [`vime_create`] /
/// [`vime_create_with`] that has not been passed to [`vime_destroy`].
#[no_mangle]
pub unsafe extern "C" fn vime_reset(engine: *mut VimeEngineHandle) -> VimeOutput {
    let engine = handle_or!(engine, VimeOutput::default());
    let result = engine.engine.reset();
    engine.output(result)
}

/// Returns the word the engine currently has parsed, rendered on demand.
///
/// The action returned by the last call decides whether this is worth asking
/// for: read it on `VIME_ACTION_CHANGED` and
/// `VIME_ACTION_CURSOR_MOVED`, skip it otherwise. The text is cached after the
/// first call, so asking twice between state changes costs one render.
///
/// The returned pointer is owned by the handle and is invalidated by the next
/// call that changes the state, or by `vime_destroy`. It must not be freed by
/// the caller.
///
/// # Safety
///
/// `engine` must be NULL or a live pointer from [`vime_create`] /
/// [`vime_create_with`] that has not been passed to [`vime_destroy`].
#[no_mangle]
pub unsafe extern "C" fn vime_parsed(engine: *mut VimeEngineHandle) -> *const c_char {
    let engine = handle_or!(engine, ptr::null());
    engine.parsed_ptr()
}

/// Returns the text to commit, as produced by the last `VIME_ACTION_COMMIT`.
///
/// Committing happens while processing the key, so this only *reports* the text
/// that processing produced; it never commits anything itself. Returns NULL
/// when nothing is pending — notably after any other action, which clears the
/// pending commit.
///
/// The returned pointer is owned by the handle and is invalidated by the next
/// call that changes the state, or by `vime_destroy`. Do not free it.
///
/// # Safety
///
/// `engine` must be NULL or a live pointer from [`vime_create`] /
/// [`vime_create_with`] that has not been passed to [`vime_destroy`].
#[no_mangle]
pub unsafe extern "C" fn vime_committed(engine: *mut VimeEngineHandle) -> *const c_char {
    let engine = handle_or!(engine, ptr::null());
    engine.committed_ptr()
}

/// Processes a key event and reports what the frontend should do about it.
///
/// # Safety
///
/// `engine` must be NULL or a live pointer from [`vime_create`] /
/// [`vime_create_with`] that has not been passed to [`vime_destroy`].
#[no_mangle]
pub unsafe extern "C" fn vime_process_key(
    engine: *mut VimeEngineHandle,
    event: VimeKeyEvent,
) -> VimeOutput {
    let engine = handle_or!(engine, VimeOutput::default());

    let Ok(key_event) = KeyEvent::try_from(event) else {
        return VimeOutput::default();
    };

    let result = engine.engine.process_key(key_event);
    engine.output(result)
}

/// Sets the active input method, clearing the buffer.
///
/// Returns whether the switch happened. False means a null handle or an unknown
/// method, in which case the engine is untouched.
///
/// On success the buffer is cleared, so the word has changed: the frontend must
/// re-read it with [`vime_parsed`]. There is no action to dispatch,
/// because nothing here consumes a key.
///
/// # Safety
///
/// `engine` must be NULL or a live pointer from [`vime_create`] /
/// [`vime_create_with`] that has not been passed to [`vime_destroy`].
#[no_mangle]
pub unsafe extern "C" fn vime_set_input_method(
    engine: *mut VimeEngineHandle,
    method: VimeInputMethod,
) -> bool {
    let engine = handle_or!(engine, false);
    let Some(keymap) = keymap_for(method) else {
        return false;
    };

    engine.engine.set_keymap(keymap);
    // `set_keymap` clears the buffer, so the word the frontend last read is
    // no longer what the engine holds.
    engine.engine.reset();
    engine.invalidate_parsed();
    true
}

/// Switches the tone-placement scheme, re-rendering the current parsed.
///
/// Returns whether the switch happened. False means a null handle or an unknown
/// scheme, in which case the engine is untouched.
///
/// On success the pending vowels render under the new scheme, so the word has
/// changed: the frontend must re-read it with [`vime_parsed`].
///
/// # Safety
///
/// `engine` must be NULL or a live pointer from [`vime_create`] /
/// [`vime_create_with`] that has not been passed to [`vime_destroy`].
#[no_mangle]
pub unsafe extern "C" fn vime_set_tone_placement(
    engine: *mut VimeEngineHandle,
    tone_placement: VimeTonePlacement,
) -> bool {
    let engine = handle_or!(engine, false);
    let Ok(tone_placement) = TonePlacement::try_from(tone_placement) else {
        return false;
    };

    engine.engine.set_tone_placement(tone_placement);
    // The pending vowels re-render under the new scheme, so the cached parsed
    // no longer describes the buffer.
    engine.invalidate_parsed();
    true
}
