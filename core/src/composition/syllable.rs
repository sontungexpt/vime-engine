use crate::phonology::{BaseVowel, Case, Coda, Onset, Tone};

/// A value paired with the letter case used to render it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cased<T> {
    pub value: T,
    pub case: Case,
}

/// A single Vietnamese syllable under construction.
///
/// The raw onset/coda characters are kept alongside their resolved kinds so
/// the renderer can replay the exact typed letters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Syllable {
    pub onset: Option<Onset>,
    pub onset_chars: Vec<char>,

    pub vowels: Vec<Cased<BaseVowel>>,

    pub coda: Option<Coda>,
    pub coda_chars: Vec<char>,

    pub tone: Tone,
}

impl Syllable {
    // Returns the total number of characters in the syllable.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.onset_chars.len() + self.vowels.len() + self.coda_chars.len()
    }
}

impl Default for Syllable {
    #[inline]
    fn default() -> Self {
        Self {
            onset: None,
            onset_chars: Vec::with_capacity(3),

            vowels: Vec::with_capacity(3),

            coda: None,
            coda_chars: Vec::with_capacity(2),

            tone: Tone::Flat,
        }
    }
}
