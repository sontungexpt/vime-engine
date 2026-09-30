//! `SmallVec` behaviour: the inline fast path, the spill, and the `ArrayVec`
//! hand-off.
//!
//! The `ArrayVec` grow/consume paths are in `array_vec.rs`; this file covers the
//! container that wraps one and can outgrow it.

use vime_engine::util::vec::{ArrayVec, SmallVec, VecLike};

/// A `SmallVec` holding `items`, spilling only if they do not fit.
fn small<const N: usize>(items: &[u8]) -> SmallVec<u8, N> {
    items.into()
}

// ─────────────────────── inline until the capacity is passed ───────────────────────

/// The whole point: an inline `SmallVec` behaves like the `ArrayVec` it wraps,
/// and only the write that overflows changes anything.
#[test]
fn stays_inline_until_the_capacity_is_passed() {
    let mut v = small::<3>(&[1, 2]);
    assert!(!v.is_spilled());
    assert_eq!(v.len(), 2);
    assert_eq!(v.as_slice(), &[1, 2]);

    v.push(3);
    assert!(!v.is_spilled(), "the third element still fits");
    assert_eq!(v.as_slice(), &[1, 2, 3]);

    // The fourth does not fit, and this is the only write that pays.
    v.push(4);
    assert!(v.is_spilled());
    assert_eq!(v.as_slice(), &[1, 2, 3, 4]);
}

/// A spilled vector keeps working: push, pop, insert, remove and index all go
/// through to the `Vec`.
#[test]
fn a_spilled_vector_still_behaves() {
    let mut v = small::<2>(&[1, 2, 3]);
    assert!(v.is_spilled());

    v.push(4);
    assert_eq!(v.as_slice(), &[1, 2, 3, 4]);

    assert_eq!(v.pop(), Some(4));
    v.insert(0, 0);
    assert_eq!(v.as_slice(), &[0, 1, 2, 3]);
    assert_eq!(v.remove(0), 0);
    assert_eq!(v.as_slice(), &[1, 2, 3]);
    assert_eq!(v[1], 2);
    assert_eq!(v.len(), 3);
}

#[test]
fn capacity_reports_the_inline_room_then_the_heap() {
    let mut v = small::<3>(&[1, 2]);
    assert_eq!(v.capacity(), 3, "inline room is exactly N");

    v.push(3);
    v.push(4);
    assert!(v.is_spilled());
    // Twice the inline capacity, so the pushes after a spill do not realloc.
    assert!(v.capacity() >= 6, "got {}", v.capacity());
}

/// Popping back under the inline capacity does *not* unspill: the vector stays
/// on the heap, and `shrink_to_inline` is the explicit way back.
#[test]
fn popping_back_does_not_unspill() {
    let mut v = small::<2>(&[1, 2, 3]);
    assert!(v.is_spilled());

    assert_eq!(v.pop(), Some(3));
    assert_eq!(v.pop(), Some(2));
    assert_eq!(v.len(), 1);
    assert!(
        v.is_spilled(),
        "moving back would be a copy nobody asked for"
    );

    assert!(v.shrink_to_inline(), "one element fits in two");
    assert!(!v.is_spilled());
    assert_eq!(v.as_slice(), &[1]);
}

#[test]
fn shrink_refuses_while_still_too_long() {
    let mut v = small::<2>(&[1, 2, 3, 4]);
    assert!(!v.shrink_to_inline(), "four elements do not fit in two");
    assert!(v.is_spilled());
    assert_eq!(v.as_slice(), &[1, 2, 3, 4]);
}

#[test]
fn reset_returns_a_bare_inline_vector() {
    let mut v = small::<2>(&[1, 2, 3]);
    assert!(v.is_spilled());

    v.reset();
    assert!(!v.is_spilled());
    assert!(v.is_empty());
    assert_eq!(v.capacity(), 2);
}

/// `clear` keeps the heap buffer, so a refill after it does not allocate again.
#[test]
fn clear_keeps_the_heap_buffer() {
    let mut v = small::<2>(&[1, 2, 3]);
    let capacity_before = v.capacity();

    v.clear();
    assert!(v.is_empty());
    assert!(v.is_spilled(), "the buffer is kept, only the length drops");
    assert_eq!(v.capacity(), capacity_before);

    v.push(9);
    assert_eq!(v.as_slice(), &[9]);
}

// ────────────────────────────── ArrayVec interop ──────────────────────────────

