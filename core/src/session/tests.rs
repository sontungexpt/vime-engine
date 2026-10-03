//! Unit tests for [`Session`].
//!
//! Kept in-crate: they drive `SessionConfig`'s `pub(crate)` setters and read the
//! generation counter, which an integration test compiling as an external
//! caller cannot see.

use super::{Session, SessionConfig, SharedSessionConfig};
use crate::keymap::DefaultKeymap;
use crate::phonology::TonePlacement;
use crate::session::Settings;

type TestSession = Session<DefaultKeymap<'static>>;

fn shared() -> SharedSessionConfig<DefaultKeymap<'static>> {
    SharedSessionConfig::new(SessionConfig::from_keymap(
        Settings::default(),
        DefaultKeymap::telex(),
    ))
}

fn session_with(shared: &SharedSessionConfig<DefaultKeymap<'static>>, input: &str) -> TestSession {
    let mut s = Session::new(shared.clone());
    for ch in input.chars() { s.insert(ch); }
    s
}

fn session_with_hoas(shared: &SharedSessionConfig<DefaultKeymap<'static>>) -> TestSession {
    session_with(shared, "hoas")
}

fn can_left(s: &TestSession) -> (bool, bool) {
    let r = s.can_move_cursor_left();
    (*r.rendered(), *r.raw())
}
fn can_right(s: &TestSession) -> (bool, bool) {
    let r = s.can_move_cursor_right();
    (*r.rendered(), *r.raw())
}

mod config_updates {
    use super::*;

    #[test]
    fn update_tone_only() {
        let s = shared();
        assert_eq!(s.generation(), 0);
        let gen = s.update(|c| c.set_tone_placement(TonePlacement::Old));
        assert_eq!(gen, 1);
        assert_eq!(s.generation(), 1);
        let after = s.snapshot();
        assert_eq!(after.tone_placement(), TonePlacement::Old);
        assert_eq!(after.keymap(), &DefaultKeymap::telex());
    }

    #[test]
    fn update_keymap_only() {
        let s = shared();
        assert_eq!(s.update(|c| c.set_keymap(DefaultKeymap::vni())), 1);
        let after = s.snapshot();
        assert_eq!(after.keymap(), &DefaultKeymap::vni());
        assert_eq!(after.tone_placement(), TonePlacement::Modern);
    }

    #[test]
    fn update_sees_current_value() {
        let s = shared();
        s.update(|c| {
            let seen = c.tone_placement();
            c.set_tone_placement(match seen {
                TonePlacement::Modern => TonePlacement::Old,
                TonePlacement::Old => TonePlacement::Modern,
            });
        });
        assert_eq!(s.snapshot().tone_placement(), TonePlacement::Old);
    }

    #[test]
    fn each_update_bumps_generation() {
        let s = shared();
        assert_eq!(s.update(|c| c.set_keymap(DefaultKeymap::telex())), 1);
        assert_eq!(s.update(|c| c.set_tone_placement(TonePlacement::Old)), 2);
        assert_eq!(s.generation(), 2);
    }
}

mod config_resolution {
    use super::*;

    #[test]
    fn rendered_resolves_pending_config() {
        let s = shared();
        let mut ss = session_with_hoas(&s);
        assert_eq!(ss.rendered().iter().collect::<String>(), "hoá");
        s.update(|c| c.set_tone_placement(TonePlacement::Old));
        assert_eq!(ss.rendered().iter().collect::<String>(), "hóa");
    }

    #[test]
    fn cursor_movement_does_not_resolve() {
        let s = shared();
        let mut ss = session_with_hoas(&s);
        let before = ss.config().tone_placement();
        let start = ss.rendered_cursor();

        s.update(|c| c.set_tone_placement(TonePlacement::Old));

        assert_eq!(can_left(&ss), (true, true));
        assert!(ss.move_cursor_left_by(1).rendered());
        assert_eq!(can_left(&ss), (true, true));
        assert_eq!(can_right(&ss), (true, true));
        assert!(ss.move_cursor_left_by(1).rendered());
        assert!(ss.move_cursor_right_by(1).rendered());
        assert_eq!(ss.rendered_cursor(), start - 1);
        assert_eq!(ss.config().tone_placement(), before);
    }

    #[test]
    fn raw_reads_do_not_resolve() {
        let s = shared();
        let ss = session_with_hoas(&s);
        let before = ss.config().tone_placement();
        s.update(|c| c.set_tone_placement(TonePlacement::Old));

        let mut out = String::new();
        ss.write_raw_to(&mut out);
        assert_eq!(out, "hoas");
        assert_eq!(ss.raw().iter().collect::<String>(), "hoas");
        assert!(ss.is_phonotactically_valid());
        assert_eq!(ss.config().tone_placement(), before);
    }

    #[test]
    fn pull_config_reports_adoption() {
        let s = shared();
        let mut ss = session_with_hoas(&s);
        s.update(|c| c.set_tone_placement(TonePlacement::Old));
        assert!(ss.pull_config());
        assert!(!ss.pull_config());
    }
}

mod buffer_queries {
    use super::*;

    fn typed(input: &str) -> TestSession {
        session_with(&shared(), input)
    }

    #[test]
    fn rendered_len_counts_syllable() {
        let s = typed("chaof");
        assert_eq!(s.raw_cursor(), 5);
        assert_eq!(s.rendered_len(), 4);
    }

    #[test]
    fn rendered_len_empty() {
        assert_eq!(typed("").rendered_len(), 0);
    }

    #[test]
    fn rendered_len_dead_buffer() {
        let s = typed("azxy");
        assert_eq!(s.rendered_len(), 4);
        assert_eq!(s.raw().len(), 4);
    }

    #[test]
    fn raw_len_counts_keystrokes() {
        let s = typed("chaof");
        assert_eq!(s.raw().len(), 5);
        assert_eq!(typed("").raw().len(), 0);
    }

    #[test]
    fn empty_buffer_cannot_move() {
        let s = typed("");
        assert_eq!(can_left(&s), (false, false));
        assert_eq!(can_right(&s), (false, false));
    }

    #[test]
    fn caret_movement_follows_position() {
        let mut s = typed("tan");
        assert_eq!(can_left(&s), (true, true));
        assert_eq!(can_right(&s), (false, false));
        s.move_cursor_left_by(1);
        assert_eq!(can_left(&s), (true, true));
        assert_eq!(can_right(&s), (true, true));
    }

    #[test]
    fn movement_clamps_at_bounds() {
        let mut s = typed("tan");
        s.move_cursor_left_by(9);
        assert_eq!(s.rendered_cursor(), 0);
        assert_eq!(can_left(&s), (false, false));
        assert_eq!(can_right(&s), (true, true));

        s.move_cursor_right_by(9);
        assert_eq!(s.rendered_cursor(), 3);
        assert_eq!(can_left(&s), (true, true));
        assert_eq!(can_right(&s), (false, false));
    }

    #[test]
    fn rendered_and_raw_cursors_differ() {
        let s = typed("chaof");
        assert_eq!(s.raw_cursor(), 5);
        assert_eq!(s.rendered_cursor(), 4);
        assert_eq!(s.rendered_len(), 4);
    }
}