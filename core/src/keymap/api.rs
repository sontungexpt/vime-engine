use crate::phonology::{RootVowel, Shape, Tone};

/// Interprets keyboard input into Vietnamese phonological transformations.
pub trait Keymap {
    /// Whether `input` triggers any transformation (tone, shape, or stroke).
    fn is_transform_key(&self, input: char) -> bool;

    /// Attempts to decode `input` as a tone mark application.
    fn decode_tone(&self, input: char) -> Option<Tone>;

    /// Whether `input` acts as the D-bar stroke modifier (d ↔ đ).
    fn is_stroke_key(&self, input: char) -> bool;

    /// Attempts to decode `input` as a vowel shape modification for `target`.
    fn decode_shape(&self, input: char, target: RootVowel) -> Option<Shape>;
}