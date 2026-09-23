use crate::phonology::{RootVowel, Shape, Tone};

/// Interprets keyboard key into Vietnamese phonological transformations.
pub trait Keymap {
    fn is_tone_key(&self, key: char) -> bool;

    fn is_shape_key(&self, key: char) -> bool;

    /// Whether `key` acts as the D-bar stroke modifier (d ↔ đ).
    fn is_stroke_key(&self, key: char) -> bool;

    /// Whether `key` triggers any transformation (tone, shape, or stroke).
    #[inline(always)]
    fn is_transform_key(&self, key: char) -> bool {
        self.is_tone_key(key) || self.is_shape_key(key) || self.is_stroke_key(key)
    }

    /// Attempts to decode `key` as a tone mark application.
    fn decode_tone(&self, key: char) -> Option<Tone>;

    /// Attempts to decode `key` as a vowel shape modification for `target`.
    fn decode_shape(&self, key: char, target: RootVowel) -> Option<Shape>;
}
