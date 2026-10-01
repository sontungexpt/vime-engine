//! The vec-like interface shared by [`ArrayVec`](super::array_vec::ArrayVec)
//! and [`SmallVec`](super::small_vec::SmallVec).

use std::ops::{Deref, DerefMut, Index, IndexMut};

/// A contiguous, indexable, growable sequence of `Copy` elements.
///
/// `ArrayVec` panics when full while `SmallVec` grows; ranges need `(*v)[a..b]`,
/// since only `usize` indexing sits on the trait itself.
pub trait VecLike<T: Copy>:
    Deref<Target = [T]> + DerefMut<Target = [T]> + Index<usize, Output = T> + IndexMut<usize>
{
    /// The number of live elements.
    fn len(&self) -> usize;

    /// Whether there are no live elements.
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The live elements, contiguously.
    fn as_slice(&self) -> &[T];

    /// The live elements, contiguously and mutably.
    fn as_mut_slice(&mut self) -> &mut [T];

    /// How many elements fit before growing; a `capacity()` check is a real
    /// "will this allocate?" test (fixed `N` for `ArrayVec`, more once spilled).
    fn capacity(&self) -> usize;

    /// Appends `value`; panics if the implementation is full and cannot grow.
    fn push(&mut self, value: T);

    /// Removes and returns the last element, or `None` when empty.
    fn pop(&mut self) -> Option<T>;

    /// Inserts `value` at `index`, shifting the tail right; panics if
    /// `index > len` or the implementation is full and cannot grow.
    fn insert(&mut self, index: usize, value: T);

    /// Removes and returns the element at `index`, shifting the tail left;
    /// panics if `index >= len`.
    fn remove(&mut self, index: usize) -> T;

    /// Drops every element, keeping the capacity.
    fn clear(&mut self);

    /// Appends every element of `values`; panics if the implementation is full
    /// and cannot grow.
    fn extend_from_slice(&mut self, values: &[T]);
}
