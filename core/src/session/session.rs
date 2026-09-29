//! Session: one typing buffer with config propagation.

use crate::composition::syllable::SyllableBuilder;
use crate::composition::Composition;
use crate::event::{Key, KeyEvent};
use crate::keymap::Keymap;
use crate::result::Result;

use super::config::{Config, SharedConfig, UNRESOLVED};

const SUFFIX_SPACE: &str = " ";

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
    pub composition: Composition<KM>,
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
        if next.context == self.active_config.context {
            // Nothing the parser or the renderer reads has changed. The engine
            // `Config` may still be new, so record it without disturbing the
            // buffer.
            self.active_config = next;
            return false;
        }

        let old_keymap = self.active_config.context.keymap();
        let new_keymap = next.context.keymap();
        let old_tone = self.active_config.context.tone_placement();
        let new_tone = next.context.tone_placement();

        let keymap_changed = *old_keymap != *new_keymap;
        let tone_changed = old_tone != new_tone;

        if keymap_changed {
            // Keymap change: keystrokes mean different things → reset buffer
            self.composition.set_keymap(new_keymap.clone());
            self.composition.reset();
        } else if tone_changed {
            // Tone placement only affects rendering → preserve buffer, update renderer
            self.composition.set_tone_placement(new_tone);
        }

        self.active_config = next;
        true
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
    pub fn parsed(&self) -> String {
        let mut output = String::new();
        self.write_parsed_to(&mut output);
        output
    }

    /// Writes the parsed word into `output`, replacing its contents.
    ///
    /// The allocation-free counterpart to [`Self::parsed`]: a caller that writes
    /// on every keystroke can keep one `String` and reuse its capacity instead
    /// of building a new one each time.
    #[inline]
    pub fn write_parsed_to(&self, output: &mut String) {
        self.composition.write_parsed_to(output);
    }

    // ----------------------------------------------------------- key event

    /// Processes a full keyboard event and dispatches it to the session.
    ///
    /// Picks up a shared-config change first, so a session never renders a
    /// keystroke under settings that have already been replaced.
    pub fn process_key(&mut self, event: KeyEvent) -> Result {
        let reconfigured = self.refresh_config();
        let result = self.dispatch(event);

        // A settings change can move the rendered word even when the key itself
        // did nothing, and `Forward` would tell the frontend to leave the
        // screen alone.
        if reconfigured && matches!(result, Result::Forward | Result::CursorMoved) {
            return Result::Changed;
        }
        result
    }

    fn dispatch(&mut self, event: KeyEvent) -> Result {
        match event.key {
            Key::Character(character) => self.insert(character),
            Key::Backspace => self.backspace(),
            Key::Delete => self.delete(),
            Key::Left => self.move_left(),
            Key::Right => self.move_right(),
            Key::Space => self.commit_with_suffix(SUFFIX_SPACE),
            Key::Enter | Key::Tab | Key::Escape => self.commit(),
        }
    }

    /// Resets the session's composition to its initial empty state.
    pub fn reset(&mut self) -> Result {
        self.composition.reset();
        Result::Changed
    }

    /// Returns the current composition cursor position in Unicode characters.
    pub fn cursor_pos(&self) -> usize {
        self.composition.cursor_pos()
    }

    /// Returns the rendered composition length in Unicode characters.
    pub fn length(&self) -> usize {
        self.composition.length()
    }

    /// Commits the current buffer and returns the resulting text.
    pub fn commit(&mut self) -> Result {
        self.commit_with_suffix("")
    }

    fn commit_with_suffix(&mut self, suffix: &str) -> Result {
        if self.composition.is_empty() {
            return Result::Forward;
        }

        let mut text = self.parsed();
        text.push_str(suffix);

        self.composition.reset();
        Result::Commit(text)
    }

    // ------------------------------------------------------------- editing

    #[inline]
    fn insert(&mut self, character: char) -> Result {
        self.composition.insert(character);
        Result::Changed
    }

    #[inline]
    fn backspace(&mut self) -> Result {
        if !self.composition.can_move_left() {
            return Result::Forward;
        }

        self.composition.backspace();
        Result::Changed
    }

    #[inline]
    fn delete(&mut self) -> Result {
        if !self.composition.can_move_right() {
            return Result::Forward;
        }

        self.composition.delete();
        Result::Changed
    }

    #[inline]
    fn move_left(&mut self) -> Result {
        if !self.composition.can_move_left() {
            return Result::Forward;
        }

        self.composition.move_left();
        Result::CursorMoved
    }

    #[inline]
    fn move_right(&mut self) -> Result {
        if !self.composition.can_move_right() {
            return Result::Forward;
        }

        self.composition.move_right();
        Result::CursorMoved
    }
}
