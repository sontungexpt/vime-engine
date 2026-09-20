mod invalid;
mod valid;

pub use invalid::{CharStatus, DeadSyllableBuilder};
pub use valid::{PushPhase, SyllableBuilder, SyllableError, TransformResult};

use crate::keymap::Keymap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyllableState {
    /// Parsing phase: accumulating and validating Vietnamese syllable components.
    Building(SyllableBuilder),

    /// Dead phase: parsing failed; remaining input is collected verbatim.
    Dead(DeadSyllableBuilder),
}

impl Default for SyllableState {
    #[inline]
    fn default() -> Self {
        Self::Building(SyllableBuilder::default())
    }
}
impl SyllableState {
    #[inline(always)]
    pub fn len(&self) -> usize {
        match self {
            Self::Building(builder) => builder.len(),
            Self::Dead(builder) => builder.len(),
        }
    }

    pub fn push<KM: Keymap>(&mut self, keymap: &KM, input: char) {
        match self {
            Self::Building(builder) => {
                if builder.push(keymap, input).is_err() {
                    let builder = std::mem::take(builder);

                    *self = Self::Dead(DeadSyllableBuilder::from_rejected(builder, input));
                }
            }

            Self::Dead(builder) => {
                builder.push(input);
            }
        }
    }

    #[inline(always)]
    pub fn insert<KM: Keymap>(&self, keymap: &KM, index: usize, input: char) {}
}
