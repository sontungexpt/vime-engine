//! Unit tests for [`SharedSessionConfig`].
//!
//! Kept in-crate: they drive `SessionConfig`'s `pub(crate)` setters, which an
//! integration test compiling as an external caller cannot see.

use super::{Session, SessionConfig, SharedSessionConfig};
use crate::keymap::DefaultKeymap;
use crate::phonology::TonePlacement;
use crate::session::Settings;

fn shared() -> SharedSessionConfig<DefaultKeymap<'static>> {
    SharedSessionConfig::new(SessionConfig::from_keymap(
        crate::session::Settings::default(),
        DefaultKeymap::telex(),
    ))
}

/// `update` changes one field, leaves the rest alone, and returns a generation
/// describing the value it left behind.
#[test]
fn update_swaps_the_tone_placement_only() {
    let shared = shared();
    assert_eq!(shared.generation(), 0);

    let generation = shared.update(|config| config.set_tone_placement(TonePlacement::Old));

    assert_eq!(generation, 1);
    assert_eq!(shared.generation(), 1);
    let after = shared.snapshot();
    assert_eq!(after.tone_placement(), TonePlacement::Old);
    assert_eq!(after.keymap(), &DefaultKeymap::telex());
}

/// The same for the keymap: a read-modify-write of a private field.
#[test]
fn update_swaps_the_keymap_only() {
    let shared = shared();

    assert_eq!(
        shared.update(|config| config.set_keymap(DefaultKeymap::vni())),
        1
    );
    let after = shared.snapshot();
    assert_eq!(after.keymap(), &DefaultKeymap::vni());
    assert_eq!(after.tone_placement(), TonePlacement::Modern);
}

/// The closure sees the live config, so it can read what it is replacing.
#[test]
fn update_sees_the_current_value() {
    let shared = shared();

    shared.update(|config| {
        let seen = config.tone_placement();
        config.set_tone_placement(match seen {
            TonePlacement::Modern => TonePlacement::Old,
            TonePlacement::Old => TonePlacement::Modern,
        });
    });

    assert_eq!(shared.snapshot().tone_placement(), TonePlacement::Old);
}

/// Every `update` publishes a distinct generation, however small the change.
#[test]
fn each_update_bumps_the_generation() {
    let shared = shared();

    assert_eq!(shared.update(|c| c.set_keymap(DefaultKeymap::telex())), 1);
    assert_eq!(
        shared.update(|c| c.set_tone_placement(TonePlacement::Old)),
        2
    );
    assert_eq!(shared.generation(), 2);
}

// ────────────── Who resolves the configuration, and when ──────────────
// A `Session` resolves configuration itself, at the start of every
// config-dependent operation; a config-independent one must leave the resolved
// config exactly as it found it.

/// A shared config holding Telex on the modern scheme.
fn shared_config() -> SharedSessionConfig<DefaultKeymap<'static>> {
    SharedSessionConfig::new(SessionConfig::from_keymap(
        Settings::default(),
        DefaultKeymap::telex(),
    ))
}

/// A session on that config, holding the word `hoas`.
fn session_with_hoas(
    shared: &SharedSessionConfig<DefaultKeymap<'static>>,
) -> Session<DefaultKeymap<'static>> {
    let mut session = Session::new(shared.clone());
    for ch in "hoas".chars() {
        session.insert(ch);
    }
    session
}

/// A config-dependent operation adopts a pending change on its own, without an
/// explicit `pull_config`.
#[test]
fn a_config_dependent_operation_resolves_the_config_itself() {
    let shared = shared_config();
    let mut session = session_with_hoas(&shared);
    assert_eq!(session.rendered().iter().collect::<String>(), "hoá");

    shared.update(|c| c.set_tone_placement(TonePlacement::Old));

    // No explicit pull: `rendered` is the thing that must resolve the change.
    assert_eq!(session.rendered().iter().collect::<String>(), "hóa");
}

/// Cursor movement reads no configuration, so it must not adopt a pending
/// change: a later config-independent read would misreport what the session
/// has resolved.
#[test]
fn cursor_movement_does_not_resolve_the_config() {
    let shared = shared_config();
    let mut session = session_with_hoas(&shared);
    let resolved_before = session.config().tone_placement();
    let start = session.rendered_cursor();

    shared.update(|c| c.set_tone_placement(TonePlacement::Old));

    assert!(
        session.move_cursor_left_by(1).rendered(),
        "left is available"
    );
    assert!(session.move_cursor_left_by(1).rendered(), "and again");
    assert!(
        session.move_cursor_right_by(1).rendered(),
        "right is available"
    );
    assert_eq!(session.rendered_cursor(), start - 1);

    assert_eq!(
        session.config().tone_placement(),
        resolved_before,
        "cursor movement must leave the resolved config alone"
    );
}

/// Nor do the raw-keystroke reads, which are the raw buffer verbatim.
#[test]
fn raw_reads_do_not_resolve_the_config() {
    let shared = shared_config();
    let session = session_with_hoas(&shared);
    let resolved_before = session.config().tone_placement();

    shared.update(|c| c.set_tone_placement(TonePlacement::Old));

    let mut out = String::new();
    session.write_raw_to(&mut out);
    assert_eq!(out, "hoas");
    assert_eq!(session.raw().iter().collect::<String>(), "hoas");
    assert!(session.is_phonotactically_valid());

    assert_eq!(
        session.config().tone_placement(),
        resolved_before,
        "raw reads must leave the resolved config alone"
    );
}

/// `pull_config` reports that a config was adopted without re-rendering: the
/// word only moves when a config-dependent operation reads it.
#[test]
fn pull_config_reports_adoption_without_rendering() {
    let shared = shared_config();
    let mut session = session_with_hoas(&shared);
    shared.update(|c| c.set_tone_placement(TonePlacement::Old));

    assert!(
        session.pull_config(),
        "a moved generation means a config was adopted"
    );
    assert!(
        !session.pull_config(),
        "the generation is resolved now, so a second pull is silent"
    );
}
