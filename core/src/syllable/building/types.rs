use crate::phonology::{Coda, Onset, Vowel, NUCLEUS_MAX_LEN};
use crate::util::vec::ArrayVec;

// ─────────────────────────── Transform ───────────────────────────

/// Which part of the syllable a transform key touched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformTarget {
    DStroke,
    /// The affected vowel's index.
    Nucleus(usize),
    UoPair,
    Tone,
}

/// Outcome of trying a transform key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformEffect {
    /// Applied a new mark to the syllable (e.g. `a` + `w` -> `ă`).
    Applied(TransformTarget),
    /// Undid an existing mark back to base (e.g. `ă` + `w` -> `a`).
    Reverted(TransformTarget),
    /// The key cannot transform the current state; pass through as a literal char.
    None,
}

pub(super) type OnsetBuffer = ArrayVec<char, { Onset::MAX_LEN }>;
pub(super) type CodaBuffer = ArrayVec<char, { Coda::MAX_LEN }>;
pub(super) type NucleusBuffer = ArrayVec<Vowel, { NUCLEUS_MAX_LEN }>;
