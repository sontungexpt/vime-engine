//! The vec-like interface shared by [`ArrayVec`](super::array_vec::ArrayVec)
//! and [`SmallVec`](super::small_vec::SmallVec).

use std::ops::{Deref, DerefMut, Index, IndexMut};

/// A contiguous, indexable, growable sequence of `Copy` elements that behaves
/// like a `Vec`.
///
/// This is the one interface both [`ArrayVec`](super::array_vec::ArrayVec) —
/// which panics when it runs out of room — and
/// [`SmallVec`](super::small_vec::SmallVec) — which spills to the heap — are
/// written against, so code that only needs "a bounded sequence of `T`" does
/// not have to care which one it was handed.
///
/// # Supertraits
///
/// `DerefMut<Target = [T]>` is what makes a `V: VecLike<T>` usable as a slice
/// without ceremony: `v.iter()`, `for x in &*v` and everything else a `[T]`
/// offers all work through the supertrait. `Index`/`IndexMut` pin the element
/// type so `v[i]` yields `T` rather than leaving it inferred.
///
/// `Deref` is not listed separately because `DerefMut: Deref` — requiring
/// `DerefMut<Target = [T]>` already implies `Deref<Target = [T]>`, since
/// `DerefMut` cannot narrow the target its own supertrait hands out.
///
/// The index bounds are `usize` deliberately — they exist to make the common
/// single-element read and write work. A *range* is not covered, so in generic
/// code `v[a..b]` is written `(*v)[a..b]`, going through the deref to the slice.
/// That is the same reach the supertrait already implies; the alternative would
/// be a blanket `Index<I>` bound, which cannot be expressed without making the
/// trait non-object-safe over the element type.
///
/// # What callers may rely on
///
/// - The live elements are always contiguous and reachable as a slice, so
///   `as_slice` is a pointer and a length rather than a walk.
/// - `clear` keeps the capacity, so it is never an allocation.
/// - `push` appends. Whether it can fail is up to the implementor, and is the
///   one thing the trait deliberately does not smooth over:
///   [`ArrayVec`](super::array_vec::ArrayVec) panics at its fixed capacity,
///   [`SmallVec`](super::small_vec::SmallVec) grows instead. Call code that must
///   not fail either pre-checks [`Self::capacity`] or uses `SmallVec`.
///
/// # Why `T` is a parameter rather than an associated type
///
/// Both implementors are already parameterised by their element type, and the
/// `T: Copy` bound is what makes the raw reads sound. A generic parameter keeps
/// the signatures monomorphic and keeps `T` spelled the same way everywhere the
/// containers already spell it, instead of forcing `V::Item` through call sites
/// that already have a `T` in hand.
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

    /// How many elements can be held before growing is required.
    ///
    /// For [`ArrayVec`](super::array_vec::ArrayVec) this is its fixed `N`. For
    /// [`SmallVec`](super::small_vec::SmallVec) it is the larger of the inline
    /// capacity and whatever the heap has reserved, so a `capacity()` check is a
    /// real "will this allocate?" test.
    fn capacity(&self) -> usize;

    /// Appends `value`.
    ///
    /// # Panics
    ///
    /// Implementations that cannot grow may panic instead.
    fn push(&mut self, value: T);

    /// Removes and returns the last element, or `None` when empty.
    fn pop(&mut self) -> Option<T>;

    /// Inserts `value` at `index`, shifting the tail right.
    ///
    /// # Panics
    ///
    /// If `index > len`, or if the implementor cannot grow.
    fn insert(&mut self, index: usize, value: T);

    /// Removes and returns the element at `index`, shifting the tail left.
    ///
    /// # Panics
    ///
    /// If `index >= len`.
    fn remove(&mut self, index: usize) -> T;

    /// Drops every element, keeping the capacity.
    fn clear(&mut self);

    /// Appends every element of `values`.
    ///
    /// # Panics
    ///
    /// Implementations that cannot grow may panic instead.
    fn extend_from_slice(&mut self, values: &[T]);
}
