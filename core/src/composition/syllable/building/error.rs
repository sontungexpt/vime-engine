/// Error type for invalid syllable construction operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyllableBuildError {
    /// The consonant cluster is not a valid Vietnamese onset.
    InvalidOnset,
    /// The vowel nucleus violates the Vietnamese vowel-rule table.
    InvalidNucleus,
    /// The final consonant cluster is not a valid Vietnamese coda.
    InvalidCoda,
}
