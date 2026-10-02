//! Unit tests for [`Session`].
//!
//! Kept in-crate: they drive `SessionConfig`'s `pub(crate)` setters and read the
//! generation counter, which an integration test compiling as an external
//! caller cannot see.

use super::{Session, SessionConfig, SharedSessionConfig};
use crate::keymap::DefaultKeymap;
use crate::phonology::TonePlacement;
use crate::session::Settings;

/// A session on the default keymap, which is what these tests are about.
type TestSession = Session<DefaultKeymap<'static>>;

/// Default shared config for tests: Telex + Modern.
fn shared() -> SharedSessionConfig<DefaultKeymap<'static>> {
    SharedSessionConfig::new(SessionConfig::from_keymap(
        Settings::default(),
        DefaultKeymap::telex(),
    ))
}

/// A session with `input` typed in. Reads `shared` rather than making its own,
/// so a test can observe the config the session has resolved.
fn session_with(shared: &SharedSessionConfig<DefaultKeymap<'static>>, input: &str) -> TestSession {
    let mut session = Session::new(shared.clone());
    for ch in input.chars() {
        session.insert(ch);
    }
    session
}

/// A session with "hoas" typed in: the two tone-placement schemes render it
/// differently (`hoá` Modern, `hóa` Old), which is what makes it the probe for
/// whether a read resolved the config.
fn session_with_hoas(shared: &SharedSessionConfig<DefaultKeymap<'static>>) -> TestSession {
    session_with(shared, "hoas")
}

/// `(rendered, raw)`: whether each caret can move one position left.
fn can_left(session: &TestSession) -> (bool, bool) {
    let reachable = session.can_move_cursor_left();
    (*reachable.rendered(), *reachable.raw())
}

/// `(rendered, raw)`: whether each caret can move one position right.
fn can_right(session: &TestSession) -> (bool, bool) {
    let reachable = session.can_move_cursor_right();
    (*reachable.rendered(), *reachable.raw())
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

    /// A config-dependent operation adopts a pending change on its own, without an
    /// explicit `pull_config`.
    #[test]
    fn a_config_dependent_operation_resolves_the_config_itself() {
        let shared = shared();
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
        let shared = shared();
        let mut session = session_with_hoas(&shared);
        let resolved_before = session.config().tone_placement();
        let start = session.rendered_cursor();

        shared.update(|c| c.set_tone_placement(TonePlacement::Old));

        assert_eq!(can_left(&session), (true, true), "left is available");
        assert!(session.move_cursor_left_by(1).rendered());
        assert_eq!(can_left(&session), (true, true), "and again");
        assert_eq!(
            can_right(&session),
            (true, true),
            "one character now sits right of the caret"
        );
        assert!(session.move_cursor_left_by(1).rendered(), "left twice");
        assert!(session.move_cursor_right_by(1).rendered(), "then right");
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
        let shared = shared();
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
        let shared = shared();
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

    /// A session with `input` typed in, on a config no test here inspects.
    fn typed(input: &str) -> TestSession {
        session_with(&shared(), input)
    }

    /// The rendered length counts the syllable, not the keys that built it.
    #[test]
    fn rendered_len_counts_the_syllable_not_the_keystrokes() {
        let session = typed("chaof");
        assert_eq!(session.raw_cursor(), 5);
        assert_eq!(session.rendered_len(), 4, "`chaof` renders as `chào`");
    }

    #[test]
    fn rendered_len_is_zero_when_empty() {
        assert_eq!(typed("").rendered_len(), 0);
    }

    /// A dead buffer renders verbatim, so its length is the keystroke count.
    #[test]
    fn rendered_len_counts_a_dead_buffer_verbatim() {
        // `z` is not a vowel/onset/coda character, so the word goes dead and the
        // rest is recorded as typed.
        let session = typed("azxy");
        assert_eq!(session.rendered_len(), 4);
        assert_eq!(session.raw().len(), 4, "and so does the raw buffer");
    }

    /// The raw buffer counts keys, so no transform ever shortens it.
    #[test]
    fn the_raw_buffer_counts_the_keystrokes_verbatim() {
        let session = typed("chaof");
        assert_eq!(session.raw().len(), 5, "5 keys, 4 rendered characters");
        assert_eq!(typed("").raw().len(), 0);
    }

    /// An empty buffer can be moved in neither direction.
    #[test]
    fn an_empty_buffer_cannot_move() {
        let session = typed("");
        assert_eq!(can_left(&session), (false, false));
        assert_eq!(can_right(&session), (false, false));
    }

    /// With the caret at the end, only left is available; after one left move,
    /// both are.
    #[test]
    fn can_move_follows_the_caret() {
        let mut session = typed("tan");

        assert_eq!(
            can_left(&session),
            (true, true),
            "the caret starts at the end"
        );
        assert_eq!(
            can_right(&session),
            (false, false),
            "nothing sits to the right of the end"
        );

        session.move_cursor_left_by(1);

        assert_eq!(can_left(&session), (true, true));
        assert_eq!(
            can_right(&session),
            (true, true),
            "one character now sits to the right of the caret"
        );
    }

    /// Moving past the start leaves the caret there, so left stops being offered
    /// while right stays available. Moving past the end is the mirror image.
    #[test]
    fn moving_past_an_end_clamps_and_stops_offering() {
        let mut session = typed("tan");

        session.move_cursor_left_by(9);
        assert_eq!(session.rendered_cursor(), 0, "clamped at the start");
        assert_eq!(can_left(&session), (false, false), "nowhere left to go");
        assert_eq!(can_right(&session), (true, true));

        session.move_cursor_right_by(9);
        assert_eq!(session.rendered_cursor(), 3, "clamped at the end");
        assert_eq!(can_left(&session), (true, true));
        assert_eq!(can_right(&session), (false, false), "nowhere right to go");
    }

    /// The rendered caret and the raw caret are separate positions; a transform
    /// collapses two keys into one character, so they do not have to agree.
    #[test]
    fn the_two_carets_are_independent_positions() {
        let session = typed("chaof");

        assert_eq!(session.raw_cursor(), 5);
        assert_eq!(session.rendered_cursor(), 4);
        assert_eq!(session.rendered_len(), 4);
    }
}
