//! The snapshot the header hands to a frontend, and the one walk that turns the
//! core's character positions into byte positions.
//!
//! # Why the byte offsets are computed here
//!
//! The header asks for a character index *and* a byte index for each of the two
//! strings. The core owns the character indices and keeps no byte ones, because
//! nothing inside the engine indexes a buffer by byte. So the FFI derives them
//! from bytes it has already written, in a single walk that sums `len_utf8` up to
//! the caret while it counts the characters.
//!
//! Both numbers come out of that one walk because [`VimeRenderState`] needs both.
//! A caller who wants only the caret throws the count away. A Vietnamese word is a
//! handful of characters, so the walk is a handful of instructions — the point is
//! that it happens on demand, next to text already in hand, instead of being
//! cached in a field that has to be kept in step with the string.
//!
//! # What the snapshot does not claim
//!
//! The `*_to_delete` fields are the only part of the ABI that is about the *host*
//! rather than the engine, and the only part with a convention attached. See
//! [`VimeRenderState::bytes_to_delete`].

use std::ffi::c_char;

/// Byte offset of a caret, and the character count of the same string.
///
/// Returns `(byte offset of the `caret`-th character, number of characters)`.
///
/// A caret past the end — which the core never produces, but which costs nothing
/// to survive — simply runs off the end and lands on the text's full length,
/// rather than panicking between a C caller and a `char *` that is about to be
/// handed out.
#[inline]
pub(crate) fn measure(text: &str, caret_chars: usize) -> (usize, usize) {
    let mut byte_idx = 0;
    let mut len_chars = 0;
    for ch in text.chars() {
        if len_chars < caret_chars {
            byte_idx += ch.len_utf8();
        }
        len_chars += 1;
    }
    (byte_idx, len_chars)
}
