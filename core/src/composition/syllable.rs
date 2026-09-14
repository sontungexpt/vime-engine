use std::ops::{Index, IndexMut};

use crate::phonology::{BaseVowel, Case, Coda, Onset, Tone};

/// A value paired with the letter case used to render it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cased<T> {
    pub value: T,
    pub case: Case,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PhonemePart<T> {
    kind: T,
    chars: Vec<char>,
}

impl<T> Index<usize> for PhonemePart<T> {
    type Output = char;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.chars[index]
    }
}

impl<T> IndexMut<usize> for PhonemePart<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.chars[index]
    }
}

impl<T: Copy> PhonemePart<T> {
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.chars.len()
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.chars.clear();
    }

    #[inline(always)]
    pub fn chars(&self) -> &[char] {
        &self.chars
    }

    #[inline(always)]
    pub fn chars_mut(&mut self) -> &mut [char] {
        &mut self.chars
    }

    #[inline(always)]
    pub fn kind(&self) -> T {
        self.kind
    }

    #[inline]
    pub fn push_as(&mut self, ch: char, kind: T) {
        self.chars.push(ch);
        self.kind = kind;
    }

    #[inline(always)]
    pub fn replace_at(&mut self, index: usize, new_char: char, new_kind: T) {
        debug_assert!(index < self.chars.len());

        self.chars[index] = new_char;
        self.kind = new_kind;
    }
}

impl PhonemePart<Onset> {
    #[inline]
    pub fn push(&mut self, ch: char) -> bool {
        self.chars.push(ch);

        match Onset::from_chars(&self.chars) {
            Ok(kind) => {
                self.kind = kind;
                true
            }
            Err(_) => {
                self.chars.pop();
                false
            }
        }
    }
}

impl PhonemePart<Coda> {
    pub fn push(&mut self, ch: char) -> bool {
        self.chars.push(ch);

        match Coda::from_chars(&self.chars) {
            Ok(kind) => {
                self.kind = kind;
                true
            }
            Err(_) => {
                self.chars.pop();
                false
            }
        }
    }
}

/// A single Vietnamese syllable under construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Syllable {
    pub onset: PhonemePart<Onset>,
    pub vowels: Vec<Cased<BaseVowel>>,
    pub coda: PhonemePart<Coda>,
    pub tone: Tone,
}

impl Syllable {
    #[inline]
    pub fn len(&self) -> usize {
        self.onset.len() + self.vowels.len() + self.coda.len()
    }

    #[inline]
    pub fn len_parts(&self) -> (usize, usize, usize, usize) {
        let onset_len = self.onset.len();
        let vowel_len = self.vowels.len();
        let coda_len = self.coda.len();
        let total_len = onset_len + vowel_len + coda_len;

        (onset_len, vowel_len, coda_len, total_len)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.onset.is_empty() && self.vowels.is_empty() && self.coda.is_empty()
    }
}

impl Default for Syllable {
    #[inline]
    fn default() -> Self {
        Self {
            onset: PhonemePart {
                kind: Onset::None,
                chars: Vec::with_capacity(3),
            },
            vowels: Vec::with_capacity(3),
            coda: PhonemePart {
                kind: Coda::None,
                chars: Vec::with_capacity(2),
            },
            tone: Tone::Flat,
        }
    }
}
