//! L. Checkpoints that leave the parse *alive* (`invalid == None`).
//!
//! The nucleus is recognised but may still need more input (a tone, a coda, a
//! shape) to finish. Only liveness is compared — the shape / `uo` cycles move
//! the internal syllable around, which this harness intentionally stays
//! agnostic to.
//!
//! Several of these were previously dead ends; the expanded nucleus validity
//! rules (see `phonology::rule::transition`) keep them alive.

use super::{status_case, StatusCase};

pub const TELEX: &[StatusCase] = &[
    // ── iêu family: `ieu` is now an accepted nucleus ──
    status_case!(['i', 'e', 'u']),
    // `ieun` used to be Dead(InvalidVowelSequence); it must stay alive now
    // that [I, E, U] is a recognised nucleus.
    status_case!(['i', 'e', 'u', 'n']),
    // ── ue / uy-e / uu: unions kept alive ──
    status_case!(['u', 'e']),
    status_case!(['u', 'y', 'e']),
    status_case!(['u', 'u']),
    // ── uo family checkpoints ──
    status_case!(['u', 'o']),
    status_case!(['u', 'o', 'i']),
    status_case!(['u', 'o', 'u']),
    status_case!(['u', 'o', 'w', 'i']), // uơi
    status_case!(['u', 'o', 'w', 'u']), // uơu
    // ── ưo family (ư = u + w in telex) ──
    status_case!(['u', 'w', 'o']),      // ưo
    status_case!(['u', 'w', 'o', 'i']), // ưoi
    status_case!(['u', 'w', 'o', 'u']), // ưou
    // ── the `w` cycle: uơ / ươ stay alive until a tone or coda arrives ──
    status_case!(['u', 'o', 'w']),      // uơ
    status_case!(['u', 'o', 'w', 'w']), // ươ
    // ── q waits for u (see dead_cases for q + non-u vowels) ──
    status_case!(['q']),
    status_case!(['q', 'u']),
    status_case!(['q', 'u', 'a']),
    status_case!(['q', 'u', 'y']),
];

pub const VNI: &[StatusCase] = &[
    // `oo` is a shape (ô) on telex; VNI keeps two plain o's, the `oo` union
    status_case!(['o', 'o']),
    status_case!(['u', 'u']),
    status_case!(['i', 'e', 'u']),
];
