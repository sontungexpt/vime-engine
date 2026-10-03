//! Unit tests for the syllable builder.
//!
//! Each module drives one entry point — `push`, `insert`, `remove`, the
//! building/dead lifecycle or rendering — through the shared harness in
//! [`common`] and the behaviour data in [`corpus`].

mod common;
mod corpus;
mod edge_cases;
mod insert;
mod lifecycle;
mod push;
mod remove;
mod render;
