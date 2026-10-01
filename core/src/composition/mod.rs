pub mod cursor;

pub use cursor::Cursor;

use crate::{
    keymap::Keymap,
    phonology::TonePlacement,
    syllable::{EditEffect, Syllable, SyllableChars},
    util::vec::SmallVec,
};

/// The raw keystroke buffer: one character per key, held inline up to
/// [`RAW_INLINE`].
///
/// The counterpart to [`SyllableChars`] (the parsed form); the two may hold
/// different numbers of characters.
type RawChars = SmallVec<char, RAW_INLINE>;

/// Inline capacity for raw keystrokes before spilling to the heap.
///
/// Sixteen characters cover most ordinary words while keeping the inline
/// buffer small. Longer input spills to the heap without imposing a limit.
const RAW_INLINE: usize = 16;

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
    raw: RawChars,

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
            raw: RawChars::new(),
            raw_cursor: Cursor::start(),
            rendered: Syllable::new(),
            rendered_cursor: Cursor::start(),
        }
    }

    // ------------------------------------------------------------ state

    /// Clears both buffers and both cursors, returning to the building phase.
    #[inline]
    pub fn reset(&mut self) {
        self.raw.clear();
        self.raw_cursor.move_to_start();

        self.rendered.reset();
        self.rendered_cursor.move_to_start();
    }

    // --------------------------------------------------------- cursor move

    /// Whether each cursor can move one position left, checked against its own
    /// buffer.
    #[inline(always)]
    pub fn can_move_left(&self) -> Parallel<bool> {
        Parallel {
            rendered: self.rendered_cursor.is_at_start(),
            raw: self.raw_cursor.is_at_start(),
        }
    }

    /// Whether each cursor can move one position right, checked against its own
    /// buffer length — the two buffers may differ.
    #[inline(always)]
    pub fn can_move_right(&self) -> Parallel<bool> {
        Parallel {
            rendered: !self.rendered_cursor.is_at_end(self.rendered.len()),
            raw: !self.raw_cursor.is_at_end(self.raw.len()),
        }
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

    // ----------------------------------------------------------- mutation

    /// Inserts `input` at both cursors, parsing under `keymap` and rendering
    /// under `tone_placement`.
    ///
    /// The raw buffer always gains a character; the parsed cursor advances only
    /// on [`EditEffect::StructurallyChanged`], since a transformation consumes
    /// the input without lengthening that buffer.
    pub fn insert<KM: Keymap>(&mut self, keymap: &KM, tone_placement: TonePlacement, input: char) {
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
            EditEffect::StructurallyChanged => {
                // SAFETY: only this arm lengthens the parsed buffer, so the next
                // position is valid; a transformation leaves it unchanged.
                unsafe {
                    self.rendered_cursor.move_right_unchecked_by(1);
                }
            }
            _ => {} // EditEffect::Transformed => {}
        }
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

    // ------------------------------------------------------------ positions

    /// The rendered cursor position, in characters from the start — the caret a
    /// frontend shows. [`Self::raw_cursor`] reports the raw buffer's own
    /// position, since the two buffers differ in length.
    #[inline(always)]
    pub const fn rendered_cursor(&self) -> usize {
        self.rendered_cursor.get()
    }

    /// The raw cursor position, in keystrokes from the start: the buffer the
    /// editing operations act on.
    #[inline(always)]
    pub const fn raw_cursor(&self) -> usize {
        self.raw_cursor.get()
    }

    /// Whether the buffered syllable spells a complete, valid Vietnamese
    /// syllable; see [`Syllable::is_phonotactically_valid`].
    #[inline]
    pub fn is_phonotactically_valid(&self) -> bool {
        self.rendered.is_phonotactically_valid()
    }

    // ----------------------------------------------------------- rendering

    /// Renders the parsed composition under `tone_placement`: the Vietnamese
    /// syllable while parsing succeeds, the dead buffer verbatim once it fails.
    #[inline]
    pub fn rendered(&self, tone_placement: TonePlacement) -> SyllableChars {
        self.rendered.to_chars(tone_placement)
    }

    /// Writes the rendered word into `output`, replacing its contents: the
    /// Vietnamese syllable while parsing, the verbatim buffer once dead.
    ///
    /// The allocation-free counterpart to [`Self::rendered`], for a caller that
    /// writes on every keystroke and can reuse one buffer.
    #[inline]
    pub fn write_rendered_to(&self, tone_placement: TonePlacement, output: &mut String) {
        self.rendered.write_to(tone_placement, output);
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
