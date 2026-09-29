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
//! - [`Session`]: one typing buffer, turning each [`KeyEvent`] into a
//!   [`Result`] (re-render, commit, ignore or forward the key).
//! - [`SessionFactory`]: owns the [`Config`] shared by its sessions and creates
//!   them, so one settings change can reach every session.

// The `unsafe fn` bodies in this crate are `const fn`s whose safety contract
// is a bounds precondition. Requiring an explicit `unsafe` block inside them
// keeps the contract visible at the point of the read, rather than relying on
// the implicit-unsafe that `unsafe fn` used to confer.
#![deny(unsafe_op_in_unsafe_fn)]

mod event;
mod keymap;
mod result;
mod session;
mod sessions;

pub mod util;

pub mod composition;
pub mod phonology;

pub use keymap::{DefaultKeymap, Keymap, Rules, ShapeRule, ToneRule};
pub use session::{Config, Session, SessionFactory, Settings, SharedConfig};
