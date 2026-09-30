//! Session: one typing buffer with config propagation.

use crate::composition::syllable::SyllableBuilder;
use crate::composition::{Composition, Parallel};
use crate::keymap::Keymap;

use super::config::{Config, SharedConfig, UNRESOLVED};

/// One typing buffer: the settings in force, and the composition they apply to.
///
/// By default a session follows a [`SharedConfig`], so a change to the shared
/// settings reaches it along with every other session. To pin a session to its
/// own settings use [`Session::with_isolated_config`], which opts it out until
/// [`Session::clear_private_config`] puts it back on the shared settings.
///
/// Settings are stored once. `active_config` is the single copy of what is in
/// force, and it is the *only* thing that decides how a session parses and
/// renders — so there is no second copy to fall out of step with the first. A
/// pinned session is the same idea: the flag says which settings these are, and
/// the flag is all it needs, because a pinned session's settings are already
/// sitting in `active_config` where a following session's would be.
pub struct Session<KM: Keymap> {
    /// The settings currently in force. Either a private config (when
    /// `has_private_config == true`) or a snapshot of the shared config
    /// (when `has_private_config == false`), resolved at `generation`.
    ///
    /// Also the record of what the composition is parsing under, since
    /// [`Session::adopt_config`] moves the two together and nothing else writes
    /// either.
    active_config: Config<KM>,
    /// Whether `active_config` is a pinned private config (`true`) or a
    /// shared-config snapshot that should be refreshed (`false`).
    has_private_config: bool,
    /// The shared configuration source. Cloning shares the same atomic
    /// generation counter and value.
    shared_config: SharedConfig<KM>,
    /// The shared-config generation `active_config` was read at. `UNRESOLVED`
    /// means the session must re-read on its next [`refresh_config`].
    generation: u64,

    /// The composition being parsed by this session.
    composition: Composition<KM>,
}

