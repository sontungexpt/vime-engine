/// Immutable engine settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config {
    pub auto_restore_english: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // Mặc định bật hoặc tắt tùy thuộc vào nhu cầu mặc định của IME
            auto_restore_english: true,
        }
    }
}
