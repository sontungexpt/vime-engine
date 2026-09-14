use crate::{
    event::{Key, KeyEvent},
    Buffer, Composition, Config, ConfiguredRuleEngine, DefaultRenderer, Renderer, Result,
    RuleEngine,
};

const SUFFIX_SPACE: &str = " ";

/// The core input method state machine: buffers raw keystrokes and renders
/// them into Vietnamese text.
pub struct Engine<R: Renderer, RE: RuleEngine> {
    config: Config,
    composition: Composition<RE>,
    renderer: R,
}

impl<R, RE> Engine<R, RE>
where
    R: Renderer,
    RE: RuleEngine + Copy,
{
    /// The engine configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Renders the current buffer as Vietnamese text, or as the raw ASCII
    /// keystrokes when the buffer cannot form a valid syllable.
    pub fn rendered(&self) -> String {
        // let mut parser = Composition::new(self.rule_engine);
        // let (status, _) = parser.parse(self.raw.chars());

        // if let ParseStatus::Dead(_) = status {
        //     return self.raw.to_string();
        // }

        // self.renderer.render(parser.syllable())
        String::new()
    }

    /// Clears the buffer.
    pub fn reset(&mut self) -> Result {
        // self.raw.clear();
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
        self.composition.append(character);
        Result::Changed
    }

    fn backspace(&mut self) -> Result {
        self.apply(Buffer::backspace)
    }

    fn delete(&mut self) -> Result {
        self.apply(Buffer::delete)
    }

    fn move_left(&mut self) -> Result {
        self.apply(Buffer::move_left)
    }

    fn move_right(&mut self) -> Result {
        self.apply(Buffer::move_right)
    }

    fn apply(&mut self, operation: fn(&mut Buffer<char>)) -> Result {
        if self.composition.is_empty() {
            return Result::Forward;
        }

        // operation(&mut self.composition);
        Result::Changed
    }
}

impl Engine<DefaultRenderer, ConfiguredRuleEngine<'static>> {
    /// Creates a Telex engine with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config,

            renderer: DefaultRenderer::default(),
            composition: Composition::new(ConfiguredRuleEngine::telex()),
        }
    }
}

impl Default for Engine<DefaultRenderer, ConfiguredRuleEngine<'static>> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
