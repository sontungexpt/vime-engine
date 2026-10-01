mod building;
mod dead;
mod iter;

use crate::{
    keymap::Keymap,
    phonology::{Coda, Onset, TonePlacement, Vowel},
    util::vec::SmallVec,
};

use building::BuildingSyllable;
use dead::DeadSyllable;

pub use building::{EditEffect, SyllableBuildError, TransformTarget};

#[cfg(test)]
mod tests;

/// The syllable's rendered characters — the form shown to the user — as a
/// [`VecLike`](crate::util::vec::VecLike) container sized for the longest
/// parseable syllable plus one slot of dead input; longer input spills to
/// the heap, so this is an allocation fast path, not a length limit.
pub type SyllableChars = SmallVec<char, { BuildingSyllable::MAX_LEN + 1 }>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Phase {
    /// Parsing phase: accumulating and validating Vietnamese syllable components.
    Building(BuildingSyllable),

    /// Dead phase: parsing failed; remaining input is collected verbatim.
    Dead(DeadSyllable),
}

impl Default for Phase {
    fn default() -> Self {
        Self::Building(BuildingSyllable::default())
    }
}

/// Incremental Vietnamese syllable buffer: parses input (building) until it
/// can no longer form a valid syllable, then records it verbatim (dead).
/// Keymap and tone placement are passed to the operations that need them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Syllable {
    phase: Phase,
}

impl Syllable {
    // --------------------------------------------------------- constructor

    /// Creates an empty syllable in the building phase.
    #[inline]
    pub fn new() -> Self {
        Self {
            phase: Phase::default(),
        }
    }

    // ---------------------------------------------------------- state

    /// Whether the syllable is still in the parsing (building) phase.
    #[inline(always)]
    pub const fn is_building(&self) -> bool {
        matches!(self.phase, Phase::Building(_))
    }

    /// Number of characters the syllable renders to (`onset + vowels + coda`).
    #[inline(always)]
    pub fn len(&self) -> usize {
        match &self.phase {
            Phase::Building(builder) => builder.len(),
            Phase::Dead(builder) => builder.len(),
        }
    }

    /// Whether the syllable holds no characters.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether the buffer is a complete, orthographically valid Vietnamese
    /// syllable: still building (a dead buffer is verbatim input, not a word)
    /// and accepted by the phonotactic rules with a nucleus present.
    #[inline]
    pub fn is_phonotactically_valid(&self) -> bool {
        match &self.phase {
            Phase::Building(builder) => builder.is_phonotactically_valid(),
            Phase::Dead(_) => false,
        }
    }

    /// Resets the syllable to an empty building state.
    #[inline]
    pub fn reset(&mut self) {
        self.phase = Phase::default();
    }

    // ------------------------------------------------------ part access

    // Once dead, every accessor below returns `None`: the buffer holds
    // verbatim characters with no onset/nucleus/coda split. See `write_to`.

    /// The onset characters, `None` once the syllable has fallen back to dead.
    #[inline(always)]
    pub fn onset(&self) -> Option<&[char]> {
        match &self.phase {
            Phase::Building(builder) => Some(builder.onset()),
            Phase::Dead(_) => None,
        }
    }

    /// The parsed onset variant, `None` once dead.
    #[inline(always)]
    pub fn onset_kind(&self) -> Option<Onset> {
        match &self.phase {
            Phase::Building(builder) => Some(builder.onset_kind()),
            Phase::Dead(_) => None,
        }
    }

    /// The nucleus vowels, `None` once dead.
    #[inline(always)]
    pub fn nucleus(&self) -> Option<&[Vowel]> {
        match &self.phase {
            Phase::Building(builder) => Some(builder.nucleus()),
            Phase::Dead(_) => None,
        }
    }

    /// The coda characters, `None` once dead.
    #[inline(always)]
    pub fn coda(&self) -> Option<&[char]> {
        match &self.phase {
            Phase::Building(builder) => Some(builder.coda()),
            Phase::Dead(_) => None,
        }
    }

    /// The parsed coda variant, `None` once dead.
    #[inline(always)]
    pub fn coda_kind(&self) -> Option<Coda> {
        match &self.phase {
            Phase::Building(builder) => Some(builder.coda_kind()),
            Phase::Dead(_) => None,
        }
    }

    // ------------------------------------------------------- rendering

