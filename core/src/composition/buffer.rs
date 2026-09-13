use std::fmt::{self, Write};

/// An editable list of raw input characters with a cursor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Buffer {
    chars: Vec<char>,
    cursor: usize,
}

impl fmt::Display for Buffer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for character in &self.chars {
            formatter.write_char(*character)?;
        }
        Ok(())
    }
}

impl Buffer {
    pub(crate) const fn new() -> Self {
        Self {
            chars: Vec::new(),
            cursor: 0,
        }
    }

    /// Whether the buffer holds no characters.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    /// The raw characters currently in the buffer.
    #[inline]
    pub fn chars(&self) -> &[char] {
        &self.chars
    }

    /// The cursor position (insertion point) in the buffer.
    #[inline]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Inserts `ch` at the cursor and moves the cursor forward.
    #[inline]
    pub(crate) fn insert(&mut self, ch: char) {
        self.chars.insert(self.cursor, ch);
        self.cursor += 1;
    }

    /// Removes the character before the cursor, if any.
    #[inline]
    pub(crate) fn backspace(&mut self) {
        if self.cursor > 0 {
            self.chars.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    /// Removes the character at the cursor, if any.
    #[inline]
    pub(crate) fn delete(&mut self) {
        if self.cursor < self.chars.len() {
            self.chars.remove(self.cursor);
        }
    }

    /// Moves the cursor one position left, clamped at the start.
    #[inline]
    pub(crate) fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Moves the cursor one position right, clamped at the end.
    #[inline]
    pub(crate) fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.chars.len());
    }

    /// Empties the buffer and resets the cursor.
    #[inline]
    pub(crate) fn clear(&mut self) {
        self.chars.clear();
        self.cursor = 0;
    }
}
