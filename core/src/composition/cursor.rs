/// A cursor position within a sequence: a bare index that stores no `len`, so
/// every method that needs a bound takes `len` as an argument.
///
/// Safe methods keep the position in `0..=len` for the `len` they are given;
/// `unsafe` ones leave that to the caller, and a safe call may be handed a `len`
/// that does not match the indexed sequence. Ordering is by position.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Cursor(usize);

impl Cursor {
    /// Creates a cursor at position `0`; same as [`Default`].
    #[inline(always)]
    pub const fn start() -> Self {
        Self(0)
    }

    /// Returns the current 0-based position.
    #[inline(always)]
    pub const fn get(self) -> usize {
        self.0
    }

    /// Sets the position to `position`, clamped into `0..=len`: an
    /// out-of-range value lands on `len` rather than panicking.
    #[inline(always)]
    pub const fn set(&mut self, position: usize, len: usize) {
        self.0 = if position < len { position } else { len };
    }

    /// Sets the position, skipping the clamp [`Self::set`] performs.
    ///
    /// # Safety
    ///
    /// `position` must be within `0..=len` of the indexed sequence; past the
    /// end, every later [`Self::get`] is an out-of-bounds index — it panics or
    /// addresses the wrong element.
    #[inline(always)]
    pub const unsafe fn set_unchecked(&mut self, position: usize) {
        self.0 = position;
    }

    /// Moves one position left; `false` means the caret was already at the
    /// start. See [`Self::move_left_by`] for moving several at once.
    #[inline(always)]
    pub const fn move_left(&mut self) -> bool {
        self.move_left_by(1)
    }

    /// Moves `by` positions left, skipping the clamp [`Self::move_left_by`] does.
    ///
    /// # Safety
    ///
    /// `by` must be `<= self.get()`; with no `len` to compare against, a larger
    /// `by` underflows and every later [`Self::get`] is an out-of-bounds index.
    #[inline(always)]
    pub const unsafe fn move_left_unchecked_by(&mut self, by: usize) {
        self.0 -= by;
    }

    /// Moves up to `by` positions left, clamping at the start. Returns whether
    /// the cursor moved: `false` only when there was nowhere to go, including
    /// `by == 0`.
    #[inline(always)]
    pub const fn move_left_by(&mut self, by: usize) -> bool {
        // One rule for every `by`: the saturated subtract is already the clamp,
        // so `by == 0` needs no separate case.
        let target = self.0.saturating_sub(by);
        let moved = target != self.0;
        self.0 = target;
        moved
    }

    /// Moves one position right, bounded by `len`: a no-op at the end, so it
    /// never moves past it. Returns whether it moved. See
    /// [`Self::move_right_by`] for moving several at once.
    #[inline(always)]
    pub const fn move_right(&mut self, len: usize) -> bool {
        self.move_right_by(1, len)
    }

    /// Moves `by` positions right with no bounds check, returning the new
    /// position. Skips the clamp [`Self::move_right_by`] does.
    ///
    /// # Safety
    ///
    /// `self.get() + by` must stay inside the sequence; with no `len` to check
    /// against, every later [`Self::get`] is then an out-of-bounds index.
    #[inline(always)]
    pub const unsafe fn move_right_unchecked_by(&mut self, by: usize) -> usize {
        self.0 += by;
        self.0
    }

    /// Moves up to `by` positions right, clamping at `len`. Returns whether it
    /// moved: `false` for `by == 0` or when the cursor already sits at `len`.
    #[inline(always)]
    pub const fn move_right_by(&mut self, by: usize, len: usize) -> bool {
        if self.is_at_end(len) || by == 0 {
            return false;
        }

        let remaining = len - self.0;
        if by < remaining {
            self.0 += by;
        } else {
            self.0 = len;
        }
        true
    }

    /// Moves the cursor to the start (position 0) and returns it.
    #[inline(always)]
    pub const fn move_to_start(&mut self) -> usize {
        self.0 = 0;
        self.0
    }

    /// Moves the cursor to `len` and returns it. `len` is taken on trust, so
    /// the result is in bounds only if it is the sequence's real length.
    #[inline(always)]
    pub const fn move_to_end(&mut self, len: usize) -> usize {
        self.0 = len;
        self.0
    }

    /// Returns whether the cursor is at the start (position 0).
    #[inline(always)]
    pub const fn is_at_start(self) -> bool {
        self.0 == 0
    }

    /// Whether the cursor is at or past `len`. One past the last element is
    /// where a caret belongs with no selection, so this is `true` there.
    #[inline(always)]
    pub const fn is_at_end(self, len: usize) -> bool {
        self.0 >= len
    }
}

// `default()` is just `start()`: zero-initialized, no other setup.
impl Default for Cursor {
    #[inline(always)]
    fn default() -> Self {
        Self::start()
    }
}
