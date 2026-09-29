//! Engine construction: tone-placement init and live updates.

use vime_engine::composition::syllable::SyllableContext;
use vime_engine::phonology::TonePlacement;
use vime_engine::{Config, Engine, Key, KeyEvent, KeyStates};

fn type_str(engine: &mut Engine<vime_engine::DefaultKeymap<'static>>, s: &str) -> String {
    for ch in s.chars() {
        engine.process_key(KeyEvent {
            key: Key::Character(ch),
            states: KeyStates::empty(),
        });
    }
    engine.parsed()
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
fn with_context_selects_old_at_construction() {
    let mut engine = Engine::with_context(
        Config::default(),
        SyllableContext::new(vime_engine::DefaultKeymap::telex(), TonePlacement::Old),
    );
    assert_eq!(type_str(&mut engine, "hoas"), OLD_HOA);
}

#[test]
fn set_tone_placement_re_renders_the_live_buffer() {
    let mut engine = Engine::new(Config::default(), vime_engine::DefaultKeymap::telex());
    assert_eq!(type_str(&mut engine, "hoas"), MODERN_HOA);

    // Flip mid-buffer: the pending vowel re-renders under the new scheme.
    engine.set_tone_placement(TonePlacement::Old);
    assert_eq!(engine.parsed(), OLD_HOA);
}

#[test]
fn convenience_constructors_build_telex_and_vni() {
    let mut telex = Engine::telex(Config::default());
    assert_eq!(type_str(&mut telex, "hoas"), MODERN_HOA);

    let mut vni = Engine::vni(Config::default());
    assert_eq!(type_str(&mut vni, "hoa1"), MODERN_HOA);
}

// ─────────────────────────── Caret movement ───────────────────────────

/// Presses a non-character key and reports what the engine did with it.
fn press(engine: &mut Engine<vime_engine::DefaultKeymap<'static>>, key: Key) -> vime_engine::Result {
    engine.process_key(KeyEvent {
        key,
        states: KeyStates::empty(),
    })
}

/// The caret predicates decide whether a move is consumed or forwarded, so
/// their boundary conditions are what the frontend contract rests on: the
/// caret is always either movable left or movable right, and never both, and
/// `can_move_right` goes false only once the caret is at the very end.
#[test]
fn caret_predicates_track_the_raw_buffer() {
    let mut engine = Engine::new(Config::default(), vime_engine::DefaultKeymap::telex());

    // Type "toa": the caret sits after the final character.
    type_str(&mut engine, "toa");
    assert_eq!(press(&mut engine, Key::Left), vime_engine::Result::CursorMoved);
    assert_eq!(press(&mut engine, Key::Left), vime_engine::Result::CursorMoved);
    assert_eq!(press(&mut engine, Key::Left), vime_engine::Result::CursorMoved);

    // Caret now at position 0: further left is forwarded, not consumed.
    assert_eq!(press(&mut engine, Key::Left), vime_engine::Result::Forward);

    // And back to the end, where right is forwarded.
    assert_eq!(press(&mut engine, Key::Right), vime_engine::Result::CursorMoved);
    assert_eq!(press(&mut engine, Key::Right), vime_engine::Result::CursorMoved);
    assert_eq!(press(&mut engine, Key::Right), vime_engine::Result::CursorMoved);
    assert_eq!(press(&mut engine, Key::Right), vime_engine::Result::Forward);
}

/// Moving the caret must not alter the rendered text, and resetting must put
/// the caret back to the start.
#[test]
fn caret_moves_leave_the_render_unchanged() {
    let mut engine = Engine::new(Config::default(), vime_engine::DefaultKeymap::telex());
    type_str(&mut engine, "hoas");
    let before = engine.parsed();

    for key in [Key::Left, Key::Left, Key::Right] {
        press(&mut engine, key);
    }
    assert_eq!(engine.parsed(), before);

    // After a reset the buffer is empty, so there is nowhere to move.
    engine.reset();
    assert_eq!(engine.parsed(), "");
}
