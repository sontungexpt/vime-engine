//! Fixed-capacity and spilling vec containers.
//!
//! Two types over one interface:
//!
//! - [`ArrayVec`] — every element inside the struct, no allocation, and a panic
//!   at its fixed capacity `N`.
//! - [`SmallVec`] — the same inline buffer, but it spills to the heap instead of
//!   panicking, for a caller that does not know the bound in advance.
//!
//! [`VecLike`] is the interface both satisfy, so code that only needs "a bounded
//! sequence of `T`" does not have to name either one.

pub mod array_vec;
pub mod small_vec;
pub mod vec_like;

pub use array_vec::ArrayVec;
pub use small_vec::SmallVec;
pub use vec_like::VecLike;
