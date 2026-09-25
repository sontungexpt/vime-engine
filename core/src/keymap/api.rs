use crate::phonology::{RootVowel, Shape, Tone};

/// Interprets keyboard input keys into Vietnamese phonological transformations.
///
/// Implementations define how raw characters map to Vietnamese tone marks
/// (e.g., sắc, huyền, hỏi, ngã, nặng), vowel shapes (e.g., circumflex, horn, breve),
/// and consonant stroke modifications (`d` ↔ `đ`).
pub trait Keymap {
    /// Returns `true` if `key` is bound to a tone mark transformation in this keymap.
    fn is_tone_key(&self, key: char) -> bool;

    /// Returns `true` if `key` is bound to any vowel shape modification in this keymap.
    fn is_shape_key(&self, key: char) -> bool;

    /// Returns `true` if `key` acts as the D-bar stroke modifier (`d` ↔ `đ`).
    fn is_stroke_key(&self, key: char) -> bool;

    /// Returns `true` if `key` triggers any transformation (tone mark, vowel shape, or D-bar stroke).
    #[inline(always)]
    fn is_transform_key(&self, key: char) -> bool {
        self.is_tone_key(key) || self.is_shape_key(key) || self.is_stroke_key(key)
    }

    /// Attempts to decode `key` as a tone mark application.
    ///
    /// Returns `Some(Tone)` if `key` represents a valid tone mark, or `None` otherwise.
    fn decode_tone(&self, key: char) -> Option<Tone>;

    /// Attempts to decode `key` as a vowel shape modification for the given `target` root vowel.
    ///
    /// Returns `Some(Shape)` if `key` modifies `target` (e.g., `a` + `w` -> breve `ă`, `a` + `a` -> circumflex `â`),
    /// or `None` if `key` is not bound to a shape modifier for `target`.
    ///
    /// NOTE: In the future may be we will change target in to context or a array vowels for dfa state implementation but for now it's too complicated and no worth.
    fn decode_shape(&self, key: char, target: RootVowel) -> Option<Shape>;
}
