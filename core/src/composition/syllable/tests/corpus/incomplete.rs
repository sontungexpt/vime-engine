//! L. Checkpoints that leave the parse *alive* (`invalid == None`).
//!
//! The nucleus is recognised but may still need more input (a tone, a coda, a
//! shape) to finish. Only liveness is compared — the shape / `uo` cycles move
//! the internal syllable around, which this harness intentionally stays
//! agnostic to.
//!
//! Several of these were previously dead ends; the expanded nucleus validity
//! rules (see `phonology::rule::transition`) keep them alive.

use super::prelude::*;

pub const TELEX: &[Case] = &[
    // ── iêu family: `ieu` is now an accepted nucleus ──
    alive_case!(['i', 'e', 'u']),
    // `ieun` used to be Dead(InvalidVowelSequence); it must stay alive now
    // that [I, E, U] is a recognised nucleus.
    alive_case!(['i', 'e', 'u', 'n']),
    // ── ue / uy-e / uu: unions kept alive ──
    alive_case!(['u', 'e']),
    alive_case!(['u', 'y', 'e']),
    alive_case!(['u', 'u']),
    // ── uo family checkpoints ──
    alive_case!(['u', 'o']),
    alive_case!(['u', 'o', 'i']),
    alive_case!(['u', 'o', 'u']),
    alive_case!(['u', 'o', 'w', 'i']), // uơi
    alive_case!(['u', 'o', 'w', 'u']), // uơu
    // ── ưo family (ư = u + w in telex) ──
    alive_case!(['u', 'w', 'o']),      // ưo
    alive_case!(['u', 'w', 'o', 'i']), // ưoi
    alive_case!(['u', 'w', 'o', 'u']), // ưou
    // ── the `w` cycle: uơ / ươ stay alive until a tone or coda arrives ──
    alive_case!(['u', 'o', 'w']),      // uơ
    alive_case!(['u', 'o', 'w', 'w']), // ươ
    // ── q waits for u (see dead_cases for q + non-u vowels) ──
    alive_case!(['q']),
    alive_case!(['q', 'u']),
    alive_case!(['q', 'u', 'a']),
    alive_case!(['q', 'u', 'y']),
];

pub const VNI: &[Case] = &[
    // `oo` is a shape (ô) on telex; VNI keeps two plain o's, the `oo` union
    alive_case!(['o', 'o']),
    alive_case!(['u', 'u']),
    alive_case!(['i', 'e', 'u']),
];