/// The inline path: an `ArrayVec` that fits is adopted with no allocation.
#[test]
fn adopting_an_array_stays_inline_when_it_fits() {
    let mut src = ArrayVec::<u8, 8>::default();
    src.extend_from_slice(&[1, 2, 3]);

    let v: SmallVec<u8, 4> = SmallVec::from(&src);
    assert!(!v.is_spilled(), "three elements fit in four");
    assert_eq!(v.as_slice(), &[1, 2, 3]);
}

/// The overflowing path: too long for the inline buffer, so it spills.
#[test]
fn adopting_an_array_spills_when_too_long() {
    let mut src = ArrayVec::<u8, 8>::default();
    src.extend_from_slice(&[1, 2, 3, 4, 5]);

    let v: SmallVec<u8, 4> = SmallVec::from(&src);
    assert!(v.is_spilled());
    assert_eq!(v.as_slice(), &[1, 2, 3, 4, 5]);
}

/// A `SmallVec` and the `ArrayVec` it came from agree at every length, whichever
/// side of the boundary each falls on.
#[test]
fn adopting_an_array_agrees_with_its_source() {
    for len in 0..=8usize {
        let mut src = ArrayVec::<u8, 8>::default();
        src.extend_from_slice(&vec![7u8; len]);

        let v: SmallVec<u8, 4> = SmallVec::from(&src);
        assert_eq!(v.as_slice(), src.as_slice(), "len {len}");
        assert_eq!(v.is_spilled(), len > 4, "len {len}");
    }
}

// ─────────────────────────────── the arm is internal ───────────────────────────────

/// The phase enum is crate-local, so an out-of-crate caller asks with
/// `is_spilled` instead. This is that public surface standing in for it.
#[test]
fn the_arm_is_reported_through_is_spilled() {
    let inline = small::<4>(&[1, 2]);
    assert!(!inline.is_spilled());

    let spilled = small::<1>(&[1, 2]);
    assert!(spilled.is_spilled());
}

// ───────────────────────── generic behaviour over the trait ─────────────────────────

/// A `ramp` of `n` elements, so each case states only its length.
fn ramp<const N: usize>(n: usize) -> ArrayVec<u8, N> {
    let mut v = ArrayVec::<u8, N>::default();
    for i in 0..n {
        v.push(i as u8);
    }
    v
}

/// Every assertion below runs through [`VecLike`] rather than the concrete
/// type, which is the point of the trait: identical behaviour whichever arm is
/// underneath.
fn behaves_like_a_vec<V: VecLike<u8>>(mut v: V) {
    assert_eq!(v.len(), 2);
    assert!(!v.is_empty());
    assert_eq!(v.as_slice(), &[0, 1]);

    v.push(2);
    assert_eq!(v.as_slice(), &[0, 1, 2]);

    assert_eq!(v.pop(), Some(2));
    assert_eq!(v.len(), 2);

    v.insert(0, 9);
    assert_eq!(v.as_slice(), &[9, 0, 1]);
    assert_eq!(v.remove(0), 9);
    assert_eq!(v.as_slice(), &[0, 1]);

    // Mutable slice access reaches the live elements in place.
    v.as_mut_slice()[1] = 42;
    assert_eq!(v.as_slice()[1], 42);

    v.clear();
    assert!(v.is_empty());

    v.extend_from_slice(&[5, 6]);
    assert_eq!(v.as_slice(), &[5, 6]);
}

#[test]
fn the_trait_agrees_across_both_containers() {
    behaves_like_a_vec(ramp::<4>(2));
    // The same two elements, in the other container, so the assertions below
    // are running twice over identical input.
    behaves_like_a_vec(SmallVec::<u8, 4>::from(&[0u8, 1][..]));
}

/// A spilled container still satisfies the trait, which is what lets caller code
/// be written once and work either way.
#[test]
fn the_trait_holds_for_a_spilled_container() {
    let mut v = SmallVec::<u8, 2>::from(&[0u8, 1, 2][..]);
    assert!(v.is_spilled());
    assert_eq!(v.len(), 3);
    assert_eq!(v.as_slice(), &[0, 1, 2]);
    v.push(3);
    assert_eq!(v.as_slice(), &[0, 1, 2, 3]);
}

// ─────────────────────────────── the `From` impls ───────────────────────────────

