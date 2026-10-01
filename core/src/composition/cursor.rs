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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Cursor(usize);

impl Cursor {
    /// Creates a new cursor starting at position `0`.
    ///
    /// Same as [`Default`].
    #[inline(always)]
    pub const fn start() -> Self {
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

    /// Moves the cursor one position to the left.
    ///
    /// Returns whether it moved, so `false` means the caret was already at the
    /// start. See [`Self::move_left_by`] for moving several positions at once.
    #[inline(always)]
    pub const fn move_left(&mut self) -> bool {
        self.move_left_by(1)
    }

    /// Moves `by` positions left, skipping the clamp [`Self::move_left_by`] does.
    ///
    /// # Safety
    ///
    /// The caller must guarantee `by <= self.get()`. There is no `len` here to
    /// compare against, so a larger `by` underflows and every later
    /// [`Self::get`] is then an out-of-bounds index.
    #[inline(always)]
    pub const unsafe fn move_left_unchecked_by(&mut self, by: usize) {
        self.0 -= by;
    }

    /// Moves up to `by` positions left, clamping at the start.
    ///
    /// Returns whether the cursor moved: `true` even when `by` overshoots and
    /// the caret stops early, `false` only when there was nowhere to go —
    /// including `by == 0`, which is a request to move nowhere.
    #[inline(always)]
    pub const fn move_left_by(&mut self, by: usize) -> bool {
        // One rule for every `by`: clamp down, then report whether the position
        // actually changed. Testing `by == 0` up front would be a second case to
        // keep in step, and the saturated subtract is already the clamp.
        let target = self.0.saturating_sub(by);
        let moved = target != self.0;
        self.0 = target;
        moved
    }

    /// Moves the cursor one position to the right, bounded by `len`.
    ///
    /// A no-op once the cursor has reached `len`, so this never moves past the
    /// end of the sequence.
    ///
    /// Returns whether it moved. See [`Self::move_right_by`] for moving several
    /// positions at once.
    #[inline(always)]
    pub const fn move_right(&mut self, len: usize) -> bool {
        self.move_right_by(1, len)
    }

    /// Moves `by` positions right with no bounds check, returning the new
    /// position. Skips the clamp [`Self::move_right_by`] does.
    ///
    /// # Safety
    ///
    /// The caller must guarantee `self.get() + by` stays inside the sequence.
    /// `len` is not a parameter, so the addition overshoots unchecked and every
    /// later [`Self::get`] is then an out-of-bounds index.
    #[inline(always)]
    pub const unsafe fn move_right_unchecked_by(&mut self, by: usize) -> usize {
        self.0 += by;
        self.0
    }

    /// Moves up to `by` positions right, clamping at `len`.
    ///
    /// Returns whether the cursor moved: `true` even when `by` overshoots and
    /// the caret stops at `len`, `false` when `by` is `0` or the cursor already
    /// sits there.
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

    /// Moves the cursor to the beginning (position 0).
    #[inline(always)]
    pub const fn move_to_start(&mut self) -> usize {
        self.0 = 0;
        self.0
    }

    /// Moves the cursor to the end of a sequence (`len`).
    ///
    /// `len` is taken on trust, so the result is only in bounds if it is the
    /// length of the sequence this cursor indexes.
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
        Self::start()
    }
}
