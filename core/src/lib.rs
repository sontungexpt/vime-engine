//! Frontend-independent Vietnamese input method engine.
//!
//! Architecture:
//!
//! ```text
//! Input → Interpreter → Processor → Unicode
//! ```
//!
//! - [`character`]: semantic Vietnamese vowels as primitive `(base, tone, case)`
//!   triples and the [`decode`]/[`encode`] codec over precomposed characters.
//! - [`Interpreter`]: turns a key plus [`rule_engine::KeyContext`] into
//!   shape or tone changes, per input-method configuration
//!   (`SimpleInterpreter`).
//! - [`Processor`]: Vietnamese rules; applies actions to semantic vowels,
//!   and parses/renders canonical ASCII syllables into Vietnamese text.
//! - [`Buffer`] + [`Engine`]: raw input buffering, cursor editing, and the
//!   frontend-facing state machine.

mod config;
mod engine;
mod event;
mod parser;
mod result;
mod rule_engine;

pub mod composition;
pub mod phonology;
pub mod renderer;

pub use composition::{Buffer, Cased, Syllable};
pub use config::Config;
pub use engine::Engine;
pub use event::{Key, KeyEvent, KeyState};
pub use parser::{DeadReason, ParsePhase, ParseStatus, Parser};
pub use phonology::{
    decode_vowel, encode_vowel, is_vowel, BaseVowel, Case, RootVowel, Shape, Tone,
};
pub use renderer::{analyze, DefaultRenderer, Orthography, Renderer};
pub use result::Result;
pub use rule_engine::{ConfiguredRuleEngine, RuleEngine, ShapeRule, ToneRule, TypingRules};
