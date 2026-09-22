mod building;
mod dead;

pub use building::{BuildingSyllableBuilder, InputEffect, SyllableError, TransformEffect};
pub use dead::{CharStatus, DeadSyllableBuilder};

use crate::{keymap::Keymap, phonology::rules::TonePlacement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyllableState {
    /// Parsing phase: accumulating and validating Vietnamese syllable components.
    Building(BuildingSyllableBuilder),

    /// Dead phase: parsing failed; remaining input is collected verbatim.
    Dead(DeadSyllableBuilder),
}

impl Default for SyllableState {
    #[inline]
    fn default() -> Self {
        Self::Building(BuildingSyllableBuilder::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyllableBuilder<KM: Keymap> {
    pub keymap: KM,
    pub tone_scheme: TonePlacement,
    pub state: SyllableState,
}

impl<KM: Keymap> SyllableBuilder<KM> {
    #[inline]
    pub fn new(keymap: KM, tone_scheme: TonePlacement) -> Self {
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

    pub fn push(&mut self, input: char) -> InputEffect {
        match &mut self.state {
            SyllableState::Dead(builder) => {
                builder.push(input);
                InputEffect::StructurallyChanged
            }
            SyllableState::Building(builder) => {
                match builder.push(&self.keymap, input) {
                    Ok(effect) => effect,
                    Err(_err) => {
                        // Chuyển sang Dead nhưng vẫn giữ lại effect gốc (Transform hay Structural)
                        let chars = builder.to_chars(self.tone_scheme);
                        let mut dead = DeadSyllableBuilder::from_accepted(chars);
                        dead.push(input);
                        self.state = SyllableState::Dead(dead);
                        InputEffect::StructurallyChanged
                    }
                }
            }
        }
    }

    pub fn insert(&mut self, index: usize, input: char) -> InputEffect {
        match &mut self.state {
            SyllableState::Dead(builder) => {
                builder.insert(index, input);
                InputEffect::StructurallyChanged
            }
            SyllableState::Building(builder) => {
                match builder.insert(&self.keymap, index, input) {
                    Ok(effect) => effect,
                    Err(_err) => {
                        // Chuyển sang Dead nhưng vẫn giữ lại effect gốc (Transform hay Structural)
                        let chars = builder.to_chars(self.tone_scheme);
                        let mut dead = DeadSyllableBuilder::from_accepted(chars);
                        dead.insert(index, input);
                        self.state = SyllableState::Dead(dead);
                        InputEffect::StructurallyChanged
                    }
                }
            }
        }
    }
}
