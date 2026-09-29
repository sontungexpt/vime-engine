use std::ffi::c_char;
use std::ptr;

use vime_engine::composition::syllable::SyllableContext;
use vime_engine::phonology::TonePlacement;
use vime_engine::{DefaultKeymap, Result, Session, SessionFactory, Config, Settings};

/// A reusable, NUL-terminated UTF-8 buffer handed to C.
///
/// Two properties matter, and pairing them here keeps callers from having to
/// track them separately:
///
/// - **The allocation is kept.** Refilling overwrites the bytes in place, so a
///   handle that renders on every keystroke stops allocating once it is warm.
///   An earlier version dropped the buffer on every state change, which
///   defeated the reuse and made each read *slower* than allocating afresh.
/// - **Staleness is explicit.** [`Self::invalidate`] marks the contents stale
///   without freeing them, and only a fresh buffer may be read.
pub(crate) struct CText {
    /// The UTF-8 bytes followed by one NUL. The NUL is always the last element.
    bytes: Vec<u8>,
    /// Whether `bytes` still describes the engine's current state.
    fresh: bool,
}

/// Complete configuration used by sessions and factories.
///
/// Mirrors the C `VimeConfig` struct. The factory stores one shared
/// configuration; sessions normally follow it, unless a session has
/// explicitly taken a private configuration.
#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct VimeConfig {
    pub auto_restore_english: bool,
    pub input_method: VimeInputMethod,
    pub tone_placement: VimeTonePlacement,
}

impl Default for VimeConfig {
    fn default() -> Self {
        Self {
            auto_restore_english: vime_engine::Settings::default().auto_restore_english,
            input_method: VimeInputMethod::Telex,
            tone_placement: VimeTonePlacement::Modern,
        }
    }
}

impl VimeConfig {
    /// Reads the caller's fields.
    ///
    /// # Safety
    ///
    /// `config` must be NULL or point to a readable `VimeConfig`.
    pub unsafe fn read(config: *const Self) -> Self {
        // SAFETY: the caller guarantees the pointer is readable. A NULL config
        // means "no configuration", the same as passing the default.
        unsafe { config.as_ref() }.copied().unwrap_or_default()
    }

    /// Converts to the engine's own configuration.
    ///
    /// One place decides what a C field means, so a frontend and the engine
    /// cannot disagree about it.
    pub fn to_engine_config(self) -> vime_engine::Config<DefaultKeymap<'static>> {
        let input_method: VimeInputMethod = self.input_method;
        let tone_placement: VimeTonePlacement = self.tone_placement;
        let keymap = match input_method {
            VimeInputMethod::Telex => vime_engine::DefaultKeymap::telex(),
            VimeInputMethod::Vni => vime_engine::DefaultKeymap::vni(),
            VimeInputMethod::Viqr => vime_engine::DefaultKeymap::viqr(),
        };
        let tone_placement = match self.tone_placement {
            VimeTonePlacement::Modern => vime_engine::phonology::TonePlacement::Modern,
            VimeTonePlacement::Old => vime_engine::phonology::TonePlacement::Old,
        };
        vime_engine::Config::new(
            vime_engine::Settings {
                auto_restore_english: self.auto_restore_english,
            },
            vime_engine::composition::syllable::SyllableContext::new(keymap, tone_placement),
        )
    }
}

impl CText {
    const fn new() -> Self {
        Self {
            bytes: Vec::new(),
            fresh: false,
        }
    }

    fn invalidate(&mut self) {
        self.fresh = false;
    }

    fn set(&mut self, text: &str) {
        self.bytes.clear();
        self.bytes.extend_from_slice(text.as_bytes());
        self.bytes.push(0);
        self.fresh = true;
    }

    fn clear(&mut self) {
        self.bytes.clear();
        self.invalidate();
    }

    fn ptr(&self) -> *const c_char {
        if self.bytes.is_empty() {
            ptr::null()
        } else {
            self.bytes.as_ptr().cast()
        }
    }
}

/// Opaque handle to a session factory (shared configuration + session creation).
#[repr(C)]
pub struct VimeSessionFactoryHandle {
    pub(crate) factory: SessionFactory<DefaultKeymap<'static>>,
}

/// Opaque handle to one independent typing session.
#[repr(C)]
pub struct VimeSessionHandle {
    pub(crate) session: Session<DefaultKeymap<'static>>,
    /// The word the engine has parsed, rendered lazily and reused across keystrokes.
    pub(crate) parsed: CText,
    /// The text the last commit produced, or empty when none is pending.
    pub(crate) committed: CText,
    /// Scratch space the word is rendered into before being copied out. Kept
    /// so the render can reuse its capacity too.
    pub(crate) scratch: String,
}

impl VimeSessionFactoryHandle {
    pub(crate) fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    pub(crate) fn new(factory: SessionFactory<DefaultKeymap<'static>>) -> Self {
        Self { factory }
    }
}

impl VimeSessionHandle {
    pub(crate) fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    pub(crate) fn new(session: Session<DefaultKeymap<'static>>) -> Self {
        Self {
            session,
            parsed: CText::new(),
            committed: CText::new(),
            scratch: String::new(),
        }
    }

    pub(crate) fn invalidate_parsed(&mut self) {
        self.parsed.invalidate();
    }

    pub(crate) fn invalidate(&mut self) {
        self.parsed.invalidate();
        self.committed.clear();
    }

    pub(crate) fn parsed_ptr(&mut self) -> *const c_char {
        if !self.parsed.fresh {
            self.scratch.clear();
            self.session.write_parsed_to(&mut self.scratch);
            self.parsed.set(&self.scratch);
        }
        self.parsed.ptr()
    }

    pub(crate) fn output(&mut self, result: Result) -> VimeOutput {
        self.invalidate();

        let action = match result {
            Result::Forward => return VimeOutput::empty(VimeAction::Forward),
            Result::Noop => return VimeOutput::empty(VimeAction::Noop),
            Result::Changed => VimeAction::Changed,
            Result::Commit(text) => {
                self.committed.set(&text);
                VimeAction::Commit
            }
            Result::CursorMoved => VimeAction::CursorMoved,
        };

        VimeOutput {
            action,
            commit: self.committed.ptr(),
        }
    }

    /// Returns the current composition cursor position in Unicode characters.
    pub(crate) fn cursor_pos(&self) -> usize {
        self.session.composition.cursor_pos()
    }

    /// Returns the rendered composition length in Unicode characters.
    pub(crate) fn length(&self) -> usize {
        self.session.composition.length()
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

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimeTonePlacement {
    #[default]
    Modern = 1,
    Old = 2,
}

impl TryFrom<VimeTonePlacement> for TonePlacement {
    type Error = ();

    #[inline]
    #[allow(unreachable_patterns)]
    fn try_from(value: VimeTonePlacement) -> std::result::Result<Self, Self::Error> {
        match value {
            VimeTonePlacement::Modern => Ok(TonePlacement::Modern),
            VimeTonePlacement::Old => Ok(TonePlacement::Old),
            _ => Err(()),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimeKey {
    #[default]
    Character = 0,
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

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimeAction {
    Forward = 0,
    Noop = 1,
    Changed = 2,
    Commit = 3,
    CursorMoved = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VimeOutput {
    pub action: VimeAction,
    pub commit: *const c_char,
}

impl VimeOutput {
    #[inline]
    pub const fn empty(action: VimeAction) -> Self {
        Self {
            action,
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

impl VimeConfig {
}