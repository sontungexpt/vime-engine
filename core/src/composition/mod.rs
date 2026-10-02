pub mod cursor;

pub use cursor::Cursor;

use crate::{
    keymap::Keymap,
    phonology::TonePlacement,
    syllable::{InsertOutcome, Syllable, SyllableChars},
    util::vec::SmallVec,
};

/// The keystroke buffer: one character per key, held inline up to
/// [`KEYSTROKE_INLINE`].
///
/// The counterpart to [`SyllableChars`] (the parsed form); the two may hold
/// different numbers of characters.
type RawBuffer = SmallVec<char, KEYSTROKE_INLINE>;

/// Inline capacity for buffered keystrokes before spilling to the heap.
///
/// Sixteen characters cover most ordinary words while keeping the inline
/// buffer small. Longer input spills to the heap without imposing a limit.
const KEYSTROKE_INLINE: usize = 16;

/// One value per buffer, so an operation reports how each side fared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parallel<T> {
    rendered: T,
    raw: T,
}

impl<T> Parallel<T> {
    /// The rendered buffer's value.
    #[inline(always)]
    pub const fn rendered(&self) -> &T {
        &self.rendered
    }

    /// The raw buffer's value.
    #[inline(always)]
    pub const fn raw(&self) -> &T {
        &self.raw
    }
}

/// Incremental syllable composition of raw keystrokes into Vietnamese text.
///
/// Keeps the raw keystrokes alongside the parsed [`Syllable`]. The buffers can
/// differ in length (`a` + `w` → `ă` collapses two keys into one character; a
/// dead syllable keeps later input verbatim), so each has its own cursor. The
/// raw buffer decides emptiness and whether the IME handles an edit; the parsed
/// buffer drives rendered output.
///
/// Like the builder, it owns state only: the keymap and the tone-placement
/// scheme are arguments of the operations that use them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition {
    raw: RawBuffer,

    // Kept separate because raw and rendered positions are not one-to-one:
    // `raw_cursor` edits the raw buffer, `rendered_cursor` drives the rest.
    raw_cursor: Cursor,

    rendered: Syllable,
    rendered_cursor: Cursor,
}

impl Composition {
    // --------------------------------------------------------- constructor

