/// Whether a recorded character belongs to the accepted syllable or to the
/// rejected input that ended the parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenState {
    /// The character was part of the last valid syllable.
    Accepted(char),
    /// The character could not be parsed and killed the composition.
    Rejected(char),
}

impl TokenState {
    /// The character carried by this status.
    #[inline(always)]
    pub const fn char(self) -> char {
        match self {
            TokenState::Accepted(ch) | TokenState::Rejected(ch) => ch,
        }
    }

    /// Returns `true` if the status is [`CharStatus::Rejected`].
    #[inline(always)]
    pub const fn is_rejected(self) -> bool {
        matches!(self, TokenState::Rejected(_))
    }
}

/// A syllable buffer in the dead state.
///
/// Once the parse fails, the active [`SyllableBuilder`](crate::composition::SyllableBuilder)
/// is replaced by this buffer: no further parsing happens and every following
/// character is recorded verbatim, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadSyllableBuilder {
    chars: Vec<TokenState>,
    rejected_count: usize,
}

impl DeadSyllableBuilder {
    pub fn from_accepted(valid_chars: impl IntoIterator<Item = char>) -> Self {
        let iter = valid_chars.into_iter();
        let (lower, _) = iter.size_hint();

        let mut chars = Vec::with_capacity(lower + 1);
        chars.extend(iter.map(TokenState::Accepted));

        Self {
            chars,
            rejected_count: 0,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    #[inline(always)]
    pub fn rejected_count(&self) -> usize {
        self.rejected_count
    }

    #[inline(always)]
    pub fn chars(&self) -> &[TokenState] {
        &self.chars
    }

    #[inline(always)]
    pub fn to_chars(&self) -> Vec<char> {
        self.chars.iter().copied().map(TokenState::char).collect()
    }

    #[inline]
    pub fn reset(&mut self) {
        self.chars.clear();
        self.rejected_count = 0;
    }

    #[inline]
    pub fn push(&mut self, input: char) {
        self.chars.push(TokenState::Rejected(input));
        self.rejected_count += 1;
    }

    #[inline]
    pub fn insert(&mut self, index: usize, input: char) {
        debug_assert!(index <= self.chars.len());
        self.chars.insert(index, TokenState::Rejected(input));
        self.rejected_count += 1;
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> TokenState {
        debug_assert!(index < self.chars.len());
        let status = self.chars.remove(index);
        if status.is_rejected() {
            self.rejected_count -= 1;
        }
        status
    }
}
