use super::{api::Renderer, tone::{self, Orthography}};

use crate::{encode_vowel, phonology::Tone, Syllable};

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
    /// Renders the syllable as a Vietnamese string, placing the tone on the
    /// tone-bearing vowel and leaving the rest unmarked.
    fn render(&self, syllable: &Syllable) -> String {
        let mut output = String::with_capacity(syllable.len());

        output.extend(syllable.onset_chars.iter().copied());

        let tone_position = tone::analyze(syllable, self.orthography);

        for (index, vowel) in syllable.vowels.iter().enumerate() {
            let tone = if Some(index) == tone_position {
                syllable.tone
            } else {
                Tone::Flat
            };

            output.push(encode_vowel(vowel.value, tone, vowel.case));
        }

        output.extend(syllable.coda_chars.iter().copied());

        output
    }
}
