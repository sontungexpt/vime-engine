use crate::{composition::SyllableBuilder, encode_vowel, Tone};

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
    previous: SyllableBuilder,
    recoverable: bool,

    chars: Vec<CharStatus>,
}

impl DeadSyllableBuilder {
    #[inline]
    pub fn from_rejected(valid: SyllableBuilder, rejected: char) -> Self {
        let mut chars = Vec::with_capacity(valid.len() + 1);

        chars.extend(valid.onset().iter().copied().map(CharStatus::Accepted));

        for cased_vowel in valid.nucleus() {
            chars.push(CharStatus::Accepted(encode_vowel(
                cased_vowel.value,
                Tone::Flat,
                cased_vowel.uppercase,
            )));
        }

        chars.extend(valid.coda().iter().copied().map(CharStatus::Accepted));

        chars.push(CharStatus::Rejected(rejected));

        Self {
            previous: valid,
            recoverable: true,
            chars,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    #[inline(always)]
    pub fn chars(&self) -> &[CharStatus] {
        &self.chars
    }

    #[inline]
    pub fn reset(&mut self) {
        self.chars.clear();
    }

    pub fn push(&mut self, input: char) {
        self.chars.push(CharStatus::Rejected(input));
    }
}
