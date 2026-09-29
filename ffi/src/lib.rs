use std::ffi::c_char;
use std::ptr;

use vime_engine::composition::syllable::SyllableContext;
use vime_engine::phonology::TonePlacement;
use vime_engine::{Config, DefaultKeymap, Key, KeyEvent, KeyStates, Result, Session, SessionFactory, Settings};

mod convert;
mod types;

pub use types::{
    VimeAction, VimeConfig, VimeInputMethod, VimeKey, VimeKeyEvent, VimeOutput,
    VimeSessionFactoryHandle, VimeSessionHandle, VimeTonePlacement,
};

macro_rules! factory_handle_or {
    ($ptr:expr, $default:expr) => {
        match $ptr.as_mut() {
            Some(handle) => handle,
            None => return $default,
        }
    };
}

macro_rules! session_handle_or {
    ($ptr:expr, $default:expr) => {
        match $ptr.as_mut() {
            Some(handle) => handle,
            None => return $default,
        }
    };
}

fn keymap_for(method: VimeInputMethod) -> Option<DefaultKeymap<'static>> {
    #[allow(unreachable_patterns)]
    match method {
        VimeInputMethod::Telex => Some(DefaultKeymap::telex()),
        VimeInputMethod::Vni => Some(DefaultKeymap::vni()),
        VimeInputMethod::Viqr => Some(DefaultKeymap::viqr()),
        _ => None,
    }
}

fn tone_placement_for(tp: VimeTonePlacement) -> Option<TonePlacement> {
    TonePlacement::try_from(tp).ok()
}

/// Creates a session factory with the default configuration.
#[no_mangle]
pub extern "C" fn vime_session_factory_create() -> *mut VimeSessionFactoryHandle {
    let factory = SessionFactory::new(Config::new(
        Settings::default(),
        SyllableContext::new(DefaultKeymap::telex(), TonePlacement::Modern),
    ));
    VimeSessionFactoryHandle::new(factory).into_raw()
}

/// Creates a factory with the supplied configuration.
///
/// `config` may be NULL, in which case VIME_CONFIG_INIT is used.
///
/// Returns NULL if the configuration contains an invalid enum value.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_create_with_config(
    config: *const VimeConfig,
) -> *mut VimeSessionFactoryHandle {
    let config = unsafe { VimeConfig::read(config) };
    let config = config.to_engine_config();
    let factory = SessionFactory::new(config);
    VimeSessionFactoryHandle::new(factory).into_raw()
}

/// Destroys a factory.
///
/// Existing sessions created by the factory remain valid.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_destroy(factory: *mut VimeSessionFactoryHandle) {
    if !factory.is_null() {
        drop(Box::from_raw(factory));
    }
}

/// Replaces the shared configuration.
///
/// Existing sessions that follow the shared configuration observe the change
/// on their next operation. Sessions with a private configuration are unaffected.
///
/// Returns false for NULL handles or invalid enum values.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_set_config(
    factory: *mut VimeSessionFactoryHandle,
    config: *const VimeConfig,
) -> bool {
    let factory = factory_handle_or!(factory, false);
    let config = unsafe { VimeConfig::read(config) };
    let config = config.to_engine_config();
    factory.factory.set_config(config);
    true
}

/// Changes the shared input method.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_set_input_method(
    factory: *mut VimeSessionFactoryHandle,
    method: VimeInputMethod,
) -> bool {
    let factory = factory_handle_or!(factory, false);
    let Some(keymap) = keymap_for(method) else {
        return false;
    };
    let config = Config::from_keymap(Settings::default(), keymap);
    factory.factory.set_config(config);
    true
}

/// Changes the shared tone-placement convention.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_set_tone_placement(
    factory: *mut VimeSessionFactoryHandle,
    tone_placement: VimeTonePlacement,
) -> bool {
    let factory = factory_handle_or!(factory, false);
    let Some(_tp) = tone_placement_for(tone_placement) else {
        return false;
    };
    let config = Config::from_keymap(Settings::default(), DefaultKeymap::telex());
    let mut config = config;
    config.context = config.context; // keep keymap
    config.context = SyllableContext::new(
        *config.context.keymap(),
        tone_placement_for(tone_placement).unwrap(),
    );
    factory.factory.set_config(config);
    true
}

/// Changes the shared English auto-restore setting.
#[no_mangle]
pub unsafe extern "C" fn vime_session_factory_set_auto_restore_english(
    factory: *mut VimeSessionFactoryHandle,
    enabled: bool,
) -> bool {
    let factory = factory_handle_or!(factory, false);
    let mut config = factory.factory.config().snapshot();
    config.settings.auto_restore_english = enabled;
    factory.factory.set_config(config);
    true
}

