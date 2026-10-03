//! `ArrayVec` tests: grow (extend, from_iter, extend_from_slice), overflow panics (no mutation),
//! IntoIterator (order, exact size_hint, iterator outlives container, Miri-catchable lifetime bugs).

use vime_engine::util::vec::{ArrayVec, SmallVec};

#[test]
fn extend_appends_within_capacity() {
    let mut v = ArrayVec::<u8, 4>::default();
    v.extend([1, 2]);
    v.extend([3, 4].iter().copied());
    assert_eq!(&v[..], &[1, 2, 3, 4]);
}

#[test]
fn extend_panics_on_overflow() {
    let mut v = ArrayVec::<u8, 2>::default();
    v.extend([1, 2]);

    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| v.extend([3]))).is_err());
    assert_eq!(&v[..], &[1, 2], "failed extend must not mutate");
}

#[test]
fn extend_from_slice_copies() {
    let mut v = ArrayVec::<u8, 4>::default();
    v.extend_from_slice(&[1, 2, 3]);
    v.extend_from_slice(&[]);
    assert_eq!(&v[..], &[1, 2, 3]);
}

#[test]
fn extend_from_slice_panics_on_overflow() {
    let mut v = ArrayVec::<u8, 1>::default();
    assert!(std::panic::catch_unwind(
        std::panic::AssertUnwindSafe(|| v.extend_from_slice(&[1, 2]))
    )
    .is_err());
    assert_eq!(&v[..], &[], "failed extend must not mutate");
}

#[test]
fn from_iter_collects() {
    let v: ArrayVec<u8, 3> = [7, 8].into_iter().collect();
    assert_eq!(&v[..], &[7, 8]);
}

// ─────────────────────────── IntoIterator ───────────────────────────

/// A `vec![0, 1, ..n]` in an `ArrayVec`, so each case states only its length.
fn ramp(n: usize) -> ArrayVec<u32, 8> {
    let mut v = ArrayVec::<u32, 8>::default();
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
/// `ArrayVec` temporary is dropped at the end of that statement.
#[test]
fn collect_in_a_single_statement() {
    assert_eq!(ramp(4).into_iter().collect::<Vec<u32>>(), vec![0, 1, 2, 3]);
    assert_eq!(ramp(0).into_iter().collect::<Vec<u32>>(), Vec::<u32>::new());
}

/// The buffer is owned by the iterator, so it stays readable after the
/// `ArrayVec` it came from is gone. A version holding `ptr`/`end` into the
/// container dangles here.
#[test]
fn iterator_outlives_the_array_vec() {
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
    let mut v = ArrayVec::<(), 4>::default();
    v.push(());
    v.push(());

    assert_eq!(v.into_iter().count(), 2);

    let mut w = ArrayVec::<(), 4>::default();
    w.push(());
    assert_eq!(w.into_iter().rev().count(), 1);
}

/// `T` that owns no heap data but is not a ZST, and a nested buffer, so the
/// generic paths are exercised with something other than integers.
#[test]
fn yields_non_copy_looking_payloads() {
    let v: ArrayVec<char, 4> = ['a', 'b', 'c'].into_iter().collect();
    assert_eq!(v.into_iter().collect::<String>(), "abc");

    let mut outer = ArrayVec::<ArrayVec<u8, 3>, 3>::default();
    let mut inner = ArrayVec::<u8, 3>::default();
    inner.extend([9, 8]);
    outer.push(inner);

    assert_eq!(outer.into_iter().flatten().collect::<Vec<u8>>(), vec![9, 8]);
}

/// Overwriting a slot after pushing must be visible through the iterator: the
/// value read is the current one, not a stale copy of the buffer.
#[test]
fn yields_overwritten_values() {
    let mut v = ArrayVec::<u8, 8>::default();
    v.extend([1, 2, 3]);
    v[0] = 200;
    v[1] = 201;

    assert_eq!(v.into_iter().collect::<Vec<u8>>(), vec![200, 201, 3]);
}

// ────────────────────── the unchecked primitives ──────────────────────
//
// `push_unchecked` and `insert_unchecked` are `unsafe`, so the guarantee is the
// caller's. What is testable here is that, under that guarantee, they do exactly
// what their checked counterparts do — and that the checked ones still refuse.

/// The unchecked push appends in the same order the checked one does.
#[test]
fn push_unchecked_appends_like_push() {
    let mut v = ArrayVec::<u8, 4>::default();
    for i in 0..4u8 {
        // SAFETY: `len == i < 4` for every `i` in `0..4`.
        unsafe { v.push_unchecked(i) };
    }
    assert_eq!(&v[..], &[0, 1, 2, 3]);
    assert_eq!(v.len(), 4);
}

/// The unchecked insert shifts the tail exactly as the checked one does.
#[test]
fn insert_unchecked_shifts_like_insert() {
    let mut v = ArrayVec::<u8, 8>::default();
    v.extend([1, 2, 3]);
    // SAFETY: `index == 0 <= len == 3`, and `len == 3 < 4`.
    unsafe { v.insert_unchecked(0, 9) };
    assert_eq!(&v[..], &[9, 1, 2, 3]);

    // SAFETY: `index == len == 4` is the end-insert case, and `4 < 8`.
    unsafe { v.insert_unchecked(v.len(), 4) };
    assert_eq!(&v[..], &[9, 1, 2, 3, 4]);
}

/// The checked versions still panic on overflow, and leave the buffer alone.
/// This is the guard the unchecked pair deliberately omits.
#[test]
fn the_checked_forms_still_refuse_to_overflow() {
    let mut v = ArrayVec::<u8, 2>::default();
    v.push(1);
    v.push(2);

    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| v.push(3))).is_err(),
        "push must refuse a full buffer"
    );
    assert_eq!(&v[..], &[1, 2], "a refused push must not mutate");

    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| v.insert(0, 9))).is_err(),
        "insert must refuse a full buffer"
    );
    assert_eq!(&v[..], &[1, 2], "a refused insert must not mutate");
}

