mod case;
mod coda;
mod nucleus;
mod onset;
mod phonotactics;
mod vowel;

pub use case::{Case, Cased};
pub use coda::{Coda, CodaParseError};

pub use nucleus::{
    nucleus_state, BaseVowelSlice, NucleusState, NucleusStateOf, TonePlacement, NUCLEUS_MAX_LEN,
};

pub use onset::{Onset, OnsetParseError};
pub use phonotactics::{DefaultPhonotacticValidator, PhonotacticError, PhonotacticValidator};
pub use vowel::{
    decode_vowel, encode_vowel, is_vowel, BaseVowel, BaseVowelId, RootVowel, Shape, Tone, Vowel,
};
