/// A cursor position within a sequence.
///
/// The position is guaranteed to remain bounded within `0..=len`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Cursor {
    position: usize,
}

impl Cursor {
    /// Creates a new cursor starting at position `0`.
    #[inline(always)]
    pub const fn new() -> Self {
        Self { position: 0 }
    }

    /// Returns the current 0-based position index.
    #[inline(always)]
    pub const fn position(self) -> usize {
        self.position
    }

    /// Resets the cursor to position `0`.
    #[inline(always)]
    pub const fn reset(&mut self) {
        self.position = 0;
    }

    /// Moves the cursor one position to the left (saturates at 0).
    #[inline(always)]
    pub const fn move_left(&mut self) {
        self.position = self.position.saturating_sub(1);
    }

    /// Moves the cursor one position to the right, bounded by `len`.
    #[inline(always)]
    pub const fn move_right(&mut self, len: usize) {
        if self.position < len {
            self.position += 1;
        }
    }

    /// Moves the cursor to the beginning (position 0).
    #[inline(always)]
    pub const fn move_to_start(&mut self) {
        self.position = 0;
    }

    /// Moves the cursor to the end of a sequence (`len`).
    #[inline(always)]
    pub const fn move_to_end(&mut self, len: usize) {
        self.position = len;
    }

    /// Sets the cursor position, clamped to `len`.
    #[inline(always)]
    pub const fn set(&mut self, position: usize, len: usize) {
        self.position = if position < len { position } else { len };
    }

    /// Returns whether the cursor is at the start (position 0).
    #[inline(always)]
    pub const fn is_at_start(self) -> bool {
        self.position == 0
    }

    /// Returns whether the cursor is at or past the end (`len`).
    #[inline(always)]
    pub const fn is_at_end(self, len: usize) -> bool {
        self.position >= len
    }
}
