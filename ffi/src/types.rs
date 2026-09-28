use std::ffi::c_char;
use std::mem::size_of;
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
    /// The UTF-8 bytes followed by one NUL. The NUL is always the last element.
    bytes: Vec<u8>,
    /// Whether `bytes` still describes the engine's current state.
    fresh: bool,
}

/// Engine settings, in the layout a C caller sees.
///
/// Each field mirrors one field of the engine's own [`vime_engine::Config`],
/// so a frontend sets its behaviour without a bespoke call per setting.
///
/// # Versioning
///
/// [`Self::struct_size`] is the size the caller compiled against. A library
/// that gains a field appends it and does not change the leading fields, so an
/// older caller still passes the smaller size and the library reads only the
/// fields that were present. This is the same convention as `XkbGetRules` and
/// `FcConfigSet`.
///
/// A zero size means "no configuration": every field takes its default. That is
/// also what a NULL pointer means, so the two are interchangeable.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VimeConfig {
    /// `size_of(VimeConfig)` as the caller knows it. Zero selects every default.
    pub struct_size: u32,

    /// Mirrors [`vime_engine::Config::auto_restore_english`].
    ///
    /// Present from revision 2. A caller compiled before this field existed
    /// reports `struct_size == 4` and does not write it, so this takes the
    /// engine default rather than whatever the padding held.
    pub auto_restore_english: bool,
}

/// The revision this library was built with.
///
/// `u32` plus `bool` plus padding, so the struct is 8 bytes. A field added
/// later is appended, and this constant becomes the size of the then-current
/// revision; the header's `VIME_CONFIG_INIT` computes the same value with
/// `sizeof`, and `config_layout` in `tests/abi.rs` pins the two together.
const VIME_CONFIG_SIZE: u32 = size_of::<VimeConfig>() as u32;

/// The size of revision 1, when `auto_restore_english` did not exist.
///
/// A caller reporting this size is treated as not having written the field.
const VIME_CONFIG_SIZE_REV1: u32 = 4;

impl VimeConfig {
    /// The configuration a caller gets by passing NULL or a zeroed struct.
    pub const fn default_config() -> Self {
        Self {
            struct_size: 0,
            // Matches `vime_engine::Config::default()`, which is `true`. Spelled
            // out rather than read from there because that `Default` is not a
            // `const fn`; `config_defaults_match_the_engine` in `tests/safety.rs`
            // pins the two together so they cannot drift.
            auto_restore_english: true,
        }
    }

    /// Reads the caller's fields, ignoring anything beyond `struct_size`.
    ///
    /// Returns `None` for a `struct_size` this library does not recognise:
    /// either a layout from a future revision, or a plausible-looking value
    /// that was never valid. Rejecting is the only safe answer, because
    /// reading fields the caller did not set would read whatever is on its
    /// stack.
    ///
    /// This is fallible rather than asserting on purpose: it is reached from an
    /// `extern "C"` function, where a panic cannot unwind and the process would
    /// abort. The entry point turns `None` into the NULL return the header
    /// promises.
    ///
    /// # Safety
    ///
    /// `config` must be NULL or point to a readable `VimeConfig`.
    pub unsafe fn read(config: *const Self) -> Option<Self> {
        // SAFETY: the caller guarantees the pointer is readable.
        let Some(config) = (unsafe { config.as_ref() }) else {
            // NULL means "no configuration", the same as a zeroed struct.
            return Some(Self::default_config());
        };

        // 0 means "no configuration", so every field takes its default.
        if config.struct_size == 0 {
            return Some(Self::default_config());
        }
        if config.struct_size < VIME_CONFIG_SIZE_REV1 {
            return None;
        }

        let mut out = Self::default_config();
        out.struct_size = config.struct_size;

        // `auto_restore_english` arrived in revision 2. A revision-1 caller
        // reports size 4 and never wrote the byte, so reading it would pick up
        // stack garbage. Below that size the field does not exist as far as
        // this caller is concerned, and the engine default stands.
        if config.struct_size >= VIME_CONFIG_SIZE {
            out.auto_restore_english = config.auto_restore_english;
        }

        Some(out)
    }