/// Creates a new empty session following the factory's shared configuration.
#[no_mangle]
pub unsafe extern "C" fn vime_session_create(
    factory: *mut VimeSessionFactoryHandle,
) -> *mut VimeSessionHandle {
    let factory = factory_handle_or!(factory, ptr::null_mut());
    let session = Session::new(factory.factory.config().clone());
    VimeSessionHandle::new(session).into_raw()
}

/// Creates a new empty session with a private configuration.
///
/// The session does not follow subsequent shared configuration changes until
/// its private configuration is cleared.
#[no_mangle]
pub unsafe extern "C" fn vime_session_create_with_config(
    factory: *mut VimeSessionFactoryHandle,
    config: *const VimeConfig,
) -> *mut VimeSessionHandle {
    let factory = factory_handle_or!(factory, ptr::null_mut());
    let config = unsafe { VimeConfig::read(config) };
    let config = config.to_engine_config();
    let session = Session::with_config_on_shared(factory.factory.config().clone(), config);
    VimeSessionHandle::new(session).into_raw()
}

/// Destroys a session.
#[no_mangle]
pub unsafe extern "C" fn vime_session_destroy(session: *mut VimeSessionHandle) {
    if !session.is_null() {
        drop(Box::from_raw(session));
    }
}

/// Replaces this session's configuration with a private configuration.
///
/// After this call, changes to the factory's shared configuration no longer
/// affect this session.
#[no_mangle]
pub unsafe extern "C" fn vime_session_set_config(
    session: *mut VimeSessionHandle,
    config: *const VimeConfig,
) -> bool {
    let session = session_handle_or!(session, false);
    let config = unsafe { VimeConfig::read(config) };
    let config = config.to_engine_config();
    session.session.set_private_config(config);
    true
}

/// Clears the session's private configuration.
///
/// The session resumes following the factory's current shared configuration.
#[no_mangle]
pub unsafe extern "C" fn vime_session_clear_config(session: *mut VimeSessionHandle) -> bool {
    let session = session_handle_or!(session, false);
    session.session.clear_private_config();
    true
}

/// Inserts a character.
///
/// Picks up shared configuration changes before processing.
/// Returns false if the session is invalid.
#[no_mangle]
pub unsafe extern "C" fn vime_session_insert(
    session: *mut VimeSessionHandle,
    character: u32,
) -> bool {
    let session = session_handle_or!(session, false);
    let ch = char::from_u32(character);
    if ch.is_none() {
        return false;
    }
    let event = KeyEvent::key(Key::Character(ch.unwrap()));
    matches!(session.session.process_key(event), Result::Changed | Result::Commit(_))
}

/// Processes a backspace key.
///
/// Picks up shared configuration changes before processing.
/// Returns false if the session is invalid or the cursor cannot move left.
#[no_mangle]
pub unsafe extern "C" fn vime_session_backspace(session: *mut VimeSessionHandle) -> bool {
    let session = session_handle_or!(session, false);
    let event = KeyEvent::key(Key::Backspace);
    matches!(session.session.process_key(event), Result::Changed | Result::Commit(_))
}

/// Processes a delete key.
///
/// Picks up shared configuration changes before processing.
/// Returns false if the session is invalid or the cursor cannot move right.
#[no_mangle]
pub unsafe extern "C" fn vime_session_delete(session: *mut VimeSessionHandle) -> bool {
    let session = session_handle_or!(session, false);
    let event = KeyEvent::key(Key::Delete);
    matches!(session.session.process_key(event), Result::Changed | Result::Commit(_))
}

/// Moves the cursor left.
///
/// Picks up shared configuration changes before processing.
/// Returns false if the session is invalid or the cursor cannot move left.
#[no_mangle]
pub unsafe extern "C" fn vime_session_move_left(session: *mut VimeSessionHandle) -> bool {
    let session = session_handle_or!(session, false);
    let event = KeyEvent::key(Key::Left);
    matches!(session.session.process_key(event), Result::CursorMoved | Result::Changed | Result::Commit(_))
}

/// Moves the cursor right.
///
/// Picks up shared configuration changes before processing.
/// Returns false if the session is invalid or the cursor cannot move right.
#[no_mangle]
pub unsafe extern "C" fn vime_session_move_right(session: *mut VimeSessionHandle) -> bool {
    let session = session_handle_or!(session, false);
    let event = KeyEvent {
        key: Key::Right,
        character: 0,
        states: 0,
    };
    matches!(session.session.process_key(event), Result::CursorMoved | Result::Changed | Result::Commit(_))
}

