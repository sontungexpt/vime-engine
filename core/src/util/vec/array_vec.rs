use super::vec_like::VecLike;

use std::{
    fmt,
    iter::FusedIterator,
    mem::MaybeUninit,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

/// An inline, fixed-capacity sequence: `N` elements stored in the struct itself,
/// no allocation, panicking at capacity.
///
/// `T: Copy` is structural: `push`/`insert`/`remove` move elements with raw
/// `ptr::copy`, and the bitwise reads of `pop`/`remove` do not invalidate the
/// slot, so a `Drop` type could be dropped twice or leaked.
#[derive(Clone, Copy)]
pub struct ArrayVec<T, const N: usize>
where
    T: Copy,
{
    buf: [MaybeUninit<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> Default for ArrayVec<T, N> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const N: usize> ArrayVec<T, N> {
    /// Creates an empty `ArrayVec`; nothing is allocated.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The number of live elements.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// The fixed maximum capacity.
    #[inline(always)]
    pub const fn capacity() -> usize {
        N
    }

    /// Returns a slice containing the live elements.
    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: only `0..len` is ever written, each slot exactly once, and
        // `len <= N`; `MaybeUninit<T>` has `T`'s layout and alignment.
        unsafe { std::slice::from_raw_parts(self.buf.as_ptr().cast::<T>(), self.len) }
    }

    /// Returns a mutable slice containing the live elements.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: as `as_slice` — every slot in `0..len` is initialized and
        // `len` never exceeds `N`.
        unsafe { std::slice::from_raw_parts_mut(self.buf.as_mut_ptr().cast::<T>(), self.len) }
    }

    /// Appends `value` at the end; panics when the array is full.
    #[inline(always)]
    pub fn push(&mut self, value: T) {
        assert!(
            self.len < N,
            "ArrayVec overflow: cannot push, capacity is {N}"
        );
        // SAFETY: `len < N` was just checked.
        unsafe { self.push_unchecked(value) };
    }

    /// Appends `value` without checking for room.
    ///
    /// # Safety
    ///
    /// `self.len()` must be less than `N`; otherwise the write lands outside
    /// the backing array and `len` claims an uninitialized slot.
    #[inline(always)]
    pub unsafe fn push_unchecked(&mut self, value: T) {
        debug_assert!(self.len < N, "push_unchecked on a full ArrayVec");
        // SAFETY: the contract gives `len < N`, so slot `len` is inside the
        // array and free for the raw write.
        unsafe {
            self.buf.as_mut_ptr().cast::<T>().add(self.len).write(value);
        }
        self.len += 1;
    }

    /// Appends `value` if there is room, handing it back if there is not.
    ///
    /// The recoverable counterpart to [`Self::push`]: on failure the buffer is
    /// unchanged and `value` comes back to be routed elsewhere.
    ///
    /// ```
    /// use vime_engine::util::vec::ArrayVec;
    ///
    /// let mut v: ArrayVec<u8, 2> = ArrayVec::default();
    /// assert_eq!(v.try_push(1), Ok(()));
    /// assert_eq!(v.try_push(2), Ok(()));
    ///
    /// // Full: the value is returned rather than lost or unwound.
    /// let rejected = v.try_push(3).unwrap_err();
    /// assert_eq!(&v[..], &[1, 2], "a refused push leaves the buffer alone");
    /// assert_eq!(rejected, 3);
    /// ```
    #[inline]
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.len == N {
            return Err(value);
        }
        // SAFETY: the branch above proved `len < N`.
        unsafe { self.push_unchecked(value) };
        Ok(())
    }

    /// Removes and returns the last element, or `None` when empty.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: `len` was non-zero, so after the decrement slot `len` was
        // initialized by a matching push; `T: Copy` makes the read a plain copy.
        Some(unsafe { self.buf.as_ptr().cast::<T>().add(self.len).read() })
    }

    /// Inserts `value` at `index`, shifting the tail right by one; panics when
    /// `index > len` or the array is full.
    #[inline]
    pub fn insert(&mut self, index: usize, value: T) {
        assert!(
            index <= self.len,
            "insertion index ({index}) out of bounds (len = {})",
            self.len
        );
        assert!(
            self.len < N,
            "ArrayVec overflow: cannot insert, capacity is {N}"
        );
        // SAFETY: both bounds were just checked.
        unsafe { self.insert_unchecked(index, value) };
    }

    /// Inserts `value` at `index` without checking either bound, shifting the
    /// tail right.
    ///
    /// # Safety
    ///
    /// `index` must be at most `self.len()` and `self.len()` less than `N`;
    /// otherwise the shift reads uninitialized slots or writes past the end
    /// of the array.
    #[inline]
    pub unsafe fn insert_unchecked(&mut self, index: usize, value: T) {
        debug_assert!(
            index <= self.len && self.len < N,
            "insert_unchecked out of bounds on a full ArrayVec"
        );
        // SAFETY: the contract gives `index <= len < N`, so the overlapping
        // `ptr::copy` stays inside the buffer and frees slot `index` for the
        // raw write.
        unsafe {
            let at = self.buf.as_mut_ptr().add(index);
            core::ptr::copy(at, at.add(1), self.len - index);
            at.cast::<T>().write(value);
        }
        self.len += 1;
    }

    /// Removes and returns the element at `index`, shifting the tail left by
    /// one; panics when `index >= len`.
    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        assert!(
            index < self.len,
            "removal index ({index}) out of bounds (len = {})",
            self.len
        );
        // SAFETY: `index < len` holds an initialized value; `T: Copy` makes the
        // read a plain bitwise copy.
        let removed = unsafe { self.buf.as_ptr().cast::<T>().add(index).read() };
        // SAFETY: `index < len`, so the copy reads only initialized slots and
        // stays within the buffer.
        unsafe {
            let at = self.buf.as_mut_ptr().add(index);
            core::ptr::copy(at.add(1), at, self.len - index - 1);
        }
        self.len -= 1;
        removed
    }

    /// Clears all elements, keeping the capacity.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Appends every element of `values`; panics if they do not all fit.
    ///
    /// One raw copy instead of repeated `push`es.
    #[inline(always)]
    pub fn extend_from_slice(&mut self, values: &[T]) {
        let count = values.len();
        assert!(
            count <= N - self.len,
            "ArrayVec overflow: cannot extend with {count} elements, capacity is {N}"
        );
        // SAFETY: `self.len + count <= N` was checked, so the destination is
        // inside the array and uninitialized; `len` then grows to cover the
        // copied slots exactly once.
        unsafe {
            core::ptr::copy_nonoverlapping(
                values.as_ptr(),
                self.buf.as_mut_ptr().cast::<T>().add(self.len),
                count,
            );
            self.len += count;
        }
    }
}