/// Each conversion reaches the same inline-or-spill decision the constructors
/// did, now through `.into()` rather than a named `from_*` method.
#[test]
fn conversions_go_through_into() {
    // slice, inline and spilled
    let inline: SmallVec<u8, 4> = (&[1u8, 2][..]).into();
    assert!(!inline.is_spilled());
    assert_eq!(inline.as_slice(), &[1, 2]);

    let spilled: SmallVec<u8, 2> = (&[1u8, 2, 3][..]).into();
    assert!(spilled.is_spilled());
    assert_eq!(spilled.as_slice(), &[1, 2, 3]);

    // array, whatever its length
    let from_an_array: SmallVec<u8, 4> = [7u8, 8].into();
    assert_eq!(from_an_array.as_slice(), &[7, 8]);

    // ArrayVec of a *different* inline capacity
    let mut av = ArrayVec::<u8, 8>::default();
    av.extend_from_slice(&[1, 2, 3]);
    let adopted: SmallVec<u8, 4> = SmallVec::from(&av);
    assert!(!adopted.is_spilled());
    assert_eq!(adopted.as_slice(), av.as_slice());

    // Vec, both ways
    let short: SmallVec<u8, 4> = Vec::from([1u8, 2]).into();
    assert!(!short.is_spilled(), "two elements fit inline");
    assert_eq!(short.as_slice(), &[1, 2]);

    let long: SmallVec<u8, 2> = Vec::from([1u8, 2, 3, 4]).into();
    assert!(long.is_spilled());
    assert_eq!(long.as_slice(), &[1, 2, 3, 4]);

    // another SmallVec, by reference
    let copy: SmallVec<u8, 4> = SmallVec::from(&av);
    assert_eq!(copy, adopted);
}

/// A `Vec` with spare capacity stays on the heap rather than having the slack
/// thrown away for an inline buffer sized to the live elements.
#[test]
fn a_vec_with_spare_capacity_stays_spilled() {
    let mut with_room = Vec::with_capacity(64);
    with_room.extend([1u8, 2]);
    assert!(with_room.capacity() > with_room.len());

    let v: SmallVec<u8, 8> = with_room.into();
    assert!(
        v.is_spilled(),
        "discarding 62 slots of room is not a saving"
    );
    assert_eq!(v.as_slice(), &[1, 2]);
}

// ───────────────────────────── std integrations ─────────────────────────────

#[test]
fn equality_ignores_which_arm_holds_the_elements() {
    let mut inline = SmallVec::<u8, 4>::default();
    inline.extend_from_slice(&[1, 2, 3]);

    let mut spilled = SmallVec::<u8, 4>::default();
    spilled.extend_from_slice(&[1, 2, 3, 4, 5]);
    spilled.pop();
    spilled.pop();

    assert!(!inline.is_spilled());
    assert!(spilled.is_spilled());
    assert_eq!(inline, spilled, "the arm is not part of the value");
}

#[test]
fn clone_preserves_the_arm_and_the_contents() {
    let inline = small::<4>(&[1, 2]);
    let inline_copy = inline.clone();
    assert!(!inline_copy.is_spilled());
    assert_eq!(inline_copy, inline);

    let spilled = small::<1>(&[1, 2, 3]);
    let spilled_copy = spilled.clone();
    assert!(spilled_copy.is_spilled());
    assert_eq!(spilled_copy, spilled);
}

