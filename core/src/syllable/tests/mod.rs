//! Unit tests for the syllable builder.
//!
//! Two entry points exercise the builder's public API:
//!
//! * [`push`] — the data-driven corpus through `BuildingSyllable::push`.
//! * [`insert`] — `BuildingSyllable::insert` at an explicit cursor.
//! * [`remove`] — `BuildingSyllable::remove` at an explicit index.
//! * [`lifecycle`] — the two-phase `Syllable` state machine
//!   (building → dead → building again).
//! * [`render`] — rendered output: tone placement and the precomposed codec.
//!
//! The shared assertion harness lives in [`common`] (the `ExpectedSyllable`
//! model and `check_syllable_eq`); the push behaviour data and its runner live
//! in [`corpus`].

mod common;
mod corpus;
mod insert;
mod lifecycle;
mod push;
mod remove;
mod render;
