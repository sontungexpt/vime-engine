//! F2. The VIQr layout.
//!
//! VIQr layout (config/rule_engine/default/viqr.rs):
//!   tones `` ` `` (grave), `?` (hook), `~` (tilde), `'` (acute), `.` (dot),
//!   `z` (flat reset)
//!   shapes `^` = circumflex (a e o), `(` = breve (a), `+` = horn (o u)
//!   stroke `d` = d → đ

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── tones ──
    case!(['a', '\''], "á"),
    case!(['a', '`'], "à"),
    case!(['a', '?'], "ả"),
    case!(['a', '~'], "ã"),
    case!(['a', '.'], "ạ"),
    case!(['e', '\''], "é"),
    case!(['e', '`'], "è"),
    case!(['o', '?'], "ỏ"),
    case!(['u', '~'], "ũ"),
    case!(['i', '.'], "ị"),
    case!(['y', '?'], "ỷ"),
    case!(['y', '`'], "ỳ"),
    // ── tone reset with `z` ──
    case!(['á', 'z'], "a"),
    case!(['ấ', 'z'], "â"),
    // ── tone replacement ──
    case!(['a', '\'', '`'], "à"),
    // ── tone after a coda ──
    case!(['a', 'n', '\''], "án"),
    case!(['a', 'm', '`'], "àm"),
    case!(['a', 't', '.'], "ạt"),
    // ── shapes ──
    case!(['a', '^'], "â"),
    case!(['a', '('], "ă"),
    case!(['e', '^'], "ê"),
    case!(['o', '^'], "ô"),
    case!(['o', '+'], "ơ"),
    case!(['u', '+'], "ư"),
    // ── shape then tone ──
    case!(['a', '^', '\''], "ấ"),
    case!(['a', '^', '?'], "ẩ"),
    case!(['a', '(', '\''], "ắ"),
    case!(['a', '(', '.'], "ặ"),
    case!(['o', '^', '.'], "ộ"),
    case!(['o', '+', '\''], "ớ"),
    case!(['o', '+', '?'], "ở"),
    case!(['u', '+', '\''], "ứ"),
    case!(['u', '+', '~'], "ữ"),
    // ── tone then shape keeps the tone ──
    case!(['a', '\'', '^'], "ấ"),
    // ── `d` stroke ──
    case!(['d', 'd'], "đ"),
    case!(['d', 'd', 'a'], "đa"),
    case!(['d', 'd', 'a', '\''], "đá"),
    case!(['D', 'd'], "Đ"),
    // ── qu ──
    case!(['q', 'u', 'a'], "qua"),
    case!(['q', 'u', 'a', '\''], "quá"),
    case!(['q', 'u', 'y'], "quy"),
    // ── uo / ươ ──
    case!(['u', 'o', '+'], "uơ"),
    case!(['u', 'o', '+', 'i'], "ươi"),
    case!(['u', 'o', '+', 'i', '\''], "ưới"),
    case!(['u', 'o', '^', 'i'], "uôi"),
    // ── precomposed vowel sequences ──
    case!(['ư', 'a'], "ưa"),
    case!(['t', 'h', 'ư', 'a', '?'], "thửa"),
    // ── real syllables ──
    case!(['n', 'g', 'u', 'o', '+', 'i', '`'], "người"),
    case!(['n', 'u', 'o', '+', 'c', '\''], "nước"),
    case!(['d', 'd', 'u', 'o', '+', 'c', '.'], "được"),
    case!(['q', 'u', 'y', 'e', '^', 't', '\''], "quyết"),
    case!(['h', 'o', 'i', '?'], "hỏi"),
    case!(['d', 'd', 'e', '^', 'n', '\''], "đến"),
    case!(['v', 'i', 'e', '^', 't', '.'], "việt"),
    case!(['t', 'h', 'a', 'n', 'h', '`'], "thành"),
    case!(['s', 'o', 'n', 'g'], "song"),
    case!(['x', 'i', 'n', 'h'], "xinh"),
    case!(['t', 'h', 'u', 'e', '^', '\''], "thuế"),
    case!(['t', 'h', 'u', 'e', '^'], "thuê"),
    case!(['h', 'o', 'a', '(', 'c', '.'], "hoặc"),
    case!(['k', 'h', 'o', 'e', '?'], "khoẻ"),
    // ── expansion: d-stroke + a following vowel ──
    case!(['d', 'd', 'i'], "đi"),
    // ── expansion: quê / yêu / mưa shapes ──
    case!(['q', 'u', 'e', '^'], "quê"),
    case!(['y', 'e', '^', 'u'], "yêu"),
    case!(['m', 'u', '+', 'a'], "mưa"),
    // ── expansion: real syllables ──
    case!(['h', 'u', '+', 'n', 'g'], "hưng"),
    case!(['a', 'n', 'h', '`'], "ành"),
    case!(['b', 'a', 'n', '?'], "bản"),
    // ── expansion: more real VIQr syllables ──
    case!(['m', 'u', 'o', '+', 'i', '`'], "mười"),
    case!(['c', 'u', 'o', '+', 'i', '`'], "cười"),
    case!(['k', 'h', 'o', '^', 'n', 'g'], "không"),
    case!(['h', 'o', '^', 'n', 'g', '`'], "hồng"),
    case!(['s', 'o', '^', 'n', 'g', '\''], "sống"),
    case!(['m', 'a', 'n', 'h', '.'], "mạnh"),
    case!(['x', 'a', 'n', 'h'], "xanh"),
    case!(['n', 'h', 'a', '`'], "nhà"),
    case!(['b', 'a', '`'], "bà"),
    case!(['a', '^', 'y', '\''], "ấy"),
    case!(['d', 'a', '^', 'y'], "dây"),
    case!(['m', 'a', '^', 'y'], "mây"),
    case!(['c', 'a', '^', 'y'], "cây"),
    case!(['k', 'h', 'u', 'y', 'a'], "khuya"),
    case!(['c', 'h', 'u', 'y', 'e', '^', 'n', '.'], "chuyện"),
    case!(['n', 'g', 'u', 'y', 'e', '^', 'n'], "nguyên"),
    case!(['y', 'e', '^', 'u'], "yêu"),
    case!(['y', 'e', '^', 'u', '\''], "yếu"),
    case!(['k', 'h', 'o', 'e'], "khoe"),
    case!(['q', 'u', 'y', '\''], "quý"),
    case!(['l', 'a', '.'], "lạ"),
    case!(['h', 'o', 'c', '.'], "học"),
    case!(['t', 'h', 'e', '^', '\''], "thế"),
];