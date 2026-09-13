//! H. Uppercase input.

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── uppercase letters ──
    case!(['A'], "A"),
    case!(['E'], "E"),
    case!(['I'], "I"),
    case!(['O'], "O"),
    case!(['U'], "U"),
    case!(['Y'], "Y"),
    case!(['B', 'A'], "BA"),
    case!(['C', 'H', 'A'], "CHA"),
    case!(['D', 'D', 'A'], "ĐA"),
    case!(['Q', 'U', 'A'], "QUA"),
    case!(['B', 'A', 'N'], "BAN"),
    case!(['C', 'H', 'U', 'Y'], "CHUY"),
    // ── shapes ──
    case!(['A', 'W'], "Ă"),
    case!(['A', 'A'], "Â"),
    case!(['E', 'E'], "Ê"),
    case!(['O', 'O'], "Ô"),
    case!(['O', 'W'], "Ơ"),
    case!(['U', 'W'], "Ư"),
    // ── shapes after a lowercase vowel keep lowercase; here uppercase → uppercase ──
    case!(['A', 'W', 'S'], "Ắ"),
    case!(['A', 'A', 'S'], "Ấ"),
    case!(['U', 'W', 'S'], "Ứ"),
    case!(['O', 'W', 'S'], "Ớ"),
    case!(['A', 'W', 'R'], "Ẳ"),
    case!(['A', 'A', 'X'], "Ẫ"),
    case!(['O', 'W', 'R'], "Ở"),
    case!(['O', 'W', 'X'], "Ỡ"),
    case!(['U', 'W', 'F'], "Ừ"),
    case!(['U', 'W', 'J'], "Ự"),
    // ── tones on uppercase vowels ──
    case!(['A', 'S'], "Á"),
    case!(['A', 'F'], "À"),
    case!(['A', 'R'], "Ả"),
    case!(['A', 'X'], "Ã"),
    case!(['A', 'J'], "Ạ"),
    case!(['E', 'F'], "È"),
    case!(['U', 'S'], "Ú"),
    case!(['E', 'R'], "Ẻ"),
    case!(['O', 'O', 'F'], "Ồ"),
    case!(['O', 'O', 'X'], "Ỗ"),
    case!(['Y', 'F'], "Ỳ"),
    // uppercase `Y` is a vowel, tone applies to it
    case!(['Y', 'S'], "Ý"),
    // ── uppercase precomposed stays uppercase ──
    case!(['Ắ'], "Ắ"),
    case!(['Ằ'], "Ằ"),
    case!(['Ấ'], "Ấ"),
    case!(['Ệ'], "Ệ"),
    case!(['Đ'], "Đ"),
    case!(['Ầ'], "Ầ"),
    case!(['Ị'], "Ị"),
    case!(['Ử'], "Ử"),
    case!(['Ỗ'], "Ỗ"),
    // ── uppercase precomposed transforms ──
    case!(['Ắ', 'f'], "Ằ"),
    case!(['Ắ', 's'], "Ăs"),
    case!(['Ắ', 'w'], "Áw"),
    case!(['Ấ', 'f'], "Ầ"),
    case!(['Ạ', 'w'], "Ặ"),
    // ── expansion: uppercase digraph onsets ──
    case!(['D', 'D', 'I'], "ĐI"),
    case!(['B', 'A', 'Y'], "BAY"),
    case!(['T', 'R', 'A'], "TRA"),
    case!(['N', 'H', 'A'], "NHA"),
    // ── expansion: uppercase shapes + vowel ──
    case!(['Y', 'E', 'E', 'U'], "YÊU"),
    // ── expansion: uppercase real syllables ──
    case!(['N', 'G', 'U', 'O', 'W', 'I', 'F'], "NGƯỜI"),
    case!(['T', 'R', 'U', 'O', 'W', 'N', 'G'], "TRƯƠNG"),
    case!(['V', 'I', 'E', 'E', 'T', 'S'], "VIẾT"),
    case!(['D', 'D', 'U', 'O', 'W', 'C', 'J'], "ĐƯỢC"),
    case!(['Q', 'U', 'A', 'S'], "QUÁ"),
    case!(['K', 'H', 'O', 'O', 'I'], "KHÔI"),
    case!(['M', 'A', 'A', 'Y'], "MÂY"),
    case!(['C', 'A', 'A', 'Y'], "CÂY"),
    case!(['D', 'D', 'A', 'A', 'S', 'Y'], "ĐẤY"),
];