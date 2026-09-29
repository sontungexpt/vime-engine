pub mod cursor;
pub mod syllable;

pub use cursor::Cursor;

use crate::{
    composition::syllable::{InputEffect, SyllableBuilder, SyllableContext},
    keymap::Keymap,
    phonology::TonePlacement,
};

/// Incremental syllable composition driven by a [`Keymap`].
///
/// Keeps the user's raw keystrokes alongside the parsed [`SyllableBuilder`].
/// The two buffers may have different lengths because input transformations
/// can collapse multiple keystrokes into a single parsed character
/// (`a` + `w` → `ă`), while a dead syllable preserves subsequent input
/// verbatim.
///
/// Each buffer has its own cursor because raw and parsed positions are not
/// necessarily one-to-one.
///
/// The raw buffer is authoritative for composition emptiness and for deciding
/// whether editing or navigation can be handled by the IME. The parsed buffer
/// is authoritative for rendered output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition<KM: Keymap> {
    raw: Vec<char>,

    // Kept separately because raw and parsed positions are not necessarily
    // one-to-one. Most editing/navigation decisions use `parsed_cursor`;
    // `raw_cursor` is used to modify the raw keystroke buffer.
    raw_cursor: Cursor,

    parsed: SyllableBuilder<KM>,
    parsed_cursor: Cursor,
}

impl<KM: Keymap> Composition<KM> {
    // --------------------------------------------------------- constructor

    /// Creates an empty composition from an already-constructed
    /// [`SyllableBuilder`].
    ///
    /// The builder supplies the keymap, tone-placement scheme and initial
    /// parser state. Both buffers and both cursors start empty at position 0.
    #[inline(always)]
    pub fn new(syllable_builder: SyllableBuilder<KM>) -> Self {
        Self {
            raw: Vec::new(),
            raw_cursor: Cursor::zero(),
            parsed: syllable_builder,
            parsed_cursor: Cursor::zero(),
        }
    }

    // --------------------------------------------------------------- context

    /// Replaces the parse context without modifying the buffered composition
    /// or either cursor.
    ///
    /// The new keymap and tone-placement scheme are used by subsequent parsing
    /// and rendering.
    #[inline]
    pub fn set_context(&mut self, context: SyllableContext<KM>) {
        self.parsed.set_context(context);
    }

    /// Replaces the active keymap without modifying the buffered composition.
    #[inline]
    pub fn set_keymap(&mut self, keymap: KM) {
        self.parsed.set_keymap(keymap);
    }

    /// Replaces the tone-placement scheme without modifying the buffered
    /// composition. The existing parsed syllable is rendered using the new
    /// scheme.
    #[inline]
    pub fn set_tone_placement(&mut self, tone_placement: TonePlacement) {
        self.parsed.set_tone_placement(tone_placement);
    }

    // ------------------------------------------------------------ state

    /// Resets the composition to an empty building state.
    ///
    /// The raw buffer, parsed state and both cursors are cleared. The parse
    /// context is preserved because it is configuration rather than input
    /// state.
    #[inline]
    pub fn reset(&mut self) {
        self.raw.clear();
        self.raw_cursor.reset();

        self.parsed.reset();
        self.parsed_cursor.reset();
    }

    /// Returns `true` when no raw keystrokes are buffered.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Returns the current composition cursor position in Unicode characters.
    pub fn cursor_pos(&self) -> usize {
        self.parsed_cursor.get()
    }

    /// Returns the rendered composition length in Unicode characters.
    pub fn length(&self) -> usize {
        self.parsed.len()
    }

    // --------------------------------------------------------- cursor move

    /// Returns `true` if the cursor can move one position to the left.
    ///
    /// This checks the parsed cursor because navigation follows the parsed
    /// composition rather than the raw keystroke count.
    #[inline(always)]
    pub fn can_move_left(&self) -> bool {
        !self.parsed_cursor.is_at_start()
    }

    /// Returns `true` if the cursor can move one position to the right.
    ///
    /// This checks the parsed cursor against the parsed buffer length because
    /// the parsed and raw buffers may contain different numbers of characters.
    #[inline(always)]
    pub fn can_move_right(&self) -> bool {
        !self.parsed_cursor.is_at_end(self.parsed.len())
    }

