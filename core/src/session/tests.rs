//! Unit tests for [`SharedConfig`].
//!
//! These stay inside the crate rather than moving to `tests/`: they drive
//! `Config`'s crate-private setters, which is the only way to exercise
//! `SharedConfig::update`. An integration test compiles against the crate as an
//! external caller and cannot see `pub(crate)` items.

use super::{Config, SharedConfig};
use crate::keymap::DefaultKeymap;
use crate::phonology::TonePlacement;

fn shared() -> SharedConfig<DefaultKeymap<'static>> {
    SharedConfig::new(Config::from_keymap(
        crate::session::Settings::default(),
        DefaultKeymap::telex(),
    ))
}

/// `update` changes one field and leaves the rest alone, and the generation
/// it returns describes the value it left behind.
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
