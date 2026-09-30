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

/// Detailed render state snapshot for UI, uinput, and IME frameworks.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VimeRenderState {
    /// Transformed Vietnamese UTF-8 text (e.g., "viê")
    pub text: *const c_char,
    /// Raw UTF-8 key sequence entered by user (e.g., "viee")
    pub raw_text: *const c_char,

    /// Rendered cursor index in Bytes
    pub cursor_byte_idx: usize,
    /// Rendered cursor index in CodePoints (Chars)
    pub cursor_char_idx: usize,

    /// Raw cursor index in Bytes
    pub raw_cursor_byte_idx: usize,
    /// Raw cursor index in CodePoints (Chars)
    pub raw_cursor_char_idx: usize,

    /// Number of UTF-8 bytes to remove from previous render
    pub bytes_to_delete: usize,
    /// Number of CodePoints (Backspaces) to delete from host buffer
    pub chars_to_delete: usize,

    /// True if current buffer forms a valid Vietnamese word
    pub is_valid_vietnamese: bool,
}

// ─────────────────────────── ABI layout locks ───────────────────────────
//
// See `config.rs` for why these exist and which C sizes they assume. The layout
// here is two pointers, six `size_t`s and a `bool`:
//
//   0  text                 8
//   8  raw_text             8
//  16  cursor_byte_idx      8
//  24  cursor_char_idx      8
//  32  raw_cursor_byte_idx  8
//  40  raw_cursor_char_idx  8
//  48  bytes_to_delete      8
//  56  chars_to_delete      8
//  64  is_valid_vietnamese 1
//  72  total (8-byte aligned)

const _: () = {
    use core::mem::{align_of, offset_of, size_of};

    assert!(size_of::<VimeRenderState>() == 72);
    assert!(align_of::<VimeRenderState>() == 8);
    assert!(offset_of!(VimeRenderState, text) == 0);
    assert!(offset_of!(VimeRenderState, raw_text) == 8);
    assert!(offset_of!(VimeRenderState, cursor_byte_idx) == 16);
    assert!(offset_of!(VimeRenderState, cursor_char_idx) == 24);
    assert!(offset_of!(VimeRenderState, raw_cursor_byte_idx) == 32);
    assert!(offset_of!(VimeRenderState, raw_cursor_char_idx) == 40);
    assert!(offset_of!(VimeRenderState, bytes_to_delete) == 48);
    assert!(offset_of!(VimeRenderState, chars_to_delete) == 56);
    assert!(offset_of!(VimeRenderState, is_valid_vietnamese) == 64);
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The rendered and raw texts are multi-byte, which is the whole reason byte
    /// and character offsets are reported separately. All three numbers describe
    /// the content, with no terminator in them.
    #[test]
    fn a_multi_byte_word_measures_in_both_units() {
        // "ương" is 4 characters in 6 bytes; the caret is at the end.
        assert_eq!(measure("ương", 4), (6, 4));

        // The caret after the first character: 2 bytes in, because 'ư' is two.
        assert_eq!(measure("ương", 1), (2, 4));

        // "hoá" — 'á' is two bytes, so the end is 4 bytes.
        assert_eq!(measure("hoá", 3), (4, 3));

        // ASCII: the two units agree, which is why this case is not interesting
        // and is only here to show the arithmetic does not drift.
        assert_eq!(measure("toan", 4), (4, 4));
    }

    #[test]
    fn an_empty_word_measures_to_zero() {
        assert_eq!(measure("", 0), (0, 0));
    }

    /// The core never produces an out-of-range caret, but a clamp here is free
    /// and beats a panic between a C caller and a pointer it is about to return.
    #[test]
    fn a_caret_past_the_end_clamps() {
        assert_eq!(measure("ab", 99), (2, 2));
    }

    /// Every byte offset has to land on a character boundary, or the frontend
    /// slices UTF-8 in half.
    #[test]
    fn every_measured_offset_is_a_character_boundary() {
        for text in ["", "a", "ă", "ương", "tiếng", "đường", "\u{1f600}ă"] {
            for caret in 0..=text.chars().count() {
                let (byte_idx, len_chars) = measure(text, caret);
                assert!(
                    text.is_char_boundary(byte_idx),
                    "{text:?} caret {caret} -> {byte_idx} is not a boundary",
                );
                assert!(byte_idx <= text.len());
                assert_eq!(len_chars, text.chars().count());
            }
        }
    }
}
