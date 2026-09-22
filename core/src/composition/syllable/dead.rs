/// Whether a recorded character belongs to the accepted syllable or to the
/// rejected input that ended the parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharStatus {
    /// The character was part of the last valid syllable.
    Accepted(char),
    /// The character could not be parsed and killed the composition.
    Rejected(char),
}

impl CharStatus {
    /// The character carried by this status.
    #[inline(always)]
    pub const fn char(self) -> char {
        match self {
            CharStatus::Accepted(ch) | CharStatus::Rejected(ch) => ch,
        }
    }
}

/// A syllable buffer in the dead state.
///
/// Once the parse fails, the active [`SyllableBuilder`](crate::composition::SyllableBuilder)
/// is replaced by this buffer: no further parsing happens and every following
/// character is recorded verbatim, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadSyllableBuilder {
    chars: Vec<CharStatus>,
}

impl DeadSyllableBuilder {
    pub fn from_accepted(valid_chars: impl IntoIterator<Item = char>) -> Self {
        let iter = valid_chars.into_iter();
        let (lower, _) = iter.size_hint();

        let mut chars = Vec::with_capacity(lower + 1);
        chars.extend(iter.map(CharStatus::Accepted));

        Self { chars }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    #[inline(always)]
    pub fn chars(&self) -> &[CharStatus] {
        &self.chars
    }

    #[inline(always)]
    pub fn to_chars(&self) -> Vec<char> {
        self.chars.iter().copied().map(CharStatus::char).collect()
    }

    #[inline]
    pub fn reset(&mut self) {
        self.chars.clear();
    }

    #[inline]
    pub fn push(&mut self, input: char) {
        self.chars.push(CharStatus::Rejected(input));
    }

    #[inline]
    pub fn insert(&mut self, index: usize, input: char) {
        debug_assert!(index <= self.chars.len());
        self.chars.insert(index, CharStatus::Rejected(input));
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> CharStatus {
        debug_assert!(index < self.chars.len());
        self.chars.remove(index)
    }
}
