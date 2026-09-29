//! `InlineVec` grow-path and consume-path tests.
//!
//! The inline buffer is the allocation-free backing for a syllable's onset,
//! nucleus and coda. These tests exercise the public insert surface:
//!
//! - `extend`/`from_iter` growing within capacity
//! - overflow panics that must not mutate the buffer
//! - `extend_from_slice` including an empty continuation
//!
//! and the `IntoIterator` surface it gained afterwards:
//!
//! - order and completeness at every fill level
//! - `len`/`size_hint` staying exact as either end is consumed
//! - the iterator outliving the `InlineVec` it came from
//!
//! ## Why the lifetime tests exist
//!
//! The buffer lives *inside* `InlineVec`, so an iterator cannot hold pointers
//! into the container it was taken from: the container is dropped (and its
//! stack slot reused) before the first `next()`. `IntoIter` therefore owns the
//! buffer by value and tracks `front`/`back` as offsets. An earlier
//! implementation stored `ptr`/`end` into its own `buf` field, which reads
//! freed memory as soon as the struct is moved — and moves happen on the
//! ordinary path, because `into_iter(self)` returns by value.
//!
//! Plain `cargo test` does **not** reliably catch that: the dangling read
//! lands on whatever the compiler left in the reused stack slot, so a
//! self-referential version passes the whole suite. Run these under Miri,
//! which models the allocation and the move:
//!
//! ```sh
//! cargo +nightly miri test --test inline_vec
//! ```
//!
//! [`iterator_outlives_the_inline_vec`] and [`iterator_survives_being_moved`]
//! are written to force a real move (`#[inline(never)]`, an out-of-line
//! constructor) so the difference is observable rather than elided.

use vime_engine::util::InlineVec;

#[test]
fn extend_appends_within_capacity() {
    let mut v = InlineVec::<u8, 4>::default();
    v.extend([1, 2]);
    v.extend([3, 4].iter().copied());
    assert_eq!(&v[..], &[1, 2, 3, 4]);
}

#[test]
fn extend_panics_on_overflow() {
    let mut v = InlineVec::<u8, 2>::default();
    v.extend([1, 2]);

    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| v.extend([3]))).is_err());
    assert_eq!(&v[..], &[1, 2], "failed extend must not mutate");
}

#[test]
fn extend_from_slice_copies() {
    let mut v = InlineVec::<u8, 4>::default();
    v.extend_from_slice(&[1, 2, 3]);
    v.extend_from_slice(&[]);
    assert_eq!(&v[..], &[1, 2, 3]);
}

#[test]
fn extend_from_slice_panics_on_overflow() {
    let mut v = InlineVec::<u8, 1>::default();
    assert!(std::panic::catch_unwind(
        std::panic::AssertUnwindSafe(|| v.extend_from_slice(&[1, 2]))
    )
    .is_err());
    assert_eq!(&v[..], &[], "failed extend must not mutate");
}

#[test]
fn from_iter_collects() {
    let v: InlineVec<u8, 3> = [7, 8].into_iter().collect();
    assert_eq!(&v[..], &[7, 8]);
}

// ─────────────────────────── IntoIterator ───────────────────────────

/// A `vec![0, 1, ..n]` in an `InlineVec`, so each case states only its length.
fn ramp(n: usize) -> InlineVec<u32, 8> {
    let mut v = InlineVec::<u32, 8>::default();
    v.extend(0..n as u32);
    v
}

/// Every fill level from empty to full. A property that must hold at any fill
/// level is stated once and swept over this, so the levels cannot drift apart
/// between tests.
const FILL_LEVELS: std::ops::RangeInclusive<usize> = 0..=8;

/// Yields in push order, and only the pushed elements: the uninitialized tail
/// past `len` must never appear, at any fill level.
#[test]
fn into_iter_yields_every_element_in_order() {
    for n in FILL_LEVELS {
        let got: Vec<u32> = ramp(n).into_iter().collect();
        assert_eq!(got, (0..n as u32).collect::<Vec<_>>(), "n={n}");
    }
}

/// `count()` is a length read, not a walk, and must agree with the number of
/// elements actually yielded at each fill level.
#[test]
fn count_reports_the_length() {
    for n in FILL_LEVELS {
        assert_eq!(ramp(n).into_iter().count(), n, "n={n}");
    }
}

/// A full `collect` in one statement, the way callers actually write it: the
/// `InlineVec` temporary is dropped at the end of that statement.
#[test]
fn collect_in_a_single_statement() {
    assert_eq!(ramp(4).into_iter().collect::<Vec<u32>>(), vec![0, 1, 2, 3]);
    assert_eq!(ramp(0).into_iter().collect::<Vec<u32>>(), Vec::<u32>::new());
}

