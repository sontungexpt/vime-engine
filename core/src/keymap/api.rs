use crate::phonology::{RootVowel, Shape, Tone};

/// Interprets keyboard input into Vietnamese phonological transformations.
pub trait Keymap {
    fn is_tone_key(&self, input: char) -> bool;

    fn is_shape_key(&self, input: char) -> bool;

    /// Whether `input` acts as the D-bar stroke modifier (d ↔ đ).
    fn is_stroke_key(&self, input: char) -> bool;

    /// Whether `input` triggers any transformation (tone, shape, or stroke).
    #[inline(always)]
    fn is_transform_key(&self, input: char) -> bool {
        self.is_tone_key(input) || self.is_shape_key(input) || self.is_stroke_key(input)
    }

    /// Attempts to decode `input` as a tone mark application.
    fn decode_tone(&self, input: char) -> Option<Tone>;

    /// Attempts to decode `input` as a vowel shape modification for `target`.
    fn decode_shape(&self, input: char, target: RootVowel) -> Option<Shape>;
}
