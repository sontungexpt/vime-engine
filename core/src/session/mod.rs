//! Sessions, and the shared settings they follow.
//!
//! An [`SessionFactory`](crate::SessionFactory) owns one [`SharedConfig`] and mints [`Session`]s
//! from it. A session is one typing buffer, several can exist at once, and by
//! default all of them run under the same settings. A session that needs its
//! own settings takes a private [`Config`] instead, and then stops
//! following the shared one.
//!
//! # How a change reaches every session
//!
//! The shared config is replaced in place behind an atomic generation counter
//! rather than pushed to each session, so a session cannot miss an update
//! because nobody remembered to notify it. Every session caches the settings it
//! resolved together with the generation it resolved them at;
//! [`Session::refresh_config`] compares that against the current generation and is
//! called for you by [`Session::process_key`]. In the steady state that costs
//! one atomic load, with no lock and no allocation.
//!
//! The counter is not only a cache check. Changing the shared settings can move
//! the rendered word even when no key was pressed, so a session has to be able
//! to notice that it happened: [`Session::refresh_config`] reports it, and
//! `process_key` turns it into a [`Result::Changed`] so the frontend re-reads
//! the word.

mod config;
mod factory;
mod session;

pub use config::{Config, Settings, SharedConfig};
pub use factory::SessionFactory;
pub use session::Session;
