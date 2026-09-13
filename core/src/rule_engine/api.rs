use crate::phonology::{RootVowel, Shape, Tone};

/// Interprets keyboard input into semantic Vietnamese actions.
pub trait RuleEngine {
    /// Returns whether `input` is configured as a tone, shape, or stroke key.
    fn is_rule_key(&self, input: char) -> bool;

    /// Interprets `input` as a tone key, returning the configured [`Tone`].
    fn tone(&self, input: char) -> Option<Tone>;

    /// Interprets `input` as the `d`/`đ` stroke key.
    fn stroke(&self, input: char) -> bool;

    /// Interprets `input` as a shape key for `target`, returning the
    /// configured [`Shape`].
    fn shape(&self, input: char, target: RootVowel) -> Option<Shape>;
}