/// The two paths agree element for element, so choosing the unchecked form is
/// only ever about dropping a check, never about behaviour.
#[test]
fn checked_and_unchecked_agree() {
    let mut checked = ArrayVec::<u8, 6>::default();
    let mut unchecked = ArrayVec::<u8, 6>::default();
    let ops: [(usize, u8); 5] = [(0, 1), (0, 2), (2, 3), (1, 4), (0, 5)];

    for (index, value) in ops {
        checked.insert(index, value);
        // SAFETY: `checked` holds the same elements at every step, so its
        // `index` is in bounds and `len == index_of_step < 6`.
        unsafe { unchecked.insert_unchecked(index, value) };
        assert_eq!(&checked[..], &unchecked[..], "after insert at {index}");
    }

    // Five inserts leave `len == 5`, so exactly one push fits in a capacity of 6.
    checked.push(6);
    // SAFETY: both are at `len == 5 < 6`.
    unsafe { unchecked.push_unchecked(6) };
    assert_eq!(&checked[..], &unchecked[..], "after push 6");
    assert_eq!(checked.len(), 6);
    assert_eq!(
        ArrayVec::<u8, 6>::capacity(),
        checked.len(),
        "now exactly full"
    );
}

/// A buffer at exactly its capacity is full, and one more checked push refuses.
#[test]
fn a_full_buffer_still_refuses_one_more() {
    let mut v = ArrayVec::<u8, 3>::default();
    v.extend([1, 2, 3]);
    assert_eq!(ArrayVec::<u8, 3>::capacity(), 3);
    assert_eq!(v.len(), 3);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| v.push(4))).is_err());
    assert_eq!(&v[..], &[1, 2, 3]);
}

// ────────────────────────── the recoverable push ──────────────────────────
//
// `try_push` completes the family: `push` panics, `try_push` hands the value
// back, `push_unchecked` trusts the caller.

/// The recoverable push fills the buffer and then refuses, without unwinding.
#[test]
fn try_push_appends_while_there_is_room() {
    let mut v = ArrayVec::<u8, 3>::default();
    assert_eq!(v.try_push(1), Ok(()));
    assert_eq!(v.try_push(2), Ok(()));
    assert_eq!(v.try_push(3), Ok(()));
    assert_eq!(&v[..], &[1, 2, 3]);
    assert_eq!(v.len(), 3);
}

