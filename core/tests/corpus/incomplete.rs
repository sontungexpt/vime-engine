//! L. Inputs that leave the parse in an *incomplete* status.
//!
//! The nucleus is recognised but still needs more input (a tone, a coda, a
//! shape) to finish. Only the `ParseStatus` is compared, never the rendered
//! syllable, so these stay honest while the shape / `uo` cycles move around.
//!
//! Several of these were previously dead ends; the expanded nucleus validity
//! rules (see `phonology::rule::transition`) turned them into `InComplete`:

use super::{status_case, DeadCase};
use vime_engine::ParseStatus;

pub const TELEX: &[DeadCase] = &[
    // ── iêu family: `ieu` is now an accepted, incomplete nucleus ──
    status_case!(['i', 'e', 'u'], ParseStatus::Incomplete),
    // `ieun` used to be Dead(InvalidVowelSequence); it must stay alive as an
    // incomplete parse now that [I, E, U] is a recognised nucleus.
    status_case!(['i', 'e', 'u', 'n'], ParseStatus::Incomplete),
    // ── ue / uy-e / uu: unions kept incomplete ──
    status_case!(['u', 'e'], ParseStatus::Incomplete),
    status_case!(['u', 'y', 'e'], ParseStatus::Incomplete),
    status_case!(['u', 'u'], ParseStatus::Incomplete),
    // ── uo family checkpoints ──
    status_case!(['u', 'o'], ParseStatus::Incomplete),
    status_case!(['u', 'o', 'i'], ParseStatus::Incomplete),
    status_case!(['u', 'o', 'u'], ParseStatus::Incomplete),
    status_case!(['u', 'o', 'w', 'i'], ParseStatus::Incomplete), // uơi
    status_case!(['u', 'o', 'w', 'u'], ParseStatus::Incomplete), // uơu
    // ── ưo family (ư = u + w in telex) ──
    status_case!(['u', 'w', 'o'], ParseStatus::Incomplete), // ưo
    status_case!(['u', 'w', 'o', 'i'], ParseStatus::Incomplete), // ưoi
    status_case!(['u', 'w', 'o', 'u'], ParseStatus::Incomplete), // ưou
    // ── the `w` cycle: uơ / ươ stay incomplete until a tone or coda arrives ──
    status_case!(['u', 'o', 'w'], ParseStatus::Incomplete), // uơ
    status_case!(['u', 'o', 'w', 'w'], ParseStatus::Incomplete), // ươ
    // ── any key before a vowel is a literal custom onset ──
    // `push_onset_literal` accepts whatever leads the slot until the first
    // vowel arrives, so symbols survive as Incomplete instead of dying.
    status_case!(['?'], ParseStatus::Incomplete),
    status_case!(['!'], ParseStatus::Incomplete),
    status_case!(['#'], ParseStatus::Incomplete),
    status_case!(['c', 'h', '?'], ParseStatus::Incomplete),
    status_case!(['c', 'h', '0'], ParseStatus::Incomplete),
];

pub const VNI: &[DeadCase] = &[
    // `oo` is a shape (ô) on telex; VNI keeps two plain o's, the `oo` union
    status_case!(['o', 'o'], ParseStatus::Incomplete),
    status_case!(['u', 'u'], ParseStatus::Incomplete),
    status_case!(['i', 'e', 'u'], ParseStatus::Incomplete),
    // a repeated stroke key spills as a literal custom onset: d,9 → đ, the
    // second 9 reverts it and lands as ['d', '9'] (rendered "d9"), so the
    // parse stays Incomplete rather than dying.
    status_case!(['d', '9', '9'], ParseStatus::Incomplete),
];
