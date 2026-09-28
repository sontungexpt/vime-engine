//! A read-only view over a vowel nucleus.
//!
//! Every nucleus consumer in the crate — the rule table in
//! [`super::state::NucleusState`] and the tone-placement rules in
//! [`super::tone_placement::TonePlacement`] — reads a nucleus through this
//! one trait, so none of them care whether the vowels are held as
//! [`BaseVowel`]s, as [`Vowel`]s (which pack the base vowel together with tone
//! and case), or in the syllable builder's inline `Nucleus` buffer.

use crate::phonology::{nucleus::state::check_nucleus_state, BaseVowel, NucleusState, Vowel};

/// A sequence of base vowels that can be inspected positionally.
pub trait NucleusView {
    /// Number of vowels in the nucleus.
    fn len(&self) -> usize;

    /// Returns the base vowel at `index`.
    ///
    /// # Safety
    ///
    /// `index` must be less than `self.len()`.
    ///
    /// Callers must ensure that the index is in bounds before calling this
    /// method. Implementations may omit bounds checking based on this
    /// invariant.
    unsafe fn at(&self, index: usize) -> BaseVowel;

    /// Returns the current validation state of the nucleus.
    ///
    /// Implementations with a cached state can override this method to return
    /// the cached value directly without recomputing the nucleus state.
    #[inline(always)]
    fn state(&self) -> NucleusState {
        check_nucleus_state(self)
    }
}

impl NucleusView for [BaseVowel] {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    unsafe fn at(&self, index: usize) -> BaseVowel {
        *self.get_unchecked(index)
    }
}

impl NucleusView for [Vowel] {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    unsafe fn at(&self, index: usize) -> BaseVowel {
        self.get_unchecked(index).base()
    }
}

impl<const N: usize> NucleusView for [BaseVowel; N] {
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }

    #[inline(always)]
    unsafe fn at(&self, index: usize) -> BaseVowel {
        *self.get_unchecked(index)
    }
}

impl<const N: usize> NucleusView for [Vowel; N] {
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }

    #[inline(always)]
    unsafe fn at(&self, index: usize) -> BaseVowel {
        self.get_unchecked(index).base()
    }
}
