//! Sessions, and the shared settings they follow.
//!
//! An [`SessionFactory`](crate::SessionFactory) owns one
//! [`SharedSessionConfig`] and mints [`Session`]s from it. A session is one
//! typing buffer, several can exist at once, and by default all of them run
//! under the same settings. A session that needs its own settings takes a
//! private [`SessionConfig`] instead, and then stops following the shared one.
//!
//! # How a change reaches every session
//!
//! The shared config is replaced in place behind an atomic generation counter
//! rather than pushed to each session, so a session cannot miss an update
//! because nobody remembered to notify it. Every session caches the settings it
//! resolved together with the generation it resolved them at;
//! [`Session::pull_config`] compares that against the current generation. Every
//! config-dependent operation calls it, so in the steady state that costs one
//! atomic load, with no lock and no allocation.
//!
//! The counter is not only a cache check. Changing the shared settings can move
//! the rendered word even when no key was pressed, so a session has to be able
//! to notice that it happened: [`Session::pull_config`] reports whether it
//! adopted anything, which is how a caller learns to re-read the word.

// Named after the type it defines, like the rest of the crate. The lint objects
// to `session::session`, which reads oddly but says the file's contents exactly.
#[allow(clippy::module_inception)]
mod session;

mod config;
mod factory;

#[cfg(test)]
mod tests;

pub use config::{SessionConfig, Settings, SharedSessionConfig};
pub use factory::SessionFactory;
pub use session::Session;
