use std::ffi::{c_char, CString};
use std::ptr;

use vime_engine::phonology::TonePlacement;
use vime_engine::{DefaultKeymap, Engine, Result};

#[repr(C)]
pub struct VimeEngineHandle {
    pub(crate) engine: Engine<DefaultKeymap<'static>>,
    pub(crate) rendered: Option<CString>,
    pub(crate) commit: Option<CString>,
}

impl VimeEngineHandle {
    /// Wraps an engine in a hand-rolled buffer-owning handle.
    pub(crate) fn new(engine: Engine<DefaultKeymap<'static>>) -> Self {
        Self {
            engine,
            rendered: None,
            commit: None,
        }
    }

    /// Builds a `VimeOutput` view whose text lives in buffers owned by this
    /// handle. Any previously returned pointers become invalidated by this call.
    pub(crate) fn output(&mut self, result: Result) -> VimeOutput {
        self.rendered = None;
        self.commit = None;

        let action = match result {
            Result::Forward => return VimeOutput::empty(VimeAction::Forward),
            Result::Noop => return VimeOutput::empty(VimeAction::Noop),
            Result::Changed => {
                self.rendered = Some(
                    CString::new(self.engine.rendered()).expect("rendered text cannot contain NUL"),
                );
                VimeAction::UpdatePreedit
            }
            Result::Commit(text) => {
                self.commit = Some(CString::new(text).expect("committed text cannot contain NUL"));
                VimeAction::Commit
            }
        };

        VimeOutput {
            action,
            rendered: self.rendered.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            commit: self.commit.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimeInputMethod {
    #[default]
    Telex = 1,
    Vni = 2,
    Viqr = 3,
}

/// Tone-placement scheme; values mirror the ABI agreement with the Rust core.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimeTonePlacement {
    #[default]
    Modern = 1,
    Old = 2,
}

impl From<VimeTonePlacement> for TonePlacement {
    #[inline]
    fn from(value: VimeTonePlacement) -> Self {
        match value {
            VimeTonePlacement::Modern => TonePlacement::Modern,
            VimeTonePlacement::Old => TonePlacement::Old,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimeKey {
    #[default]
    None = 0,
    Backspace = 1,
    Delete = 2,
    Left = 3,
    Right = 4,
    Enter = 5,
    Escape = 6,
    Tab = 7,
    Space = 8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VimeInputContextProperties {
    pub enabled: bool,
    pub password: bool,
    pub ascii_only: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VimeKeyEvent {
    pub key: VimeKey,
    pub character: u32,
    pub states: u32,
}

/// Action directive returned to native frontends (Fcitx5, IBus, macOS).
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimeAction {
    /// Key was ignored by IME; frontend must forward key to active application.
    Forward = 0,
    /// Key was consumed by IME, but preedit/commit state did not change.
    Noop = 1,
    /// Preedit text was updated; update the client preedit window.
    UpdatePreedit = 2,
    /// Text was committed; clear the preedit window and insert committed text.
    Commit = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VimeOutput {
    /// High-level action for the frontend state machine.
    pub action: VimeAction,
    /// Preedit display text (UTF-8, null-terminated). NULL if empty/unchanged.
    /// Owned by the engine handle; valid until the next call or vime_destroy.
    pub rendered: *const c_char,
    /// Text to commit to the input context (UTF-8, null-terminated). NULL if none.
    /// Owned by the engine handle; valid until the next call or vime_destroy.
    pub commit: *const c_char,
}

impl VimeOutput {
    #[inline]
    pub const fn empty(action: VimeAction) -> Self {
        Self {
            action,
            rendered: ptr::null(),
            commit: ptr::null(),
        }
    }
}

impl Default for VimeOutput {
    #[inline]
    fn default() -> Self {
        Self::empty(VimeAction::Forward)
    }
}
