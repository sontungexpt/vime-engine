pub mod cursor;
pub mod syllable;

pub use cursor::Cursor;
pub use syllable::{
    CharStatus, DeadSyllableBuilder, SyllableError, SyllableState, TransformEffect,
    ValidSyllableBuilder,
};

use crate::{
    composition::syllable::SyllableBuilder, keymap::Keymap, phonology::rules::ToneScheme,
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
    /// Creates a parser backed by `keymap`, starting in the `Onset` phase.
    #[inline(always)]
    pub fn new(keymap: KM) -> Self {
        Self {
            raw: Vec::new(),
            raw_cursor: Cursor::default(),

            syllable: SyllableBuilder::new(keymap, ToneScheme::Modern),
            syllable_cursor: Cursor::default(),
        }
    }

    /// Resets the composition to its initial empty state: the raw input
    /// buffer, the syllable, the parse phase and any fallback state are all
    /// cleared.
    #[inline]
    pub fn reset(&mut self) {
        self.raw.clear();
        self.raw_cursor.reset();

        self.syllable_cursor.reset();
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    #[inline(always)]
    pub const fn syllable(&self) -> &SyllableBuilder<KM> {
        &self.syllable
    }

    #[inline]
    pub fn backspace(&mut self) {
        todo!()
    }

    #[inline]
    pub fn delete(&mut self) {
        todo!()
    }

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

    /// Inserts `input` into the syllable at `index`.
    ///
    /// Not implemented yet: anything at or past the end of the syllable simply
    /// falls through to [`Self::append`].
    pub fn insert(&mut self, input: char) {
        self.raw.insert(self.raw_cursor.position(), input);

        let composed_len = self.syllable.len();

        // If the index is beyond the end of the syllable, push the input as-is.
        if self.syllable_cursor.is_at_end(composed_len) {
            self.syllable.push(input);
        }
        // Example: "trường"
        //
        // chars: ['t', 'r', 'ư', 'ờ', 'n', 'g']
        //        [0]  [1]  [2]  [3]  [4]  [5]
        //
        // onset: ['t', 'r']
        // vowels: ['ư', 'ờ']
        // coda: ['n', 'g']
        //
        // Vowel range is between onset end and coda start:
        //
        // "tr|ườ|ng"
        //     ^   ^
        //     2   4
        //
        // onset_end = 2
        // coda_start = 4

        // Onset insertion positions: 0..=1 in example
        // if index < onset_len {
        // Only one effect can happens that is stroke d
        // }
        // Vowel insertion positions: 2..=4 in example
        // else if index < onset_len + vowel_len + 1 {
        // 1. Stroke modifiers:
        //    Any position in the vowel sequence may update/revert the stroke.
        //
        // 2. Tone modifiers:
        //    Any position in the vowel sequence may update/revert the tone.
        //    Tone is stored at syllable level; the renderer recalculates
        //    the tone position.
        //
        // 3. Shape modifiers:
        //    Apply/revert only when the modifier is immediately after
        //    a compatible vowel.
        // }
        // Coda insertion positions: 5.. in example
        // else {
        // 1. Any stroke modifier key typed in any position in coda sequence will update or revert the stroke
        // 2. Any tone modifier key typed in any position in coda sequence will update or revert the tone
        // 3. Any shape modifier key typed in any position in coda sequence will update or revert the shape
        // }
    }

    /// Removes the character at `index` from the syllable.
    ///
    /// Not implemented yet; placeholder for in-place editing of the current
    /// syllable.
    pub fn remove_at(&mut self, index: usize) {
        // let (onset_len, vowel_len, coda_len, len) = self.syllable.len_parts();

        // Example: "trường"
        //
        // chars: ['t', 'r', 'ư', 'ờ', 'n', 'g']
        //        [0]  [1]  [2]  [3]  [4]  [5]
        //
        // onset: ['t', 'r']
        // vowels: ['ư', 'ờ']
        // coda: ['n', 'g']
        //
        // Vowel range is between onset end and coda start:
        //
        // "tr|ườ|ng"
        //     ^   ^
        //     2   4
        //
        // onset_end = 2
        // coda_start = 4

        // Onset removal positions: 0..=1 in example
        // if index < onset_len {
        // Just allow remove of onset characters
        // Need to revalidate the onset after removal
        // May be need to revalidate all syllable
        // }
        // Vowel removal positions: 2..=4 in example
        // else if index < onset_len + vowel_len + 1 {
        // If the syllable had tone. And the removal position is not the tone position after caculated.
        // the tone needs to be recalculated position after removal (May be do in another method)

        // May need to revalidate the nucleus after removal
        // May be need to revalidate all syllable

        // NOTE: Do not recaculate `uo` because it is hard to predict
        // For example `ư ơ o` and we remove the `ơ` at position 2
        // then should we change the o at position 3 to `ơ`?
        // Because current nucleus is `ưo`?
        // }
        // Coda removal positions: 5.. in example
        // else {
        // Just allow remove of coda characters
        // Need to revalidate the coda after removal
        // May be need to revalidate all syllable
        // }
    }
}
