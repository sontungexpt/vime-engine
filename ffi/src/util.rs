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
}
