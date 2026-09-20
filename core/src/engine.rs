use crate::{
    composition::Composition,
    config::Config,
    event::{Key, KeyEvent},
    keymap::{DefaultKeymap, Keymap},
    renderer::{DefaultRenderer, Renderer},
    result::Result,
};

const SUFFIX_SPACE: &str = " ";

/// The core input method state machine: buffers raw keystrokes and renders
/// them into Vietnamese text.
pub struct Engine<R: Renderer, KM: Keymap> {
    config: Config,
    composition: Composition<KM>,
    renderer: R,
}

impl<R, RE> Engine<R, RE>
where
    R: Renderer,
    RE: Keymap + Copy,
{
    /// The engine configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Renders the current buffer as Vietnamese text, or as the raw
    /// characters when the composition can no longer form a valid syllable.
    pub fn rendered(&self) -> String {
        self.renderer.render(&self.composition)
    }

    /// Resets the engine's composition to its initial empty state.
    pub fn reset(&mut self) -> Result {
        self.composition.reset();
        Result::Changed
    }

    /// Processes a full keyboard event. Handles modifier policy (Ctrl/Alt/Super
    /// are forwarded) and dispatches the key to the engine.
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
        // self.composition.clear();
        Result::Commit(text)
    }

    fn insert(&mut self, character: char) -> Result {
        self.composition.push(character);
        Result::Changed
    }

    fn backspace(&mut self) -> Result {
        self.apply(Composition::backspace)
    }

    fn delete(&mut self) -> Result {
        self.apply(Composition::delete)
    }

    fn move_left(&mut self) -> Result {
        self.apply(Composition::move_left)
    }

    fn move_right(&mut self) -> Result {
        self.apply(Composition::move_right)
    }

    fn apply(&mut self, operation: fn(&mut Composition<RE>)) -> Result {
        if self.composition.is_empty() {
            return Result::Forward;
        }

        operation(&mut self.composition);
        Result::Changed
    }
}

impl Engine<DefaultRenderer, DefaultKeymap<'static>> {
    /// Creates a Telex engine with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config,

            renderer: DefaultRenderer::default(),
            composition: Composition::new(DefaultKeymap::telex()),
        }
    }

    /// Switches the active input keymap (Telex, VNI, …), resetting the
    /// composition to its initial empty state.
    pub fn set_layout(&mut self, keymap: DefaultKeymap<'static>) {
        self.composition = Composition::new(keymap);
    }
}

impl Default for Engine<DefaultRenderer, DefaultKeymap<'static>> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
