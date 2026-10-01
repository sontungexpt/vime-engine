//! Sessions and the settings they follow.
//!
//! The shared config is replaced in place behind an atomic generation counter;
//! every config-dependent operation compares generations to adopt a change —
//! one atomic load, no lock, no allocation.

// Clippy's `module_inception` objects to `session::session`, which reads oddly
// but names the file's contents exactly.
#[allow(clippy::module_inception)]
mod session;

mod config;
mod factory;

#[cfg(test)]
mod tests;

pub use config::{SessionConfig, Settings, SharedSessionConfig};
pub use factory::SessionFactory;
pub use session::Session;
