//! The render-state snapshot the header hands to a frontend, and the one place
//! text is measured.
//!
//! # Why the byte offsets are computed here
//!
//! A frontend integrating at the text level needs four positions, and the header
//! asks for all of them: a character index and a byte index for each of the two
//! buffers. Only the core knows the *character* positions; the byte positions
//! have to come from walking the rendered UTF-8. Walking it once, in a single
//! pass that both sums `len_utf8` up to the caret and counts the characters, is
//! what keeps [`measure`] from being four separate scans.
//!
//! A Vietnamese word is a handful of characters and Vietnamese characters are
//! two or three bytes, so this is a handful of instructions either way. The point
//! is that it is done once, at the point where the text is already in hand, and
//! never repeated for a second caller who wants a different index.
//!
//! # What the snapshot does not claim
//!
//! The `*_to_delete` fields are the only part of the ABI that is about the *host*
//! rather than about the engine, and they are the only part with a convention
//! attached. See [`VimeRenderState::bytes_to_delete`].

use std::ffi::c_char;

/// Byte and character offsets of one caret, plus the text's own length.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub(crate) struct Measured {
    /// UTF-8 byte offset of the caret from the start of the text.
    pub(crate) byte_idx: usize,
    /// The text's length in UTF-8 bytes.
    pub(crate) len_bytes: usize,
    /// The text's length in Unicode characters.
    pub(crate) len_chars: usize,
}

/// Measures a rendered buffer in one pass, getting all three numbers at once.
///
/// `text` is the content only — the caller appends the NUL terminator
/// afterwards, so a length here is a length a host can act on rather than one
/// that includes a byte only the C side can see.
///
/// `caret_chars` is a character index. A caret past the end — which the core never
/// produces, but which costs nothing to survive — simply runs off the end and
/// lands on the text's full length, rather than panicking between a C caller and
/// a `char *` that is about to be handed out.
#[inline]
pub(crate) fn measure(text: &str, caret_chars: usize) -> Measured {
    let mut byte_idx = 0;
    let mut len_chars = 0;
    for ch in text.chars() {
        if len_chars < caret_chars {
            byte_idx += ch.len_utf8();
        }
        len_chars += 1;
    }
    Measured {
        byte_idx,
        len_bytes: text.len(),
        len_chars,
    }
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
        assert_eq!(
            measure("ương", 4),
            Measured {
                byte_idx: 6,
                len_bytes: 6,
                len_chars: 4
            }
        );

        // The caret after the first character: 2 bytes in, because 'ư' is two.
        assert_eq!(
            measure("ương", 1),
            Measured {
                byte_idx: 2,
                len_bytes: 6,
                len_chars: 4
            }
        );

        // "hoá" — 'á' is two bytes, so the end is 4 bytes.
        assert_eq!(
            measure("hoá", 3),
            Measured {
                byte_idx: 4,
                len_bytes: 4,
                len_chars: 3
            }
        );

        // ASCII: the two units agree, which is why this case is not interesting
        // and is only here to show the arithmetic does not drift.
        assert_eq!(
            measure("toan", 4),
            Measured {
                byte_idx: 4,
                len_bytes: 4,
                len_chars: 4
            }
        );
    }

    #[test]
    fn an_empty_word_measures_to_zero() {
        assert_eq!(
            measure("", 0),
            Measured {
                byte_idx: 0,
                len_bytes: 0,
                len_chars: 0
            }
        );
    }

    /// The core never produces an out-of-range caret, but a clamp here is free
    /// and beats a panic between a C caller and a pointer it is about to return.
    #[test]
    fn a_caret_past_the_end_clamps() {
        assert_eq!(
            measure("ab", 99),
            Measured {
                byte_idx: 2,
                len_bytes: 2,
                len_chars: 2
            }
        );
    }

    /// Every byte offset has to land on a character boundary, or the frontend
    /// slices UTF-8 in half.
    #[test]
    fn every_measured_offset_is_a_character_boundary() {
        for text in ["", "a", "ă", "ương", "tiếng", "đường", "\u{1f600}ă"] {
            for caret in 0..=text.chars().count() {
                let m = measure(text, caret);
                assert!(
                    text.is_char_boundary(m.byte_idx),
                    "{text:?} caret {caret} -> {} is not a boundary",
                    m.byte_idx,
                );
                assert!(m.byte_idx <= text.len());
            }
        }
    }
}
