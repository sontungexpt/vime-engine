use super::SyllableChars;
use crate::util::vec::SmallVec;

/// Which part of the dead buffer a recorded character belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharState {
    /// The character was part of the last valid parse.
    Accepted(char),
    /// The character that failed to parse, or one typed after the parse had
    /// already died; either way it is not part of the accepted syllable.
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

/// A syllable buffer that no longer parses: the accepted prefix of the failed
/// parse plus every character typed since, recorded verbatim until a removal
/// revives parsing (see [`Self::is_all_accepted`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadSyllable {
    chars: SmallVec<CharState, INLINE>,
    rejected_count: usize,
}

/// How many characters the dead buffer holds inline before spilling to the
/// heap; longer input still works, it just allocates.
const INLINE: usize = 12;

impl DeadSyllable {
    /// Builds a buffer holding `valid_chars` as `Accepted`, the prefix of the
    /// parse that just failed; the rejecting character follows via `push` or
    /// `insert`. Takes a slice so callers need no iterator.
    #[inline]
    pub fn from_accepted(valid_chars: &[char]) -> Self {
        // Collecting uses an exact size hint, so an over-long prefix spills to
        // the heap once instead of per character.
        let chars: SmallVec<CharState, INLINE> = valid_chars
            .iter()
            .copied()
            .map(CharState::Accepted)
            .collect();

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

    /// Returns `true` when every buffered character is `Accepted` — the state
    /// before the rejecting character is appended, where re-parsing the buffer
    /// reproduces it, which is how a removal revives parsing.
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

    /// Yields all buffered characters, accepted and rejected, with no intermediate buffer.
    #[inline(always)]
    pub fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars.iter().map(|status| status.char())
    }

    /// Collects all buffered characters into a [`SyllableChars`];
    /// see [`Self::write_to`] to write them with no intermediate buffer.
    #[inline(always)]
    pub fn to_chars(&self) -> SyllableChars {
        self.chars.iter().copied().map(CharState::char).collect()
    }

    /// Appends all buffered characters to `output` in input order, with no
    /// intermediate buffer; prefer [`Self::to_chars`] when the characters are
    /// wanted as a value to keep.
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

    /// Appends `input` as a `Rejected` character. Nothing is parsed, so this
    /// is only correct on a buffer that is already dead.
    #[inline(always)]
    pub fn push(&mut self, input: char) {
        self.chars.push(CharState::Rejected(input));
        self.rejected_count += 1;
    }

    /// Inserts `input` at `index` as a `Rejected` character, recording
    /// without parsing. `index` may be `0..=Self::len()`; past that panics.
    #[inline(always)]
    pub fn insert(&mut self, index: usize, input: char) {
        self.chars.insert(index, CharState::Rejected(input));
        self.rejected_count += 1;
    }

    /// Removes the character at `index` and returns its state, decrementing
    /// the rejected count only if it was rejected. `index` must be
    /// `0..Self::len()`; the one operation that can restore [`Self::is_all_accepted`].
    #[inline(always)]
    pub fn remove(&mut self, index: usize) -> CharState {
        let status = self.chars.remove(index);
        if status.is_rejected() {
            self.rejected_count -= 1;
        }
        status
    }
}
