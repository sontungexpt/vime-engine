use crate::{Composition, Keymap};

/// Renders a canonical parsed syllable into Unicode Vietnamese text.
pub trait Renderer {
    /// Renders `composition`'s syllable as a Unicode Vietnamese string with the
    /// tone mark placed according to the renderer's orthography.
    fn render<KM: Keymap>(&self, composition: &Composition<KM>) -> String;
}
