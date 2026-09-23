mod building;
mod dead;
mod input_effect;

use building::BuildingSyllableBuilder;
use dead::DeadSyllableBuilder;

use crate::{
    keymap::Keymap,
    phonology::{rules::TonePlacement, CasedBaseVowel, Coda, Onset},
};

pub(crate) use input_effect::InputEffect;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
enum SyllableState {
    /// Parsing phase: accumulating and validating Vietnamese syllable components.
    Building(BuildingSyllableBuilder),

    /// Dead phase: parsing failed; remaining input is collected verbatim.
    Dead(DeadSyllableBuilder),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyllableBuilder<KM: Keymap> {
    keymap: KM,
    tone_placement: TonePlacement,
    state: SyllableState,
}

impl<KM: Keymap> SyllableBuilder<KM> {
    #[inline]
    pub fn new(keymap: KM, tone_placement: TonePlacement) -> Self {
        Self {
            keymap,
            tone_placement,
            state: SyllableState::Building(BuildingSyllableBuilder::default()),
        }
    }

    #[inline(always)]
    pub const fn is_dead(&self) -> bool {
        matches!(self.state, SyllableState::Dead(_))
    }

    #[inline(always)]
    pub const fn is_building(&self) -> bool {
        matches!(self.state, SyllableState::Building(_))
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        match &self.state {
            SyllableState::Building(builder) => builder.len(),
            SyllableState::Dead(builder) => builder.len(),
        }
    }

    #[inline(always)]
    pub fn onset(&self) -> Option<&[char]> {
        match &self.state {
            SyllableState::Building(builder) => Some(builder.onset()),
            SyllableState::Dead(_) => None,
        }
    }

    #[inline(always)]
    pub fn onset_kind(&self) -> Option<Onset> {
        match &self.state {
            SyllableState::Building(builder) => Some(builder.onset_kind()),
            SyllableState::Dead(_) => None,
        }
    }

    #[inline(always)]
    pub fn vowels(&self) -> Option<&[CasedBaseVowel]> {
        match &self.state {
            SyllableState::Building(builder) => Some(builder.vowels()),
            SyllableState::Dead(_) => None,
        }
    }

    #[inline(always)]
    pub fn coda(&self) -> Option<&[char]> {
        match &self.state {
            SyllableState::Building(builder) => Some(builder.coda()),
            SyllableState::Dead(_) => None,
        }
    }

    #[inline(always)]
    pub fn coda_kind(&self) -> Option<Coda> {
        match &self.state {
            SyllableState::Building(builder) => Some(builder.coda_kind()),
            SyllableState::Dead(_) => None,
        }
    }

    #[inline(always)]
    pub fn to_chars(&self) -> Vec<char> {
        match &self.state {
            SyllableState::Building(builder) => builder.to_chars(self.tone_placement),
            SyllableState::Dead(builder) => builder.to_chars(),
        }
    }

    pub(crate) fn push(&mut self, input: char) -> InputEffect {
        match &mut self.state {
            SyllableState::Dead(builder) => {
                builder.push(input);
                InputEffect::StructurallyChanged
            }
            SyllableState::Building(builder) => match builder.push(&self.keymap, input) {
                Ok(effect) => effect,
                Err(_err) => {
                    let chars = builder.to_chars(self.tone_placement);
                    let mut dead = DeadSyllableBuilder::from_accepted(chars);
                    dead.push(input);
                    self.state = SyllableState::Dead(dead);
                    InputEffect::StructurallyChanged
                }
            },
        }
    }

    pub(crate) fn insert(&mut self, index: usize, input: char) -> InputEffect {
        match &mut self.state {
            SyllableState::Dead(builder) => {
                builder.insert(index, input);
                InputEffect::StructurallyChanged
            }
            SyllableState::Building(builder) => match builder.insert(&self.keymap, index, input) {
                Ok(effect) => effect,
                Err(_err) => {
                    let chars = builder.to_chars(self.tone_placement);
                    let mut dead = DeadSyllableBuilder::from_accepted(chars);
                    dead.insert(index, input);
                    self.state = SyllableState::Dead(dead);
                    InputEffect::StructurallyChanged
                }
            },
        }
    }
}