/// A full buffer hands the value back and is left exactly as it was, so the
/// caller can route the element somewhere else instead of unwinding.
#[test]
fn try_push_hands_the_value_back_when_full() {
    let mut v = ArrayVec::<u8, 2>::default();
    v.push(1);
    v.push(2);

    let rejected = v.try_push(3).expect_err("a full buffer must refuse");
    assert_eq!(rejected, 3, "the value comes back, not lost");
    assert_eq!(&v[..], &[1, 2], "and the buffer is untouched");
    assert_eq!(v.len(), 2);
}

/// Refusing is repeatable: a caller can keep offering the same value.
#[test]
fn try_push_refuses_every_time_while_full() {
    let mut v = ArrayVec::<u8, 1>::default();
    v.push(7);
    for _ in 0..3 {
        assert_eq!(v.try_push(9), Err(9));
    }
    assert_eq!(&v[..], &[7]);

    // And it succeeds again once there is room.
    assert_eq!(v.pop(), Some(7));
    assert_eq!(v.try_push(9), Ok(()));
    assert_eq!(&v[..], &[9]);
}

/// It never panics, which is the whole point: the failure is a value, not an
/// unwind crossing whatever boundary the caller is behind.
#[test]
fn try_push_never_unwinds() {
    let mut v = ArrayVec::<u8, 1>::default();
    v.push(1);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| v.try_push(2)));
    assert_eq!(outcome.expect("try_push must not panic"), Err(2));
}

/// `try_push` and `push` agree on every element they both accept.
#[test]
fn try_push_matches_push() {
    let mut pushed = ArrayVec::<u8, 4>::default();
    let mut tried = ArrayVec::<u8, 4>::default();
    for i in 0..4u8 {
        pushed.push(i);
        assert_eq!(tried.try_push(i), Ok(()));
        assert_eq!(&pushed[..], &tried[..], "after {i}");
    }
    // The fifth is where they part: one panics, the other returns.
    assert_eq!(tried.try_push(9), Err(9));
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pushed.push(9))).is_err());
    assert_eq!(&pushed[..], &tried[..]);
}

// ───────────────────── conversion from any vec-like ─────────────────────
//
// A fixed-capacity buffer can be too small for its source, so these are
// `TryFrom`: failure hands the elements back rather than losing them.

/// An `ArrayVec` of any capacity converts when it fits, and the result is the
/// live prefix — not the whole source buffer.
#[test]
fn try_from_an_array_vec_of_another_capacity() {
    let mut wide = ArrayVec::<u8, 8>::new();
    wide.extend([1u8, 2, 3]);

    let narrow = ArrayVec::<u8, 4>::try_from(&wide).expect("3 fits in 4");
    assert_eq!(&narrow[..], &[1, 2, 3]);

    let exact = ArrayVec::<u8, 3>::try_from(&wide).expect("3 fits in 3");
    assert_eq!(&exact[..], &[1, 2, 3]);
}

/// A spilled `SmallVec` holds more than its inline capacity, so this is the case
/// that makes the conversion fallible rather than a plain `From`.
#[test]
fn try_from_a_spilled_small_vec() {
    let spilled = SmallVec::<u8, 2>::from([1u8, 2, 3, 4, 5]);
    assert!(spilled.is_spilled());

    let too_long = ArrayVec::<u8, 4>::try_from(&spilled).expect_err("5 does not fit in 4");
    assert_eq!(too_long, vec![1, 2, 3, 4, 5]);

    let fits = ArrayVec::<u8, 5>::try_from(&spilled).expect("5 fits in 5");
    assert_eq!(&fits[..], &[1, 2, 3, 4, 5]);
}

/// A fixed-capacity buffer has no `From<&VecLike>` that can panic its way
/// through an overflow, so `TryFrom` is the whole conversion: too long is an
/// `Err` carrying the elements, not a panic and not a truncation.
#[test]
fn try_from_a_vec_like_never_truncates() {
    let mut wide = ArrayVec::<u8, 8>::new();
    wide.extend([1u8, 2, 3, 4, 5]);

    let ok = ArrayVec::<u8, 5>::try_from(&wide).expect("5 fits in 5");
    assert_eq!(&ok[..], &[1, 2, 3, 4, 5]);

    let too_long = ArrayVec::<u8, 3>::try_from(&wide).expect_err("5 does not fit in 3");
    assert_eq!(
        too_long,
        vec![1, 2, 3, 4, 5],
        "the elements come back whole"
    );
    assert_eq!(&wide[..], &[1, 2, 3, 4, 5], "the source is untouched");
}
