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
    tone_mask: u128,
    shape_mask: u128,
    stroke_mask: u128,
}

impl<'a> DefaultKeymap<'a> {
    /// Creates a key mapping from a declarative configuration.
    pub const fn new(config: &'a Rules<'a>) -> Self {
        Self {
            rules: config,
            tone_mask: Self::build_tone_mask(config.tones),
            shape_mask: Self::build_shape_mask(config.shapes),
            stroke_mask: Self::build_stroke_mask(config.strokes),
        }
    }

    const fn build_tone_mask(tones: &[ToneRule]) -> u128 {
        let mut mask = 0u128;
        let mut i = 0;
        while i < tones.len() {
            let key = tones[i].key;
            if key < 128 {
                mask |= 1u128 << key;
            }
            i += 1;
        }
        mask
    }

    /// Generates bitmask for shape keys at compile-time.
    const fn build_shape_mask(shapes: &[ShapeRule]) -> u128 {
        let mut mask = 0u128;
        let mut i = 0;
        while i < shapes.len() {
            let key = shapes[i].key;
            if key < 128 {
                mask |= 1u128 << key;
            }
            i += 1;
        }
        mask
    }

    /// Generates bitmask for stroke keys at compile-time.
    const fn build_stroke_mask(strokes: &[u8]) -> u128 {
        let mut mask = 0u128;
        let mut i = 0;
        while i < strokes.len() {
            let key = strokes[i];
            if key < 128 {
                mask |= 1u128 << key;
            }
            i += 1;
        }
        mask
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

    /// Branchless check whether a character matches the mask.
    #[inline(always)]
    fn has_key(mask: u128, input: char) -> bool {
        let lower = input.to_ascii_lowercase() as u32;
        lower < 128 && (mask & (1u128 << lower)) != 0
    }
}

impl Keymap for DefaultKeymap<'_> {
    #[inline(always)]
    fn is_tone_key(&self, input: char) -> bool {
        Self::has_key(self.tone_mask, input)
    }

    #[inline(always)]
    fn is_shape_key(&self, input: char) -> bool {
        Self::has_key(self.shape_mask, input)
    }

    #[inline(always)]
    fn is_stroke_key(&self, input: char) -> bool {
        Self::has_key(self.stroke_mask, input)
    }

    #[inline(always)]
    fn decode_tone(&self, input: char) -> Option<Tone> {
        let lower = input.to_ascii_lowercase();
        self.rules
            .tones
            .iter()
            .find(|map| (map.key as char) == lower)
            .map(|map| map.tone)
    }

    #[inline(always)]
    fn decode_shape(&self, input: char, target: RootVowel) -> Option<Shape> {
        let lower = input.to_ascii_lowercase();

        self.rules
            .shapes
            .iter()
            .find(|map| (map.key as char) == lower && map.on == target)
            .map(|map| map.shape)
    }
}
