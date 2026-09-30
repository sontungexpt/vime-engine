//! Small utility containers used by the engine.
//!
//! Currently one family: the [`mod@vec`] module, holding the fixed-capacity
//! [`vec::ArrayVec`], the spilling [`vec::SmallVec`], and the [`vec::VecLike`]
//! interface the two share.

pub mod vec;
