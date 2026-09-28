//! The nucleus: the vowel core of a Vietnamese syllable.
//!
//! ```text
//! nucleus/
//! ├── state.rs           the rule table -> Valid / InComplete / Dead
//! ├── slice.rs           the read-only view every consumer reads through
//! ├── tone_placement.rs  which vowel in the nucleus carries the tone mark
//! └── dfa.rs             generated DFA, kept as an unused backup
//! ```
//!
//! `dfa.rs` is intentionally absent from the module tree below: it is a backup
//! that is not compiled, and `state.rs` is the live implementation. See its
//! module docs for what would have to change to revive it.
//!
//! The three pieces share one vocabulary: a nucleus is up to
//! [`NUCLEUS_MAX_LEN`] base vowels, read through [`BaseVowelSlice`] so callers
//! can pass `BaseVowel`s, `Vowel`s or the syllable builder's inline buffer
//! alike. [`NucleusState`] decides whether a nucleus is a real one;
//! [`TonePlacement`] decides which of its vowels the tone mark lands on.

mod slice;
mod state;
mod tone_placement;

pub use slice::BaseVowelSlice;
pub use state::{nucleus_state, NucleusState, NucleusStateOf, NUCLEUS_MAX_LEN};
pub use tone_placement::TonePlacement;
