use crate::phonology::{RootVowel, Shape, Tone};

mod config;
mod telex;
mod viqr;
mod vni;

pub use config::{Rules, ShapeRule, ToneRule};

use super::Keymap;

/// Configuration-driven key mapping implementation.
///
/// This is used by input methods whose behavior can be described
/// declaratively through a [`Rules`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DefaultKeymap<'a> {
    rules: &'a Rules<'a>,
}

impl<'a> DefaultKeymap<'a> {
    /// Creates a key mapping from a declarative configuration.
    pub const fn new(config: &'a Rules<'a>) -> Self {
        Self { rules: config }
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
    pub const fn config(&self) -> &Rules<'a> {
        self.rules
    }
}

impl Keymap for DefaultKeymap<'_> {
    /// Returns whether `key` is configured as a tone, shape, or stroke key.
    #[inline(always)]
    fn is_transform_key(&self, key: char) -> bool {
        let key = key.to_ascii_lowercase();

        self.rules.tones.iter().any(|map| map.key == key)
            || self.rules.strokes.iter().any(|&k| k == key)
            || self.rules.shapes.iter().any(|map| map.key == key)
    }

    #[inline(always)]
    fn decode_tone(&self, input: char) -> Option<Tone> {
        let loinput = input.to_ascii_lowercase();

        self.rules
            .tones
            .iter()
            .find(|map| map.key == loinput)
            .map(|map| map.tone)
    }

    #[inline(always)]
    fn is_stroke_key(&self, input: char) -> bool {
        let loinput = input.to_ascii_lowercase();
        self.rules.strokes.iter().any(|c| *c == loinput)
    }

    #[inline(always)]
    fn decode_shape(&self, input: char, target: RootVowel) -> Option<Shape> {
        let loinput = input.to_ascii_lowercase();

        self.rules
            .shapes
            .iter()
            .find(|map| map.key == loinput && map.on == target)
            .map(|map| map.shape)
    }
}
