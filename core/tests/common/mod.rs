//! Shared harness for the session-facing integration tests.
//!
//! `config.rs` and `session.rs` both drive a Telex session through the public
//! API and both assert on the same `hoa` rendering, so the type alias, the two
//! expected words, and the type/render helpers live here rather than being
//! written twice. Helpers only one of them uses stay in that file.

use vime_engine::phonology::TonePlacement;
use vime_engine::{DefaultKeymap, Session, SessionConfig, Settings};

/// A session on the default keymap, which is what these tests are about.
pub type TelexSession = Session<DefaultKeymap<'static>>;

/// "hoa" + sắc: the two schemes place the mark on different vowels
/// (Modern 2-vowel rule: second vowel; Old open syllable: first vowel).
pub const MODERN_HOA: &str = "hoá";
pub const OLD_HOA: &str = "hóa";

/// A config on the given keymap and tone-placement scheme.
pub fn make_config(
    keymap: DefaultKeymap<'static>,
    tone: TonePlacement,
) -> SessionConfig<DefaultKeymap<'static>> {
    SessionConfig::new(Settings::default(), keymap, tone)
}

/// The word as the session renders it.
pub fn rendered_to_string(session: &mut TelexSession) -> String {
    session.rendered().into_iter().collect()
}

/// Types `s` one keystroke at a time, then returns what the session renders.
pub fn type_str(session: &mut TelexSession, s: &str) -> String {
    for ch in s.chars() {
        session.insert(ch);
    }
    rendered_to_string(session)
}
