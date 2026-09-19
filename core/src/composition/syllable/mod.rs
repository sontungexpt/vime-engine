mod invalid;
mod valid;

pub use invalid::{CharStatus, DeadSyllableBuilder};
pub use valid::{PushPhase, SyllableBuilder, SyllableError, TransformResult};

use crate::Keymap;

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
}

impl SyllableState {
    pub fn push<KM: Keymap>(&mut self, keymap: &KM, input: char) {
        match self {
            // Fast-path: Thử push directly vào Building
            Self::Building(builder) => {
                if builder.push(keymap, input).is_err() {
                    // Slow-path: Chỉ khi parse lỗi mới swap state sang Dead
                    let old_stage = std::mem::replace(self, Self::default());
                    if let Self::Building(old_builder) = old_stage {
                        *self = Self::Dead(DeadSyllableBuilder::from_rejected(old_builder, input));
                    }
                }
            }

            // Đang ở trạng thái Dead: push trực tiếp ký tự thô
            Self::Dead(builder) => {
                builder.push(input);
            }
        }
    }

    #[inline(always)]
    pub fn insert<KM: Keymap>(&self, keymap: &KM, index: usize, input: char) {}
}
