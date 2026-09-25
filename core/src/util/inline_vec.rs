use std::{
    fmt,
    mem::MaybeUninit,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

/// An inline, fixed-capacity growable sequence.
///
/// The storage lives *inside the struct* — there is no heap allocation and no
/// pointer indirection. `InlineVec` is `Copy` exactly because its payload is
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
/// - Because element copies are free of side effects, `InlineVec` can itself
///   derive [`Copy`] (and `Clone`) and be returned cheaply by value.
///
/// Use a different container (e.g. `Vec<T>` or a `Box`ed array) when the
/// element type cannot be `Copy`.
#[derive(Clone, Copy)]
pub struct InlineVec<T, const N: usize>
where
    T: Copy,
{
    buf: [MaybeUninit<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> Default for InlineVec<T, N> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }
}

impl<T: Copy, const N: usize> InlineVec<T, N> {
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

    /// Appends `value` at the end; panics when the array is full.
    #[inline(always)]
    pub fn push(&mut self, value: T) {
        if self.len == N {
            panic!("InlineVec overflow: cannot push, capacity is {N}");
        }
        // SAFETY: `len < N` was checked, so slot `len` is uninitialized and a
        // raw `T` write lands inside the backing array.
        unsafe {
            self.buf.as_mut_ptr().cast::<T>().add(self.len).write(value);
        }
        self.len += 1;
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
        if self.len == N {
            panic!("InlineVec overflow: cannot insert, capacity is {N}");
        }
        // SAFETY: `index <= self.len` and `len < N`, so the copy reads the `len
        // - index` initialized slots `index..len` and writes them to `index+1..
        // =len`, which stays within the buffer; `ptr::copy` permits overlap.
        // Slot `index` is free after the shift, so the raw `T` write is sound.
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
    /// `&InlineVec<T, N>`.
    #[inline(always)]
    pub fn extend_from_slice(&mut self, values: &[T]) {
        let count = values.len();
        assert!(
            count <= N - self.len,
            "InlineVec overflow: cannot extend with {count} elements, capacity is {N}"
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

impl<T: Copy, const N: usize> Deref for InlineVec<T, N> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &[T] {
        // SAFETY: only slots `0..len` are ever written, each initialized
        // exactly once before `len` grows to cover it, and `len` never
        // exceeds N. `MaybeUninit<T>` has the same layout/alignment as `T`.
        unsafe { std::slice::from_raw_parts(self.buf.as_ptr().cast::<T>(), self.len) }
    }
}

impl<T: Copy, const N: usize> DerefMut for InlineVec<T, N> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [T] {
        // SAFETY: the `Deref` invariant also holds here: `len` never exceeds
        // N and every covered slot is initialized.
        unsafe { std::slice::from_raw_parts_mut(self.buf.as_mut_ptr().cast::<T>(), self.len) }
    }
}

impl<T: Copy, const N: usize, I> Index<I> for InlineVec<T, N>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;

    #[inline(always)]
    fn index(&self, index: I) -> &I::Output {
        &self.deref()[index]
    }
}

impl<T: Copy, const N: usize, I> IndexMut<I> for InlineVec<T, N>
where
    I: SliceIndex<[T]>,
{
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut I::Output {
        &mut self.deref_mut()[index]
    }
}

impl<T: Copy + PartialEq, const N: usize> PartialEq for InlineVec<T, N> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        &self[..] == &other[..]
    }
}

impl<T: Copy + Eq, const N: usize> Eq for InlineVec<T, N> {}

impl<T: Copy + fmt::Debug, const N: usize> fmt::Debug for InlineVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Copy, const N: usize> Extend<T> for InlineVec<T, N> {
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
                "InlineVec overflow: cannot extend with {max} elements, capacity is {N}"
            );
        }

        let orig_len = self.len;
        let space = N - orig_len;
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
                    ptr.write(value);
                    ptr = ptr.add(1);
                },
                None => {
                    // The iterator produced fewer items than its hint promised;
                    // `ptr - start` already counts what was actually written.
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
                panic!("InlineVec overflow: cannot extend, capacity is {N}");
            }
            unsafe {
                ptr.write(value);
                ptr = ptr.add(1);
            }
            space_left -= 1;
        }

        self.len = N - space_left;
    }
}

impl<T: Copy, const N: usize> FromIterator<T> for InlineVec<T, N> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = InlineVec::default();
        vec.extend(iter);
        vec
    }
}