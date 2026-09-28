pub mod cursor;
pub mod syllable;

pub use cursor::Cursor;

use crate::{
    composition::syllable::{InputEffect, SyllableBuilder},
    keymap::Keymap,
    phonology::TonePlacement,
};

/// Incremental syllable parser driven by a [`Keymap`].
///
/// Holds the keystrokes exactly as typed (`raw`) alongside the parsed
/// [`SyllableBuilder`] syllable, each with its own cursor. The two buffers are
/// deliberately allowed to differ in length: a transform key collapses one
/// syllable character out of two keystrokes (`a` + `w` → `ă`), and once the
/// parse has died every later keystroke is recorded as rejected without being
/// parsed at all. Hence two cursors rather than one.
///
/// `len` and `cursor` describe the raw buffer, which is what an
/// `empty`/`Forward` decision is judged on; [`Self::rendered`] reports the
/// parsed syllable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition<KM: Keymap> {
    raw_input: Vec<char>,
    raw_cursor: Cursor,

    parsed: SyllableBuilder<KM>,
    parsed_cursor: Cursor,
}

impl<KM: Keymap> Composition<KM> {
    // ------------------------------------------------------------- constructor

    /// Creates a parser wrapping an already-constructed [`SyllableBuilder`],
    /// which supplies the keymap, the tone-placement scheme and the initial
    /// empty syllable state. Both buffers start empty with their cursors at
    /// position 0.
    #[inline(always)]
    pub fn new(syllable_builder: SyllableBuilder<KM>) -> Self {
        Self {
            raw_input: Vec::new(),
            raw_cursor: Cursor::zero(),
            parsed: syllable_builder,
            parsed_cursor: Cursor::zero(),
        }
    }

    // ---------------------------------------------------------------- keymap

    /// Swaps the active keymap without touching the buffered composition.
    #[inline]
    pub fn set_keymap(&mut self, keymap: KM) {
        self.parsed.set_keymap(keymap);
    }

    // --------------------------------------------------------- tone placement

    /// Replaces the tone-placement scheme without touching the buffered
    /// composition; pending vowels are re-rendered under the new scheme.
    #[inline]
    pub fn set_tone_placement(&mut self, tone_placement: TonePlacement) {
        self.parsed.set_tone_placement(tone_placement);
    }

    // --------------------------------------------------------------- state

    /// Resets the composition to its initial empty state: the raw buffer and
    /// both cursors are cleared, and the syllable returns to a fresh, empty
    /// building buffer, discarding any dead fallback.
    ///
    /// The keymap and tone-placement scheme are left alone — they are
    /// configuration rather than buffered input.
    #[inline]
    pub fn reset(&mut self) {
        self.raw_input.clear();
        self.raw_cursor.reset();

        self.parsed.reset();
        self.parsed_cursor.reset();
    }

    /// Whether the raw input buffer holds no characters.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.raw_input.is_empty()
    }

    /// The number of buffered characters (from the raw input buffer).
    #[inline(always)]
    pub(crate) const fn len(&self) -> usize {
        self.raw_input.len()
    }

    /// The current 0-based caret position within the raw input buffer.
    #[inline(always)]
    pub(crate) const fn cursor(&self) -> usize {
        self.raw_cursor.get()
    }

    // ------------------------------------------------------------ cursor move

    /// Moves both cursors one position to the left, saturating at position 0.
    ///
    /// The two cursors move independently of each other's buffer length, so
    /// they are clamped by their own `Cursor::move_left` rather than against
    /// either buffer.
    #[inline]
    pub fn move_left(&mut self) {
        self.raw_cursor.move_left();
        self.parsed_cursor.move_left();
    }

    /// Moves both cursors one position to the right, each bounded by its own
    /// buffer length, so either may stop earlier than the other.
    #[inline]
    pub fn move_right(&mut self) {
        self.raw_cursor.move_right(self.raw_input.len());
        self.parsed_cursor.move_right(self.parsed.len());
    }

    // ------------------------------------------------------------- mutation

    /// Inserts `input` at the raw cursor and mirrors it into the syllable.
    ///
    /// A transform key (`a` + `w` → `ă`) is consumed by the syllable without
    /// becoming a new character, so only [`InputEffect::StructurallyChanged`]
    /// advances the syllable cursor.
    pub fn insert(&mut self, input: char) {
        self.raw_input.insert(self.raw_cursor.get(), input);
        self.raw_cursor.move_right(self.raw_input.len());

        match self.parsed.insert(self.parsed_cursor.get(), input) {
            InputEffect::StructurallyChanged => {
                self.parsed_cursor.move_right(self.parsed.len());
            }
            InputEffect::Transformed => {}
        }
    }

    /// Deletes the character before each cursor, moving both cursors one
    /// position left.
    ///
    /// # Panics / silent misbehaviour
    ///
    /// Requires both cursors to sit strictly inside their buffers. Checking the
    /// raw buffer for emptiness is *not* enough: [`Cursor::move_left`] saturates
    /// at position 0, so a cursor already at the start leaves its index at 0 and
    /// the removal then deletes the first character instead of doing nothing.
    /// Undo a transform key (`ă` ← `a` + `w`) by deleting the syllable
    /// character, not the keystroke that produced it.
    #[inline]
    pub fn backspace(&mut self) {
        self.raw_cursor.move_left();
        self.raw_input.remove(self.raw_cursor.get());

        self.parsed_cursor.move_left();
        match self.parsed.remove(self.parsed_cursor.get()) {
            InputEffect::StructurallyChanged => {}
            InputEffect::Transformed => {}
        }
    }

    /// Deletes the character at each cursor, leaving both cursors where they
    /// are. The syllable cursor does not move, since the character that
    /// followed the cursor is the one going away.
    ///
    /// # Panics
    ///
    /// Panics if either cursor sits at the end of its buffer, where the
    /// removal index is out of bounds. The raw cursor is at the end whenever the
    /// caret is at the tail of the input, so a caller must check the raw
    /// `cursor` against the raw `len` first.
    #[inline]
    pub fn delete(&mut self) {
        self.raw_input.remove(self.raw_cursor.get());
        match self.parsed.remove(self.parsed_cursor.get()) {
            InputEffect::StructurallyChanged => {}
            InputEffect::Transformed => {}
        }
    }

    // -------------------------------------------------------------- rendering

    /// Renders the parsed syllable as Vietnamese characters: the precomposed
    /// form while building, or the dead buffer's verbatim contents once the
    /// parse has failed.
    #[inline]
    pub fn rendered(&self) -> String {
        self.parsed.to_chars().iter().collect()
    }
}
