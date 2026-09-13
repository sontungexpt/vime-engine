/// Semantic result of processing one input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Result {
    /// Input buffer state changed; frontends should re-render the preedit
    /// from the current state's rendered text.
    Changed,
    /// A word was finalized; the string must be committed to the
    /// application and the input buffer cleared.
    Commit(String),
    /// The input was consumed but produced no visible change.
    Noop,
    /// The input was not consumed; frontends should pass the key through.
    Forward,
}
