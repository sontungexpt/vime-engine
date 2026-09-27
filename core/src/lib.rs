//! Frontend-independent Vietnamese input method engine.
//!
//! Architecture:
//!
//! ```text
//! KeyEvent → Keymap → Composition → SyllableBuilder → phonology → Vietnamese text
//! ```
//!
//! - [`KeyEvent`]: a [`Key`] plus [`KeyStates`]; the only input the engine takes.
//! - [`keymap::Keymap`]: classifies a key as a tone, vowel-shape or
//!   `d`/`đ` stroke key and decodes it (`DefaultKeymap` for Telex/VNI/VIQR).
//! - [`Composition`]: the raw keystroke buffer, its cursor, and the incremental
//!   syllable parser driven by that keymap.
//! - [`SyllableBuilder`](composition::syllable::SyllableBuilder): the two-phase
//!   syllable buffer — a validated `BuildingSyllable`, or a verbatim
//!   `DeadSyllable` once the input can no longer form a Vietnamese syllable.
//! - [`phonology`]: the shared model both phases rely on — the `Vowel` codec
//!   (`encode_vowel`/`decode_vowel`), [`NucleusState`](phonology::NucleusState),
//!   [`PhonotacticValidator`](phonology::PhonotacticValidator) and
//!   [`TonePlacement`](phonology::TonePlacement).
//! - [`Engine`]: the frontend-facing state machine, turning each [`KeyEvent`]
//!   into a [`Result`] (re-render, commit, ignore or forward the key).

mod config;
mod engine;
mod event;
mod keymap;
mod result;

pub mod util;

pub mod composition;
pub mod phonology;

pub use composition::Composition;
pub use config::Config;
pub use engine::Engine;
pub use event::{Key, KeyEvent, KeyStates};
pub use keymap::{DefaultKeymap, Keymap, Rules, ShapeRule, ToneRule};
pub use result::Result;
