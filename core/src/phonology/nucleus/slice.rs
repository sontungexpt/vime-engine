//! A read-only view over a vowel nucleus.
//!
//! The nucleus and tone-placement rules read through this trait, so it does not
//! matter whether the vowels are [`BaseVowel`]s, [`Vowel`]s, or the inline buffer.

use super::super::{BaseVowel, Vowel};

/// A sequence of base vowels that can be inspected positionally.
pub trait BaseVowelSlice {
    /// Number of vowels in the nucleus.
    fn len(&self) -> usize;

    /// Returns the base vowel at `index`.
    ///
    /// # Safety
    ///
    /// `index` must be less than `self.len()`; implementations may skip bounds
    /// checks on that invariant.
    unsafe fn at_unchecked(&self, index: usize) -> BaseVowel;
}

impl BaseVowelSlice for [BaseVowel] {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    unsafe fn at_unchecked(&self, index: usize) -> BaseVowel {
        // SAFETY: the caller's contract is `index < self.len()`.
        unsafe { *self.get_unchecked(index) }
    }
}

impl BaseVowelSlice for [Vowel] {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    unsafe fn at_unchecked(&self, index: usize) -> BaseVowel {
        // SAFETY: the caller's contract is `index < self.len()`.
        unsafe { self.get_unchecked(index).base() }
    }
}

impl<const N: usize> BaseVowelSlice for [BaseVowel; N] {
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }

    #[inline(always)]
    unsafe fn at_unchecked(&self, index: usize) -> BaseVowel {
        // SAFETY: the caller's contract is `index < self.len()`.
        unsafe { *self.get_unchecked(index) }
    }
}

impl<const N: usize> BaseVowelSlice for [Vowel; N] {
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }

    #[inline(always)]
    unsafe fn at_unchecked(&self, index: usize) -> BaseVowel {
        // SAFETY: the caller's contract is `index < self.len()`.
        unsafe { self.get_unchecked(index).base() }
    }
}
