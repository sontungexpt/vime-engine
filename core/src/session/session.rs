//! Session: one typing buffer with config propagation.

use crate::composition::Composition;
use crate::composition::Parallel;
use crate::keymap::Keymap;
use crate::syllable::InsertOutcome;
use crate::syllable::SyllableChars;

use super::config::{SessionConfig, SharedSessionConfig, UNRESOLVED_GENERATION};

/// One typing buffer with config propagation.
///
/// By default follows a [`SharedSessionConfig`]; use
/// [`with_isolated_config`] for a private config, and
/// [`clear_private_config`] to rejoin the shared config.
pub struct Session<KM: Keymap> {
    /// Active settings: private config if pinned, else shared snapshot at `generation`.
    active_config: SessionConfig<KM>,
    /// True if pinned to a private config (not following shared changes).
    has_private_config: bool,
    /// Shared config source; clones share one generation counter.
    shared_config: SharedSessionConfig<KM>,
    /// Generation `active_config` was read at; `UNRESOLVED_GENERATION` forces re-read.
    generation: u64,

    /// The composition being parsed by this session.
    composition: Composition,
}

impl<KM: Keymap> Session<KM>
where
    KM: Clone + PartialEq,
{
    /// Creates an empty session following `shared`.
    pub fn new(shared: SharedSessionConfig<KM>) -> Self {
        let active_config = shared.snapshot();
        let mut session = Self {
            composition: Composition::new(),
            active_config,
            has_private_config: false,
            shared_config: shared,
            // Force initial resolution so a racing config change isn't lost.
            generation: UNRESOLVED_GENERATION,
        };
        session.pull_config();
        session
    }

    /// Creates a session with isolated settings; `clear_private_config`
    /// returns it to the shared config.
    pub fn with_isolated_config(private: SessionConfig<KM>) -> Self {
        Self::with_config_on_shared(SharedSessionConfig::new(private.clone()), private)
    }

    /// Creates a session with its own settings, attached to `shared` so
    /// `clear_private_config` restores it to that config's current state.
    pub fn with_config_on_shared(
        shared_config: SharedSessionConfig<KM>,
        private: SessionConfig<KM>,
    ) -> Self {
        Self {
            composition: Composition::new(),
            active_config: private,
            has_private_config: true,
            shared_config,
            generation: UNRESOLVED_GENERATION,
        }
    }

    // ------------------------------------------------------------- settings

    /// Active settings (private if pinned, else shared snapshot).
    #[inline(always)]
    pub fn config(&self) -> &SessionConfig<KM> {
        &self.active_config
    }

    /// Shared config source all unpinned sessions follow.
    #[inline(always)]
    pub fn shared_config(&self) -> &SharedSessionConfig<KM> {
        &self.shared_config
    }

    /// Whether this session is pinned to its own config.
    #[inline(always)]
    pub fn has_private_config(&self) -> bool {
        self.has_private_config
    }

    /// Pins this session to its own settings. Pass current `config()` to opt out.
    pub fn set_private_config(&mut self, private: SessionConfig<KM>) {
        self.activate_config(private);
        self.has_private_config = true;
    }

    /// Unpins the session, making it follow the shared config again.
    pub fn clear_private_config(&mut self) {
        if !self.has_private_config {
            return;
        }
        self.has_private_config = false;
        self.generation = UNRESOLVED_GENERATION;
        self.pull_config();
    }

    /// Adopts shared config if it moved. Returns `true` if adopted.
    /// Private configs never adopt; returns `false` for them.
    /// No re-rendering; callers must call a config-dependent op to render.
    #[inline(always)]
    pub fn pull_config(&mut self) -> bool {
        let gen = self.shared_config.generation();
        if self.generation == gen {
            return false;
        }
        self.generation = gen;

        if self.has_private_config {
            return false;
        }
        self.activate_config(self.shared_config.snapshot())
    }

    /// Makes `next` the active config. The sole writer of `active_config`.
    fn activate_config(&mut self, next: SessionConfig<KM>) -> bool {
        self.active_config = next;
        true
    }

    // ---------------------------------------------------------------- state

    /// Parsed buffer as a `String`: `aw` → `ă`; failed parse = raw keystrokes.
    /// Pulls config first (tone placement affects which nucleus carries tone).
    #[inline(always)]
    pub fn rendered(&mut self) -> SyllableChars {
        self.pull_config();
        self.composition
            .rendered(self.active_config.tone_placement())
    }

    /// Allocation-free `rendered`; reuses `output` capacity. Pulls config first.
    #[inline(always)]
    pub fn write_rendered_to(&mut self, output: &mut String) {
        self.pull_config();
        self.composition
            .write_rendered_to(self.active_config.tone_placement(), output);
    }

    #[inline(always)]
    pub fn raw(&self) -> &[char] {
        self.composition.raw()
    }

    /// Allocation-free `raw`; reuses `output` capacity. No config read.
    #[inline(always)]
    pub fn write_raw_to(&self, output: &mut String) {
        self.composition.write_raw_to(output);
    }

    // ----------------------------------------------------------- key event

    /// Resets the session's composition to empty.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.composition.reset();
    }

    /// Rendered caret position (characters, not bytes).
    #[inline(always)]
    pub const fn rendered_cursor(&self) -> usize {
        self.composition.rendered_cursor()
    }

    /// Raw keystroke cursor position. Not interchangeable with `rendered_cursor`:
    /// a transform consumes a keystroke without lengthening the rendered word.
    #[inline(always)]
    pub const fn raw_cursor(&self) -> usize {
        self.composition.raw_cursor()
    }

    /// Length of the rendered buffer, in characters.
    #[inline(always)]
    pub fn rendered_len(&self) -> usize {
        self.composition.rendered_len()
    }

    /// Total UTF-8 length of the rendered buffer.
    #[inline(always)]
    pub fn rendered_len_utf8(&self) -> usize {
        self.composition.rendered_len_utf8()
    }

    /// Length of the raw buffer, in characters.
    #[inline(always)]
    pub fn raw_len(&self) -> usize {
        self.composition.raw_len()
    }

    /// Total UTF-8 length of the raw buffer.
    #[inline(always)]
    pub fn raw_len_utf8(&self) -> usize {
        self.composition.raw_len_utf8()
    }

    /// Whether the buffer is a complete, valid Vietnamese syllable.
    #[inline(always)]
    pub fn is_phonotactically_valid(&self) -> bool {
        self.composition.is_phonotactically_valid()
    }

    #[inline(always)]
    pub fn move_cursor_left_by(&mut self, by: usize) -> Parallel<bool> {
        self.composition.move_cursor_left_by(by)
    }

    #[inline(always)]
    pub fn move_cursor_right_by(&mut self, by: usize) -> Parallel<bool> {
        self.composition.move_cursor_right_by(by)
    }

    /// Whether the rendered cursor can move one position left.
    #[inline(always)]
    pub fn can_move_cursor_left(&self) -> Parallel<bool> {
        self.composition.can_move_cursor_left()
    }

    /// Whether the rendered cursor can move one position right.
    #[inline(always)]
    pub fn can_move_cursor_right(&self) -> Parallel<bool> {
        self.composition.can_move_cursor_right()
    }

    // ------------------------------------------------------------- editing
    //
    // Edits parse under keymap and re-render under tone placement,
    // so each pulls config first. Cursor movement does not.

    #[inline(always)]
    pub fn insert(&mut self, character: char) -> Parallel<InsertOutcome> {
        self.pull_config();
        self.composition.insert(
            self.active_config.keymap(),
            self.active_config.tone_placement(),
            character,
        )
    }

    #[inline(always)]
    pub fn backspace(&mut self) -> Parallel<bool> {
        self.pull_config();
        self.composition.backspace(
            self.active_config.keymap(),
            self.active_config.tone_placement(),
        )
    }

    #[inline(always)]
    pub fn delete(&mut self) -> Parallel<bool> {
        self.pull_config();
        self.composition.delete(
            self.active_config.keymap(),
            self.active_config.tone_placement(),
        )
    }
}
