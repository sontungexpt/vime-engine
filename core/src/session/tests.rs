//! Unit tests for [`SharedSessionConfig`].
//!
//! Kept in-crate: they drive `SessionConfig`'s `pub(crate)` setters, which an
//! integration test compiling as an external caller cannot see.

use super::{Session, SessionConfig, SharedSessionConfig};
use crate::keymap::DefaultKeymap;
use crate::phonology::TonePlacement;
use crate::session::Settings;

/// Default shared config for tests: Telex + Modern.
fn shared() -> SharedSessionConfig<DefaultKeymap<'static>> {
    SharedSessionConfig::new(SessionConfig::from_keymap(
        Settings::default(),
        DefaultKeymap::telex(),
    ))
}

/// A session with "hoas" typed in.
fn session_with_hoas(
    shared: &SharedSessionConfig<DefaultKeymap<'static>>,
) -> Session<DefaultKeymap<'static>> {
    let mut session = Session::new(shared.clone());
    for ch in "hoas".chars() {
        session.insert(ch);
    }
    session
}

mod config_updates {
    use super::*;

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
}

mod session_config_resolution {
    use super::*;

    fn shared_config() -> SharedSessionConfig<DefaultKeymap<'static>> {
        shared()
    }

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
}

mod buffer_queries {
    use super::*;

    fn session_with(raw: &str) -> Session<DefaultKeymap<'static>> {
        let shared = shared();
        let mut session = Session::new(shared);
        for ch in raw.chars() {
            session.insert(ch);
        }
        session
    }

    /// The rendered length counts the syllable, not the keys that built it.
    #[test]
    fn rendered_len_counts_the_syllable_not_the_keystrokes() {
        let session = session_with("chaof");
        assert_eq!(session.raw_cursor(), 5);
        assert_eq!(session.rendered_len(), 4, "`chaof` renders as `chào`");
    }

    #[test]
    fn rendered_len_is_zero_when_empty() {
        assert_eq!(session_with("").rendered_len(), 0);
    }

    /// A dead buffer renders verbatim, so its length is the keystroke count.
    #[test]
    fn rendered_len_counts_a_dead_buffer_verbatim() {
        // `z` is not a vowel/onset/coda character, so the word goes dead and the
        // rest is recorded as typed.
        let session = session_with("azxy");
        assert_eq!(session.rendered_len(), 4);
    }

    /// An empty buffer can be moved in neither direction.
    #[test]
    fn an_empty_buffer_cannot_move() {
        let session = session_with("");

        assert!(!*session.can_move_cursor_left().rendered());
        assert!(!*session.can_move_cursor_right().rendered());
    }

    /// With the caret at the end, only left is available; after one left move,
    /// both are.
    #[test]
    fn can_move_follows_the_caret() {
        let mut session = session_with("tan");

        assert!(
            *session.can_move_cursor_left().rendered(),
            "caret starts at the end"
        );
        assert!(
            !*session.can_move_cursor_right().rendered(),
            "nothing sits to the right of the end"
        );

        session.move_cursor_left_by(1);

        assert!(*session.can_move_cursor_left().rendered());
        assert!(
            *session.can_move_cursor_right().rendered(),
            "one character now sits to the right of the caret"
        );
    }

    /// Moving past the start leaves the caret there, so left stops being offered
    /// while right stays available.
    #[test]
    fn can_move_left_stops_at_the_start() {
        let mut session = session_with("tan");

        session.move_cursor_left_by(9);

        assert_eq!(session.rendered_cursor(), 0, "clamped at the start");
        assert!(!*session.can_move_cursor_left().rendered());
        assert!(*session.can_move_cursor_right().rendered());
    }

    /// The rendered caret and the raw caret are separate positions; a transform
    /// collapses two keys into one character, so they do not have to agree.
    #[test]
    fn the_two_carets_are_independent_positions() {
        let session = session_with("chaof");

        assert_eq!(session.raw_cursor(), 5);
        assert_eq!(session.rendered_cursor(), 4);
        assert_eq!(session.rendered_len(), 4);
    }
}
