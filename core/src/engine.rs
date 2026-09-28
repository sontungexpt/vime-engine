use crate::{
    composition::syllable::{SyllableBuilder, SyllableContext},
    composition::Composition,
    config::Config,
    event::{Key, KeyEvent},
    keymap::{DefaultKeymap, Keymap},
    phonology::TonePlacement,
    result::Result,
};

const SUFFIX_SPACE: &str = " ";

/// The core input method state machine: buffers raw keystrokes and renders
/// them into Vietnamese text.
pub struct Engine<KM: Keymap> {
    config: Config,
    composition: Composition<KM>,
}

impl<KM: Keymap> Engine<KM> {
    // ------------------------------------------------------------- constructor

    /// Creates an engine with the given configuration and `keymap`, using the
    /// modern tone-placement convention.
    #[inline]
    pub fn new(config: Config, keymap: KM) -> Self {
        Self::with_context(config, SyllableContext::new(keymap, TonePlacement::Modern))
    }

    /// Creates an engine with the given configuration and parse `context`,
    /// which supplies the keymap and the tone-placement scheme.
    #[inline]
    pub fn with_context(config: Config, context: SyllableContext<KM>) -> Self {
        Self {
            config,
            composition: Composition::new(SyllableBuilder::new(context)),
        }
    }

    // ------------------------------------------------------------- config

    /// The engine configuration.
    #[inline(always)]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Replaces the tone-placement scheme, re-rendering the live composition
    /// without discarding the current buffer.
    #[inline]
    pub fn set_tone_placement(&mut self, tone_placement: TonePlacement) {
        self.composition.set_tone_placement(tone_placement);
    }

    /// Switches the active keymap, resetting the composition to its initial
    /// empty state.
    #[inline]
    pub fn set_keymap(&mut self, keymap: KM) {
        self.composition.set_keymap(keymap);
        self.composition.reset();
    }

    // --------------------------------------------------------------- state

    /// The buffer as this engine parses it, as a fresh [`String`].
    ///
    /// Parsing is what turns raw keystrokes into a word, so this is the
    /// spelled-out form: `aw` reads back as `ă`, not `aw`. Once the parse has
    /// failed there is nothing left to apply, and the raw keystrokes come back
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

    // ------------------------------------------------------------ key event

    /// Processes a full keyboard event and dispatches it to the engine.
    pub fn process_key(&mut self, event: KeyEvent) -> Result {
        // if event
        //     .state
        //     .intersects(KeyState::CTRL | KeyState::ALT | KeyState::SUPER)
        // {
        //     return Result::Forward;
        // }

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

    /// Resets the engine's composition to its initial empty state.
    pub fn reset(&mut self) -> Result {
        self.composition.reset();
        Result::Changed
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

impl Engine<DefaultKeymap<'static>> {
    // -------------------------------------------------- convenience ctor

    /// Creates a Telex engine with the given configuration.
    #[inline]
    pub fn telex(config: Config) -> Self {
        Self::new(config, DefaultKeymap::telex())
    }

    /// Creates a VNI engine with the given configuration.
    #[inline]
    pub fn vni(config: Config) -> Self {
        Self::new(config, DefaultKeymap::vni())
    }

    #[inline]
    pub fn viqr(config: Config) -> Self {
        Self::new(config, DefaultKeymap::viqr())
    }
}