impl<T: Copy, const N: usize> VecLike<T> for ArrayVec<T, N> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn as_slice(&self) -> &[T] {
        ArrayVec::as_slice(self)
    }

    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [T] {
        ArrayVec::as_mut_slice(self)
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        N
    }

    #[inline(always)]
    fn push(&mut self, value: T) {
        ArrayVec::push(self, value);
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
        ArrayVec::pop(self)
    }

    #[inline]
    fn insert(&mut self, index: usize, value: T) {
        ArrayVec::insert(self, index, value);
    }

    #[inline]
    fn remove(&mut self, index: usize) -> T {
        ArrayVec::remove(self, index)
    }

    #[inline(always)]
    fn clear(&mut self) {
        ArrayVec::clear(self);
    }

    #[inline(always)]
    fn extend_from_slice(&mut self, values: &[T]) {
        ArrayVec::extend_from_slice(self, values);
    }
}
impl<T: Copy, const N: usize> Deref for ArrayVec<T, N> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Copy, const N: usize> DerefMut for ArrayVec<T, N> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Copy, const N: usize, I> Index<I> for ArrayVec<T, N>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;

    #[inline(always)]
    fn index(&self, index: I) -> &I::Output {
        &self.deref()[index]
    }
}

impl<T: Copy, const N: usize, I> IndexMut<I> for ArrayVec<T, N>
where
    I: SliceIndex<[T]>,
{
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut I::Output {
        &mut self.deref_mut()[index]
    }
}

impl<T: Copy + PartialEq, const N: usize> PartialEq for ArrayVec<T, N> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        &self[..] == &other[..]
    }
}

impl<T: Copy + Eq, const N: usize> Eq for ArrayVec<T, N> {}

