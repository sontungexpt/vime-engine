use crate::{
    event::{Key, KeyEvent},
    parser::ParseStatus,
    Buffer, Config, ConfiguredRuleEngine, DefaultRenderer, Parser, Renderer, Result, RuleEngine,
};

const SUFFIX_SPACE: &str = " ";

/// The core input method state machine: buffers raw keystrokes and renders
/// them into Vietnamese text.
pub struct Engine<R: Renderer, KM: RuleEngine> {
    config: Config,
    keystrokes: Buffer,
    renderer: R,
    mapping: KM,
}

impl<R, KM> Engine<R, KM>
where
    R: Renderer,
    KM: RuleEngine + Copy,
{
    /// The engine configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// The raw keystroke buffer.
    pub fn keystrokes(&self) -> &Buffer {
        &self.keystrokes
    }

    /// Renders the current buffer as Vietnamese text, or as the raw ASCII
    /// keystrokes when the buffer cannot form a valid syllable.
    pub fn rendered(&self) -> String {
        let mut parser = Parser::new(self.mapping);
        let (status, _) = parser.parse(self.keystrokes.chars());

        if let ParseStatus::Dead(_) = status {
            return self.keystrokes.to_string();
        }

        self.renderer.render(parser.syllable())
    }

    /// The input layout the engine is currently using.
    #[inline(always)]
    pub const fn layout(&self) -> KM {
        self.mapping
    }

    /// Switches the engine to `layout` for subsequent renders and commits.
    #[inline(always)]
    pub fn set_layout(&mut self, layout: KM) {
        self.mapping = layout;
    }

    /// Clears the buffer.
    pub fn reset(&mut self) -> Result {
        self.keystrokes.clear();
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
        if self.keystrokes.is_empty() {
            return Result::Forward;
        }

        let mut text = self.rendered();
        text.push_str(suffix);
        self.keystrokes.clear();
        Result::Commit(text)
    }

    fn insert(&mut self, character: char) -> Result {
        self.keystrokes.insert(character);
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

    fn apply(&mut self, operation: fn(&mut Buffer)) -> Result {
        if self.keystrokes.is_empty() {
            return Result::Forward;
        }

        operation(&mut self.keystrokes);
        Result::Changed
    }
}

impl Engine<DefaultRenderer, ConfiguredRuleEngine<'static>> {
    /// Creates a Telex engine with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            keystrokes: Buffer::new(),
            renderer: DefaultRenderer::default(),
            mapping: ConfiguredRuleEngine::telex(),
        }
    }

    /// Creates an engine using `layout` with the default configuration.
    #[inline(always)]
    pub fn with_layout(layout: ConfiguredRuleEngine<'static>) -> Self {
        Self {
            config: Config::default(),
            keystrokes: Buffer::new(),
            renderer: DefaultRenderer::default(),
            mapping: layout,
        }
    }
}

impl Default for Engine<DefaultRenderer, ConfiguredRuleEngine<'static>> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
