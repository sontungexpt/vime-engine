//! Shared harness for the FFI integration tests.
//!
//! Wraps the C ABI in an RAII-friendly driver that copies out every piece of
//! text immediately (pointers are invalidated by the next call on the same
//! handle) and destroys the handle on drop.
//!
//! The word is fetched through `vime_session_render` only when the action says it
//! changed, which is how a real frontend should use the lazy accessor.

#![allow(dead_code)]

use std::ffi::CStr;

use vime::{
    VimeAction, VimeConfig, VimeInputMethod, VimeKey, VimeKeyEvent, VimeOutput,
    VimeTonePlacement, VimeSessionFactoryHandle, VimeSessionHandle,
};

/// A processed key/command response with the strings already copied out.
#[derive(Debug)]
pub struct Outcome {
    pub action: VimeAction,
    pub rendered: Option<String>,
    pub commit: Option<String>,
}

/// A live session factory handle; destroyed automatically when the `SessionFactory` is dropped.
pub struct SessionFactory(*mut VimeSessionFactoryHandle);

/// A live session handle; destroyed automatically when the `Session` is dropped.
pub struct Session(*mut VimeSessionHandle);

/// Builds a character key event (no modifiers, no special key).
pub fn char_event(ch: char) -> VimeKeyEvent {
    VimeKeyEvent {
        key: VimeKey::Character,
        character: ch as u32,
        states: 0,
    }
}

/// Builds a special-key event (no modifiers, no character payload).
pub fn key_event(key: VimeKey) -> VimeKeyEvent {
    VimeKeyEvent {
        key,
        character: 0,
        states: 0,
    }
}

unsafe fn read_output(handle: *mut VimeSessionHandle, out: VimeOutput) -> Outcome {
    let rendered = match out.action {
        VimeAction::Changed | VimeAction::CursorMoved => {
            let ptr = unsafe { vime::vime_session_render(handle) };
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_str().unwrap().to_string())
            }
        }
        _ => None,
    };
    let commit = if out.commit.is_null() {
        None
    } else {
        Some(CStr::from_ptr(out.commit).to_str().unwrap().to_string())
    };
    Outcome {
        action: out.action,
        rendered,
        commit,
    }
}

impl SessionFactory {
    fn from_raw(raw: *mut VimeSessionFactoryHandle) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self(raw))
        }
    }

    /// Creates a default (Telex) factory.
    pub fn create() -> Option<Self> {
        Self::from_raw(unsafe { vime::vime_session_factory_create() })
    }

    /// Creates a factory with the given configuration.
    pub fn create_with(config: VimeConfig) -> Option<Self> {
        Self::from_raw(unsafe { vime::vime_session_factory_create_with_config(&config) })
    }

    /// Changes the shared input method.
    pub fn set_input_method(&mut self, method: VimeInputMethod) -> bool {
        unsafe { vime::vime_session_factory_set_input_method(self.0, method) }
    }

    /// Changes the shared tone-placement convention.
    pub fn set_tone_placement(&mut self, tone: VimeTonePlacement) -> bool {
        unsafe { vime::vime_session_factory_set_tone_placement(self.0, tone) }
    }

    /// Changes the shared English auto-restore setting.
    pub fn set_auto_restore_english(&mut self, enabled: bool) -> bool {
        unsafe { vime::vime_session_factory_set_auto_restore_english(self.0, enabled) }
    }

    /// Creates a new empty session following the factory's shared configuration.
    pub fn open_session(&mut self) -> Session {
        Session(unsafe { vime::vime_session_create(self.0) })
    }

    /// Creates a session with a private configuration.
    pub fn open_session_with_config(&mut self, config: VimeConfig) -> Session {
        Session(unsafe { vime::vime_session_create_with_config(self.0, &config) })
    }

    /// The raw factory handle, for tests that need to drive the C ABI directly.
    pub fn handle(&self) -> *mut VimeSessionFactoryHandle {
        self.0
    }
}

impl Drop for SessionFactory {
    fn drop(&mut self) {
        unsafe { vime::vime_session_factory_destroy(self.0) };
    }
}

