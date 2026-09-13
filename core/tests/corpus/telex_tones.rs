//! B. Telex tone keys.

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── all five tones on single vowels ──
    // a
    case!(['a', 's'], "á"),
    case!(['a', 'f'], "à"),
    case!(['a', 'r'], "ả"),
    case!(['a', 'x'], "ã"),
    case!(['a', 'j'], "ạ"),
    // e
    case!(['e', 's'], "é"),
    case!(['e', 'f'], "è"),
    case!(['e', 'r'], "ẻ"),
    case!(['e', 'x'], "ẽ"),
    case!(['e', 'j'], "ẹ"),
    // i
    case!(['i', 's'], "í"),
    case!(['i', 'f'], "ì"),
    case!(['i', 'r'], "ỉ"),
    case!(['i', 'x'], "ĩ"),
    case!(['i', 'j'], "ị"),
    // o
    case!(['o', 's'], "ó"),
    case!(['o', 'f'], "ò"),
    case!(['o', 'r'], "ỏ"),
    case!(['o', 'x'], "õ"),
    case!(['o', 'j'], "ọ"),
    // u
    case!(['u', 's'], "ú"),
    case!(['u', 'f'], "ù"),
    case!(['u', 'r'], "ủ"),
    case!(['u', 'x'], "ũ"),
    case!(['u', 'j'], "ụ"),
    // y is a vowel; `yếu`-style spellings work (see KNOWN_DEVIATIONS for `nhà`)
    case!(['y', 's'], "ý"),
    case!(['y', 'f'], "ỳ"),
    case!(['y', 'r'], "ỷ"),
    // ── `z` resets an existing tone to flat ──
    case!(['á', 'z'], "a"),
    case!(['à', 'z'], "a"),
    // ── tone after an onset ──
    case!(['b', 'a', 's'], "bá"),
    case!(['c', 'h', 'a', 'f'], "chà"),
    case!(['g', 'i', 'a', 's'], "giá"),
    // ── tone after a coda ──
    case!(['a', 'n', 's'], "án"),
    case!(['b', 'a', 'c', 'j'], "bạc"),
    case!(['t', 'o', 'a', 'n', 'j'], "toạn"),
    case!(['a', 'm', 'f'], "àm"),
    case!(['a', 'p', 'j'], "ạp"),
    case!(['a', 't', 'j'], "ạt"),
    // ── tone on two-vowel nuclei (flat placement on the first) ──
    case!(['a', 'i', 's'], "ái"),
    case!(['o', 'i', 's'], "ói"),
    case!(['u', 'i', 's'], "úi"),
    case!(['a', 'u', 's'], "áu"),
    case!(['a', 'y', 's'], "áy"),
    case!(['u', 'a', 's'], "úa"),
    case!(['i', 'a', 's'], "ía"),
    case!(['o', 'i', 'r'], "ỏi"),
    case!(['a', 'y', 'r'], "ảy"),
    case!(['a', 'u', 'r'], "ảu"),
    case!(['a', 'i', 'x'], "ãi"),
    case!(['i', 'a', 'x'], "ĩa"),
    case!(['u', 'a', 'j'], "ụa"),
    case!(['b', 'u', 'i', 'j'], "bụi"),
    // ── `u y` places the tone on the first letter ──
    case!(['u', 'y', 's'], "úy"),
    case!(['u', 'y', 'f'], "ùy"),
    case!(['u', 'y', 'r'], "ủy"),
    case!(['u', 'y', 'x'], "ũy"),
    case!(['u', 'y', 'j'], "ụy"),
    // ── `oa` / `oe` place the tone on the second letter ──
    case!(['o', 'a', 's'], "oá"),
    case!(['o', 'e', 'f'], "oè"),
    case!(['x', 'o', 'e', 'f'], "xoè"),
    // ── tone replacement ──
    case!(['a', 's', 'f'], "à"),
    case!(['a', 'f', 'r'], "ả"),
    case!(['e', 's', 'r'], "ẻ"),
    case!(['o', 'f', 's'], "ó"),
    case!(['u', 's', 'j'], "ụ"),
    // ── expansion: remaining y tones ──
    case!(['y', 'x'], "ỹ"),
    case!(['y', 'j'], "ỵ"),
    // ── expansion: tones after digraph onsets ──
    case!(['p', 'h', 'a', 's'], "phá"),
    case!(['t', 'r', 'o', 'j'], "trọ"),
    case!(['k', 'h', 'a', 'f'], "khà"),
    // ── expansion: tone after a `nh` / `ng` coda ──
    case!(['b', 'a', 'n', 'h', 's'], "bánh"),
    // ── expansion: `oa` / `oe` tones (placed on the second letter) ──
    case!(['o', 'a', 'r'], "oả"),
    case!(['o', 'e', 'j'], "oẹ"),
    case!(['x', 'o', 'a', 'f'], "xoà"),
    // ── expansion: two-letter nucleus tones ──
    case!(['e', 'u', 'f'], "èu"),
    case!(['a', 'o', 'j'], "ạo"),
    // ── expansion: `iêu` / `yếu`-style three-vowel tones ──
    case!(['i', 'e', 'u', 'f'], "ièu"),
    // ── expansion: tones on plain vowel sequences ──
    case!(['b', 'a', 'y', 's'], "báy"),
    case!(['c', 'h', 'o', 'a', 'y', 'j'], "choạy"),
    case!(['m', 'ư', 'a', 'r'], "mửa"),
    case!(['n', 'ư', 'a', 'f'], "nừa"),
    case!(['o', 'a', 'n', 'f'], "oàn"),
    case!(['o', 'a', 's'], "oá"),
    case!(['i', 'ê', 'u', 's'], "iếu"),
    case!(['ư', 'ơ', 'n', 's'], "ướn"),
    case!(['b', 'o', 'a', 'y', 's'], "boáy"),
    case!(['t', 'i', 'u', 'f'], "tìu"),
];