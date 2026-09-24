use crate::{
    composition::syllable::SyllableBuilder,
    composition::Composition,
    config::Config,
    event::{Key, KeyEvent},
    keymap::{DefaultKeymap, Keymap},
    phonology::rules::TonePlacement,
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
        Self::with_tone_placement(config, keymap, TonePlacement::Modern)
    }

    /// Creates an engine with the given configuration, `keymap` and
    /// `tone_placement`.
    #[inline]
    pub fn with_tone_placement(config: Config, keymap: KM, tone_placement: TonePlacement) -> Self {
        Self {
            config,
            composition: Composition::new(SyllableBuilder::new(keymap, tone_placement)),
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

    // --------------------------------------------------------------- state

    /// Renders the current buffer as Vietnamese text, or as the raw characters
    /// when the composition can no longer form a valid syllable.
    #[inline]
    pub fn rendered(&self) -> String {
        self.composition.rendered()
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

        let mut text = self.rendered();
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
        if self.composition.is_empty() {
            return Result::Forward;
        }

        self.composition.backspace();
        Result::Changed
    }

    #[inline]
    fn delete(&mut self) -> Result {
        if self.composition.cursor() >= self.composition.len() {
            return Result::Forward;
        }

        self.composition.delete();
        Result::Changed
    }

    #[inline]
    fn move_left(&mut self) -> Result {
        if self.composition.cursor() == 0 {
            return Result::Forward;
        }

        self.composition.move_left();
        Result::Changed
    }

    #[inline]
    fn move_right(&mut self) -> Result {
        if self.composition.cursor() >= self.composition.len() {
            return Result::Forward;
        }

        self.composition.move_right();
        Result::Changed
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

    /// Switches the active keymap, resetting the composition to its initial
    /// empty state.
    #[inline]
    pub fn set_keymap(&mut self, keymap: DefaultKeymap<'static>) {
        self.composition.set_keymap(keymap);
        self.composition.reset();
    }
}