impl Session {
    fn from_raw(raw: *mut VimeSessionHandle) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self(raw))
        }
    }

    /// Feeds `text` character-by-character; returns the last word.
    pub fn type_text(&mut self, text: &str) -> String {
        let mut last = String::new();
        for ch in text.chars() {
            let out = self.process(char_event(ch));
            assert_eq!(out.action, VimeAction::Changed);
            if let Some(rendered) = out.rendered {
                last = rendered;
            }
        }
        last
    }

    /// Processes one event, copying the output strings.
    pub fn process(&mut self, event: VimeKeyEvent) -> Outcome {
        let out = match event.key {
            vime::VimeKey::Character => {
                let ok = unsafe { vime::vime_session_insert(self.0, event.character) };
                // Create a synthetic output for insert
                let out = if ok {
                    VimeOutput { action: VimeAction::Changed, commit: std::ptr::null() }
                } else {
                    VimeOutput::empty(VimeAction::Forward)
                };
                out
            }
            vime::VimeKey::Backspace => {
                let ok = unsafe { vime::vime_session_backspace(self.0) };
                let out = if ok {
                    VimeOutput { action: VimeAction::Changed, commit: std::ptr::null() }
                } else {
                    VimeOutput::empty(VimeAction::Forward)
                };
                out
            }
            vime::VimeKey::Delete => {
                let ok = unsafe { vime::vime_session_delete(self.0) };
                let out = if ok {
                    VimeOutput { action: VimeAction::Changed, commit: std::ptr::null() }
                } else {
                    VimeOutput::empty(VimeAction::Forward)
                };
                out
            }
            vime::VimeKey::Left => {
                let ok = unsafe { vime::vime_session_move_left(self.0) };
                let out = if ok {
                    VimeOutput { action: VimeAction::CursorMoved, commit: std::ptr::null() }
                } else {
                    VimeOutput::empty(VimeAction::Forward)
                };
                out
            }
            vime::VimeKey::Right => {
                let ok = unsafe { vime::vime_session_move_right(self.0) };
                let out = if ok {
                    VimeOutput { action: VimeAction::CursorMoved, commit: std::ptr::null() }
                } else {
                    VimeOutput::empty(VimeAction::Forward)
                };
                out
            }
            vime::VimeKey::Enter => {
                // Enter commits - we need to read the current word and reset
                let word = unsafe { vime::vime_session_render(self.0) };
                let word_str = if word.is_null() {
                    String::new()
                } else {
                    unsafe { CStr::from_ptr(word).to_str().unwrap().to_string() }
                };
                let ok = unsafe { vime::vime_session_reset(self.0) };
                let out = if ok {
                    VimeOutput {
                        action: VimeAction::Commit,
                        commit: if word_str.is_empty() { std::ptr::null() } else { std::ptr::null() }
                    }
                } else {
                    VimeOutput::empty(VimeAction::Forward)
                };
                out
            }
            vime::VimeKey::Escape => {
                let ok = unsafe { vime::vime_session_reset(self.0) };
                let out = if ok {
                    VimeOutput { action: VimeAction::Changed, commit: std::ptr::null() }
                } else {
                    VimeOutput::empty(VimeAction::Forward)
                };
                out
            }
            vime::VimeKey::Tab => {
                // Tab commits
                let word = unsafe { vime::vime_session_render(self.0) };
                let _ = unsafe { vime::vime_session_reset(self.0) };
                VimeOutput { action: VimeAction::Commit, commit: std::ptr::null() }
            }
            _ => VimeOutput::empty(VimeAction::Forward),
        };
        unsafe { read_output(self.0, out) }
    }

    /// Switches the input method for this session (via private config).
    pub fn set_input_method(&mut self, method: VimeInputMethod) -> Outcome {
        let ok = unsafe { vime::vime_session_set_config(self.0, &VimeConfig {
            auto_restore_english: true,
            input_method: method,
            tone_placement: VimeTonePlacement::Modern,
        }) };
        self.read_flag_output(ok)
    }

    /// Switches the tone-placement convention for this session (via private config).
    pub fn set_tone_placement(&mut self, tone: VimeTonePlacement) -> Outcome {
        let ok = unsafe { vime::vime_session_set_config(self.0, &VimeConfig {
            auto_restore_english: true,
            input_method: VimeInputMethod::Telex,
            tone_placement: tone,
        }) };
        self.read_flag_output(ok)
    }

    /// Returns the currently rendered composition.
    pub fn word(&mut self) -> Option<String> {
        let ptr = unsafe { vime::vime_session_render(self.0) };
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { CStr::from_ptr(ptr).to_str() }.unwrap().to_string())
    }

    /// Clears the current composition.
    pub fn reset_composition(&mut self) -> Outcome {
        let ok = unsafe { vime::vime_session_reset(self.0) };
        self.read_flag_output(ok)
    }

    /// Commits the current composition and resets the buffer.
    pub fn commit(&mut self) -> Outcome {
        // Read the current word before reset
        let word = unsafe { vime::vime_session_render(self.0) };
        let word_str = if word.is_null() {
            String::new()
        } else {
            unsafe { CStr::from_ptr(word).to_str().unwrap().to_string() }
        };
        let ok = unsafe { vime::vime_session_reset(self.0) };
        let out = if ok {
            VimeOutput {
                action: if word_str.is_empty() { VimeAction::Forward } else { VimeAction::Commit },
                commit: if word_str.is_empty() { std::ptr::null() } else { std::ptr::null() },
            }
        } else {
            VimeOutput::empty(VimeAction::Forward)
        };
        unsafe { read_output(self.0, out) }
    }

    /// Resets the current composition, returning the full outcome.
    pub fn reset(&mut self) -> Outcome {
        let ok = unsafe { vime::vime_session_reset(self.0) };
        self.read_flag_output(ok)
    }

    /// Resets the current composition, returning only the success flag.
    pub fn reset_flag(&mut self) -> bool {
        unsafe { vime::vime_session_reset(self.0) }
    }

    /// The raw handle, for tests that need to drive the C ABI directly.
    pub fn handle(&self) -> *mut VimeSessionHandle {
        self.0
    }

    /// The synthetic `VimeOutput` for a status-only entry point.
    fn read_flag_output(&mut self, ok: bool) -> Outcome {
        let out = if ok {
            VimeOutput {
                action: VimeAction::Changed,
                commit: std::ptr::null(),
            }
        } else {
            VimeOutput::empty(VimeAction::Forward)
        };
        unsafe { read_output(self.0, out) }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe { vime::vime_session_destroy(self.0) };
    }
}