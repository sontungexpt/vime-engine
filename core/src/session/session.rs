//! Session: one typing buffer with config propagation.

use crate::composition::Composition;
use crate::composition::Parallel;
use crate::keymap::Keymap;
use crate::syllable::SyllableChars;

use super::config::{SessionConfig, SharedSessionConfig, UNRESOLVED_GENERATION};

/// One typing buffer: the settings in force, and the composition they apply to.
///
/// By default it follows a [`SharedSessionConfig`]; [`Session::with_isolated_config`]
/// pins it to its own settings until [`Session::clear_private_config`] puts it
/// back on the shared ones.
pub struct Session<KM: Keymap> {
    /// The settings in force: a private config when pinned, otherwise a shared
    /// snapshot resolved at `generation`. Also what the composition parses
    /// under — [`Session::activate_config`] moves the two together.
    active_config: SessionConfig<KM>,
    /// Whether `active_config` is a pinned private config rather than a shared
    /// snapshot to refresh.
    has_private_config: bool,
    /// The shared configuration source; clones share one generation counter.
    shared_config: SharedSessionConfig<KM>,
    /// The generation `active_config` was read at. `UNRESOLVED_GENERATION` means
    /// re-read on the next [`Self::pull_config`].
    generation: u64,

    /// The composition being parsed by this session.
    composition: Composition,
}

impl<KM: Keymap> Session<KM>
where
    KM: Clone + PartialEq,
{
    /// Creates an empty session that follows `shared`.
    pub fn new(shared: SharedSessionConfig<KM>) -> Self {
        let active_config = shared.snapshot();
        let mut session = Self {
            composition: Composition::new(),
            active_config,
            has_private_config: false,
            shared_config: shared,
            // Force one resolution so a replacement racing with construction
            // cannot leave settings from a generation never named.
            generation: UNRESOLVED_GENERATION,
        };
        session.pull_config();
        session
    }

    /// Creates an empty session with its own settings, isolated from the shared
    /// config; `clear_private_config` falls back to these settings.
    pub fn with_isolated_config(private: SessionConfig<KM>) -> Self {
        Self::with_config_on_shared(SharedSessionConfig::new(private.clone()), private)
    }

    /// Creates an empty session with its own settings, attached to `shared`, so
    /// [`Session::clear_private_config`] puts it back on `shared` as it stands
    /// then.
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

    /// The settings in force: the private config if pinned, otherwise the
    /// shared snapshot.
    #[inline(always)]
    pub fn config(&self) -> &SessionConfig<KM> {
        &self.active_config
    }

    /// The shared configuration source all unpinned sessions follow.
    #[inline(always)]
    pub fn shared_config(&self) -> &SharedSessionConfig<KM> {
        &self.shared_config
    }

    /// Whether this session is pinned to its own config rather than following
    /// the shared one.
    #[inline(always)]
    pub fn has_private_config(&self) -> bool {
        self.has_private_config
    }

    /// Gives this session its own settings, which then stop following the
    /// shared config. Pass [`Session::config`]'s current value to opt out
    /// without changing anything.
    pub fn set_private_config(&mut self, private: SessionConfig<KM>) {
        self.activate_config(private);
        self.has_private_config = true;
    }

    /// Drops the private config, so the session follows the shared one again
    /// and picks up whatever it says right now.
    pub fn clear_private_config(&mut self) {
        if !self.has_private_config {
            return;
        }
        self.has_private_config = false;
        self.generation = UNRESOLVED_GENERATION;
        self.pull_config();
    }

    /// Adopts the shared config if it moved since this session last looked.
    /// Returns `true` when a newer config was adopted — a config-dependent
    /// operation is about to behave differently even though no key was pressed;
    /// a session with a private config returns `false`. Nothing is re-rendered
    /// here, and config-dependent operations call this for themselves, so a
    /// caller needs it only to learn whether anything moved.
    #[inline(always)]
    pub fn pull_config(&mut self) -> bool {
        let gen = self.shared_config.generation();
        if self.generation == gen {
            return false;
        }
        self.generation = gen;

        // A private config is not replaced, but the generation is still tracked.
        if self.has_private_config {
            return false;
        }
        self.activate_config(self.shared_config.snapshot())
    }

    /// Makes `next` the settings in force for every later composition
    /// operation. The only writer of `active_config`, which the composition
    /// reads on each call, so there is no second copy to fall out of step.
    /// Returns `true` always: the caller has already seen the generation move.
    fn activate_config(&mut self, next: SessionConfig<KM>) -> bool {
        self.active_config = next;
        true
    }

    // ---------------------------------------------------------------- state

    /// The buffer as this session parses it, as a fresh [`String`]: `aw` reads
    /// back as `ă`, and a failed parse returns the raw keystrokes verbatim.
    /// Pulls the config first, since tone placement decides which nucleus
    /// carries the tone mark. See [`Self::write_rendered_to`] for the
    /// allocation-free version.
    #[inline(always)]
    pub fn rendered(&mut self) -> SyllableChars {
        self.pull_config();
        self.composition
            .rendered(self.active_config.tone_placement())
    }

    /// Writes the parsed word into `output`, replacing its contents. Pulls the
    /// config first, like [`Self::rendered`]. The allocation-free counterpart
    /// to [`Self::rendered`]: keep one `String` and reuse its capacity.
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

    /// Writes the raw keystroke buffer into `output`, replacing its contents.
    /// The allocation-free counterpart to [`Self::raw`]: keep one `String` and
    /// reuse its capacity. Reads no configuration.
    #[inline(always)]
    pub fn write_raw_to(&self, output: &mut String) {
        self.composition.write_raw_to(output);
    }

    // ----------------------------------------------------------- key event

    /// Resets the session's composition to its initial empty state.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.composition.reset();
    }

    // Caret position inside the rendered word, in characters, not bytes: a
    // caller that needs bytes has to walk the text this session renders.
    #[inline(always)]
    pub const fn rendered_cursor(&self) -> usize {
        self.composition.rendered_cursor()
    }

    /// The caret position inside the raw keystroke buffer, in keystrokes. Not
    /// interchangeable with [`Self::rendered_cursor`]: a transform consumes a
    /// keystroke without lengthening the rendered word.
    #[inline(always)]
    pub const fn raw_cursor(&self) -> usize {
        self.composition.raw_cursor()
    }

    /// Whether the buffer currently spells a complete, valid Vietnamese
    /// syllable: a building syllable with a nucleus the phonotactic rules
    /// accept; see
    /// [`Syllable::is_phonotactically_valid`](crate::syllable::Syllable::is_phonotactically_valid).
    #[inline(always)]
    pub fn is_phonotactically_valid(&self) -> bool {
        self.composition.is_phonotactically_valid()
    }

    // ------------------------------------------------------------- editing
    //
    // The three edits parse under the keymap and re-render under tone
    // placement, so each pulls the config first. Cursor movement deliberately
    // does not: it only walks positions that already exist.

    #[inline(always)]
    pub fn insert(&mut self, character: char) {
        self.pull_config();
        self.composition.insert(
            self.active_config.keymap(),
            self.active_config.tone_placement(),
            character,
        );
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

    #[inline(always)]
    pub fn move_cursor_left_by(&mut self, by: usize) -> Parallel<bool> {
        self.composition.move_cursor_left_by(by)
    }

    #[inline(always)]
    pub fn move_cursor_right_by(&mut self, by: usize) -> Parallel<bool> {
        self.composition.move_cursor_right_by(by)
    }
}
