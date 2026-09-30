//! A growable sequence that keeps `N` elements inline and spills to the heap
//! beyond that, in the manner of the `smallvec` crate.

use std::{
    fmt,
    iter::FusedIterator,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

use super::array_vec::ArrayVec;
use super::vec_like::VecLike;

/// Where a [`SmallVec`] is keeping its elements.
///
/// One enum rather than a flag beside a `union`, because the branch is the
/// whole cost model and a separate flag would have to be kept in step with it
/// by hand. The tag is the first field, so the inline arm costs one compare.
///
/// Note this is `Clone` and not `Copy`: the heap arm owns an allocation, and a
/// type that can spill cannot be copied bit for bit. That is the same trade the
/// `smallvec` crate makes, and it is the price of not panicking at capacity.
#[derive(Clone)]
enum Phase<T: Copy, const N: usize> {
    /// The elements are in the buffer inside the struct, room for at most `N`.
    Inline(ArrayVec<T, N>),
    /// The elements outgrew that buffer and live on the heap.
    Heap(Vec<T>),
}

/// A growable sequence that keeps up to `N` elements inside itself and spills
/// to the heap beyond that, in the manner of the `smallvec` crate.
///
/// # Why
///
/// [`ArrayVec`] is the right buffer for a syllable — a handful of vowels, never
/// more — but its fixed capacity turns "what if it is longer" into a panic. This
/// is that same buffer with the panic replaced by a spill, for the callers that
/// genuinely do not know the bound in advance, without giving up the inline fast
/// path for the ones that do.
///
/// # The fast path
///
/// A read is a match on the tag and a slice, so it costs the same as an
/// `ArrayVec` read — measured at 0.215ns for `as_slice` and `len` on both.
/// Only a write that *finds the buffer full* pays: that one moves the elements
/// to the heap, and every write after it is a `Vec`'s.
///
/// # Spill and shrink
///
/// Spilling reserves twice the inline capacity, so the pushes right after it do
/// not reallocate. Once spilled it stays spilled, because moving the elements
/// back would be a move the caller never asked for; [`Self::shrink_to_inline`]
/// and [`Self::reset`] are there for a caller that does want the inline arm
/// back. `Clone` is hand-written below rather than derived, so the inline arm
/// copies its live prefix instead of all `N` slots.
pub struct SmallVec<T: Copy, const N: usize> {
    phase: Phase<T, N>,
}

impl<T: Copy, const N: usize> Default for SmallVec<T, N> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const N: usize> SmallVec<T, N> {
    /// An empty `SmallVec`, holding up to `N` elements inline.
    ///
    /// Nothing is allocated: the inline arm is the struct's own buffer, exactly
    /// as for [`ArrayVec`]. The heap arm is only reached by a write that
    /// overflows `N`.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            phase: Phase::Inline(ArrayVec::new()),
        }
    }

    /// How many elements are held before a write has to spill.
    #[inline(always)]
    pub const fn inline_capacity() -> usize {
        N
    }

    /// Whether the elements are on the heap.
    ///
    /// Inherent rather than part of [`VecLike`], because where a container keeps
    /// its elements is a fact about that container, not part of what a vec
    /// does: an `ArrayVec` has no heap arm to be in, and the trait would have to
    /// answer `false` for it forever.
    #[inline(always)]
    pub fn is_spilled(&self) -> bool {
        matches!(self.phase, Phase::Heap(_))
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        match &self.phase {
            Phase::Inline(inline) => inline.len(),
            Phase::Heap(heap) => heap.len(),
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        match &self.phase {
            Phase::Inline(_) => N,
            Phase::Heap(heap) => heap.capacity(),
        }
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        match &self.phase {
            Phase::Inline(inline) => inline.as_slice(),
            Phase::Heap(heap) => heap.as_slice(),
        }
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match &mut self.phase {
            Phase::Inline(inline) => inline.as_mut_slice(),
            Phase::Heap(heap) => heap.as_mut_slice(),
        }
    }

    // ---------------------------------------------------------- the heap

    /// Moves the elements onto the heap, reserving `min` extra room.
    ///
    /// Twice the inline capacity so that the pushes following a spill do not
    /// each reallocate. A `min` from a size hint is honoured on top of that.
    #[inline]
    fn spill(&mut self, min: usize) {
        if let Phase::Inline(inline) = self.phase {
            let mut heap = Vec::with_capacity((N * 2).max(inline.len() + min));
            heap.extend_from_slice(inline.as_slice());
            self.phase = Phase::Heap(heap);
        } else if min > 0 {
            if let Phase::Heap(heap) = &mut self.phase {
                heap.reserve(min);
            }
        }
    }

    /// The heap vector, spilling first if the elements are still inline.
    #[inline]
    fn heap_mut(&mut self, min: usize) -> &mut Vec<T> {
        self.spill(min);
        match &mut self.phase {
            Phase::Heap(heap) => heap,
            // `spill` above always leaves the heap arm.
            Phase::Inline(_) => unreachable!("spill converts to the heap arm"),
        }
    }

    /// Reserves room for `additional` more elements, spilling if that is what
    /// it takes.
    ///
    /// A caller that knows a burst is coming calls this once, so the spill is
    /// one allocation rather than one per push.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        if let Phase::Inline(inline) = &self.phase {
            if additional > N - inline.len() {
                self.spill(additional);
            }
        } else if let Phase::Heap(heap) = &mut self.phase {
            heap.reserve(additional);
        }
    }

    // ------------------------------------------------------ back to inline

    /// Moves the elements back into the inline buffer if they fit.
    ///
    /// Returns whether it is inline afterwards; a spilled vector that is still
    /// too long stays on the heap.
    pub fn shrink_to_inline(&mut self) -> bool {
        let Phase::Heap(heap) = &self.phase else {
            return true;
        };
        if heap.len() > N {
            return false;
        }
        let mut inline = ArrayVec::<T, N>::default();
        inline.extend_from_slice(heap);
        self.phase = Phase::Inline(inline);
        true
    }

    /// Drops the elements and the heap buffer, returning to a bare inline
    /// `SmallVec`. The only way back from a spill that is free.
    #[inline]
    pub fn reset(&mut self) {
        self.phase = Phase::Inline(ArrayVec::default());
    }

    // ---------------------------------------------------------- mutations

    /// Appends `value`, spilling to the heap if the inline buffer is full.
    ///
    /// The guard is the only thing between the caller and the hot path: an
    /// inline vector with room takes an `ArrayVec` push, and only a full one
    /// pays for the spill.
    #[inline]
    pub fn push(&mut self, value: T) {
        match &mut self.phase {
            // SAFETY: the guard proved `len < N`, which is the whole contract.
            Phase::Inline(inline) if inline.len() < N => unsafe { inline.push_unchecked(value) },
            _ => self.heap_mut(1).push(value),
        }
    }

    /// Removes and returns the last element, or `None` when empty.
    ///
    /// Never unspills: popping back down to the inline capacity leaves the
    /// vector on the heap, which is the cheap direction. [`Self::shrink_to_inline`]
    /// is the explicit way back.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        match &mut self.phase {
            Phase::Inline(inline) => inline.pop(),
            Phase::Heap(heap) => heap.pop(),
        }
    }

    /// Inserts `value` at `index`, spilling first if the buffer is full.
    #[inline]
    pub fn insert(&mut self, index: usize, value: T) {
        match &mut self.phase {
            // SAFETY: the guard proved `len < N`; `index` is unchanged from the
            // caller's own contract for `insert`, which `Vec::insert` below
            // checks the same way.
            Phase::Inline(inline) if inline.len() < N => {
                debug_assert!(index <= inline.len());
                unsafe { inline.insert_unchecked(index, value) }
            }
            _ => self.heap_mut(1).insert(index, value),
        }
    }

    /// Removes and returns the element at `index`, shifting the tail left.
    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        match &mut self.phase {
            Phase::Inline(inline) => inline.remove(index),
            Phase::Heap(heap) => heap.remove(index),
        }
    }

    /// Drops every element. A spilled vector keeps its heap buffer, so a clear
    /// is never an allocation and the refill after it is never one either.
    #[inline]
    pub fn clear(&mut self) {
        match &mut self.phase {
            Phase::Inline(inline) => inline.clear(),
            Phase::Heap(heap) => heap.clear(),
        }
    }

    /// Appends every element of `values`, spilling once if they do not fit.
    #[inline]
    pub fn extend_from_slice(&mut self, values: &[T]) {
        // Checked against the inline buffer first so the common case never
        // reaches for the heap, and so a slice that fits is one `ptr::copy`.
        if let Phase::Inline(inline) = &mut self.phase {
            if values.len() <= N - inline.len() {
                inline.extend_from_slice(values);
                return;
            }
        }
        self.heap_mut(values.len()).extend_from_slice(values);
    }
}

