use std::ffi::c_char;
use std::ptr;

use vime_engine::phonology::TonePlacement;
use vime_engine::{DefaultKeymap, Engine, Result};

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
struct CText {
    /// The UTF-8 bytes followed by one NUL. Empty means "nothing to report".
    bytes: Vec<u8>,
    /// Whether `bytes` still describes the engine's current state.
    fresh: bool,
}

impl CText {
    const fn new() -> Self {
        Self {
            bytes: Vec::new(),
            fresh: false,
        }
    }

    /// Marks the contents stale. The next [`Self::ptr`] must not be trusted
    /// until [`Self::set`] has run again.
    fn invalidate(&mut self) {
        self.fresh = false;
    }

    /// Replaces the contents with `text` and marks them fresh.
    fn set(&mut self, text: &str) {
        self.bytes.clear();
        self.bytes.extend_from_slice(text.as_bytes());
        self.bytes.push(0);
        self.fresh = true;
    }

    /// Empties the buffer, so [`Self::ptr`] reports nothing.
    fn clear(&mut self) {
        self.bytes.clear();
        self.invalidate();
    }

    /// The C string view, or NULL when the buffer holds nothing.
    ///
    /// Only call on a fresh buffer, or the caller will read text from before
    /// the last state change.
    fn ptr(&self) -> *const c_char {
        if self.bytes.is_empty() {
            ptr::null()
        } else {
            self.bytes.as_ptr().cast()
        }
    }
}

/// A live Vietnamese input engine plus the text buffers it hands to C.
///
/// The engine owns no C-visible memory; the two text buffers below do, and
/// outlive every call so a frontend can hold a pointer across one state change.
#[repr(C)]
pub struct VimeEngineHandle {
    pub(crate) engine: Engine<DefaultKeymap<'static>>,
    /// The preedit, rendered lazily and reused across keystrokes.
    preedit: CText,
    /// The text a commit produced, or empty when none is pending.
    commit: CText,
    /// Scratch space the preedit is rendered into before being copied out.
    /// Kept so the render can reuse its capacity too.
    scratch: String,
}

impl VimeEngineHandle {
    /// Hands the handle to C as an owning raw pointer.
    ///
    /// The matching [`crate::vime_destroy`] turns it back into a `Box` and
    /// drops it, so the allocation is owned by the caller across the ABI.
    pub(crate) fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    /// Wraps an engine in a hand-rolled buffer-owning handle.
    pub(crate) fn new(engine: Engine<DefaultKeymap<'static>>) -> Self {
        Self {
            engine,
            preedit: CText::new(),
            commit: CText::new(),
            scratch: String::new(),
        }
    }

    /// Marks the preedit stale without touching the buffer.
    ///
    /// For state changes made outside [`Self::output`] — the config setters,
    /// which return no [`VimeOutput`]. Dropping the text here would defeat the
    /// buffer reuse that keeps a warm handle allocation-free.
    pub(crate) fn invalidate_preedit(&mut self) {
        self.preedit.invalidate();
    }

    /// The current preedit text, as a NUL-terminated UTF-8 string owned by this
    /// handle, or NULL when there is nothing to show.
    ///
    /// Rendered on first use after a state change and cached until the next
    /// [`Self::output`], so a frontend that never reads the preedit never pays
    /// for it. The returned pointer stays valid until the next call on this
    /// handle or [`vime_destroy`](crate::vime_destroy).
    pub(crate) fn preedit_ptr(&mut self) -> *const c_char {
        if !self.preedit.fresh {
            // Render into the reusable scratch, then copy into the buffer we
            // already own: neither step allocates once the handle is warm.
            self.scratch.clear();
            self.engine.write_parsed_to(&mut self.scratch);
            self.preedit.set(&self.scratch);
        }
        self.preedit.ptr()
    }

    /// The text to commit, as a NUL-terminated UTF-8 string owned by this
    /// handle, or NULL when there is nothing pending.
    ///
    /// Non-empty only after an action of
    /// [`VimeAction::Commit`]: committing happens as part of processing the
    /// key, so this reports the text that key produced rather than producing
    /// any itself. Cleared by the next state change, as with the preedit.
    pub(crate) fn committed_ptr(&self) -> *const c_char {
        self.commit.ptr()
    }

    /// Builds a `VimeOutput` view whose text lives in buffers owned by this
    /// handle. Any previously returned pointers become invalidated by this call.
    ///
    /// The preedit is *not* rendered here: read it with
    /// [`Self::preedit_ptr`] only when the frontend actually needs it.
    pub(crate) fn output(&mut self, result: Result) -> VimeOutput {
        // Any text pointer handed out earlier is stale once the state moves on.
        self.preedit.invalidate();
        self.commit.clear();

        let action = match result {
            Result::Forward => return VimeOutput::empty(VimeAction::Forward),
            Result::Noop => return VimeOutput::empty(VimeAction::Noop),
            Result::Changed => VimeAction::UpdatePreedit,
            Result::Commit(text) => {
                self.commit.set(&text);
                VimeAction::Commit
            }
            Result::CursorMoved => VimeAction::CursorMoved,
        };

        VimeOutput {
            action,
            commit: self.commit.ptr(),
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

/// A `repr(u32)` enum accepts any value a C caller passes, so the conversion
/// is fallible: an unknown discriminant has no `TonePlacement` counterpart.
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
    /// The caret moved within the preedit; the preedit text is unchanged, so
    /// refresh the window only if the frontend tracks the caret.
    CursorMoved = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VimeOutput {
    /// High-level action for the frontend state machine.
    pub action: VimeAction,
    /// Text to commit to the input context (UTF-8, null-terminated). NULL if none.
    /// Owned by the engine handle; valid until the next call or vime_destroy.
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