    /// Converts to the engine's own configuration.
    ///
    /// One place decides what a C field means, so a frontend and the engine
    /// cannot disagree about it.
    pub fn to_engine_config(self) -> vime_engine::Config {
        vime_engine::Config {
            auto_restore_english: self.auto_restore_english,
        }
    }
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
    /// A rendered-but-empty word is one NUL byte, so it reads as a valid empty
    /// string rather than NULL. That is deliberate: NULL means "there is no
    /// buffer to read" (a cleared or never-set field), while `""` means "the
    /// word is empty", which a frontend must act on by clearing its preedit.
    /// Collapsing the two would leave a stale preedit on screen.
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
    /// The word the engine has parsed, rendered lazily and reused across keystrokes.
    parsed: CText,
    /// The text the last commit produced, or empty when none is pending.
    committed: CText,
    /// Scratch space the word is rendered into before being copied out. Kept
    /// so the render can reuse its capacity too.
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
            parsed: CText::new(),
            committed: CText::new(),
            scratch: String::new(),
        }
    }

    /// Marks the word stale without touching the buffer.
    ///
    /// For state changes made outside [`Self::output`] — the config setters,
    /// which return no [`VimeOutput`]. Dropping the text here would defeat the
    /// buffer reuse that keeps a warm handle allocation-free.
    pub(crate) fn invalidate_parsed(&mut self) {
        self.parsed.invalidate();
    }

    /// The word the engine currently has parsed, as a NUL-terminated UTF-8
    /// string owned by this handle, or NULL when there is nothing to show.
    ///
    /// Rendered on first use after a state change and cached until the next
    /// [`Self::output`], so a frontend that never reads the word never pays
    /// for it. The returned pointer stays valid until the next call on this
    /// handle or [`vime_destroy`](crate::vime_destroy).
    pub(crate) fn parsed_ptr(&mut self) -> *const c_char {
        if !self.parsed.fresh {
            // Render into the reusable scratch, then copy into the buffer we
            // already own: neither step allocates once the handle is warm.
            self.scratch.clear();
            self.engine.write_parsed_to(&mut self.scratch);
            self.parsed.set(&self.scratch);
        }
        self.parsed.ptr()
    }

    /// Marks the cached word stale and drops any pending commit.
    ///
    /// Any pointer handed out earlier describes state that has since moved on,
    /// so the next read has to go back to the engine. Every entry point that
    /// changes state calls this, whether or not it returns a `VimeOutput`:
    /// `vime_reset` does not, which is why this is not folded into
    /// [`Self::output`] alone.
    #[inline]
    pub(crate) fn invalidate(&mut self) {
        self.parsed.invalidate();
        self.committed.clear();
    }

    /// Builds a `VimeOutput` view whose text lives in buffers owned by this
    /// handle. Any previously returned pointers become invalidated by this call.
    ///
    /// The parsed is *not* rendered here: read it with
    /// [`Self::parsed_ptr`] only when the frontend actually needs it.
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
    /// The event carries a character, in [`VimeKeyEvent::character`].
    ///
    /// The core enum's `Key::Character(char)` holds a `char`, which a flat C
    /// struct cannot, so this variant is payload-free and the character travels
    /// beside it. Every other variant is a discrete key with no character.
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

/// Action directive returned to native frontends (Fcitx5, IBus, macOS).
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimeAction {
    /// Key was ignored by IME; frontend must forward key to active application.
    Forward = 0,
    /// Key was consumed by IME, but nothing in the buffer changed.
    Noop = 1,
    /// The buffer changed; re-read the word with [`crate::vime_parsed`] and
    /// redraw whatever the frontend shows for it. Mirrors the core's
    /// `Result::Changed`.
    Changed = 2,
    /// Text was committed; clear the displayed word and insert the committed
    /// text.
    Commit = 3,
    /// The caret moved within the word, which is unchanged. Re-read with
    /// [`crate::vime_parsed`] only if the frontend shows the caret.
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