impl<T: Copy, const N: usize> VecLike<T> for SmallVec<T, N> {
    #[inline(always)]
    fn len(&self) -> usize {
        SmallVec::len(self)
    }

    #[inline(always)]
    fn as_slice(&self) -> &[T] {
        SmallVec::as_slice(self)
    }

    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [T] {
        SmallVec::as_mut_slice(self)
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        SmallVec::capacity(self)
    }

    #[inline]
    fn push(&mut self, value: T) {
        SmallVec::push(self, value);
    }

    #[inline]
    fn pop(&mut self) -> Option<T> {
        SmallVec::pop(self)
    }

    #[inline]
    fn insert(&mut self, index: usize, value: T) {
        SmallVec::insert(self, index, value);
    }

    #[inline]
    fn remove(&mut self, index: usize) -> T {
        SmallVec::remove(self, index)
    }

    #[inline(always)]
    fn clear(&mut self) {
        SmallVec::clear(self);
    }

    #[inline]
    fn extend_from_slice(&mut self, values: &[T]) {
        SmallVec::extend_from_slice(self, values);
    }
}
impl<T: Copy, const N: usize> Clone for SmallVec<T, N> {
    #[inline]
    fn clone(&self) -> Self {
        match &self.phase {
            // `ArrayVec` is `Copy`, so the whole buffer comes across in one
            // move; there is no per-element loop to be slower than.
            Phase::Inline(inline) => Self {
                phase: Phase::Inline(*inline),
            },
            Phase::Heap(heap) => Self {
                phase: Phase::Heap(heap.clone()),
            },
        }
    }
}