/// Returns the currently rendered composition.
///
/// Returns NULL for invalid session, "" for empty composition.
///
/// The returned pointer is owned by the session and must not be freed.
/// Valid until the next operation that changes the session state or vime_session_destroy().
#[no_mangle]
pub unsafe extern "C" fn vime_session_render(session: *mut VimeSessionHandle) -> *const c_char {
    let session = session_handle_or!(session, ptr::null());
    session.parsed_ptr()
}

/// Clears the current composition.
///
/// Returns false for a NULL session.
#[no_mangle]
pub unsafe extern "C" fn vime_session_reset(session: *mut VimeSessionHandle) -> bool {
    let session = session_handle_or!(session, false);
    session.session.reset();
    session.invalidate();
    true
}

/// Returns the current composition cursor position in Unicode characters.
///
/// Returns SIZE_MAX for an invalid session.
#[no_mangle]
pub unsafe extern "C" fn vime_session_cursor(session: *mut VimeSessionHandle) -> usize {
    let session = session_handle_or!(session, usize::MAX);
    session.cursor_pos()
}

/// Returns the rendered composition length in Unicode characters.
///
/// Returns SIZE_MAX for an invalid session.
#[no_mangle]
pub unsafe extern "C" fn vime_session_length(session: *mut VimeSessionHandle) -> usize {
    let session = session_handle_or!(session, usize::MAX);
    session.length()
}

/// Gets the transformed Vietnamese UTF-8 display string.
///
/// Returned pointer is managed by Session and remains valid until next session call.
/// Returns NULL for invalid session or empty string for empty buffer.
#[no_mangle]
pub unsafe extern "C" fn vime_session_render_text(session: *mut VimeSessionHandle) -> *const c_char {
    let session = session_handle_or!(session, ptr::null());
    session.parsed_ptr()
}

/// Gets the raw UTF-8 sequence typed by user.
///
/// Not yet implemented in core engine. Returns NULL.
#[no_mangle]
pub unsafe extern "C" fn vime_session_render_raw_text(session: *mut VimeSessionHandle) -> *const c_char {
    let _ = session_handle_or!(session, ptr::null());
    ptr::null()
}

/// Checks if current buffer conforms to Vietnamese orthography rules.
///
/// Not yet implemented in core engine. Returns false.
#[no_mangle]
pub unsafe extern "C" fn vime_session_is_valid_vietnamese(session: *mut VimeSessionHandle) -> bool {
    let _ = session_handle_or!(session, false);
    false
}

/// Gets rendered cursor position in CodePoints (Unicode characters).
///
/// Returns 0 for invalid session.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_cursor_char_idx(session: *mut VimeSessionHandle) -> usize {
    let session = session_handle_or!(session, 0);
    session.cursor_pos()
}

/// Gets rendered cursor position in UTF-8 Byte offset.
///
/// Not yet implemented in core engine. Returns 0.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_cursor_byte_idx(session: *mut VimeSessionHandle) -> usize {
    let _ = session_handle_or!(session, 0);
    0
}

/// Gets raw cursor position in CodePoints (Unicode characters).
///
/// Not yet implemented in core engine. Returns 0.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_raw_cursor_char_idx(session: *mut VimeSessionHandle) -> usize {
    let _ = session_handle_or!(session, 0);
    0
}

/// Gets raw cursor position in UTF-8 Byte offset.
///
/// Not yet implemented in core engine. Returns 0.
#[no_mangle]
pub unsafe extern "C" fn vime_session_get_raw_cursor_byte_idx(session: *mut VimeSessionHandle) -> usize {
    let _ = session_handle_or!(session, 0);
    0
}

/// Retrieves a complete snapshot of current session render state.
///
/// Not yet implemented in core engine. Returns NULL.
#[no_mangle]
pub unsafe extern "C" fn vime_session_render_state(session: *mut VimeSessionHandle) -> *const VimeRenderState {
    let _ = session_handle_or!(session, ptr::null());
    ptr::null()
}

/// Clears input buffers and resets session state to initial conditions.
///
/// Safe to call with NULL pointer (no-op).
#[no_mangle]
pub unsafe extern "C" fn vime_session_reset(session: *mut VimeSessionHandle) {
    if session.is_null() {
        return;
    }
    let session = &mut *session;
    session.session.reset();
    session.invalidate();
}

/// Returns the VIME ABI version.
///
/// The returned string is static and must not be freed.
#[no_mangle]
pub extern "C" fn vime_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}