impl<T: Copy + fmt::Debug, const N: usize> fmt::Debug for ArrayVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Copy, const N: usize> Extend<T> for ArrayVec<T, N> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();

        // Fail fast on an advertised overflow rather than panicking mid-write;
        // the guarded drain below only fires for a hint-lying iterator.
        if let Some(max) = upper {
            assert!(
                max <= N - self.len,
                "ArrayVec overflow: cannot extend with {max} elements, capacity is {N}"
            );
        }

        let orig_len = self.len;
        let space = N - orig_len;
        // SAFETY: `len <= N`, so `orig_len` addresses a slot in the array and
        // the free space after it is exactly `space`.
        let start = unsafe { self.buf.as_mut_ptr().cast::<T>().add(orig_len) };
        let mut ptr = start;

        // SAFETY of the write block: `ptr` stays within the free tail
        // `start..start + space`, every write lands in an uninitialized slot
        // exactly once (trust loop capped at `space`, drain guarded by
        // `space_left`), and `T: Copy` keeps the transfers drop-free.
        let trusted = lower.min(space);
        let mut left = trusted;
        while left > 0 {
            match iter.next() {
                Some(value) => unsafe {
                    // SAFETY: `left` counts down from `trusted <= space`, so
                    // the writes fill exactly the free slots past `orig_len`,
                    // once each.
                    ptr.write(value);
                    ptr = ptr.add(1);
                },
                None => {
                    // The iterator undersold its hint; `ptr - start` already
                    // counts what was written. SAFETY: both pointers address
                    // the same array with `ptr >= start`, so the distance is
                    // non-negative and at most `N`.
                    self.len = orig_len + unsafe { ptr.offset_from(start) } as usize;
                    return;
                }
            }
            left -= 1;
        }

        // Drain any surplus that the lower bound understated.
        let mut space_left = space - trusted;
        for value in &mut iter {
            if space_left == 0 {
                panic!("ArrayVec overflow: cannot extend, capacity is {N}");
            }
            // SAFETY: the `space_left == 0` guard runs first, so this write
            // targets a free slot and the count-down matches the space left.
            unsafe {
                ptr.write(value);
                ptr = ptr.add(1);
            }
            space_left -= 1;
        }

        self.len = N - space_left;
    }
}

impl<T: Copy, const N: usize> FromIterator<T> for ArrayVec<T, N> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = ArrayVec::default();
        vec.extend(iter);
        vec
    }
}

/// Owning iterator over an [`ArrayVec`]'s live elements.
///
/// It owns the buffer rather than pointing into an `ArrayVec`: the elements
/// live inside the struct, so pointers into a dropped temporary would dangle.
pub struct IntoIter<T: Copy, const N: usize> {
    buf: [MaybeUninit<T>; N],
    front: usize,
    back: usize,
}

impl<T: Copy, const N: usize> IntoIter<T, N> {
    /// Elements not yet yielded from either end.
    #[inline(always)]
    fn remaining(&self) -> usize {
        self.back - self.front
    }
}

impl<T: Copy, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }

        // SAFETY: `front < back <= len`, slots `0..len` are initialized, and
        // `T: Copy` makes the read a non-dropping copy.
        let value = unsafe { self.buf.get_unchecked(self.front).assume_init_read() };
        self.front += 1;

        Some(value)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.remaining();
        (n, Some(n))
    }

    #[inline(always)]
    fn count(self) -> usize {
        self.remaining()
    }
}

impl<T: Copy, const N: usize> ExactSizeIterator for IntoIter<T, N> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.remaining()
    }
}

impl<T: Copy, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }

        // SAFETY: as in `next`, from the back: `back` is exclusive, so
        // `back - 1` is still inside the initialized `0..len` prefix.
        self.back -= 1;
        Some(unsafe { self.buf.get_unchecked(self.back).assume_init_read() })
    }
}

impl<T: Copy, const N: usize> FusedIterator for IntoIter<T, N> {}

impl<T: Copy, const N: usize> IntoIterator for ArrayVec<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len;
        IntoIter {
            buf: self.buf,
            front: 0,
            back: len,
        }
    }
}

impl<T: Copy, const N: usize> From<[T; N]> for ArrayVec<T, N> {
    #[inline(always)]
    fn from(array: [T; N]) -> Self {
        Self {
            buf: array.map(MaybeUninit::new),
            len: N,
        }
    }
}

/// The elements a conversion into an `ArrayVec` could not take, handed back.
///
/// Named for the reason it fails: a fixed-capacity buffer simply has no room.
pub type TooLong<T> = Vec<T>;

/// Adopts an [`ArrayVec`] of any capacity, handing the elements back if it is
/// too long.
///
/// Fallible because a source `VecLike` may hold more than `N` elements, so
/// there is no panicking `From<&V>`.
impl<T: Copy, const M: usize, const N: usize> TryFrom<&ArrayVec<T, M>> for ArrayVec<T, N> {
    type Error = TooLong<T>;

    #[inline]
    fn try_from(src: &ArrayVec<T, M>) -> Result<Self, Self::Error> {
        if src.len() > N {
            return Err(src.as_slice().to_vec());
        }
        let mut out = Self::new();
        out.extend_from_slice(src.as_slice());
        Ok(out)
    }
}

/// Adopts a [`SmallVec`](super::SmallVec) of any capacity, handing the elements
/// back if it is too long — a spilled one can exceed `N`, which is why the
/// conversion is fallible.
impl<T: Copy, const M: usize, const N: usize> TryFrom<&super::SmallVec<T, M>> for ArrayVec<T, N> {
    type Error = TooLong<T>;

    #[inline]
    fn try_from(src: &super::SmallVec<T, M>) -> Result<Self, Self::Error> {
        if src.len() > N {
            return Err(src.as_slice().to_vec());
        }
        let mut out = Self::new();
        out.extend_from_slice(src.as_slice());
        Ok(out)
    }
}
