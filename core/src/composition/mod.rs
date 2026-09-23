pub mod cursor;
pub mod syllable;

pub use cursor::Cursor;

use crate::{
    composition::syllable::{InputEffect, SyllableBuilder},
    keymap::Keymap,
    phonology::rules::TonePlacement,
};

/// Incremental syllable parser driven by a [`RuleEngine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition<KM: Keymap> {
    raw: Vec<char>,
    raw_cursor: Cursor,

    syllable: SyllableBuilder<KM>,
    syllable_cursor: Cursor,
}

impl<KM: Keymap> Composition<KM> {
    // ------------------------------------------------------------- constructor

    /// Creates a parser backed by `keymap`, starting in the `Onset` phase.
    #[inline(always)]
    pub fn new(syllable_builder: SyllableBuilder<KM>) -> Self {
        Self {
            raw: Vec::new(),
            raw_cursor: Cursor::default(),
            syllable: syllable_builder,
            syllable_cursor: Cursor::default(),
        }
    }

    // ---------------------------------------------------------------- keymap

    /// Swaps the active keymap without touching the buffered composition.
    #[inline]
    pub fn set_keymap(&mut self, keymap: KM) {
        self.syllable.set_keymap(keymap);
    }

    // --------------------------------------------------------- tone placement

    /// Replaces the tone-placement scheme without touching the buffered
    /// composition; pending vowels are re-rendered under the new scheme.
    #[inline]
    pub fn set_tone_placement(&mut self, tone_placement: TonePlacement) {
        self.syllable.set_tone_placement(tone_placement);
    }

    // --------------------------------------------------------------- state

    /// Resets the composition to its initial empty state: the raw input
    /// buffer, the syllable, the parse phase and any fallback state are all
    /// cleared.
    #[inline]
    pub fn reset(&mut self) {
        self.raw.clear();
        self.raw_cursor.reset();

        self.syllable.reset();
        self.syllable_cursor.reset();
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// The number of buffered characters (from the raw input buffer).
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.raw.len()
    }

    /// The current 0-based cursor position within the raw input buffer.
    #[inline(always)]
    pub const fn cursor(&self) -> usize {
        self.raw_cursor.position()
    }

    #[inline(always)]
    pub const fn syllable(&self) -> &SyllableBuilder<KM> {
        &self.syllable
    }

    // ------------------------------------------------------------ cursor move

    #[inline]
    pub fn move_left(&mut self) {
        self.raw_cursor.move_left();
        self.syllable_cursor.move_left();
    }

    #[inline]
    pub fn move_right(&mut self) {
        self.raw_cursor.move_right(self.raw.len());
        self.syllable_cursor.move_right(self.syllable.len());
    }

    // ------------------------------------------------------------- mutation

    /// Inserts `input` into the syllable at `index`.
    ///
    /// Not implemented yet: anything at or past the end of the syllable simply
    /// falls through to [`Self::append`].
    pub fn insert(&mut self, input: char) {
        self.raw.insert(self.raw_cursor.position(), input);
        self.raw_cursor.move_right(self.raw.len());

        match self.syllable.insert(self.syllable_cursor.position(), input) {
            InputEffect::StructurallyChanged => {
                self.syllable_cursor.move_right(self.syllable.len());
            }
            InputEffect::Transformed => {}
        }
    }

    #[inline]
    pub fn backspace(&mut self) {
        self.raw_cursor.move_left();
        self.raw.remove(self.raw_cursor.position());

        self.syllable_cursor.move_left();
        match self.syllable.remove(self.syllable_cursor.position()) {
            InputEffect::StructurallyChanged => {}
            InputEffect::Transformed => {}
        }
    }

    #[inline]
    pub fn delete(&mut self) {
        self.raw.remove(self.raw_cursor.position());
        match self.syllable.remove(self.syllable_cursor.position()) {
            InputEffect::StructurallyChanged => {}
            InputEffect::Transformed => {}
        }
    }

    // -------------------------------------------------------------- rendering

    pub fn rendered(&self) -> String {
        self.syllable.to_chars().iter().collect()
    }
}
