use std::ffi::c_char;

/// Returns `(byte offset of `caret`-th char, char count)`. Caret past end runs off to text length.
#[inline]
pub(crate) fn measure(text: &str, caret_chars: usize) -> (usize, usize) {
    let mut byte_idx = 0;
    let mut len_chars = 0;
    for ch in text.chars() {
        if len_chars < caret_chars { byte_idx += ch.len_utf8(); }
        len_chars += 1;
    }
    (byte_idx, len_chars)
}
