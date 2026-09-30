use super::vec_like::VecLike;

use std::{
    fmt,
    iter::FusedIterator,
    mem::MaybeUninit,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

/// An inline, fixed-capacity growable sequence.
///
/// The storage lives *inside the struct* — there is no heap allocation and no
/// pointer indirection. `ArrayVec` is `Copy` exactly because its payload is
/// embedded: moving or cloning the value copies the whole buffer wholesale,
/// and the type size is the constant `size_of::<[MaybeUninit<T>; N]>() +
/// size_of::<usize>()`, regardless of `len`.
///
/// It holds at most `N` elements and tracks the live length separately. The
/// backing buffer starts uninitialized (`[MaybeUninit<T>; N]`), so `T` needs
/// neither `Default` nor a placeholder value; only the slots in `0..len` are
/// ever observed.
///
/// ## The `T: Copy` bound
///
/// `T` is required to be [`Copy`], and that boundary is structural, not
/// incidental:
///
/// - `push`/`insert`/`remove` move elements with raw `ptr::copy`, recreating
///   duplicate (and orphaned) slots until `len` catches up. `Copy` guarantees
///   such bytewise duplication is *observationally identical* to a normal
///   move: no `Drop` ever runs on a slot the buffer gives up on, so no value
///   is destroyed twice or leaked.
/// - `pop`/`remove` hand out values by bitwise read without invalidating the
///   slot in `buf`; a `Drop`-owning `T` would be dropped again when the
///   buffer is someday reused. `Copy` rules that out.
/// - Because element copies are free of side effects, `ArrayVec` can itself
///   derive [`Copy`] (and `Clone`) and be returned cheaply by value.
///
/// Use a different container (e.g. `Vec<T>` or a `Box`ed array) when the
/// element type cannot be `Copy`.
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
    /// Creates an empty `ArrayVec`.
    ///
    /// The backing storage is uninitialized and no heap allocation occurs.
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
        // SAFETY: only slots `0..len` are ever written, each initialized
        // exactly once before `len` grows to cover it, and `len` never
        // exceeds N. `MaybeUninit<T>` has the same layout/alignment as `T`.
        unsafe { std::slice::from_raw_parts(self.buf.as_ptr().cast::<T>(), self.len) }
    }

    /// Returns a mutable slice containing the live elements.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: the `Deref` invariant also holds here: `len` never exceeds
        // N and every covered slot is initialized.
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

    /// Appends `value` at the end without checking that there is room.
    ///
    /// For a caller that has already established `len < N` — one that knows the
    /// buffer has space, so a full-buffer panic would be dead weight. `SmallVec`
    /// is the case that motivates it: its own guard decides whether to push
    /// inline or spill, so re-checking inside the push repeats a comparison the
    /// branch already made.
    ///
    /// # Safety
    ///
    /// `self.len()` must be less than `N`. Writing into a slot at or past the
    /// end of the backing array is undefined behaviour, and leaving `len`
    /// claiming an uninitialized slot makes every later read unsound.
    #[inline(always)]
    pub unsafe fn push_unchecked(&mut self, value: T) {
        debug_assert!(self.len < N, "push_unchecked on a full ArrayVec");
        // SAFETY: the contract says `len < N`, so slot `len` is inside the
        // backing array and uninitialized, and a raw `T` write lands there.
        unsafe {
            self.buf.as_mut_ptr().cast::<T>().add(self.len).write(value);
        }
        self.len += 1;
    }

    /// Appends `value` if there is room, handing it back if there is not.
    ///
    /// The recoverable counterpart to [`Self::push`], for a caller that treats a
    /// full buffer as an outcome rather than a bug. On failure the buffer is
    /// left exactly as it was and `value` comes back, so it can be routed
    /// somewhere else instead of unwinding the stack. `T` is `Copy`, so handing
    /// it back costs nothing.
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
        // SAFETY: `len` was non-zero and has been decremented, so slot `len`
        // was initialized by a matching push/insert. `T: Copy` makes the raw
        // read a plain bitwise copy.
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
    /// tail right by one.
    ///
    /// # Safety
    ///
    /// `index` must be at most `self.len()`, and `self.len()` must be less than
    /// `N`. A `index > len` makes the shift read uninitialized slots, and a full
    /// buffer makes the shift write past the end of the array; either is
    /// undefined behaviour.
    #[inline]
    pub unsafe fn insert_unchecked(&mut self, index: usize, value: T) {
        debug_assert!(
            index <= self.len && self.len < N,
            "insert_unchecked out of bounds on a full ArrayVec"
        );
        // SAFETY: the contract says `index <= len` and `len < N`, so the copy
        // reads the `len - index` initialized slots `index..len` and writes them
        // to `index+1..=len`, which stays within the buffer; `ptr::copy` permits
        // overlap. Slot `index` is free after the shift, so the raw `T` write is
        // sound.
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
        // SAFETY: `index < len` was checked, so this slot holds an initialized
        // value; `T: Copy` makes the raw read a plain bitwise copy.
        let removed = unsafe { self.buf.as_ptr().cast::<T>().add(index).read() };
        // SAFETY: `index < len`, so `index + 1..len` covers only initialized
        // slots and the destination `index..=len - 1` stays within the buffer.
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

    /// Appends every element of `values`; panics when the array is full.
    ///
    /// Uses a single raw copy up front instead of repeated `push` calls. Works
    /// on any `&[T]`, which deref coercion also makes available for
    /// `&ArrayVec<T, N>`.
    #[inline(always)]
    pub fn extend_from_slice(&mut self, values: &[T]) {
        let count = values.len();
        assert!(
            count <= N - self.len,
            "ArrayVec overflow: cannot extend with {count} elements, capacity is {N}"
        );
        // SAFETY: `self.len + count <= N` was checked, so the destination range
        // `self.len..self.len + count` sits inside the backing array and is
        // uninitialized. The whole tail of `values` is copied and `len` is then
        // grown to cover it, so every written slot is observed exactly once.
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

        // Fail fast when the iterator already advertises overflow instead of
        // panicking mid-write; the guarded drain below then only fires for a
        // hint-lying iterator.
        if let Some(max) = upper {
            assert!(
                max <= N - self.len,
                "ArrayVec overflow: cannot extend with {max} elements, capacity is {N}"
            );
        }

        let orig_len = self.len;
        let space = N - orig_len;
        // SAFETY: `len` never exceeds `N`, so `orig_len` addresses a slot
        // inside the array, and the free space after it is exactly `space`.
        let start = unsafe { self.buf.as_mut_ptr().cast::<T>().add(orig_len) };
        let mut ptr = start;

        // SAFETY of the write block:
        // - `start` points at the first free slot and `space` counts the
        //   slots left in the backing array, so `ptr` never leaves it.
        // - The trust loop writes at most `trusted = lower.min(space)` items
        //   with no per-item overflow check; the drain refuses to write once
        //   `space_left == 0`. Every write therefore lands in an
        //   uninitialized slot and no slot is written twice. `T: Copy` makes
        //   the transfers free of `Drop` effects.
        let trusted = lower.min(space);
        let mut left = trusted;
        while left > 0 {
            match iter.next() {
                Some(value) => unsafe {
                    // SAFETY: `left` counts down from `trusted <= space`, so at
                    // most `space` writes land, filling exactly the free slots
                    // past `orig_len`. No slot is written twice.
                    ptr.write(value);
                    ptr = ptr.add(1);
                },
                None => {
                    // The iterator produced fewer items than its hint promised;
                    // `ptr - start` already counts what was actually written.
                    // SAFETY: both pointers address the same array and `ptr`
                    // never precedes `start`, so the distance is non-negative
                    // and at most `N`.
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
            // SAFETY: the `space_left == 0` guard above runs first, so this
            // write targets a free slot and `space_left` counts down to zero
            // exactly as the space runs out.
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
/// It holds the buffer **by value** instead of pointing into an `ArrayVec`
/// that has already been dropped. That is what makes it sound: `ArrayVec`
/// stores its elements inline, in the struct itself, so an iterator that only
/// kept `start`/`end` pointers into it would dangle the moment the temporary
/// was dropped at the end of the `into_iter()` statement.
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

        // SAFETY: `front < back <= len`, and every slot in `0..len` was
        // initialized before `len` was advanced to cover it. `T: Copy`, so
        // reading the value out and forgetting the slot cannot drop it twice.
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

        // SAFETY: as in `next`, but from the back. `back` is exclusive, so the
        // slot at `back - 1` is still inside the initialized `0..len` prefix.
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
/// Named for the reason the conversion fails: a fixed-capacity buffer has no
/// room, not because anything is wrong with the elements themselves. It is the
/// error type of the [`TryFrom`] impls below.
pub type TooLong<T> = Vec<T>;

/// Adopts an [`ArrayVec`] of any capacity, handing the elements back if it is
/// too long.
///
/// The fallible conversion, and the only one into a fixed-capacity buffer: a
/// `VecLike` may hold more than `N` elements — a spilled
/// [`SmallVec`](super::SmallVec) is the reachable case — so there is no
/// `From<&V>` here to panic with.
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
/// back if it is too long.
///
/// The case that makes a fallible conversion necessary at all: a `SmallVec` that
/// has spilled to the heap holds more than a fixed-capacity buffer can take.
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
