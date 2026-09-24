/// A cursor position within a sequence.
///
/// The position is guaranteed to remain bounded within `0..=len`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Cursor(usize);

impl Cursor {
    /// Creates a new cursor starting at position `0`.
    #[inline(always)]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Returns the current 0-based position index.
    #[inline(always)]
    pub const fn get(self) -> usize {
        self.0
    }

    /// Sets the cursor position, clamped to `len`.
    #[inline(always)]
    pub const fn set(&mut self, position: usize, len: usize) {
        self.0 = if position < len { position } else { len };
    }

    /// Resets the cursor to position `0`.
    #[inline(always)]
    pub const fn reset(&mut self) {
        self.0 = 0;
    }

    /// Moves the cursor one position to the left (saturates at 0).
    #[inline(always)]
    pub const fn move_left(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }

    /// Moves the cursor one position to the right, bounded by `len`.
    #[inline(always)]
    pub const fn move_right(&mut self, len: usize) {
        if self.0 < len {
            self.0 += 1;
        }
    }

    /// Moves the cursor to the beginning (position 0).
    #[inline(always)]
    pub const fn move_to_start(&mut self) {
        self.0 = 0;
    }

    /// Moves the cursor to the end of a sequence (`len`).
    #[inline(always)]
    pub const fn move_to_end(&mut self, len: usize) {
        self.0 = len;
    }

    /// Returns whether the cursor is at the start (position 0).
    #[inline(always)]
    pub const fn is_at_start(self) -> bool {
        self.0 == 0
    }

    /// Returns whether the cursor is at or past the end (`len`).
    #[inline(always)]
    pub const fn is_at_end(self, len: usize) -> bool {
        self.0 >= len
    }
}
