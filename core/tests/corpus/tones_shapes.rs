//! D. Tone + shape combinations.

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── shape then tone ──
    case!(['a', 'w', 's'], "ắ"),
    case!(['a', 'w', 'f'], "ằ"),
    case!(['a', 'w', 'r'], "ẳ"),
    case!(['a', 'w', 'x'], "ẵ"),
    case!(['a', 'w', 'j'], "ặ"),
    case!(['a', 'a', 's'], "ấ"),
    case!(['a', 'a', 'f'], "ầ"),
    case!(['a', 'a', 'r'], "ẩ"),
    case!(['a', 'a', 'x'], "ẫ"),
    case!(['a', 'a', 'j'], "ậ"),
    case!(['e', 'e', 's'], "ế"),
    case!(['e', 'e', 'f'], "ề"),
    case!(['e', 'e', 'r'], "ể"),
    case!(['e', 'e', 'x'], "ễ"),
    case!(['e', 'e', 'j'], "ệ"),
    case!(['o', 'o', 's'], "ố"),
    case!(['o', 'o', 'f'], "ồ"),
    case!(['o', 'o', 'r'], "ổ"),
    case!(['o', 'o', 'x'], "ỗ"),
    case!(['o', 'o', 'j'], "ộ"),
    case!(['o', 'w', 's'], "ớ"),
    case!(['o', 'w', 'f'], "ờ"),
    case!(['o', 'w', 'r'], "ở"),
    case!(['o', 'w', 'x'], "ỡ"),
    case!(['o', 'w', 'j'], "ợ"),
    case!(['u', 'w', 's'], "ứ"),
    case!(['u', 'w', 'f'], "ừ"),
    case!(['u', 'w', 'r'], "ử"),
    case!(['u', 'w', 'x'], "ữ"),
    case!(['u', 'w', 'j'], "ự"),
    // ── tone then shape: the shape applies, the tone is preserved ──
    case!(['a', 's', 'w'], "ắ"),
    case!(['a', 'f', 'w'], "ằ"),
    case!(['a', 's', 'a'], "ấ"),
    case!(['e', 's', 'e'], "ế"),
    case!(['o', 's', 'o'], "ố"),
    case!(['o', 's', 'w'], "ớ"),
    case!(['u', 's', 'w'], "ứ"),
    // ── shape then second tone replaces ──
    case!(['a', 'w', 's', 'f'], "ằ"),
    case!(['a', 'a', 's', 'r'], "ẩ"),
    // ── shapes + tones after an onset / with coda ──
    case!(['c', 'h', 'a', 'w', 'n', 's'], "chắn"),
    case!(['m', 'a', 'a', 'n', 's'], "mấn"),
    case!(['k', 'h', 'a', 'a', 's'], "khấ"),
    case!(['a', 'a', 'n', 's'], "ấn"),
    case!(['e', 'e', 'n', 'j'], "ện"),
    case!(['o', 'w', 'n', 'f'], "ờn"),
    // ── shape swap keeps tone ──
    case!(['ấ', 'w'], "ắ"),
    // ── expansion: tones on shaped nuclei with a following vowel ──
    case!(['e', 'e', 'u', 'f'], "ều"),
    case!(['o', 'o', 'i', 'f'], "ồi"),
    case!(['u', 'w', 'a', 'r'], "ửa"),
    // ── expansion: tones on shaped nuclei with a coda ──
    case!(['b', 'a', 'w', 'n', 's'], "bắn"),
    case!(['b', 'a', 'a', 'n', 'f'], "bần"),
    // ── expansion: tone + shape + coda syllables ──
    case!(['o', 'a', 'w', 't', 's'], "oắt"),
    case!(['l', 'o', 'a', 'w', 't', 's'], "loắt"),
    case!(['t', 'h', 'o', 'o', 'i', 's'], "thối"),
    case!(['d', 'd', 'a', 'a', 'u', 'r'], "đẩu"),
    case!(['c', 'u', 'o', 'o', 'n', 'j'], "cuộn"),
    case!(['b', 'a', 'a', 'y', 's'], "bấy"),
];