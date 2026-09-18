use super::api::Renderer;

use crate::{
    analyze_modern, analyze_old,
    composition::{SyllableBuilder, SyllableState},
    encode_vowel,
    phonology::{BaseVowel, Tone},
    Composition, Keymap, VowelSequence,
};

/// Tone-placement orthography: the modern standard or the pre-1975 "old style".
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Orthography {
    /// Modern standard orthography ("học sinh" placement).
    #[default]
    Modern,
    /// Pre-1975 "old style" ("học sinh" placement).
    Old,
}

/// A [`SyllableBuilder`] is a vowel sequence for tone placement: its nucleus
/// is exactly the `vowels` list.
impl VowelSequence for SyllableBuilder {
    #[inline(always)]
    fn len(&self) -> usize {
        self.nucleus().len()
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<BaseVowel> {
        self.nucleus().get(index).map(|v| v.value)
    }
}

/// Renders a syllable to a Vietnamese string using a given tone orthography.
#[derive(Clone, Copy, Debug)]
pub struct DefaultRenderer {
    orthography: Orthography,
}

impl DefaultRenderer {
    /// Creates a renderer for the given orthography.
    pub const fn new(orthography: Orthography) -> Self {
        Self { orthography }
    }

    /// Returns the orthography this renderer uses.
    #[inline(always)]
    pub const fn orthography(&self) -> Orthography {
        self.orthography
    }
}

impl Default for DefaultRenderer {
    fn default() -> Self {
        Self::new(Orthography::default())
    }
}

impl Renderer for DefaultRenderer {
    /// Renders the composition's syllable as a Vietnamese string, placing the
    /// tone on the tone-bearing vowel and leaving the rest unmarked. A dead
    /// syllable is rendered verbatim, in input order.
    fn render<KM: Keymap>(&self, composition: &Composition<KM>) -> String {
        match composition.syllable() {
            SyllableState::Building(syllable) => self.render_building(syllable),
            SyllableState::Dead(builder) => {
                builder.chars().iter().map(|status| status.char()).collect()
            }
        }
    }
}

impl DefaultRenderer {
    fn render_building(&self, syllable: &SyllableBuilder) -> String {
        let mut output = String::with_capacity(syllable.len());

        output.extend(syllable.onset().iter().copied());

        let tone_position = match self.orthography {
            Orthography::Modern => analyze_modern(syllable),
            Orthography::Old => analyze_old(syllable, syllable.coda().is_empty()),
        };

        for (index, vowel) in syllable.nucleus().iter().enumerate() {
            let tone = if Some(index) == tone_position {
                syllable.tone()
            } else {
                Tone::Flat
            };

            output.push(encode_vowel(vowel.value, tone, vowel.case));
        }

        output.extend(syllable.coda().iter().copied());

        output
    }
}