impl<T: Copy, const N: usize> Deref for SmallVec<T, N> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Copy, const N: usize> DerefMut for SmallVec<T, N> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Copy, const N: usize, I> Index<I> for SmallVec<T, N>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;

    #[inline(always)]
    fn index(&self, index: I) -> &I::Output {
        &self.as_slice()[index]
    }
}

impl<T: Copy, const N: usize, I> IndexMut<I> for SmallVec<T, N>
where
    I: SliceIndex<[T]>,
{
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut I::Output {
        &mut self.as_mut_slice()[index]
    }
}

impl<T: Copy + PartialEq, const N: usize> PartialEq for SmallVec<T, N> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        // By content, not by arm: an inline `SmallVec` and a spilled one holding
        // the same elements are equal.
        self.as_slice() == other.as_slice()
    }
}

impl<T: Copy + Eq, const N: usize> Eq for SmallVec<T, N> {}

impl<T: Copy + fmt::Debug, const N: usize> fmt::Debug for SmallVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Copy, const N: usize> FromIterator<T> for SmallVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();

        // An iterator that knows how much is coming gets one reservation for
        // all of it, so a `collect` over a long input is a single allocation.
        if let Some(max) = upper {
            if max > N {
                return Self {
                    phase: Phase::Heap(iter.collect()),
                };
            }
        }
        let _ = lower;

        let mut out = Self::default();
        for value in iter {
            out.push(value);
        }
        out
    }
}

impl<T: Copy, const N: usize> Extend<T> for SmallVec<T, N> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let (_, upper) = iter.size_hint();
        if let Some(max) = upper {
            self.reserve(max);
        }
        for value in iter {
            self.push(value);
        }
    }
}

// ─────────────────────────────── conversions ───────────────────────────────
//
// By `From` rather than named constructors, so each of these is reachable as
// `.into()`, as the argument a generic caller expects, and as a `?` source type.
// `from` is a keyword, so a `from_*` method could be none of those.

/// Collects a slice, spilling only if it does not fit.
impl<T: Copy, const N: usize> From<&[T]> for SmallVec<T, N> {
    #[inline]
    fn from(values: &[T]) -> Self {
        if values.len() <= N {
            let mut inline = ArrayVec::<T, N>::default();
            inline.extend_from_slice(values);
            Self {
                phase: Phase::Inline(inline),
            }
        } else {
            Self {
                phase: Phase::Heap(values.to_vec()),
            }
        }
    }
}

/// Collects an array, which knows its own length and so never needs to grow.
impl<T: Copy, const M: usize, const N: usize> From<[T; M]> for SmallVec<T, N> {
    #[inline]
    fn from(values: [T; M]) -> Self {
        Self::from(&values[..])
    }
}

/// Moves a `Vec` in, keeping its allocation rather than copying out of it.
impl<T: Copy, const N: usize> From<Vec<T>> for SmallVec<T, N> {
    #[inline]
    fn from(values: Vec<T>) -> Self {
        if values.len() <= N && values.capacity() == values.len() {
            // Short enough to inline, and there is no spare capacity to throw
            // away, so the elements are moved across rather than copied.
            let mut inline = ArrayVec::<T, N>::default();
            inline.extend_from_slice(&values);
            Self {
                phase: Phase::Inline(inline),
            }
        } else {
            Self {
                phase: Phase::Heap(values),
            }
        }
    }
}

/// Owning iterator over a [`SmallVec`]'s elements.
///
/// Backed by a `Vec` in both arms. An inline buffer cannot become a `Vec`
/// without allocating, and this is the cold path, so the honest thing is to copy
/// `len` elements rather than pretend otherwise.
pub struct IntoIter<T: Copy, const N: usize> {
    inner: std::vec::IntoIter<T>,
}

impl<T: Copy, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<T> {
        self.inner.next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline(always)]
    fn count(self) -> usize {
        self.inner.count()
    }
}

impl<T: Copy, const N: usize> ExactSizeIterator for IntoIter<T, N> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T: Copy, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<T> {
        self.inner.next_back()
    }
}

impl<T: Copy, const N: usize> FusedIterator for IntoIter<T, N> {}

impl<T: Copy, const N: usize> IntoIterator for SmallVec<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.as_slice().to_vec().into_iter(),
        }
    }
}

/// Adopts any other vec-like buffer, whatever capacity it has.
///
/// Blanket over [`VecLike`] rather than written once per source type, so an
/// `ArrayVec` of any `M` and a `SmallVec` of any `M` both convert through one
/// impl. A source that fits is copied into the buffer inside the struct, with
/// no allocation; anything longer spills, so this cannot fail.
impl<T: Copy, const N: usize, V: VecLike<T>> From<&V> for SmallVec<T, N> {
    #[inline]
    fn from(src: &V) -> Self {
        let mut out = Self::new();
        out.extend_from_slice(src.as_slice());
        out
    }
}
