//! Shared harness for the FFI integration tests.
//!
//! Wraps the C ABI in an RAII-friendly driver that copies out every piece of
//! `VimeOutput` text immediately (pointers are invalidated by the next call on
//! the same handle) and destroys the handle on drop.

#![allow(dead_code)]

use std::ffi::CStr;

use vime::{
    VimeAction, VimeInputMethod, VimeEngineHandle, VimeKey, VimeKeyEvent, VimeOutput,
    VimeTonePlacement,
};

/// A processed key/command response with the strings already copied out.
#[derive(Debug)]
pub struct Outcome {
    pub action: VimeAction,
    pub rendered: Option<String>,
    pub commit: Option<String>,
}

/// A live engine handle; destroyed automatically when the `Engine` is dropped.
pub struct Engine(*mut VimeEngineHandle);

/// Builds a character key event (no modifiers, no special key).
pub fn char_event(ch: char) -> VimeKeyEvent {
    VimeKeyEvent {
        key: VimeKey::None,
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

unsafe fn read_output(out: VimeOutput) -> Outcome {
    let rendered = if out.rendered.is_null() {
        None
    } else {
        Some(CStr::from_ptr(out.rendered).to_str().unwrap().to_string())
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

impl Engine {
    fn from_raw(raw: *mut VimeEngineHandle) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self(raw))
        }
    }

    /// Creates a default (Telex) engine.
    pub fn create() -> Option<Self> {
        Self::from_raw(vime::vime_create())
    }

    /// Creates an engine for the given method and tone-placement scheme.
    pub fn create_with(method: VimeInputMethod, tone: VimeTonePlacement) -> Option<Self> {
        Self::from_raw(vime::vime_create_with(method, tone))
    }

    /// Feeds `text` character-by-character; returns the last preedit.
    pub fn type_text(&mut self, text: &str) -> String {
        let mut last = String::new();
        for ch in text.chars() {
            let out = self.process(char_event(ch));
            assert_eq!(out.action, VimeAction::UpdatePreedit);
            if let Some(rendered) = out.rendered {
                last = rendered;
            }
        }
        last
    }

    /// Processes one event, copying the output strings.
    pub fn process(&mut self, event: VimeKeyEvent) -> Outcome {
        // SAFETY: `self.0` is the live handle owned by this struct.
        let out = unsafe { vime::vime_process_key(self.0, event) };
        // SAFETY: `out` is a plain-repr struct of pointers; copying is safe.
        unsafe { read_output(out) }
    }

    /// Commits the buffer, returning the text (or `Forward` when empty).
    pub fn commit(&mut self) -> Outcome {
        // SAFETY: `self.0` is live.
        let out = unsafe { vime::vime_commit(self.0) };
        unsafe { read_output(out) }
    }

    /// Resets the buffer, returning the new (usually empty) preedit.
    pub fn reset(&mut self) -> Outcome {
        // SAFETY: `self.0` is live.
        let out = unsafe { vime::vime_reset(self.0) };
        unsafe { read_output(out) }
    }

    /// Switches the input method, returning the new preedit.
    pub fn set_input_method(&mut self, method: VimeInputMethod) -> Outcome {
        // SAFETY: `self.0` is live.
        let out = unsafe { vime::vime_set_input_method(self.0, method) };
        unsafe { read_output(out) }
    }

    /// Switches the tone-placement scheme, returning the re-rendered preedit.
    pub fn set_tone_placement(&mut self, tone: VimeTonePlacement) -> Outcome {
        // SAFETY: `self.0` is live.
        let out = unsafe { vime::vime_set_tone_placement(self.0, tone) };
        unsafe { read_output(out) }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // SAFETY: `self.0` is null-free (from_raw guarantees it); Idempotent per
        // handle, and called exactly once per engine by the C ABI contract.
        unsafe { vime::vime_destroy(self.0) };
    }
}