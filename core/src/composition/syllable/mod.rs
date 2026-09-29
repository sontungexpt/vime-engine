mod building;
mod dead;
mod input_effect;
mod iter;
mod context;

use crate::{
    keymap::Keymap,
    phonology::{Coda, Onset, TonePlacement, Vowel},
};

pub use building::BuildingSyllable;
pub use dead::DeadSyllable;
pub use input_effect::InputEffect;
pub use context::SyllableContext;

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyllableBuilder<KM: Keymap> {
    context: SyllableContext<KM>,
    state: SyllableState,
}

impl<KM: Keymap> SyllableBuilder<KM> {
    // --------------------------------------------------------- constructor

    /// Creates a building syllable that parses and renders under `context`.
    #[inline]
    pub fn new(context: SyllableContext<KM>) -> Self {
        Self {
            context,
            state: SyllableState::default(),
        }
    }

    // --------------------------------------------------------- context

    /// The context this syllable is parsed and rendered under.
    #[inline(always)]
    pub const fn context(&self) -> &SyllableContext<KM> {
        &self.context
    }

    /// Replaces the context wholesale, leaving the buffered syllable in its
    /// current phase. The new keymap and tone-placement scheme take effect
    /// from the next parse or render; already-buffered characters are not
    /// re-parsed.
    #[inline]
    pub fn set_context(&mut self, context: SyllableContext<KM>) {
        self.context = context;
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

    /// Resets the syllable to an empty building state, keeping the
    /// [`SyllableContext`] (keymap and tone placement) as it is.
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
    #[inline(always)]
    pub fn to_chars(&self) -> Vec<char> {
        match &self.state {
            SyllableState::Building(builder) => {
                builder.to_chars(self.context.tone_placement()).to_vec()
            }
            SyllableState::Dead(builder) => builder.to_chars(),
        }
    }

    /// Appends the syllable to `output` as Vietnamese characters, either
    /// precomposed (building) or as the verbatim dead-buffer contents.
    ///
    /// Appends rather than replaces, and unlike [`Self::to_chars`] needs no
    /// intermediate `Vec` on the dead path.
    #[inline(always)]
    pub fn write_to(&self, output: &mut String) {
        match &self.state {
            SyllableState::Building(builder) => {
                builder.write_to(self.context.tone_placement(), output)
            }
            SyllableState::Dead(builder) => {
                builder.write_to(output);
            }
        }
    }

    /// Returns a lazy Iterator over the rendered characters of the syllable,
    /// whether in the building phase (with precomposed tones) or verbatim dead phase.
    #[inline]
    pub fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        use crate::composition::syllable::iter::SyllableChars;
        match &self.state {
            SyllableState::Building(builder) => {
                SyllableChars::Building(builder.iter_chars(self.context.tone_placement()))
            }
            SyllableState::Dead(builder) => SyllableChars::Dead(builder.iter_chars()),
        }
    }

    // -------------------------------------------------------- mutation

    /// Appends `input` at the end. When the building phase rejects it, the
    /// accepted prefix is frozen into a dead buffer and the input appended
    /// verbatim.
    #[allow(dead_code)]
    pub(crate) fn push(&mut self, input: char) -> InputEffect {
        // Split the borrow: `context` is read while `state` is mutated.
        let Self { context, state } = self;
        let keymap = context.keymap();
        match state {
            SyllableState::Dead(builder) => {
                builder.push(input);
                InputEffect::StructurallyChanged
            }
            SyllableState::Building(builder) => match builder.push(keymap, input) {
                Ok(effect) => effect,
                Err(_err) => {
                    let chars = builder.to_chars(context.tone_placement());
                    let mut dead = DeadSyllable::from_accepted(chars.iter().copied());
                    dead.push(input);
                    self.state = SyllableState::Dead(dead);
                    InputEffect::StructurallyChanged
                }
            },
        }
    }

    /// Inserts `input` at `index`, falling back to the dead buffer on an
    /// out-of-range index or a rejected parse, exactly as [`Self::push`] does.
    pub(crate) fn insert(&mut self, index: usize, input: char) -> InputEffect {
        // Split the borrow: `context` is read while `state` is mutated.
        let Self { context, state } = self;
        match state {
            SyllableState::Dead(builder) => {
                builder.insert(index, input);
                InputEffect::StructurallyChanged
            }
            SyllableState::Building(builder) => {
                match builder.insert(context.keymap(), index, input) {
                    Ok(effect) => effect,
                    Err(_err) => {
                        let chars = builder.to_chars(context.tone_placement());
                        let mut dead = DeadSyllable::from_accepted(chars.iter().copied());
                        dead.insert(index, input);
                        self.state = SyllableState::Dead(dead);
                        InputEffect::StructurallyChanged
                    }
                }
            }
        }
    }

    /// Removes the character at `index` from the syllable.
    ///
    /// Transactional on the building path: a deletion that leaves the syllable
    /// invalid is rolled back and the builder keeps its previous contents. On
    /// the dead path it is the reverse — once every character is accepted
    /// again, the buffer is re-parsed and the building phase resumes.
    pub(crate) fn remove(&mut self, index: usize) -> InputEffect {
        // Split the borrow: `context` is read while `state` is mutated.
        let Self { context, state } = self;
        match state {
            SyllableState::Dead(builder) => {
                builder.remove(index);
                if builder.is_all_accepted() {
                    let mut building = BuildingSyllable::default();
                    for ch in builder.iter_chars() {
                        if building.push(context.keymap(), ch).is_err() {
                            return InputEffect::StructurallyChanged;
                        }
                    }
                    // Every character parsed cleanly: resume building.
                    *state = SyllableState::Building(building);
                }
                InputEffect::StructurallyChanged
            }
            SyllableState::Building(builder) => {
                match builder.remove(index, context.tone_placement()) {
                    Ok(effect) => effect,
                    Err(_) => InputEffect::StructurallyChanged,
                }
            }
        }
    }
}