#[test]
fn debug_and_iteration_read_as_a_list() {
    let v = small::<4>(&[1, 2, 3]);
    assert_eq!(format!("{v:?}"), "[1, 2, 3]");
    assert_eq!(v.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);
    assert_eq!(v.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
}

#[test]
fn collect_uses_the_heap_only_when_it_has_to() {
    // An array iterator knows its length, so the size hint is exact.
    let inline: SmallVec<u8, 8> = [1, 2, 3].into_iter().collect();
    assert!(!inline.is_spilled());

    let spilled: SmallVec<u8, 2> = [1, 2, 3].into_iter().collect();
    assert!(spilled.is_spilled());
    assert_eq!(spilled.as_slice(), &[1, 2, 3]);
}

#[test]
fn extend_reserves_up_front_when_it_knows_the_size() {
    let mut v = small::<2>(&[]);
    v.extend(0u8..6);
    assert!(v.is_spilled());
    assert_eq!(v.as_slice(), &[0, 1, 2, 3, 4, 5]);
}

#[test]
#[should_panic(expected = "out of bounds")]
fn indexing_past_the_end_panics() {
    let v = small::<4>(&[1, 2]);
    let _ = v[9];
}

// ─────────────────────────────── construction ───────────────────────────────

/// `new` and `default` are the same empty inline vector, and `new` is usable in
/// a `const`, so a buffer can live in a static rather than being built at runtime.
#[test]
fn new_and_default_agree() {
    let a: SmallVec<u8, 4> = SmallVec::new();
    let b: SmallVec<u8, 4> = SmallVec::default();
    assert_eq!(a, b);
    assert!(a.is_empty());
    assert_eq!(a.len(), 0);
    assert!(!a.is_spilled(), "an empty vector is always inline");
    assert_eq!(a.as_slice(), &[] as &[u8]);
}

/// A `static` buffer, which only compiles if `new` is `const`.
#[test]
fn new_is_usable_in_a_static() {
    static EMPTY: SmallVec<u8, 4> = SmallVec::new();
    assert!(EMPTY.is_empty());
    assert_eq!(EMPTY.as_slice(), &[] as &[u8]);
}

/// `new` starts inline, so the first `N` pushes never allocate.
#[test]
fn a_new_vector_takes_the_inline_path() {
    let mut v: SmallVec<u8, 3> = SmallVec::new();
    for i in 0..3u8 {
        v.push(i);
        assert!(!v.is_spilled(), "push {i} must stay inline");
    }
    v.push(3);
    assert!(v.is_spilled(), "and the fourth spills");
    assert_eq!(v.as_slice(), &[0, 1, 2, 3]);
}

// ────────────────── indexing and slicing through the trait ──────────────────
//
// The supertraits are what make a `V: VecLike<T>` usable as a slice without
// ceremony. These run the same code against both containers, so neither can
// quietly lose the ability.

/// Indexing, index-assignment and slicing all work on a bare `V: VecLike<T>`.
#[test]
fn a_vec_like_can_be_indexed_and_sliced() {
    fn exercise<V: VecLike<u8>>(mut v: V) -> (u8, u8, Vec<u8>, usize) {
        let first = v[0]; // Index, via the supertrait
        v[1] = 20; // IndexMut, via the supertrait
                   // A range goes through the deref, because the supertrait pins
                   // `Index<usize>`: `(*v)[a..b]` is a slice range, not a container index.
        let tail: Vec<u8> = (*v)[1..].to_vec();
        let summed: usize = v.iter().map(|&x| x as usize).sum(); // Deref
        (first, v[1], tail, summed)
    }

    let mut array = ArrayVec::<u8, 4>::new();
    array.extend([1u8, 2, 3]);
    let (first, second, tail, sum) = exercise(array);
    assert_eq!(first, 1);
    assert_eq!(second, 20, "the write went through IndexMut");
    assert_eq!(tail, vec![20, 3]);
    assert_eq!(sum, 1 + 20 + 3);

    let small = SmallVec::<u8, 4>::from([1u8, 2, 3]);
    let (first, second, tail, sum) = exercise(small);
    assert_eq!((first, second, tail, sum), (1, 20, vec![20, 3], 24));
}

/// The same holds for a spilled one, where the elements are on the heap.
#[test]
fn a_spilled_vec_like_still_indexes() {
    let mut v = SmallVec::<u8, 2>::from([7u8, 8, 9]);
    assert!(v.is_spilled(), "three elements into a capacity of two");
    v[0] = 70;
    assert_eq!(v[0], 70);
    assert_eq!((*v), [70, 8, 9]);
    assert_eq!(v.iter().sum::<u8>(), 70 + 8 + 9);
}

/// `DerefMut` gives a mutable slice, so slice methods come along for free.
#[test]
fn a_vec_like_yields_a_mutable_slice() {
    fn double_all<V: VecLike<u8>>(v: &mut V) {
        for x in v.iter_mut() {
            *x *= 2;
        }
        v.reverse(); // a slice method, through the supertrait
    }

    let mut array = ArrayVec::<u8, 3>::from([1u8, 2, 3]);
    double_all(&mut array);
    assert_eq!((*array), [6, 4, 2]);

    let mut small = SmallVec::<u8, 3>::from([1u8, 2, 3]);
    double_all(&mut small);
    assert_eq!((*small), [6, 4, 2]);
}

/// `Deref` is *not* named in the `VecLike` supertrait list, because
/// `DerefMut: Deref` — requiring `DerefMut<Target = [T]>` already implies
/// `Deref<Target = [T]>`. This test would not compile if that were not so.
#[test]
fn the_trait_bound_alone_gives_deref() {
    fn as_slice_through_deref<V: VecLike<u8>>(v: &V) -> &[u8] {
        v
    }
    fn as_mut_slice_through_deref<V: VecLike<u8>>(v: &mut V) -> &mut [u8] {
        v
    }

    let mut v: SmallVec<u8, 4> = SmallVec::from([1u8, 2, 3]);
    let seen: &[u8] = as_slice_through_deref(&v);
    assert_eq!(seen, &[1, 2, 3], "Deref reached the slice");

    as_mut_slice_through_deref(&mut v)[0] = 9;
    assert_eq!(v[0], 9, "DerefMut reached the slice");
}
