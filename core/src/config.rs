/// Immutable engine settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config {
    /// Restore English when the word contains characters that are not
    /// Vietnamese, committing the literal text instead of interpreting it.
    pub auto_restore_english: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // Whether this is on by default is a product decision: it changes
            // what happens to a word that is not Vietnamese at all.
            auto_restore_english: true,
        }
    }
}
