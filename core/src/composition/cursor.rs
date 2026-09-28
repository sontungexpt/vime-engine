/// A cursor position within a sequence.
///
/// A cursor is a bare index: it stores no `len` and cannot check its own
/// bounds, so every method that needs a bound takes `len` as an argument. Safe
/// methods keep the position inside `0..=len` for the `len` they are given; the
/// `unsafe` methods leave that to the caller, and nothing stops a safe call from
/// being handed a `len` that does not match the sequence the cursor actually
/// indexes.
///
/// Ordering is by position.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Cursor(usize);

impl Cursor {
    /// Creates a new cursor starting at position `0`.
    ///
    /// Same as [`Default`].
    #[inline(always)]
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Returns the current 0-based position index.
    #[inline(always)]
    pub const fn get(self) -> usize {
        self.0
    }

    /// Sets the cursor position to `position`, clamped into `0..=len`.
    ///
    /// Equivalent to `position.min(len)`, so an out-of-range `position` lands on
    /// `len` (one past the last element) rather than panicking.
    #[inline(always)]
    pub const fn set(&mut self, position: usize, len: usize) {
        self.0 = if position < len { position } else { len };
    }

    /// Sets the cursor position, skipping the clamp that [`Self::set`] performs.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `position` is within `0..=len` of the
    /// sequence this cursor indexes. `len` is not a parameter here, so the
    /// clamp cannot happen even in principle; a position past the end makes
    /// every later [`Self::get`] an out-of-bounds slice index, which panics or
    /// silently addresses the wrong element.
    #[inline(always)]
    pub const unsafe fn set_unchecked(&mut self, position: usize) {
        self.0 = position;
    }

    /// Resets the cursor to position `0`.
    ///
    /// Same as [`Self::move_to_start`].
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
    ///
    /// A no-op once the cursor has reached `len`, so this never moves past the
    /// end of the sequence.
    #[inline(always)]
    pub const fn move_right(&mut self, len: usize) {
        if self.0 < len {
            self.0 += 1;
        }
    }

    /// Moves the cursor one position to the right, skipping the bound check
    /// that [`Self::move_right`] performs.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the cursor is strictly inside the
    /// sequence, that is `self.get() < len`. `len` is not a parameter here, so
    /// the increment is unconditional and lands one past the end when the
    /// cursor already sat at `len`; every later [`Self::get`] is then an
    /// out-of-bounds slice index.
    #[inline(always)]
    pub const unsafe fn move_right_unchecked(&mut self) {
        self.0 += 1;
    }

    /// Moves the cursor to the beginning (position 0).
    #[inline(always)]
    pub const fn move_to_start(&mut self) {
        self.0 = 0;
    }

    /// Moves the cursor to the end of a sequence (`len`).
    ///
    /// `len` is taken on trust, so the result is only in bounds if it is the
    /// length of the sequence this cursor indexes.
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
    ///
    /// A cursor at `len` is one past the last element, which is where a caret
    /// belongs when it has no selection, so this returns `true` there.
    #[inline(always)]
    pub const fn is_at_end(self, len: usize) -> bool {
        self.0 >= len
    }
}

// Explictly zero-initializes the cursor, so that `Cursor::default()` is equivalent to `Cursor::zero()`.
impl Default for Cursor {
    #[inline(always)]
    fn default() -> Self {
        Self::zero()
    }
}
