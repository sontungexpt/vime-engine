use crate::{RootVowel, Shape, Tone};

mod config;
mod telex;
mod viqr;
mod vni;

pub use config::{ShapeRule, ToneRule, TypingRules};

use super::RuleEngine;

/// Configuration-driven key mapping implementation.
///
/// This is used by input methods whose behavior can be described
/// declaratively through a [`TypingRules`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfiguredRuleEngine<'a> {
    config: &'a TypingRules<'a>,
}

impl<'a> ConfiguredRuleEngine<'a> {
    /// Creates a key mapping from a declarative configuration.
    pub const fn new(config: &'a TypingRules<'a>) -> Self {
        Self { config }
    }

    /// The Telex input method.
    #[inline(always)]
    pub const fn telex() -> Self {
        Self::new(telex::CONFIG)
    }

    /// The VNI input method.
    #[inline(always)]
    pub const fn vni() -> Self {
        Self::new(vni::CONFIG)
    }

    /// The VIQR input method.
    #[inline(always)]
    pub const fn viqr() -> Self {
        Self::new(viqr::CONFIG)
    }

    /// The underlying configuration.
    #[inline(always)]
    pub const fn config(&self) -> &TypingRules<'a> {
        self.config
    }
}

impl RuleEngine for ConfiguredRuleEngine<'_> {
    /// Returns whether `key` is configured as a tone, shape, or stroke key.
    #[inline(always)]
    fn is_rule_key(&self, key: char) -> bool {
        let key = key.to_ascii_lowercase();

        self.config.tones.iter().any(|map| map.key == key)
            || self.config.strokes.iter().any(|&k| k == key)
            || self.config.shapes.iter().any(|map| map.key == key)
    }

    #[inline(always)]
    fn tone(&self, input: char) -> Option<Tone> {
        let loinput = input.to_ascii_lowercase();

        self.config
            .tones
            .iter()
            .find(|map| map.key == loinput)
            .map(|map| map.tone)
    }

    #[inline(always)]
    fn stroke(&self, input: char) -> bool {
        let loinput = input.to_ascii_lowercase();
        self.config.strokes.iter().any(|c| *c == loinput)
    }

    #[inline(always)]
    fn shape(&self, input: char, target: RootVowel) -> Option<Shape> {
        let loinput = input.to_ascii_lowercase();

        self.config
            .shapes
            .iter()
            .find(|map| map.key == loinput && map.on == target)
            .map(|map| map.shape)
    }
}
