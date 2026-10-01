//! Frontend-independent Vietnamese input method engine.
//!
//! Pipeline: key → [`keymap::Keymap`] → [`composition::Composition`] → [`syllable::Syllable`] → phonology → text.
//!
//! - [`keymap::Keymap`]: classifies and decodes tone, vowel-shape and `d`/`đ` stroke keys (`DefaultKeymap` covers Telex/VNI/VIQR).
//! - [`composition::Composition`]: the raw keystroke buffer, its cursors, and the incremental syllable parser.
//! - [`syllable::Syllable`]: the two-phase buffer — a building syllable, or a verbatim dead one once input can no longer form Vietnamese.
//! - [`Session`]: one typing buffer, turning each key into a [`Result`] (re-render, commit, ignore or forward).

// `unsafe fn` bodies are `const fn`s with a bounds precondition; an explicit
// `unsafe` block keeps that contract visible at the read site.
#![deny(unsafe_op_in_unsafe_fn)]

mod keymap;
mod session;
// mod sessions;

pub mod util;

pub mod composition;
pub mod phonology;
pub mod syllable;

pub use keymap::{DefaultKeymap, Keymap, Rules, ShapeRule, ToneRule};
pub use session::{Session, SessionConfig, SessionFactory, Settings, SharedSessionConfig};
