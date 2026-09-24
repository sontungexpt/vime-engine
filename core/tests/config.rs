//! Engine construction: tone-placement init and live updates.

use vime_engine::phonology::TonePlacement;
use vime_engine::{Config, Engine, Key, KeyEvent, KeyStates};

fn type_str(engine: &mut Engine<vime_engine::DefaultKeymap<'static>>, s: &str) -> String {
    for ch in s.chars() {
        engine.process_key(KeyEvent {
            key: Key::Character(ch),
            states: KeyStates::empty(),
        });
    }
    engine.rendered()
}

// "hoa" + sắc: the two schemes place the mark on different vowels
// (Modern 2-vowel rule: second vowel; Old open syllable: first vowel).
const MODERN_HOA: &str = "hoá";
const OLD_HOA: &str = "hóa";

#[test]
fn engine_defaults_to_modern_tone_placement() {
    let mut engine = Engine::new(Config::default(), vime_engine::DefaultKeymap::telex());
    assert_eq!(type_str(&mut engine, "hoas"), MODERN_HOA);
}

#[test]
fn with_tone_placement_selects_old_at_construction() {
    let mut engine = Engine::with_tone_placement(
        Config::default(),
        vime_engine::DefaultKeymap::telex(),
        TonePlacement::Old,
    );
    assert_eq!(type_str(&mut engine, "hoas"), OLD_HOA);
}

#[test]
fn set_tone_placement_re_renders_the_live_buffer() {
    let mut engine = Engine::new(Config::default(), vime_engine::DefaultKeymap::telex());
    assert_eq!(type_str(&mut engine, "hoas"), MODERN_HOA);

    // Flip mid-buffer: the pending vowel re-renders under the new scheme.
    engine.set_tone_placement(TonePlacement::Old);
    assert_eq!(engine.rendered(), OLD_HOA);
}

#[test]
fn convenience_constructors_build_telex_and_vni() {
    let mut telex = Engine::telex(Config::default());
    assert_eq!(type_str(&mut telex, "hoas"), MODERN_HOA);

    let mut vni = Engine::vni(Config::default());
    assert_eq!(type_str(&mut vni, "hoa1"), MODERN_HOA);
}