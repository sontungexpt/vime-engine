use crate::phonology::{BaseVowel, Coda, NucleusView, Onset, Vowel, NUCLEUS_MAX_LEN};
use crate::util::InlineVec;

pub type Nucleus = InlineVec<Vowel, NUCLEUS_MAX_LEN>;

impl NucleusView for Nucleus {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    unsafe fn at(&self, index: usize) -> BaseVowel {
        self.get_unchecked(index).base()
    }
}

pub type OnsetChars = InlineVec<char, { Onset::MAX_LEN }>;
pub type CodaChars = InlineVec<char, { Coda::MAX_LEN }>;
