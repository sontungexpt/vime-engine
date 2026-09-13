//! C. Telex shape keys.

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── core shapes ──
    case!(['a', 'w'], "ă"),
    case!(['a', 'a'], "â"),
    case!(['e', 'e'], "ê"),
    case!(['o', 'o'], "ô"),
    case!(['o', 'w'], "ơ"),
    case!(['u', 'w'], "ư"),
    // ── shapes on shaped vowels: replace or revert the shape ──
    case!(['â', 'w'], "ă"),
    case!(['ă', 'a'], "â"),
    case!(['â', 'a'], "aa"), // same shape reverts â → a, key leaks as a vowel
    // ── shapes after an onset ──
    case!(['c', 'h', 'a', 'w'], "chă"),
    case!(['c', 'h', 'a', 'a'], "châ"),
    case!(['c', 'o', 'o'], "cô"),
    case!(['b', 'u', 'w'], "bư"),
    case!(['t', 'h', 'o', 'w'], "thơ"),
    // ── shape on plain a after a coda already started ──
    case!(['a', 'n', 'w'], "ăn"),
    case!(['a', 'n', 'a'], "ân"),
    // ── o/w interplay outside uo context (single vowel) ──
    case!(['o', 'w'], "ơ"),
    case!(['o', 'w', 'o'], "ô"),
    // ── o/w interplay: circumflex replaced by the horn on a second key ──
    case!(['o', 'o', 'w'], "ơ"),
    // ── shape toggles ──
    case!(['a', 'w', 'w'], "aw"),
    case!(['e', 'e', 'e'], "ee"),
    case!(['o', 'o', 'o'], "oo"),
    case!(['u', 'w', 'w'], "uw"),
    case!(['o', 'w', 'w'], "ow"),
    // ── shapes in front of onsets / codas ──
    case!(['k', 'e', 'e'], "kê"),
    case!(['c', 'h', 'e', 'e'], "chê"),
    case!(['t', 'h', 'e', 'e'], "thê"),
    case!(['n', 'g', 'a', 'w'], "ngă"),
    case!(['d', 'd', 'a', 'w'], "đă"),
    case!(['b', 'u', 'w', 'n'], "bưn"),
    // ── expansion: shapes with an onset ──
    case!(['t', 'h', 'a', 'a'], "thâ"),
    // ── expansion: shaped nuclei with a following vowel ──
    case!(['e', 'e', 'u'], "êu"),
    case!(['o', 'o', 'i'], "ôi"),
    case!(['o', 'w', 'i'], "ơi"),
    case!(['u', 'w', 'a'], "ưa"),
    // ── expansion: shaped nuclei with a coda ──
    case!(['a', 'a', 'n'], "ân"),
    case!(['b', 'a', 'w', 'n'], "băn"),
    case!(['u', 'w', 'n'], "ưn"),
    // ── expansion: shapes on more onsets ──
    case!(['l', 'a', 'w'], "lă"),
    case!(['r', 'a', 'a'], "râ"),
    case!(['h', 'a', 'a'], "hâ"),
    case!(['y', 'e', 'e'], "yê"),
    case!(['o', 'a', 'w'], "oă"),
    // ── expansion: shaped nuclei with codas ──
    case!(['n', 'g', 'a', 'a', 'n'], "ngân"),
    case!(['t', 'h', 'a', 'a', 'n'], "thân"),
];