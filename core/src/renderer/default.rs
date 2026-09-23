use super::api::Renderer;

use crate::{composition::Composition, keymap::Keymap};

/// Renders a syllable to a Vietnamese string using a given tone orthography.
#[derive(Clone, Copy, Debug)]
pub struct DefaultRenderer {}

impl DefaultRenderer {
    /// Creates a renderer for the given orthography.
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for DefaultRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for DefaultRenderer {
    fn render<KM: Keymap>(&self, composition: &Composition<KM>) -> String {
        composition.syllable().to_chars().iter().collect()
    }
}
