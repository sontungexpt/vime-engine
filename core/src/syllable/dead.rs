/// Which part of the dead buffer a recorded character belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharState {
    /// The character was part of the last valid parse.
    Accepted(char),
    /// The character is not part of the accepted syllable: either it is the one
    /// that failed to parse, or it was typed after the parse had already died
    /// and so was never parsed at all.
    Rejected(char),
}

impl CharState {
    /// The character this state carries, whichever variant it is.
    #[inline(always)]
    pub const fn char(self) -> char {
        match self {
            CharState::Accepted(ch) | CharState::Rejected(ch) => ch,
        }
    }

    /// Returns `true` if the character is `Rejected`.
    #[inline(always)]
    pub const fn is_rejected(self) -> bool {
        matches!(self, CharState::Rejected(_))
    }
}

/// A syllable buffer in the dead state.
///
/// When the building phase rejects a character, the accepted prefix it had
/// produced so far is frozen into one of these, the rejecting character is
/// appended, and
/// [`Syllable`](crate::syllable::Syllable) switches
/// state. From then on nothing is parsed: every further character is recorded
/// verbatim, in order, until a removal revives parsing (see
/// [`Self::is_all_accepted`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadSyllable {
    chars: Vec<CharState>,
    rejected_count: usize,
}

impl DeadSyllable {
    /// Builds a buffer in which every character of `valid_chars` is
    /// `Accepted`, so the rejected count starts at zero.
    ///
    /// This is the accepted prefix of the parse that just failed; the
    /// character that caused the rejection is not part of it and is expected to
    /// be [`Self::push`]ed or [`Self::insert`]ed straight after.
    #[inline]
    pub fn from_accepted(valid_chars: impl IntoIterator<Item = char>) -> Self {
        let iter = valid_chars.into_iter();
        let (lower, upper) = iter.size_hint();
        let cap = upper.unwrap_or(lower);
        // Room for the one character that will end the parse, so the caller
        // does not reallocate on the first push.
        let mut chars = Vec::with_capacity(cap + 1);
        chars.extend(iter.map(CharState::Accepted));

        Self {
            chars,
            rejected_count: 0,
        }
    }

    /// The number of buffered characters, accepted and rejected alike.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    /// Returns `true` when nothing has been buffered.
    #[allow(dead_code)]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    /// How many buffered characters are `Rejected`, i.e. how many are not part
    /// of the accepted syllable.
    #[allow(dead_code)]
    #[inline(always)]
    pub fn rejected_count(&self) -> usize {
        self.rejected_count
    }

    /// Returns `true` when every buffered character is `Accepted`.
    ///
    /// This is the state a buffer is in between the parse failing and the
    /// rejecting character being appended: the whole text came from the last
    /// valid parse. A caller can exploit that by feeding the buffer to a fresh
    /// builder and re-running the parse, which is how a removal that drops the
    /// last rejected character revives parsing. Once anything is rejected the
    /// text is verbatim, and re-parsing it would not reproduce what is
    /// buffered.
    #[inline(always)]
    pub fn is_all_accepted(&self) -> bool {
        self.rejected_count == 0
    }

    /// The buffered characters with their state, in input order.
    #[allow(dead_code)]
    #[inline(always)]
    pub fn chars(&self) -> &[CharState] {
        &self.chars
    }

    /// Yields the buffered characters, accepted and rejected alike, without
    /// allocating an intermediate buffer.
    #[inline(always)]
    pub fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars.iter().map(|status| status.char())
    }

    /// Collects the buffered characters, accepted and rejected alike, into a
    /// [`Vec`]. See also [`Self::write_to`], which writes the same characters
    /// without allocating.
    #[inline(always)]
    pub fn to_chars(&self) -> Vec<char> {
        self.chars.iter().copied().map(CharState::char).collect()
    }

    /// Appends the buffered characters to `output`, accepted and rejected
    /// alike, in input order.
    ///
    /// Needs no intermediate buffer. Prefer it when the characters are only
    /// being written somewhere; use [`Self::to_chars`] when they are wanted as
    /// a value to keep.
    #[inline(always)]
    pub fn write_to(&self, output: &mut String) {
        output.extend(self.iter_chars());
    }

    /// Empties the buffer, dropping both the characters and the rejected
    /// count.
    #[allow(dead_code)]
    #[inline]
    pub fn reset(&mut self) {
        self.chars.clear();
        self.rejected_count = 0;
    }

    /// Appends `input` as a `Rejected` character.
    ///
    /// Nothing is parsed: this records the character, so it is only correct to
    /// call this on a buffer that is already dead.
    #[inline(always)]
    pub fn push(&mut self, input: char) {
        self.chars.push(CharState::Rejected(input));
        self.rejected_count += 1;
    }

    /// Inserts `input` at `index` as a `Rejected` character.
    ///
    /// `index` may be `0..=Self::len()`; anything past the end panics. Like
    /// [`Self::push`], this records without parsing.
    #[inline(always)]
    pub fn insert(&mut self, index: usize, input: char) {
        self.chars.insert(index, CharState::Rejected(input));
        self.rejected_count += 1;
    }

    /// Removes the character at `index` and returns its state, dropping the
    /// rejected count by one only if that character was rejected.
    ///
    /// `index` must be `0..Self::len()`; anything past the end panics. This is
    /// the one operation that can take a buffer back to
    /// [`Self::is_all_accepted`], by removing the last rejected character.
    #[inline(always)]
    pub fn remove(&mut self, index: usize) -> CharState {
        let status = self.chars.remove(index);
        if status.is_rejected() {
            self.rejected_count -= 1;
        }
        status
    }
}