    /// Creates an empty composition in the building phase: both buffers and
    /// both cursors start empty at position 0.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            raw: RawBuffer::new(),
            raw_cursor: Cursor::start(),
            rendered: Syllable::new(),
            rendered_cursor: Cursor::start(),
        }
    }

    // ------------------------------------------------------------- queries

    /// Caret position in the rendered buffer, in characters — the position a
    /// frontend shows.
    #[inline(always)]
    pub const fn rendered_cursor(&self) -> usize {
        self.rendered_cursor.get()
    }

    /// Caret position in the raw keystroke buffer, in keystrokes: the buffer the
    /// editing operations act on.
    #[inline(always)]
    pub const fn raw_cursor(&self) -> usize {
        self.raw_cursor.get()
    }

    /// Length of the rendered buffer, in characters — the extent
    /// [`Self::rendered_cursor`] moves within.
    #[inline(always)]
    pub fn rendered_len(&self) -> usize {
        self.rendered.len()
    }

    /// Number of UTF-8 bytes the rendered buffer renders to.
    #[inline(always)]
    pub fn rendered_len_utf8(&self) -> usize {
        self.rendered.len_utf8()
    }

    /// Length of the raw keystroke buffer, in keystrokes.
    #[inline(always)]
    pub fn raw_len(&self) -> usize {
        self.raw.len()
    }

    /// Number of UTF-8 bytes the raw buffer renders to.
    #[inline(always)]
    pub fn raw_len_utf8(&self) -> usize {
        let mut sum = 0;
        let mut i = 0;
        let len = self.raw.len();
        while i < len {
            sum += self.raw[i].len_utf8();
            i += 1;
        }
        sum
    }

    /// The raw keystrokes, in the order typed.
    #[inline(always)]
    pub fn raw(&self) -> &[char] {
        &self.raw
    }

    /// Whether each cursor can move one position left, against its own buffer.
    #[inline(always)]
    pub fn can_move_cursor_left(&self) -> Parallel<bool> {
        Parallel {
            rendered: !self.rendered_cursor.is_at_start(),
            raw: !self.raw_cursor.is_at_start(),
        }
    }

    /// Whether each cursor can move one position right. The buffers may differ in
    /// length, so each is checked against its own.
    #[inline(always)]
    pub fn can_move_cursor_right(&self) -> Parallel<bool> {
        Parallel {
            rendered: !self.rendered_cursor.is_at_end(self.rendered.len()),
            raw: !self.raw_cursor.is_at_end(self.raw.len()),
        }
    }

    /// Whether the buffered syllable spells a complete, valid Vietnamese
    /// syllable; see [`Syllable::is_phonotactically_valid`].
    #[inline]
    pub fn is_phonotactically_valid(&self) -> bool {
        self.rendered.is_phonotactically_valid()
    }

    // ------------------------------------------------------------ mutation

    /// Clears both buffers and both cursors, returning to the building phase.
    #[inline]
    pub fn reset(&mut self) {
        self.raw.clear();
        self.raw_cursor.move_to_start();

        self.rendered.reset();
        self.rendered_cursor.move_to_start();
    }

    /// Inserts `input` at both cursors, parsing under `keymap` and rendering
    /// under `tone_placement`.
    ///
    /// The raw buffer always gains a character; the parsed cursor advances only
    /// on [`InsertOutcome::Extended`], since a transformation consumes the input
    /// without lengthening that buffer.
    pub fn insert<KM: Keymap>(
        &mut self,
        keymap: &KM,
        tone_placement: TonePlacement,
        input: char,
    ) -> Parallel<InsertOutcome> {
        let mut outcome = Parallel {
            rendered: InsertOutcome::Extended,
            raw: InsertOutcome::Extended,
        };

        self.raw.insert(self.raw_cursor.get(), input);

        // SAFETY: insertion always increases the raw buffer length by one, so
        // advancing the cursor by one stays within the new bounds.
        unsafe {
            self.raw_cursor.move_right_unchecked_by(1);
        }

        match self
            .rendered
            .insert(keymap, tone_placement, self.rendered_cursor.get(), input)
        {
            InsertOutcome::Extended => {
                // SAFETY: only this arm lengthens the parsed buffer, so the next
                // position is valid; a transformation leaves it unchanged.
                unsafe {
                    self.rendered_cursor.move_right_unchecked_by(1);
                }
            }
            edit_outcome @ InsertOutcome::Transformed { .. } => {
                outcome.rendered = edit_outcome;
            }
        }
        outcome
    }

    /// Removes the character immediately before each cursor, parsing under
    /// `keymap`, rendering under `tone_placement`. Each side reports whether it
    /// removed anything; a transformation can make the two buffers diverge.
    #[inline]
    pub fn backspace<KM: Keymap>(
        &mut self,
        keymap: &KM,
        tone_placement: TonePlacement,
    ) -> Parallel<bool> {
        let mut result = Parallel {
            rendered: false,
            raw: false,
        };

        if self.raw_cursor.move_left() {
            self.raw.remove(self.raw_cursor.get());
            result.raw = true;
        }

        if self.rendered_cursor.move_left() {
            self.rendered
                .remove(keymap, tone_placement, self.rendered_cursor.get());
            result.rendered = true;
        }
        result
    }

    /// Removes the character at each cursor without moving either cursor,
    /// parsing under `keymap`, rendering under `tone_placement`.
    #[inline]
    pub fn delete<KM: Keymap>(
        &mut self,
        keymap: &KM,
        tone_placement: TonePlacement,
    ) -> Parallel<bool> {
        let mut result = Parallel {
            rendered: false,
            raw: false,
        };

        if !self.raw_cursor.is_at_end(self.raw.len()) {
            self.raw.remove(self.raw_cursor.get());
            result.raw = true;
        }

        if !self.rendered_cursor.is_at_end(self.rendered.len()) {
            self.rendered
                .remove(keymap, tone_placement, self.rendered_cursor.get());
            result.rendered = true;
        }
        result
    }

    /// Moves both cursors up to `by` positions left, clamping each at the start.
    ///
    /// Raw and parsed positions differ, so each field reports whether *that*
    /// cursor moved.
    #[inline]
    pub fn move_cursor_left_by(&mut self, by: usize) -> Parallel<bool> {
        Parallel {
            rendered: self.rendered_cursor.move_left_by(by),
            raw: self.raw_cursor.move_left_by(by),
        }
    }

    /// Moves both cursors up to `by` positions right, clamping each at its own
    /// buffer length.
    #[inline]
    pub fn move_cursor_right_by(&mut self, by: usize) -> Parallel<bool> {
        Parallel {
            rendered: self.rendered_cursor.move_right_by(by, self.rendered.len()),
            raw: self.raw_cursor.move_right_by(by, self.raw.len()),
        }
    }

    // ----------------------------------------------------------- rendering

    /// Renders the parsed composition under `tone_placement`: the Vietnamese
    /// syllable while parsing succeeds, the dead buffer verbatim once it fails.
    #[inline]
    pub fn rendered(&self, tone_placement: TonePlacement) -> SyllableChars {
        self.rendered.to_chars(tone_placement)
    }

    /// Appends the rendered word to `output`: the Vietnamese syllable while
    /// parsing, the verbatim buffer once dead.
    ///
    /// The allocation-free counterpart to [`Self::rendered`]. `output` is not
    /// cleared, so a caller reusing one buffer must empty it first.
    #[inline]
    pub fn write_rendered_to(&self, tone_placement: TonePlacement, output: &mut String) {
        self.rendered.write_to(tone_placement, output);
    }

    /// Appends the raw keystrokes to `output`, which is not cleared.
    #[inline]
    pub fn write_raw_to(&self, output: &mut String) {
        self.raw.iter().for_each(|c| output.push(*c));
    }
}