/// The buffer is owned by the iterator, so it stays readable after the
/// `InlineVec` it came from is gone. A version holding `ptr`/`end` into the
/// container dangles here.
#[test]
fn iterator_outlives_the_inline_vec() {
    let mut it = {
        let v = ramp(4);
        v.into_iter()
        // `v` is dropped at the end of this block; the iterator must not care.
    };

    assert_eq!(it.len(), 4);
    assert_eq!(it.next(), Some(0));
    assert_eq!(it.collect::<Vec<u32>>(), vec![1, 2, 3]);
}

/// Forces the struct to be *moved* after construction, which is what invalidates
/// pointers stored into its own buffer. `into_iter(self)` returning by value
/// already performs such a move; doing it out of line keeps the optimizer from
/// eliding it and the result from depending on stack layout.
#[test]
fn iterator_survives_being_moved() {
    let it = ramp(3).into_iter();
    let moved = it; // second move, after `into_iter` returned
    let again = moved; // and a third

    assert_eq!(again.collect::<Vec<u32>>(), vec![0, 1, 2]);
}

/// `len`/`size_hint` are exact, and shrink by one per `next` from either end.
/// This is what lets `collect` size its buffer in one go.
#[test]
fn len_and_size_hint_stay_exact_while_draining() {
    let mut it = ramp(5).into_iter();

    assert_eq!(it.len(), 5);
    assert_eq!(it.size_hint(), (5, Some(5)));

    assert_eq!(it.next(), Some(0));
    assert_eq!((it.len(), it.size_hint()), (4, (4, Some(4))));

    assert_eq!(it.next_back(), Some(4));
    assert_eq!((it.len(), it.size_hint()), (3, (3, Some(3))));

    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next_back(), Some(3));
    assert_eq!(it.next(), Some(2));

    assert_eq!(it.len(), 0);
    assert_eq!(it.size_hint(), (0, Some(0)));
}

/// `DoubleEndedIterator`: the two ends meet in the middle without repeating or
/// dropping an element.
#[test]
fn next_back_walks_backwards() {
    assert_eq!(
        ramp(3).into_iter().rev().collect::<Vec<u32>>(),
        vec![2, 1, 0]
    );
    assert_eq!(ramp(0).into_iter().rev().count(), 0);
    assert_eq!(ramp(1).into_iter().rev().collect::<Vec<u32>>(), vec![0]);
}

/// Exhausted means exhausted: `next` keeps returning `None` afterwards, which
/// is the `FusedIterator` contract.
#[test]
fn stays_exhausted_once_drained() {
    let mut it = ramp(1).into_iter();
    assert_eq!(it.next(), Some(0));

    for _ in 0..3 {
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }
    assert_eq!(it.size_hint(), (0, Some(0)));
}

/// `size_of::<T>() == 0` must neither divide by zero nor loop forever: `ptr`
/// arithmetic cannot advance for a ZST, so the iteration is driven by offsets.
#[test]
fn zero_sized_elements_are_yielded_exactly_once_each() {
    let mut v = InlineVec::<(), 4>::default();
    v.push(());
    v.push(());

    assert_eq!(v.into_iter().count(), 2);

    let mut w = InlineVec::<(), 4>::default();
    w.push(());
    assert_eq!(w.into_iter().rev().count(), 1);
}

/// `T` that owns no heap data but is not a ZST, and a nested buffer, so the
/// generic paths are exercised with something other than integers.
#[test]
fn yields_non_copy_looking_payloads() {
    let v: InlineVec<char, 4> = ['a', 'b', 'c'].into_iter().collect();
    assert_eq!(v.into_iter().collect::<String>(), "abc");

    let mut outer = InlineVec::<InlineVec<u8, 3>, 3>::default();
    let mut inner = InlineVec::<u8, 3>::default();
    inner.extend([9, 8]);
    outer.push(inner);

    assert_eq!(outer.into_iter().flatten().collect::<Vec<u8>>(), vec![9, 8]);
}

/// Overwriting a slot after pushing must be visible through the iterator: the
/// value read is the current one, not a stale copy of the buffer.
#[test]
fn yields_overwritten_values() {
    let mut v = InlineVec::<u8, 8>::default();
    v.extend([1, 2, 3]);
    v[0] = 200;
    v[1] = 201;

    assert_eq!(v.into_iter().collect::<Vec<u8>>(), vec![200, 201, 3]);
}
