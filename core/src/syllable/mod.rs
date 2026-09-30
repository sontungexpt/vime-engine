mod building;
mod dead;
mod input_effect;
mod iter;

use crate::{
    keymap::Keymap,
    phonology::{Coda, Onset, TonePlacement, Vowel},
};

// The two phases are local to this module: `Syllable` is the only way in, and
// nothing outside `syllable` names `BuildingSyllable` or `DeadSyllable`.
use building::BuildingSyllable;
use dead::DeadSyllable;

pub use input_effect::InputEffect;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
enum SyllableState {
    /// Parsing phase: accumulating and validating Vietnamese syllable components.
    Building(BuildingSyllable),

    /// Dead phase: parsing failed; remaining input is collected verbatim.
    Dead(DeadSyllable),
}

impl Default for SyllableState {
    fn default() -> Self {
        Self::Building(BuildingSyllable::default())
    }
}

/// Incremental Vietnamese syllable buffer with a two-phase lifecycle:
/// parsing (building) first, falling back to a verbatim (dead) buffer once
/// the input can no longer form a valid syllable.
///
/// A [`Syllable`] owns the incremental syllable parsing state and nothing
/// else, which is why it is not generic: the [`Keymap`] that decodes transform
/// keys and the [`TonePlacement`] scheme that picks the tone-bearing nucleus
/// vowel are configuration rather than input, so the operations that need them
/// take them as arguments, the keymap as a generic parameter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Syllable {
    state: SyllableState,
}

impl Syllable {
    // --------------------------------------------------------- constructor

    /// Creates an empty syllable in the building phase.
    ///
    /// The keymap and tone-placement scheme are not captured here: they are
    /// supplied to the operations that need them.
    #[inline]
    pub fn new() -> Self {
        Self {
            state: SyllableState::default(),
        }
    }

    // ---------------------------------------------------------- state

    /// Whether the syllable is still in the parsing (building) phase.
    #[inline(always)]
    pub const fn is_building(&self) -> bool {
        matches!(self.state, SyllableState::Building(_))
    }

    /// Number of characters the syllable renders to (`onset + vowels + coda`).
    #[inline(always)]
    pub fn len(&self) -> usize {
        match &self.state {
            SyllableState::Building(builder) => builder.len(),
            SyllableState::Dead(builder) => builder.len(),
        }
    }

    /// Whether the syllable holds no characters.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether the buffer spells a complete, orthographically valid Vietnamese
    /// syllable — the question a frontend asks to decide whether the word it is
    /// showing is real.
    ///
    /// Two things have to hold. The syllable must still be in the building
    /// phase, because a dead buffer is verbatim input and not a word at all; and
    /// the phonotactic rules must accept its onset / nucleus / coda / tone, with
    /// a nucleus present so that a bare onset is not mistaken for a word.
    ///
    /// This asks the validator directly rather than going through
    /// [`BuildingSyllable::validate`], whose incomplete-nucleus guard reads the
    /// cached `nucleus_state`. That cache only tracks nuclei of two vowels or
    /// more — a one-vowel nucleus is accepted as soon as it is pushed, without
    /// consulting it — so it answers "may this edit stand?", not "is this word
    /// finished?".
    #[inline]
    pub fn is_valid(&self) -> bool {
        use crate::phonology::{DefaultPhonotacticValidator, PhonotacticValidator};
        match &self.state {
            SyllableState::Building(builder) => {
                let nucleus = builder.nucleus();
                !nucleus.is_empty()
                    && DefaultPhonotacticValidator
                        .validate(
                            builder.onset_kind(),
                            nucleus,
                            builder.coda_kind(),
                            builder.tone(),
                        )
                        .is_ok()
            }
            SyllableState::Dead(_) => false,
        }
    }

    /// Resets the syllable to an empty building state, discarding whatever the
    /// keymap and tone placement had accepted.
    #[inline]
    pub fn reset(&mut self) {
        self.state = SyllableState::default();
    }

    // ------------------------------------------------------ part access

