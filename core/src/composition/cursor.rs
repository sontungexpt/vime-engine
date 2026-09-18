/// A cursor position within a sequence.
///
/// The position is expected to be in `0..=len`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Cursor {
    position: usize,
}

impl Cursor {
    #[inline(always)]
    pub const fn new() -> Self {
        return Self { position: 0 };
    }

    #[inline(always)]
    pub const fn position(self) -> usize {
        self.position
    }

    #[inline(always)]
    pub const fn reset(&mut self) {
        self.position = 0
    }

    /// Moves the cursor one position to the left.
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

    /// Moves the cursor to the beginning.
    #[inline(always)]
    pub const fn move_to_start(&mut self) {
        self.position = 0;
    }

    /// Moves the cursor to the end of a sequence.
    #[inline(always)]
    pub const fn move_to_end(&mut self, len: usize) {
        self.position = len;
    }

    /// Sets the cursor position, clamped to `len`.
    #[inline(always)]
    pub const fn set(&mut self, position: usize, len: usize) {
        self.position = if position < len { position } else { len };
    }

    /// Returns whether the cursor is at the beginning.
    #[inline(always)]
    pub const fn is_at_start(self) -> bool {
        self.position == 0
    }

    /// Returns whether the cursor is at the end.
    #[inline(always)]
    pub const fn is_at_end(self, len: usize) -> bool {
        self.position >= len
    }
}

impl Default for Cursor {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
