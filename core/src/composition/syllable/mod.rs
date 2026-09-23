mod building;
mod dead;
mod input_effect;

use crate::{
    keymap::Keymap,
    phonology::{rules::TonePlacement, CasedBaseVowel, Coda, Onset},
};

pub use building::BuildingSyllableBuilder;
pub use dead::DeadSyllableBuilder;
pub use input_effect::InputEffect;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
enum SyllableState {
    /// Parsing phase: accumulating and validating Vietnamese syllable components.
    Building(BuildingSyllableBuilder),

    /// Dead phase: parsing failed; remaining input is collected verbatim.
    Dead(DeadSyllableBuilder),
}

/// Incremental Vietnamese syllable buffer with a two-phase lifecycle:
/// parsing (building) first, falling back to a verbatim (dead) buffer once
/// the input can no longer form a valid syllable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyllableBuilder<KM: Keymap> {
    keymap: KM,
    tone_placement: TonePlacement,
    state: SyllableState,
}

impl<KM: Keymap> SyllableBuilder<KM> {
    // ------------------------------------------------------------- constructor

    /// Creates a building syllable backed by `keymap`, using `tone_placement`
    /// to decide where tone marks land.
    #[inline]
    pub fn new(keymap: KM, tone_placement: TonePlacement) -> Self {
        Self {
            keymap,
            tone_placement,
            state: SyllableState::Building(BuildingSyllableBuilder::default()),
        }
    }

    // ---------------------------------------------------------------- keymap

    /// The active keymap.
    #[inline(always)]
    pub fn keymap(&self) -> &KM {
        &self.keymap
    }

    /// Swaps the active keymap without touching the buffered syllable.
    #[inline]
    pub fn set_keymap(&mut self, keymap: KM) {
        self.keymap = keymap;
    }

    // --------------------------------------------------------- tone placement

    /// The tone-placement scheme used when rendering.
    #[inline(always)]
    pub const fn tone_placement(&self) -> TonePlacement {
        self.tone_placement
    }

    /// Replaces the tone-placement scheme.
    #[inline]
    pub fn set_tone_placement(&mut self, tone_placement: TonePlacement) {
        self.tone_placement = tone_placement;
    }

    // --------------------------------------------------------------- state

    /// Whether the syllable is still in the parsing (building) phase.
    #[inline(always)]
    pub const fn is_building(&self) -> bool {
        matches!(self.state, SyllableState::Building(_))
    }

    /// Rendered length of the syllable (`onset + vowels + coda`).
    #[inline(always)]
    pub fn len(&self) -> usize {
        match &self.state {
            SyllableState::Building(builder) => builder.len(),
            SyllableState::Dead(builder) => builder.len(),
        }
    }

    // ----------------------------------------------------------- part access

    /// The onset characters, `None` once the syllable has fallen back to dead.
    #[inline(always)]
    pub fn onset(&self) -> Option<&[char]> {
        match &self.state {
            SyllableState::Building(builder) => Some(builder.onset()),
            SyllableState::Dead(_) => None,
        }
    }

    /// The parsed onset variant, `None` once dead.
    #[inline(always)]
    pub fn onset_kind(&self) -> Option<Onset> {
        match &self.state {
            SyllableState::Building(builder) => Some(builder.onset_kind()),
            SyllableState::Dead(_) => None,
        }
    }

    /// The nucleus vowels, `None` once dead.
    #[inline(always)]
    pub fn vowels(&self) -> Option<&[CasedBaseVowel]> {
        match &self.state {
            SyllableState::Building(builder) => Some(builder.vowels()),
            SyllableState::Dead(_) => None,
        }
    }

    /// The coda characters, `None` once dead.
    #[inline(always)]
    pub fn coda(&self) -> Option<&[char]> {
        match &self.state {
            SyllableState::Building(builder) => Some(builder.coda()),
            SyllableState::Dead(_) => None,
        }
    }

    /// The parsed coda variant, `None` once dead.
    #[inline(always)]
    pub fn coda_kind(&self) -> Option<Coda> {
        match &self.state {
            SyllableState::Building(builder) => Some(builder.coda_kind()),
            SyllableState::Dead(_) => None,
        }
    }

    // -------------------------------------------------------------- rendering

    /// Resets the syllable to an empty building state, keeping `keymap` and
    /// `tone_placement`.
    #[inline]
    pub fn reset(&mut self) {
        self.state = SyllableState::Building(BuildingSyllableBuilder::default());
    }

    /// Renders the syllable as Vietnamese characters, either precomposed
    /// (building) or as the verbatim dead-buffer contents.
    #[inline(always)]
    pub fn to_chars(&self) -> Vec<char> {
        match &self.state {
            SyllableState::Building(builder) => builder.to_chars(self.tone_placement),
            SyllableState::Dead(builder) => builder.to_chars(),
        }
    }

    // ------------------------------------------------------------- mutation

    /// Appends `input` at the end. When the building phase rejects it, the
    /// accepted prefix is frozen into a dead buffer and the input appended
    /// verbatim.
    pub(crate) fn push(&mut self, input: char) -> InputEffect {
        let keymap = &self.keymap;
        match &mut self.state {
            SyllableState::Dead(builder) => {
                builder.push(input);
                InputEffect::StructurallyChanged
            }
            SyllableState::Building(builder) => match builder.push(keymap, input) {
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

    /// Inserts `input` at `index`; out-of-range indexes either delegate to the
    /// underlying builders or fall back to the dead buffer, same as [`Self::push`].
    pub(crate) fn insert(&mut self, index: usize, input: char) -> InputEffect {
        // let keymap = &self.keymap;
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

    /// Removes the character at `index` from the syllable.
    ///
    /// The removal is transactional on the building path: a deletion that
    /// leaves the syllable invalid is rolled back, and the builder keeps its
    /// previous contents.
    pub(crate) fn remove(&mut self, index: usize) -> InputEffect {
        match &mut self.state {
            SyllableState::Dead(builder) => {
                builder.remove(index);
                InputEffect::StructurallyChanged
            }
            SyllableState::Building(builder) => match builder.remove(index, self.tone_placement) {
                Ok(effect) => effect,
                Err(_) => InputEffect::StructurallyChanged,
            },
        }
    }
}
