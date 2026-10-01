use crate::phonology::{RootVowel, Shape, Tone};

/// Interprets keyboard keys into Vietnamese phonological transformations: tone
/// marks, vowel shapes, and the `d` ↔ `đ` stroke.
pub trait Keymap: Clone {
    /// Whether `key` is bound to a tone mark in this keymap.
    fn is_tone_key(&self, key: char) -> bool;

    /// Whether `key` is bound to a vowel shape in this keymap.
    fn is_shape_key(&self, key: char) -> bool;

    /// Whether `key` acts as the `d` ↔ `đ` stroke modifier.
    fn is_stroke_key(&self, key: char) -> bool;

    /// Whether `key` triggers any transformation: tone, shape or stroke.
    #[inline(always)]
    fn is_transform_key(&self, key: char) -> bool {
        self.is_tone_key(key) || self.is_shape_key(key) || self.is_stroke_key(key)
    }

    /// Decodes `key` as a tone mark, or `None` if it is not one.
    fn decode_tone(&self, key: char) -> Option<Tone>;

    /// Decodes `key` as a shape change on `target` — e.g. `a` + `w` → breve
    /// `ă`, `a` + `a` → circumflex `â` — or `None` if `key` is not a shape key
    /// for `target`.
    ///
    /// NOTE: `target` may one day become a context or an array of vowels for a
    /// DFA; too complicated to be worth it now.
    fn decode_shape(&self, key: char, target: RootVowel) -> Option<Shape>;
}