    // Every accessor below yields `None` once the syllable is dead: a dead
    // buffer holds characters verbatim, with no onset/nucleus/coda split left
    // to report. See `write_to` for reading them in that phase.

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
    pub fn nucleus(&self) -> Option<&[Vowel]> {
        match &self.state {
            SyllableState::Building(builder) => Some(builder.nucleus()),
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

    // ------------------------------------------------------- rendering

    /// Renders the syllable as Vietnamese characters, either precomposed
    /// (building) or as the verbatim dead-buffer contents.
    ///
    /// `tone_placement` only decides which nucleus vowel carries the tone mark,
    /// so it matters on the building path alone.
    #[inline(always)]
    pub fn to_chars(&self, tone_placement: TonePlacement) -> Vec<char> {
        match &self.state {
            SyllableState::Building(builder) => builder.to_chars(tone_placement).to_vec(),
            SyllableState::Dead(builder) => builder.to_chars(),
        }
    }

    /// Appends the syllable to `output` as Vietnamese characters, either
    /// precomposed (building) or as the verbatim dead-buffer contents.
    ///
    /// Appends rather than replaces, and unlike [`Self::to_chars`] needs no
    /// intermediate `Vec` on the dead path.
    #[inline(always)]
    pub fn write_to(&self, tone_placement: TonePlacement, output: &mut String) {
        match &self.state {
            SyllableState::Building(builder) => builder.write_to(tone_placement, output),
            SyllableState::Dead(builder) => {
                builder.write_to(output);
            }
        }
    }

    /// Returns a lazy Iterator over the rendered characters of the syllable,
    /// whether in the building phase (with precomposed tones) or verbatim dead phase.
    #[inline]
    pub fn iter_chars(&self, tone_placement: TonePlacement) -> impl Iterator<Item = char> + '_ {
        use crate::syllable::iter::SyllableChars;
        match &self.state {
            SyllableState::Building(builder) => {
                SyllableChars::Building(builder.iter_chars(tone_placement))
            }
            SyllableState::Dead(builder) => SyllableChars::Dead(builder.iter_chars()),
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
    ) -> InputEffect {
        match &mut self.state {
            SyllableState::Dead(builder) => {
                builder.push(input);
                InputEffect::StructurallyChanged
            }
            SyllableState::Building(builder) => match builder.push(keymap, input) {
                Ok(effect) => effect,
                Err(_err) => {
                    let chars = builder.to_chars(tone_placement);
                    let mut dead = DeadSyllable::from_accepted(chars.iter().copied());
                    dead.push(input);
                    self.state = SyllableState::Dead(dead);
                    InputEffect::StructurallyChanged
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
    ) -> InputEffect {
        match &mut self.state {
            SyllableState::Dead(builder) => {
                builder.insert(index, input);
                InputEffect::StructurallyChanged
            }
            SyllableState::Building(builder) => match builder.insert(keymap, index, input) {
                Ok(effect) => effect,
                Err(_err) => {
                    let chars = builder.to_chars(tone_placement);
                    let mut dead = DeadSyllable::from_accepted(chars.iter().copied());
                    dead.insert(index, input);
                    self.state = SyllableState::Dead(dead);
                    InputEffect::StructurallyChanged
                }
            },
        }
    }

    /// Removes the character at `index` from the syllable.
    ///
    /// Transactional on the building path: a deletion that leaves the syllable
    /// invalid is rolled back and the builder keeps its previous contents. On
    /// the dead path it is the reverse — once every character is accepted
    /// again, the buffer is re-parsed under `keymap` and the building phase
    /// resumes.
    pub(crate) fn remove<KM: Keymap>(
        &mut self,
        keymap: &KM,
        tone_placement: TonePlacement,
        index: usize,
    ) -> InputEffect {
        match &mut self.state {
            SyllableState::Dead(builder) => {
                builder.remove(index);
                if builder.is_all_accepted() {
                    let mut building = BuildingSyllable::default();
                    for ch in builder.iter_chars() {
                        if building.push(keymap, ch).is_err() {
                            return InputEffect::StructurallyChanged;
                        }
                    }
                    // Every character parsed cleanly: resume building.
                    self.state = SyllableState::Building(building);
                }
                InputEffect::StructurallyChanged
            }
            SyllableState::Building(builder) => match builder.remove(index, tone_placement) {
                Ok(effect) => effect,
                Err(_) => InputEffect::StructurallyChanged,
            },
        }
    }
}
