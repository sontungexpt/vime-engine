//! I. Toggle / revert behaviour.
//!
//! Transform keys produce one of three effects:
//!   Applied      → the transform took effect (tone/shape set)
//!   Reverted     → an equal tone/shape was toggled off; the key then falls
//!                  through as a literal and lands in the coda
//!   NotApplicable→ nothing to transform; the key falls through as literal
//! The expected strings below reflect that *exact* semantics.

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    // ── tone → different tone (Applied) ──
    case!(['a', 's'], "á"),
    case!(['á', 'f'], "à"),
    case!(['à', 'r'], "ả"),
    case!(['ả', 'x'], "ã"),
    case!(['ã', 'j'], "ạ"),
    case!(['ả', 'f'], "à"),
    // ── tone → same tone (Reverted + literal spill) ──
    case!(['a', 's', 's'], "as"),
    case!(['á', 's'], "as"),
    case!(['à', 'f'], "af"),
    // ── `z` resets a tone (Applied when the vowel is toned) ──
    case!(['á', 'z'], "a"),
    case!(['ạ', 'z'], "a"),
    // ── shape → same shape (Reverted + literal spill) ──
    case!(['a', 'w', 'w'], "aw"),
    case!(['ă', 'w'], "aw"),
    case!(['â', 'a'], "aa"),
    case!(['ô', 'o'], "oo"),
    case!(['ê', 'e'], "ee"),
    case!(['ơ', 'w'], "ow"),
    case!(['ư', 'w'], "uw"),
    // ── tone → same tone (Reverted + literal spill) ──
    case!(['ẻ', 'r'], "er"),
    case!(['õ', 'x'], "ox"),
    case!(['ị', 'j'], "ij"),
    // ── shape → different shape (Applied) ──
    case!(['a', 'a'], "â"),
    case!(['â', 'w'], "ă"),
    case!(['ă', 'a'], "â"),
    case!(['a', 'w'], "ă"),
    case!(['o', 'o'], "ô"),
    // ── tone + shape toggles keep the other attribute ──
    case!(['a', 's', 'w'], "ắ"),
    case!(['ă', 's'], "ắ"),
    case!(['ắ', 'w'], "áw"),
    case!(['a', 'w', 's'], "ắ"),
    // ── d-stroke toggle ──
    case!(['d', 'd'], "đ"),
    case!(['D', 'D'], "Đ"),
    case!(['d', 'd', 'a'], "đa"),
    // ── d-stroke revert: the stroke key spills as a literal ──
    case!(['d', 'd', 'd'], "dd"),
    case!(['D', 'D', 'D'], "DD"),
    // ── uo/ươ revert cycles (see section E) ──
    case!(['u', 'o', 'w'], "uơ"),
    case!(['u', 'o', 'w', 'w'], "ươ"),
    case!(['u', 'o', 'w', 'w', 'w'], "uow"),
    // ── tone on a literal that is a tone key in onset: `s` becomes onset ──
    case!(['s'], "s"),
    // ── expansion: more revert spills ──
    case!(['o', 's', 's'], "os"),
    case!(['i', 'x', 'x'], "ix"),
    case!(['u', 'r', 'r'], "ur"),
    case!(['o', 's', 'r'], "ỏ"),
];