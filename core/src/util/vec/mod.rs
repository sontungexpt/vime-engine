//! Fixed-capacity and spilling vec containers.
//!
//! [`ArrayVec`] stores `N` elements inline and panics at capacity; [`SmallVec`]
//! spills to the heap instead; both satisfy [`VecLike`].

pub mod array_vec;
pub mod small_vec;
pub mod vec_like;

pub use array_vec::ArrayVec;
pub use small_vec::SmallVec;
pub use vec_like::VecLike;
