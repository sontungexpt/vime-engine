use super::api::Renderer;

use crate::{
    composition::{BuildingSyllableBuilder, Composition, SyllableState},
    keymap::Keymap,
    phonology::{rules::TonePlacement, Tone},
};

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
    /// Renders the composition's syllable as a Vietnamese string, placing the
    /// tone on the tone-bearing vowel and leaving the rest unmarked. A dead
    /// syllable is rendered verbatim, in input order.
    fn render<KM: Keymap>(&self, composition: &Composition<KM>) -> String {
        match &composition.syllable().state {
            SyllableState::Building(syllable) => self.render_building(syllable),
            SyllableState::Dead(builder) => {
                builder.chars().iter().map(|status| status.char()).collect()
            }
        }
    }
}

impl DefaultRenderer {
    fn render_building(&self, syllable: &BuildingSyllableBuilder) -> String {
        let mut output = String::with_capacity(syllable.len());

        output.extend(syllable.onset().iter().copied());

        let tone_position = syllable.tone_index(TonePlacement::Modern);

        for (index, vowel) in syllable.vowels().iter().enumerate() {
            let tone = if Some(index) == tone_position {
                syllable.tone()
            } else {
                Tone::Flat
            };

            output.push(vowel.to_char_tone(tone));
        }

        output.extend(syllable.coda().iter().copied());

        output
    }
}
