//! `InlineVec` grow-path tests.
//!
//! The inline buffer is the allocation-free backing for a syllable's onset,
//! nucleus and coda. These tests exercise the public insert surface only:
//!
//! - `extend`/`from_iter` growing within capacity
//! - overflow panics that must not mutate the buffer
//! - `extend_from_slice` including an empty continuation

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

    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| v.extend([3]))).is_err()
    );
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
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| v.extend_from_slice(&[1, 2])))
            .is_err()
    );
    assert_eq!(&v[..], &[], "failed extend must not mutate");
}

#[test]
fn from_iter_collects() {
    let v: InlineVec<u8, 3> = [7, 8].into_iter().collect();
    assert_eq!(&v[..], &[7, 8]);
}