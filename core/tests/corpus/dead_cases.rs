//! J. Invalid / dead cases: inputs that leave the parse in a dead status.
//! The syllable a dead parse renders is not compared, only the `ParseStatus`.

use super::{dead_case, DeadCase};

use vime_engine::{DeadReason, ParseStatus};

pub const TELEX: &[DeadCase] = &[
    // ── InvalidOnset ──
    dead_case!(['b', 'c', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['w', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['q', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)), // q needs u
    dead_case!(['z', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['j', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['f', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['g', 'r', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(
        ['t', 'r', 'g', 'a'],
        ParseStatus::Dead(DeadReason::InvalidOnset)
    ),
    dead_case!(
        ['k', 'h', 'h', 'a'],
        ParseStatus::Dead(DeadReason::InvalidOnset)
    ),
    dead_case!(
        ['n', 'g', 'g', 'a'],
        ParseStatus::Dead(DeadReason::InvalidOnset)
    ),
    // ── InvalidVowelSequence ──
    // `ieu` is now a valid (incomplete) nucleus: `ieun` parses Incomplete.
    dead_case!(
        ['a', 'i', 'u', 'n'],
        ParseStatus::Dead(DeadReason::InvalidVowelSequence)
    ),
    dead_case!(
        ['a', 'o', 'i', 'u'],
        ParseStatus::Dead(DeadReason::InvalidVowelSequence)
    ), // fourth vowel
    dead_case!(
        ['b', 'a', 'o', 'i', 'u', 'n'],
        ParseStatus::Dead(DeadReason::InvalidVowelSequence)
    ), // fourth vowel after onset
    // ── InvalidCoda ──
    dead_case!(['a', 'k', 't'], ParseStatus::Dead(DeadReason::InvalidCoda)),
    dead_case!(['a', 'b', 'd'], ParseStatus::Dead(DeadReason::InvalidCoda)),
    dead_case!(['a', 'n', 'n'], ParseStatus::Dead(DeadReason::InvalidCoda)),
    dead_case!(
        ['c', 'h', 'a', 't', 'c'],
        ParseStatus::Dead(DeadReason::InvalidCoda)
    ),
    dead_case!(['a', 'm', 'h'], ParseStatus::Dead(DeadReason::InvalidCoda)),
    dead_case!(
        ['a', 'n', 'g', 'g'],
        ParseStatus::Dead(DeadReason::InvalidCoda)
    ),
    dead_case!(['a', 't', 'c'], ParseStatus::Dead(DeadReason::InvalidCoda)),
    dead_case!(['o', 'n', 'm'], ParseStatus::Dead(DeadReason::InvalidCoda)),
    // ── InvalidCharacter ──
    // Only a non-letter typed *after* a vowel kills the parse. Before any
    // vowel it is accepted as a literal custom onset and stays Incomplete
    // (see `incomplete`).
    dead_case!(['a', '?'], ParseStatus::Dead(DeadReason::InvalidCharacter)),
    dead_case!(
        ['a', 'n', 'o'],
        ParseStatus::Dead(DeadReason::InvalidCharacter)
    ), // vowel inside a coda
    // ── expansion ──
    // InvalidOnset: too long a digraph
    dead_case!(
        ['t', 'r', 'l', 'a'],
        ParseStatus::Dead(DeadReason::InvalidOnset)
    ),
    dead_case!(
        ['n', 'g', 'c', 'a'],
        ParseStatus::Dead(DeadReason::InvalidOnset)
    ),
    dead_case!(
        ['p', 'h', 'f', 'a'],
        ParseStatus::Dead(DeadReason::InvalidOnset)
    ),
    // InvalidCoda: `p h` / `ng h` are not valid codas
    dead_case!(['a', 'p', 'h'], ParseStatus::Dead(DeadReason::InvalidCoda)),
    dead_case!(
        ['a', 'n', 'g', 'h'],
        ParseStatus::Dead(DeadReason::InvalidCoda)
    ),
    // InvalidCharacter: a non-letter after a vowel; before one it is a
    // literal custom onset
    dead_case!(['a', '$'], ParseStatus::Dead(DeadReason::InvalidCharacter)),
    dead_case!(
        ['c', 'h', 'a', '0'],
        ParseStatus::Dead(DeadReason::InvalidCharacter)
    ),
    // ── expansion ──
    // InvalidOnset: invalid consonant clusters
    dead_case!(['b', 'd', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['m', 'n', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    // InvalidCoda: doubled consonant codas
    dead_case!(['a', 't', 't'], ParseStatus::Dead(DeadReason::InvalidCoda)),
    dead_case!(['a', 'k', 'k'], ParseStatus::Dead(DeadReason::InvalidCoda)),
    dead_case!(['o', 'n', 'm'], ParseStatus::Dead(DeadReason::InvalidCoda)),
];

pub const VNI: &[DeadCase] = &[
    // A VNI key that is not a vowel is kept as a literal custom onset until
    // the first vowel arrives; with a vowel it then fails onset validation.
    dead_case!(['1', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['9', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['6', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['7', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    // stroke key with no `d` to revert → literal in the vowel phase kills
    dead_case!(['a', '9'], ParseStatus::Dead(DeadReason::InvalidCharacter)),
    // coda-invalid via VNI layout
    dead_case!(['a', 'b', 'd'], ParseStatus::Dead(DeadReason::InvalidCoda)),
    // flat-reset key lands as a literal inside the vowel phase
    dead_case!(['a', '0'], ParseStatus::Dead(DeadReason::InvalidCharacter)),
    // ── expansion ──
    // InvalidOnset: invalid consonant clusters in the VNI layout too
    dead_case!(['b', 'c', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    dead_case!(['w', 'a'], ParseStatus::Dead(DeadReason::InvalidOnset)),
    // InvalidCoda: doubled consonant codas
    dead_case!(['a', 't', 't'], ParseStatus::Dead(DeadReason::InvalidCoda)),
    dead_case!(['a', 'k', 'k'], ParseStatus::Dead(DeadReason::InvalidCoda)),
];
