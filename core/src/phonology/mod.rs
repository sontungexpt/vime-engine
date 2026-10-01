//! Vietnamese phonology: the parts of a syllable and the rules they must satisfy.
mod case;
mod coda;
mod nucleus;
mod onset;
mod phonotactics;
mod tone_placement;
mod vowel;

pub use case::{Case, Cased};
pub use coda::{Coda, CodaParseError};

pub use nucleus::{
    nucleus_state, BaseVowelSlice, NucleusState, NucleusStateResolver, NUCLEUS_MAX_LEN,
};
pub use vowel::{
    decode_vowel, encode_vowel, is_vowel, BaseVowel, BaseVowelId, RootVowel, Shape, Tone, Vowel,
};

pub use onset::{Onset, OnsetParseError};
pub use phonotactics::{validate_phonotactics, PhonotacticError};
pub use tone_placement::TonePlacement;
