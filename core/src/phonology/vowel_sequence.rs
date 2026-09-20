use arrayvec::ArrayVec;

use super::{BaseVowel, CasedBaseVowel};

/// A read-only view over a vowel nucleus.
pub trait VowelSequence {
    fn len(&self) -> usize;
    fn at(&self, index: usize) -> BaseVowel;
}

impl VowelSequence for [BaseVowel] {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn at(&self, index: usize) -> BaseVowel {
        self[index]
    }
}

impl<const N: usize> VowelSequence for ArrayVec<CasedBaseVowel, N> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn at(&self, index: usize) -> BaseVowel {
        self[index].value
    }
}
