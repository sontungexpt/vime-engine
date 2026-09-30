//! Shared harness for the FFI integration tests.
//!
//! Wraps the two opaque handles in RAII so a test cannot leak one, and copies
//! every string out before the next call, because a returned `const char *` is
//! only valid until the next call on the same handle. The point of copying is
//! that a test comparing two renders needs both at once, which the ABI does not
//! otherwise allow.

#![allow(dead_code)]

use std::ffi::{c_char, CStr};
use std::mem::size_of;
use std::ptr;

use vime::{
    vime_session_backspace, vime_session_clear_config, vime_session_create,
    vime_session_create_with_config, vime_session_delete, vime_session_destroy,
    vime_session_factory_create, vime_session_factory_create_with_config,
    vime_session_factory_destroy, vime_session_factory_set_config,
    vime_session_get_cursor_byte_idx, vime_session_get_cursor_char_idx,
    vime_session_get_raw_cursor_byte_idx, vime_session_get_raw_cursor_char_idx,
    vime_session_insert, vime_session_is_valid_vietnamese, vime_session_move_cursor_left,
    vime_session_move_cursor_right, vime_session_render_raw_text, vime_session_render_state,
    vime_session_render_text, vime_session_reset, vime_session_set_config, VimeConfig,
    VimeRenderState, VimeSessionFactoryHandle, VimeSessionHandle,
    VIME_INPUT_METHOD_TELEX, VIME_TONE_PLACEMENT_MODERN,
};

/// Reads a `const char *` the session owns.
pub fn read_str(ptr: *const c_char) -> String {
    assert!(!ptr.is_null(), "expected a string, got NULL");
    // SAFETY: every pointer passed here came from a live session, which owns the
    // buffer and keeps it alive for the session's lifetime.
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .expect("the engine only ever writes UTF-8")
        .to_owned()
}

/// A live session factory, destroyed on drop.
pub struct Factory(*mut VimeSessionFactoryHandle);

/// A live session, destroyed on drop.
pub struct Session(*mut VimeSessionHandle);

impl Factory {
    /// A factory with the header's default configuration.
    pub fn create() -> Option<Self> {
        Self::from_raw(vime_session_factory_create())
    }

    /// A factory with an explicit configuration.
    pub fn create_with(config: &VimeConfig) -> Option<Self> {
        // SAFETY: `config` outlives the call.
        Self::from_raw(unsafe { vime_session_factory_create_with_config(config) })
    }

    /// Replaces the shared configuration. `false` means it was rejected.
    pub fn set_config(&mut self, config: &VimeConfig) -> bool {
        // SAFETY: the handle is live and `config` outlives the call.
        unsafe { vime_session_factory_set_config(self.0, config) }
    }

    /// A session following the shared configuration.
    pub fn open_session(&mut self) -> Session {
        // SAFETY: the factory handle is live.
        Session(unsafe { vime_session_create(self.0) })
    }

    /// A session with a private configuration.
    pub fn open_session_with(&mut self, config: &VimeConfig) -> Session {
        // SAFETY: the factory handle is live and `config` outlives the call.
        Session(unsafe { vime_session_create_with_config(self.0, config) })
    }

    /// The raw handle, for tests that need to drive the ABI directly.
    pub fn handle(&self) -> *mut VimeSessionFactoryHandle {
        self.0
    }
}

impl Factory {
    fn from_raw(raw: *mut VimeSessionFactoryHandle) -> Option<Self> {
        (!raw.is_null()).then_some(Self(raw))
    }
}

impl Drop for Factory {
    fn drop(&mut self) {
        // SAFETY: the handle is live and dropped exactly once.
        unsafe { vime_session_factory_destroy(self.0) };
    }
}

impl Session {
    fn from_raw(raw: *mut VimeSessionHandle) -> Option<Self> {
        (!raw.is_null()).then_some(Self(raw))
    }

    /// Types `text` one character at a time, resetting first.
    pub fn type_text(&mut self, text: &str) -> String {
        self.reset();
        for ch in text.chars() {
            assert!(self.insert(ch), "insert({ch:?}) should succeed");
        }
        self.render_text()
    }

    /// Inserts one character.
    pub fn insert(&mut self, ch: char) -> bool {
        // SAFETY: the handle is live.
        unsafe { vime_session_insert(self.0, ch as u32) }
    }

    /// Inserts a raw `uint32_t`, for testing what C can pass.
    pub fn insert_raw(&mut self, character: u32) -> bool {
        // SAFETY: the handle is live.
        unsafe { vime_session_insert(self.0, character) }
    }

    pub fn backspace(&mut self) -> bool {
        // SAFETY: the handle is live.
        unsafe { vime_session_backspace(self.0) }
    }

    pub fn delete(&mut self) -> bool {
        // SAFETY: the handle is live.
        unsafe { vime_session_delete(self.0) }
    }

    pub fn move_cursor_left(&mut self) -> bool {
        // SAFETY: the handle is live.
        unsafe { vime_session_move_cursor_left(self.0) }
    }

    pub fn move_cursor_right(&mut self) -> bool {
        // SAFETY: the handle is live.
        unsafe { vime_session_move_cursor_right(self.0) }
    }

    /// The rendered word.
    pub fn render_text(&mut self) -> String {
        // SAFETY: the handle is live.
        read_str(unsafe { vime_session_render_text(self.0) })
    }

    /// The rendered word, or `None` for a NULL return.
    pub fn render_text_ptr(&mut self) -> Option<String> {
        // SAFETY: the handle is live.
        let ptr = unsafe { vime_session_render_text(self.0) };
        (!ptr.is_null()).then(|| read_str(ptr))
    }

    /// The raw keystrokes.
    pub fn render_raw_text(&mut self) -> String {
        // SAFETY: the handle is live.
        read_str(unsafe { vime_session_render_raw_text(self.0) })
    }

