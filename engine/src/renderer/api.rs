use crate::Syllable;

/// Renders a canonical parsed syllable into Unicode Vietnamese text.
pub trait Renderer {
    /// Renders `syllable` as a Unicode Vietnamese string with the tone mark
    /// placed according to the renderer's orthography.
    fn render(&self, syllable: &Syllable) -> String;
}