    /// Moves both cursors one position to the left.
    ///
    /// Each cursor is clamped independently at position 0 because the raw and
    /// parsed buffers may have different lengths.
    #[inline]
    pub fn move_left(&mut self) {
        self.raw_cursor.move_left();
        self.parsed_cursor.move_left();
    }

    /// Moves both cursors one position to the right.
    ///
    /// Each cursor is bounded by its own buffer length because raw and parsed
    /// positions are not necessarily one-to-one.
    #[inline]
    pub fn move_right(&mut self) {
        self.raw_cursor.move_right(self.raw.len());
        self.parsed_cursor.move_right(self.parsed.len());
    }

    // ----------------------------------------------------------- mutation

    /// Inserts `input` at the current cursor position.
    ///
    /// The raw buffer always gains one character. The parsed buffer may either
    /// gain a character or consume the input as a transformation, so the
    /// parsed cursor advances only for [`InputEffect::StructurallyChanged`].
    pub fn insert(&mut self, input: char) {
        self.raw.insert(self.raw_cursor.get(), input);

        // SAFETY: insertion always increases the raw buffer length by one, so
        // advancing the cursor by one stays within the new bounds.
        unsafe {
            self.raw_cursor.move_right_unchecked();
        }

        match self.parsed.insert(self.parsed_cursor.get(), input) {
            InputEffect::StructurallyChanged => {
                // SAFETY: a structural insertion increases the parsed buffer
                // length by one, making the next cursor position valid. A
                // transformed key consumes the input without lengthening the
                // buffer, which is why this arm is the only one that moves.
                unsafe {
                    self.parsed_cursor.move_right_unchecked();
                }
            }
            InputEffect::Transformed => {}
        }
    }

    /// Removes the character immediately before each cursor.
    ///
    /// The raw buffer removes one keystroke, while the parsed buffer removes
    /// the corresponding parsed character. A transformed input may therefore
    /// affect the parsed buffer differently from the raw buffer.
    ///
    /// # Panics
    ///
    /// Panics if either cursor is already at the beginning of its buffer.
    /// Callers should check the appropriate cursor before calling this method.
    #[inline]
    pub fn backspace(&mut self) {
        self.raw.remove(self.raw_cursor.move_left());

        match self.parsed.remove(self.parsed_cursor.move_left()) {
            InputEffect::StructurallyChanged => {}
            InputEffect::Transformed => {}
        }
    }

    /// Removes the character at each cursor without moving either cursor.
    ///
    /// The character immediately following the cursor is removed, so the
    /// cursor remains at the same position.
    ///
    /// # Panics
    ///
    /// Panics if either cursor is at the end of its buffer.
    #[inline]
    pub fn delete(&mut self) {
        self.raw.remove(self.raw_cursor.get());

        match self.parsed.remove(self.parsed_cursor.get()) {
            InputEffect::StructurallyChanged => {}
            InputEffect::Transformed => {}
        }
    }

    // ----------------------------------------------------------- rendering

    /// Renders the current parsed composition.
    ///
    /// While the syllable is valid, rendering produces its Vietnamese form.
    /// Once parsing enters the dead state, rendering preserves the dead
    /// buffer's characters verbatim.
    /// The keystrokes as this composition parses them, as a fresh [`String`].
    ///
    /// While the parse succeeds this is the spelled-out syllable; once it has
    /// failed the raw buffer comes back verbatim.
    #[inline]
    pub fn parsed(&self) -> String {
        let mut output = String::new();
        self.write_parsed_to(&mut output);
        output
    }

    /// Writes the parsed word into `output`, replacing its contents: the
    /// rendered syllable while parsing, the verbatim buffer once dead.
    ///
    /// The allocation-free counterpart to [`Self::parsed`], for a caller that
    /// writes on every keystroke and can reuse one buffer.
    #[inline]
    pub fn write_parsed_to(&self, output: &mut String) {
        self.parsed.write_to(output);
    }
}
