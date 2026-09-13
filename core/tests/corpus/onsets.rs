//! A. Basic onsets and plain vowel sequences, before shapes and tones.

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── single vowels ──
    case!(['a'], "a"),
    case!(['e'], "e"),
    case!(['i'], "i"),
    case!(['o'], "o"),
    case!(['u'], "u"),
    case!(['y'], "y"),
    // zero onset requires a vowel; plain onset chars render as themselves
    case!(['b'], "b"),
    case!(['c'], "c"),
    case!(['d'], "d"),
    case!(['g'], "g"),
    case!(['h'], "h"),
    case!(['k'], "k"),
    case!(['l'], "l"),
    case!(['m'], "m"),
    case!(['n'], "n"),
    case!(['q'], "q"),
    case!(['r'], "r"),
    case!(['s'], "s"),
    case!(['t'], "t"),
    case!(['v'], "v"),
    case!(['x'], "x"),
    // ── single consonant + vowel ──
    case!(['b', 'a'], "ba"),
    case!(['c', 'a'], "ca"),
    case!(['g', 'a'], "ga"),
    case!(['h', 'a'], "ha"),
    case!(['k', 'a'], "ka"),
    case!(['l', 'a'], "la"),
    case!(['m', 'a'], "ma"),
    case!(['n', 'a'], "na"),
    case!(['p', 'a'], "pa"),
    case!(['r', 'a'], "ra"),
    case!(['t', 'a'], "ta"),
    case!(['v', 'a'], "va"),
    case!(['x', 'a'], "xa"),
    // tone keys double as consonants while no vowel exists yet
    case!(['s', 'a'], "sa"),
    case!(['x', 'a'], "xa"),
    // ── digraph onsets ──
    case!(['c', 'h', 'a'], "cha"),
    case!(['g', 'h', 'a'], "gha"),
    case!(['g', 'i', 'a'], "gia"),
    case!(['k', 'h', 'a'], "kha"),
    case!(['n', 'g', 'a'], "nga"),
    case!(['n', 'g', 'h', 'a'], "ngha"),
    case!(['p', 'h', 'a'], "pha"),
    case!(['t', 'h', 'a'], "tha"),
    case!(['t', 'r', 'a'], "tra"),
    // ── d / đ stroke ──
    case!(['d', 'a'], "da"),
    case!(['d', 'd', 'a'], "đa"),
    // qu keeps `u` as a consonant inside the onset
    case!(['q', 'u', 'a'], "qua"),
    case!(['q', 'u', 'e'], "que"),
    case!(['q', 'u', 'i'], "qui"),
    case!(['q', 'u', 'o'], "quo"),
    // ── vowel sequences without onset ──
    case!(['a', 'i'], "ai"),
    case!(['a', 'o'], "ao"),
    case!(['a', 'u'], "au"),
    case!(['a', 'y'], "ay"),
    case!(['i', 'a'], "ia"),
    case!(['u', 'a'], "ua"),
    case!(['ư', 'a'], "ưa"),
    case!(['u', 'y'], "uy"),
    case!(['o', 'a'], "oa"),
    case!(['o', 'e'], "oe"),
    case!(['e', 'u'], "eu"),
    case!(['i', 'u'], "iu"),
    case!(['i', 'e'], "ie"),
    case!(['o', 'i'], "oi"),
    case!(['ư', 'u'], "ưu"),
    // ── ngh onset ──
    case!(['n', 'g', 'h', 'e'], "nghe"),
    case!(['n', 'g', 'h', 'i'], "nghi"),
    // ── đ standalone ──
    case!(['d', 'd'], "đ"),
    // ── expansion: more consonants + vowels ──
    case!(['d', 'd', 'i'], "đi"),
    case!(['x', 'e'], "xe"),
    case!(['p', 'h', 'o'], "pho"),
    case!(['k', 'h', 'u'], "khu"),
    case!(['t', 'r', 'e'], "tre"),
    case!(['n', 'h', 'u'], "nhu"),
    case!(['n', 'h', 'a', 'n', 'h'], "nhanh"),
    // ── expansion: gi with a plain vowel ──
    case!(['g', 'i', 'u'], "giu"),
    // ── expansion: vowel sequences with an onset ──
    case!(['c', 'a', 'y'], "cay"),
    case!(['b', 'o', 'i'], "boi"),
    case!(['m', 'i', 'e'], "mie"),
];