pub mod cursor;
pub mod syllable;

pub use cursor::Cursor;

use crate::{
    composition::syllable::{InputEffect, SyllableBuilder, SyllableContext},
    keymap::Keymap,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parallel<T> {
    rendered: T,
    raw: T,
}

impl<T> Parallel<T> {
    #[inline(always)]
    pub const fn rendered(&self) -> &T {
        &self.rendered
    }

    #[inline(always)]
    pub const fn raw(&self) -> &T {
        &self.raw
    }
}

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

    rendered: SyllableBuilder<KM>,
    rendered_cursor: Cursor,
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
            raw_cursor: Cursor::start(),
            rendered: syllable_builder,
            rendered_cursor: Cursor::start(),
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
        self.rendered.set_context(context);
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
        self.raw_cursor.move_to_start();

        self.rendered.reset();
        self.rendered_cursor.move_to_start();
    }

    // --------------------------------------------------------- cursor move

    /// Returns `true` if the cursor can move one position to the left.
    ///
    /// This checks the parsed cursor because navigation follows the parsed
    /// composition rather than the raw keystroke count.
    #[inline(always)]
    pub fn can_move_left(&self) -> Parallel<bool> {
        Parallel {
            rendered: self.rendered_cursor.is_at_start(),
            raw: self.raw_cursor.is_at_start(),
        }
    }

    /// Returns `true` if the cursor can move one position to the right.
    ///
    /// This checks the parsed cursor against the parsed buffer length because
    /// the parsed and raw buffers may contain different numbers of characters.
    #[inline(always)]
    pub fn can_move_right(&self) -> Parallel<bool> {
        Parallel {
            rendered: !self.rendered_cursor.is_at_end(self.rendered.len()),
            raw: !self.raw_cursor.is_at_end(self.raw.len()),
        }
    }

    /// Moves both cursors one position to the left.
    ///
    /// Each cursor is clamped independently at position 0 because the raw and
    /// parsed buffers may have different lengths.
    /// Returns `true` if the parsed cursor moved.
    #[inline]
    pub fn move_cursor_left(&mut self) -> Parallel<bool> {
        // Raw cursor always true if parsed_cursor is true so do not need to check
        Parallel {
            rendered: self.rendered_cursor.move_left(),
            raw: self.raw_cursor.move_left(),
        }
    }

    /// Moves both cursors one position to the right.
    ///
    /// Each cursor is bounded by its own buffer length because raw and parsed
    /// positions are not necessarily one-to-one.
    #[inline]
    pub fn move_cursor_right(&mut self) -> Parallel<bool> {
        Parallel {
            rendered: self.rendered_cursor.move_right(self.rendered.len()),
            raw: self.raw_cursor.move_right(self.raw.len()),
        }
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

        match self.rendered.insert(self.rendered_cursor.get(), input) {
            InputEffect::StructurallyChanged => {
                // SAFETY: a structural insertion increases the parsed buffer
                // length by one, making the next cursor position valid. A
                // transformed key consumes the input without lengthening the
                // buffer, which is why this arm is the only one that moves.
                unsafe {
                    self.rendered_cursor.move_right_unchecked();
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
    #[inline]
    pub fn backspace(&mut self) -> Parallel<bool> {
        let mut result = Parallel {
            rendered: false,
            raw: false,
        };

        if self.raw_cursor.move_left() {
            self.raw.remove(self.raw_cursor.get());
            result.raw = true;
        }

        if self.rendered_cursor.move_left() {
            self.rendered.remove(self.rendered_cursor.get());
            result.rendered = true;
        }
        result
    }

    /// Removes the character at each cursor without moving either cursor.
    ///
    /// The character immediately following the cursor is removed, so the
    /// cursor remains at the same position.
    #[inline]
    pub fn delete(&mut self) -> Parallel<bool> {
        let mut result = Parallel {
            rendered: false,
            raw: false,
        };

        if !self.raw_cursor.is_at_end(self.raw.len()) {
            self.raw.remove(self.raw_cursor.get());
            result.raw = true;
        }

        if !self.rendered_cursor.is_at_end(self.rendered.len()) {
            self.rendered.remove(self.rendered_cursor.get());
            result.rendered = true;
        }
        result
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
    pub fn rendered(&self) -> Vec<char> {
        self.rendered.to_chars()
    }

    /// Writes the parsed word into `output`, replacing its contents: the
    /// rendered syllable while parsing, the verbatim buffer once dead.
    ///
    /// The allocation-free counterpart to [`Self::parsed`], for a caller that
    /// writes on every keystroke and can reuse one buffer.
    #[inline]
    pub fn write_rendered_to(&self, output: &mut String) {
        self.rendered.write_to(output);
    }

    #[inline]
    pub fn raw(&self) -> &[char] {
        &self.raw
    }

    #[inline]
    pub fn write_raw_to(&self, output: &mut String) {
        self.raw.iter().for_each(|c| output.push(*c));
    }
}