impl<KM: Keymap> Session<KM>
where
    KM: Clone + PartialEq,
{
    /// Creates an empty session that follows `shared`.
    pub fn new(shared: SharedConfig<KM>) -> Self {
        let active_config = shared.snapshot();
        let mut session = Self {
            composition: Composition::new(SyllableBuilder::new(active_config.context.clone())),
            active_config,
            has_private_config: false,
            shared_config: shared,
            // Force one resolution so a replacement racing with construction
            // cannot leave the session holding settings from a generation it
            // never named.
            generation: UNRESOLVED,
        };
        session.refresh_config();
        session
    }

    /// Creates an empty session with its own settings, isolated from any shared
    /// config. `clear_private_config` falls back to the original settings.
    pub fn with_isolated_config(private: Config<KM>) -> Self {
        Self::with_config_on_shared(SharedConfig::new(private.clone()), private)
    }

    /// Creates an empty session with its own settings, attached to `shared`.
    ///
    /// Unlike [`Session::with_isolated_config`] the session remembers
    /// `shared_config`, so [`Session::clear_private_config`] puts it back on
    /// those settings — the pair of operations an application needs to give one
    /// buffer special treatment for a while and then take it back.
    pub fn with_config_on_shared(shared_config: SharedConfig<KM>, private: Config<KM>) -> Self {
        Self {
            composition: Composition::new(SyllableBuilder::new(private.context.clone())),
            active_config: private,
            has_private_config: true,
            shared_config,
            generation: UNRESOLVED,
        }
    }

    // ------------------------------------------------------------- settings

    /// The settings currently in force: the private config if pinned, otherwise
    /// the shared config snapshot.
    #[inline]
    pub fn config(&self) -> &Config<KM> {
        &self.active_config
    }

    /// The shared configuration source all unpinned sessions follow.
    #[inline]
    pub fn shared_config(&self) -> &SharedConfig<KM> {
        &self.shared_config
    }

    /// Whether this session is pinned to its own config (`true`) or follows
    /// the shared config (`false`).
    #[inline]
    pub fn has_private_config(&self) -> bool {
        self.has_private_config
    }

    /// Gives this session its own settings, which then stop following the
    /// shared config.
    ///
    /// A session that was following the shared config becomes a private one
    /// simply by being given a config; pass [`Session::config`]'s current value
    /// to opt out of following without changing anything.
    pub fn set_private_config(&mut self, private: Config<KM>) {
        self.adopt_config(private);
        self.has_private_config = true;
    }

    /// Drops the private config, so the session follows the shared one again
    /// and picks up whatever it says right now.
    pub fn clear_private_config(&mut self) {
        if !self.has_private_config {
            return;
        }
        self.has_private_config = false;
        self.generation = UNRESOLVED;
        self.refresh_config();
    }

    /// Re-resolves the settings if the shared config has moved since this
    /// session last looked.
    ///
    /// Returns `true` when the live composition was re-rendered as a result,
    /// meaning the word on screen changed without a key being pressed. A
    /// session with a private config is unaffected and returns `false`.
    pub fn refresh_config(&mut self) -> bool {
        let gen = self.shared_config.generation();
        if self.generation == gen {
            return false;
        }
        self.generation = gen;

        // Private config so do not activate config
        if self.has_private_config {
            return false;
        }
        self.adopt_config(self.shared_config.snapshot())
    }

    /// Makes `next` the settings in force, pushing whatever changed through to
    /// the live composition. Returns whether the word moved.
    ///
    /// The only place that writes `active_config` or the composition's context, so
    /// those two cannot disagree about what this session is parsing under.
    fn adopt_config(&mut self, next: Config<KM>) -> bool {
        self.active_config = next;
        self.composition
            .set_context(self.active_config.context.clone());
        true
    }

    #[inline(always)]
    fn with_refreshed_config<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.refresh_config();
        f(self)
    }

    // ---------------------------------------------------------------- state

    /// The buffer as this session parses it, as a fresh [`String`].
    ///
    /// Parsing is what turns raw keystrokes into a word, so this is the
    /// spelled-out form: `aw` reads back as `ă`, not `aw`. Once the parse has
    /// failed there is nothing left to apply and the raw keystrokes come back
    /// verbatim.
    ///
    /// See [`Self::write_parsed_to`] for the version that does not allocate.
    #[inline]
    pub fn rendered(&self) -> Vec<char> {
        self.composition.rendered()
    }

    /// Writes the parsed word into `output`, replacing its contents.
    ///
    /// The allocation-free counterpart to [`Self::parsed`]: a caller that writes
    /// on every keystroke can keep one `String` and reuse its capacity instead
    /// of building a new one each time.
    #[inline]
    pub fn write_rendered_to(&self, output: &mut String) {
        self.composition.write_rendered_to(output);
    }

    #[inline]
    pub fn raw(&self) -> &[char] {
        self.composition.raw()
    }

    /// Writes the parsed word into `output`, replacing its contents.
    ///
    /// The allocation-free counterpart to [`Self::parsed`]: a caller that writes
    /// on every keystroke can keep one `String` and reuse its capacity instead
    /// of building a new one each time.
    #[inline]
    pub fn write_raw_to(&self, output: &mut String) {
        self.composition.write_raw_to(output);
    }

    // ----------------------------------------------------------- key event

    /// Resets the session's composition to its initial empty state.
    #[inline]
    pub fn reset(&mut self) {
        self.composition.reset();
    }

    // /// Returns the caret position inside the rendered word, in Unicode
    // /// characters.
    // ///
    // /// The position is a character index, not a byte offset: the caller that
    // /// needs bytes has to walk the text this session renders.
    #[inline(always)]
    pub const fn cursor_pos(&self) -> usize {
        self.composition.cursor_pos()
    }

    /// Returns the caret position inside the raw keystroke buffer, in keystrokes.
    ///
    /// Reported separately from [`Self::cursor_pos`] because a transform consumes
    /// a keystroke without lengthening the rendered word, so the two positions
    /// are not interchangeable.
    #[inline(always)]
    pub const fn raw_cursor_pos(&self) -> usize {
        self.composition.raw_cursor_pos()
    }

    /// Whether the buffer currently spells a complete, valid Vietnamese
    /// syllable.
    ///
    /// See [`SyllableBuilder::is_valid`](crate::composition::syllable::SyllableBuilder::is_valid)
    /// for what "valid" means here; in short, a building syllable with a nucleus
    /// that the phonotactic rules accept.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.composition.is_valid()
    }

    // ------------------------------------------------------------- editing

    #[inline]
    pub fn insert(&mut self, character: char) {
        self.with_refreshed_config(|c| c.composition.insert(character));
    }

    #[inline]
    pub fn backspace(&mut self) -> Parallel<bool> {
        self.with_refreshed_config(|c| c.composition.backspace())
    }

    #[inline]
    pub fn delete(&mut self) -> Parallel<bool> {
        self.with_refreshed_config(|c| c.composition.delete())
    }

    #[inline]
    pub fn move_cursor_left(&mut self) -> Parallel<bool> {
        self.with_refreshed_config(|c| c.composition.move_cursor_left())
    }

    #[inline]
    pub fn move_cursor_right(&mut self) -> Parallel<bool> {
        self.with_refreshed_config(|c| c.composition.move_cursor_right())
    }
}
