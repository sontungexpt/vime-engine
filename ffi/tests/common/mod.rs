//! Shared harness for the FFI integration tests.
//!
//! Wraps the C ABI in an RAII-friendly driver that copies out every piece of
//! text immediately (pointers are invalidated by the next call on the same
//! handle) and destroys the handle on drop.
//!
//! The word is fetched through `vime_parsed` only when the action says it
//! changed, which is how a real frontend should use the lazy accessor.

#![allow(dead_code)]

use std::ffi::CStr;

use vime::{
    VimeAction, VimeEngineHandle, VimeInputMethod, VimeKey, VimeKeyEvent, VimeOutput,
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

unsafe fn read_output(handle: *mut VimeEngineHandle, out: VimeOutput) -> Outcome {
    // The word is only rendered when the action says the text changed,
    // mirroring how a frontend is meant to drive the lazy accessor.
    let rendered = match out.action {
        VimeAction::Changed | VimeAction::CursorMoved => {
            let ptr = vime::vime_parsed(handle);
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
        // SAFETY: `self.0` is the live handle owned by this struct.
        let out = unsafe { vime::vime_process_key(self.0, event) };
        // SAFETY: `out` is a plain-repr struct of pointers; copying is safe.
        unsafe { read_output(self.0, out) }
    }

    /// Presses Enter, which is how a frontend commits: the key handler
    /// produces the commit text and reports `VIME_ACTION_COMMIT`.
    pub fn commit(&mut self) -> Outcome {
        self.process(key_event(VimeKey::Enter))
    }

    /// Presses Enter and returns the text it committed, if any.
    ///
    /// The commit text arrives in the `VimeOutput` that `Enter` produced, so
    /// there is no second call to make and no chance of reading a commit that
    /// a later key has already cleared.
    pub fn commit_and_read(&mut self) -> Option<String> {
        self.commit().commit
    }

    /// Resets the buffer, returning the new (usually empty) word.
    pub fn reset(&mut self) -> Outcome {
        // SAFETY: `self.0` is live.
        let out = unsafe { vime::vime_reset(self.0) };
        unsafe { read_output(self.0, out) }
    }

    /// Switches the input method, returning the new word.
    ///
    /// The C entry point reports only success, so the word is read here on
    /// the way out: a switch clears the buffer, so a caller needs the new
    /// text.
    pub fn set_input_method(&mut self, method: VimeInputMethod) -> Outcome {
        // SAFETY: `self.0` is live.
        let ok = unsafe { vime::vime_set_input_method(self.0, method) };
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

    /// The raw handle, for tests that need to drive the C ABI directly.
    pub fn raw(&self) -> *mut VimeEngineHandle {
        self.0
    }

    /// Switches the tone-placement scheme, returning the re-rendered word.
    ///
    /// As with `set_input_method`, the entry point reports only success.
    pub fn set_tone_placement(&mut self, tone: VimeTonePlacement) -> Outcome {
        // SAFETY: `self.0` is live.
        let ok = unsafe { vime::vime_set_tone_placement(self.0, tone) };
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

impl Drop for Engine {
    fn drop(&mut self) {
        // SAFETY: `self.0` is null-free (from_raw guarantees it); Idempotent per
        // handle, and called exactly once per engine by the C ABI contract.
        unsafe { vime::vime_destroy(self.0) };
    }
}