    pub fn is_valid_vietnamese(&mut self) -> bool {
        // SAFETY: the handle is live.
        unsafe { vime_session_is_valid_vietnamese(self.0) }
    }

    pub fn cursor_char_idx(&mut self) -> usize {
        // SAFETY: the handle is live.
        unsafe { vime_session_get_cursor_char_idx(self.0) }
    }

    pub fn cursor_byte_idx(&mut self) -> usize {
        // SAFETY: the handle is live.
        unsafe { vime_session_get_cursor_byte_idx(self.0) }
    }

    pub fn raw_cursor_char_idx(&mut self) -> usize {
        // SAFETY: the handle is live.
        unsafe { vime_session_get_raw_cursor_char_idx(self.0) }
    }

    pub fn raw_cursor_byte_idx(&mut self) -> usize {
        // SAFETY: the handle is live.
        unsafe { vime_session_get_raw_cursor_byte_idx(self.0) }
    }

    /// The full snapshot, with the two strings copied out.
    pub fn render_state(&mut self) -> Snapshot {
        // SAFETY: the handle is live.
        let ptr = unsafe { vime_session_render_state(self.0) };
        assert!(!ptr.is_null());
        // SAFETY: the pointer is owned by the live session. The two string fields
        // are read before anything else can overwrite the buffers, and the scalar
        // fields are copied out by value.
        let raw: VimeRenderState = unsafe { *ptr };
        Snapshot {
            text: read_str(raw.text),
            raw_text: read_str(raw.raw_text),
            cursor_byte_idx: raw.cursor_byte_idx,
            cursor_char_idx: raw.cursor_char_idx,
            raw_cursor_byte_idx: raw.raw_cursor_byte_idx,
            raw_cursor_char_idx: raw.raw_cursor_char_idx,
            bytes_to_delete: raw.bytes_to_delete,
            chars_to_delete: raw.chars_to_delete,
            is_valid_vietnamese: raw.is_valid_vietnamese,
        }
    }

    /// The snapshot's pointer, for tests that care about address stability.
    pub fn render_state_ptr(&mut self) -> *const VimeRenderState {
        // SAFETY: the handle is live.
        unsafe { vime_session_render_state(self.0) }
    }

    pub fn set_config(&mut self, config: &VimeConfig) -> bool {
        // SAFETY: the handle is live and `config` outlives the call.
        unsafe { vime_session_set_config(self.0, config) }
    }

    pub fn clear_config(&mut self) -> bool {
        // SAFETY: the handle is live.
        unsafe { vime_session_clear_config(self.0) }
    }

    pub fn reset(&mut self) {
        // SAFETY: the handle is live.
        unsafe { vime_session_reset(self.0) };
    }

    /// The raw handle, for tests that need to drive the ABI directly.
    pub fn handle(&self) -> *mut VimeSessionHandle {
        self.0
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // SAFETY: the handle is live and dropped exactly once.
        unsafe { vime_session_destroy(self.0) };
    }
}

/// A `VimeRenderState` with its strings owned, so a test can hold on to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub text: String,
    pub raw_text: String,
    pub cursor_byte_idx: usize,
    pub cursor_char_idx: usize,
    pub raw_cursor_byte_idx: usize,
    pub raw_cursor_char_idx: usize,
    pub bytes_to_delete: usize,
    pub chars_to_delete: usize,
    pub is_valid_vietnamese: bool,
}

impl Snapshot {
    /// The snapshot of an empty session, which is what a fresh handle reports.
    pub fn empty() -> Self {
        Self {
            text: String::new(),
            raw_text: String::new(),
            cursor_byte_idx: 0,
            cursor_char_idx: 0,
            raw_cursor_byte_idx: 0,
            raw_cursor_char_idx: 0,
            bytes_to_delete: 0,
            chars_to_delete: 0,
            is_valid_vietnamese: false,
        }
    }
}

/// A NULL session handle, for the tests that check the null path.
pub fn null_session() -> *mut VimeSessionHandle {
    ptr::null_mut()
}

/// A NULL factory handle.
pub fn null_factory() -> *mut VimeSessionFactoryHandle {
    ptr::null_mut()
}

/// The header's `VIME_CONFIG_INIT`, spelled out in Rust.
pub fn config_init() -> VimeConfig {
    VimeConfig::default()
}

/// A `VimeConfig` holding an input method outside the header's range.
///
/// Built by writing bytes rather than by transmuting an integer into the enum,
/// because forming the enum at all is what a debug build panics on — and the
/// adapter's whole job is to survive a C caller that never formed it. This is
/// exactly what an out-of-range discriminant looks like on the wire: a struct with
/// the right size and the wrong number in it.
pub fn config_with_raw_input_method(input_method: u32) -> Vec<u8> {
    raw_config(input_method, VIME_TONE_PLACEMENT_MODERN)
}

/// A `VimeConfig` holding a tone placement outside the header's range.
pub fn config_with_raw_tone_placement(tone_placement: u32) -> Vec<u8> {
    raw_config(VIME_INPUT_METHOD_TELEX, tone_placement)
}

/// The byte pattern of a `VimeConfig`, `#[repr(C)]`-encoded by hand.
fn raw_config(input_method: u32, tone_placement: u32) -> Vec<u8> {
    // Laid out as the header declares it: int, int.
    assert_eq!(
        size_of::<VimeConfig>(),
        8,
        "the offsets below assume 8 bytes"
    );
    let mut bytes = vec![0u8; size_of::<VimeConfig>()];
    bytes[0..4].copy_from_slice(&input_method.to_ne_bytes());
    bytes[4..8].copy_from_slice(&tone_placement.to_ne_bytes());
    bytes
}