    /// Renders the syllable as Vietnamese characters, precomposed (building)
    /// or verbatim (dead). `tone_placement` only matters on the building path,
    /// where it picks the vowel that carries the tone mark.
    #[inline(always)]
    pub fn to_chars(&self, tone_placement: TonePlacement) -> SyllableChars {
        match &self.phase {
            Phase::Building(builder) => builder.to_chars(tone_placement),
            Phase::Dead(builder) => builder.to_chars(),
        }
    }

    /// Appends the syllable to `output` as Vietnamese characters, precomposed
    /// (building) or verbatim (dead). Appends rather than replaces, and avoids
    /// the intermediate buffer [`Self::to_chars`] builds.
    #[inline(always)]
    pub fn write_to(&self, tone_placement: TonePlacement, output: &mut String) {
        match &self.phase {
            Phase::Building(builder) => builder.write_to(tone_placement, output),
            Phase::Dead(builder) => {
                builder.write_to(output);
            }
        }
    }

    /// Iterates the rendered characters, precomposed (building) or verbatim (dead).
    #[inline]
    pub fn iter_chars(&self, tone_placement: TonePlacement) -> impl Iterator<Item = char> + '_ {
        use crate::syllable::iter::SyllableCharsIter;
        match &self.phase {
            Phase::Building(builder) => {
                SyllableCharsIter::Building(builder.iter_chars(tone_placement))
            }
            Phase::Dead(builder) => SyllableCharsIter::Dead(builder.iter_chars()),
        }
    }

    // -------------------------------------------------------- mutation

    /// Appends `input` at the end under `keymap`. When the building phase
    /// rejects it, the accepted prefix is frozen into a dead buffer, rendered
    /// with `tone_placement`, and the input appended verbatim.
    #[allow(dead_code)]
    pub(crate) fn push<KM: Keymap>(
        &mut self,
        keymap: &KM,
        tone_placement: TonePlacement,
        input: char,
    ) -> EditEffect {
        match &mut self.phase {
            Phase::Dead(builder) => {
                builder.push(input);
                EditEffect::StructurallyChanged
            }
            Phase::Building(builder) => match builder.push(keymap, input) {
                Ok(effect) => effect,
                Err(_err) => {
                    let chars = builder.to_chars(tone_placement);
                    let mut dead = DeadSyllable::from_accepted(&chars);
                    dead.push(input);
                    self.phase = Phase::Dead(dead);
                    EditEffect::StructurallyChanged
                }
            },
        }
    }

    /// Inserts `input` at `index` under `keymap`, falling back to the dead
    /// buffer on an out-of-range index or a rejected parse, exactly as
    /// [`Self::push`] does.
    pub(crate) fn insert<KM: Keymap>(
        &mut self,
        keymap: &KM,
        tone_placement: TonePlacement,
        index: usize,
        input: char,
    ) -> EditEffect {
        match &mut self.phase {
            Phase::Dead(builder) => {
                builder.insert(index, input);
                EditEffect::StructurallyChanged
            }
            Phase::Building(builder) => match builder.insert(keymap, index, input) {
                Ok(effect) => effect,
                Err(_err) => {
                    let chars = builder.to_chars(tone_placement);
                    let mut dead = DeadSyllable::from_accepted(&chars);
                    dead.insert(index, input);
                    self.phase = Phase::Dead(dead);
                    EditEffect::StructurallyChanged
                }
            },
        }
    }

    /// Removes the character at `index`. On the building path an invalidating
    /// deletion is rolled back; on the dead path, once every character is
    /// accepted again the buffer is re-parsed under `keymap` and building resumes.
    pub(crate) fn remove<KM: Keymap>(
        &mut self,
        keymap: &KM,
        tone_placement: TonePlacement,
        index: usize,
    ) -> EditEffect {
        match &mut self.phase {
            Phase::Dead(builder) => {
                builder.remove(index);
                if builder.is_all_accepted() {
                    let mut building = BuildingSyllable::default();
                    for ch in builder.iter_chars() {
                        if building.push(keymap, ch).is_err() {
                            return EditEffect::StructurallyChanged;
                        }
                    }
                    self.phase = Phase::Building(building);
                }
                EditEffect::StructurallyChanged
            }
            Phase::Building(builder) => match builder.remove(index, tone_placement) {
                Ok(effect) => effect,
                Err(_) => EditEffect::StructurallyChanged,
            },
        }
    }
}
