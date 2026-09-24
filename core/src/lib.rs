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
//! - [`Interpreter`]: turns a key plus [`keymap::Keymap`] into
//!   shape or tone changes, per input-method configuration
//!   (`SimpleInterpreter`).
//! - [`Processor`]: Vietnamese rules; applies actions to semantic vowels,
//!   and parses/renders canonical ASCII syllables into Vietnamese text.
//! - [`Composition`] + [`Engine`]: raw input buffering, cursor editing, and
//!   the frontend-facing state machine.

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
