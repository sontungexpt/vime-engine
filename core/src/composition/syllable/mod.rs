mod invalid;
mod valid;

pub use invalid::{CharStatus, DeadSyllableBuilder};
pub use valid::{PushPhase, SyllableError, TransformResult, ValidSyllableBuilder};

use crate::{keymap::Keymap, phonology::ToneScheme};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyllableState {
    /// Parsing phase: accumulating and validating Vietnamese syllable components.
    Building(ValidSyllableBuilder),

    /// Dead phase: parsing failed; remaining input is collected verbatim.
    Dead(DeadSyllableBuilder),
}

impl Default for SyllableState {
    #[inline]
    fn default() -> Self {
        Self::Building(ValidSyllableBuilder::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyllableBuilder<'a, KM: Keymap> {
    pub keymap: &'a KM,
    pub tone_scheme: ToneScheme,
    pub state: SyllableState,
}

impl<'a, KM: Keymap> SyllableBuilder<'a, KM> {
    pub fn new(keymap: &'a KM, tone_scheme: ToneScheme) -> Self {
        Self {
            keymap,
            tone_scheme,
            state: SyllableState::default(),
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        match &self.state {
            SyllableState::Building(builder) => builder.len(),
            SyllableState::Dead(builder) => builder.len(),
        }
    }

    /// Clean push signature — only needs the character!
    pub fn push(&mut self, input: char) {
        match &mut self.state {
            SyllableState::Building(builder) => {
                if builder.push(self.keymap, input).is_err() {
                    let builder = std::mem::take(builder);

                    self.state =
                        SyllableState::Dead(DeadSyllableBuilder::from_rejected(builder, input));
                }
            }

            SyllableState::Dead(builder) => {
                builder.push(input);
            }
        }
    }
}
