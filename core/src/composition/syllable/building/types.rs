use crate::phonology::{BaseVowel, BaseVowelSlice, Coda, Onset, Vowel, NUCLEUS_MAX_LEN};
use crate::util::InlineVec;

pub type Nucleus = InlineVec<Vowel, NUCLEUS_MAX_LEN>;

impl BaseVowelSlice for Nucleus {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    unsafe fn at_unchecked(&self, index: usize) -> BaseVowel {
        // SAFETY: the caller's contract is `index < self.len()`; the nucleus
        // is only indexed by indices the tone-placement dispatch admitted.
        unsafe { self.get_unchecked(index).base() }
    }
}

pub type OnsetChars = InlineVec<char, { Onset::MAX_LEN }>;
pub type CodaChars = InlineVec<char, { Coda::MAX_LEN }>;
