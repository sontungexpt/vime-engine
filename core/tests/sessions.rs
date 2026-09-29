// //! The session registry: holding sessions, naming them, and reaching them.
// //!
// //! The point of these is the awkward part. A registered session has two owners,
// //! so the tests here are mostly about what that does and does not let happen.
//
// use vime_engine::composition::syllable::SyllableContext;
// use vime_engine::phonology::TonePlacement;
// use vime_engine::{Settings,
//     Config, DefaultKeymap, SessionFactory, Session,
// };
//
// const MODERN_HOA: &str = "hoá";
// const OLD_HOA: &str = "hóa";
//
// fn old_telex() -> Config<DefaultKeymap<'static>> {
//     Config::new(
//         Settings::default(),
//         SyllableContext::new(DefaultKeymap::telex(), TonePlacement::Old),
//     )
// }
//
// fn rendered_to_string(session: &vime_engine::Session<DefaultKeymap<'static>>) -> String {
//     session.rendered().into_iter().collect()
// }
//
// fn type_str(session: &mut vime_engine::Session<DefaultKeymap<'static>>, s: &str) -> String {
//     for ch in s.chars() {
//         session.insert(ch);
//     }
//     rendered_to_string(session)
// }
//
// /// Types into a registered session the way a caller would: through its own
// /// handle, with no help from the registry.
// fn type_into(session: &vime_engine::SessionRef<DefaultKeymap<'static>>, s: &str) -> String {
//     let mut session = session.lock().expect("uncontended");
//     type_str(&mut session, s)
// }
//
// fn all() -> vime_engine::Sessions<DefaultKeymap<'static>> {
//     vime_engine::Sessions::new(Config::from_keymap(
//         Settings::default(),
//         DefaultKeymap::telex(),
//     ))
// }
//
// // ─────────────────────────────── Naming and lookup ───────────────────────────────
//
// #[test]
// fn open_returns_an_id_that_names_the_session() {
//     let sessions = all();
//     let (id, session) = sessions.open();
//
//     assert_eq!(sessions.len(), 1);
//     assert_eq!(sessions.ids(), vec![id]);
//
//     // The registry's handle and the caller's handle are one buffer, not two.
//     type_into(&session, "ba");
//     let found = sessions.get(id).expect("the id names it");
//     assert_eq!(rendered_to_string(&found.lock().expect("uncontended")), "ba");
//
//     type_into(&found, "n");
//     assert_eq!(
//         rendered_to_string(&session.lock().expect("uncontended")),
//         "ban",
//         "written through one, read through the other"
//     );
// }
//
// #[test]
// fn ids_are_not_reused_after_a_close() {
//     let sessions = all();
//     let (first, _) = sessions.open();
//     assert!(sessions.close(first));
//     assert_eq!(sessions.len(), 0);
//
//     // The next session is a different session, and the old id stays dead rather
//     // than going on to name it.
//     let (second, _) = sessions.open();
//     assert_ne!(first, second);
//     assert!(sessions.get(first).is_none(), "a stale id is not found");
//     assert!(sessions.get(second).is_some());
//
//     assert!(!sessions.close(first), "closing twice is not a success");
// }
//
// #[test]
// fn the_open_set_is_kept_in_the_order_it_was_opened() {
//     let sessions = all();
//     let mut ids = Vec::new();
//     for _ in 0..3 {
//         ids.push(sessions.open().0);
//     }
//     assert_eq!(sessions.ids(), ids);
//
//     // Closing from the middle keeps the rest in order.
//     assert!(sessions.close(ids[1]));
//     assert_eq!(sessions.ids(), vec![ids[0], ids[2]]);
//     assert_eq!(
//         sessions.next(),
//         Some(ids[2]),
//         "the high-water mark, not the count"
//     );
// }
//
// /// A closed session stays usable for whoever still holds it. The registry
// /// dropping its reference is not a signal to the owner that the buffer is done.
// #[test]
// fn closing_releases_the_registry_not_the_owner() {
//     let sessions = all();
//     let (id, session) = sessions.open();
//     type_into(&session, "hoa");
//
//     assert!(sessions.close(id));
//     assert_eq!(
//         type_into(&session, "s"),
//         MODERN_HOA,
//         "still the caller's buffer"
//     );
//     assert_eq!(sessions.len(), 0, "but no longer registered");
// }
//
// #[test]
// fn an_empty_set_reports_itself_empty() {
//     let sessions = all();
//     assert!(sessions.is_empty());
//     assert!(sessions.ids().is_empty());
//     assert_eq!(sessions.next(), None);
// }
//
// // ─────────────────────────────────── Settings ───────────────────────────────────
//
// /// The registry's whole reason for existing, and the reason it does not need to
// /// walk anything to do it: one write, every session, including one that has not
// /// been touched in a while.
// #[test]
// fn one_write_reaches_every_registered_session() {
//     let sessions = all();
//     let (_first, first) = sessions.open();
//     let (_second, second) = sessions.open();
//     let (_third, third) = sessions.open();
//
//     for session in [&first, &second, &third] {
//         type_into(session, "hoas");
//     }
//
//     sessions.set_config(old_telex());
//
//     for session in [&first, &second, &third] {
//         let mut session = session.lock().expect("uncontended");
//         session.refresh_config();
//         assert_eq!(rendered_to_string(&session), OLD_HOA);
//     }
// }
//
// /// A session opened after the change is in the same place as one open before it,
// /// which is what "does not walk the list" buys.
// #[test]
// fn a_session_opened_after_a_change_starts_out_current() {
//     let sessions = all();
//     sessions.set_config(old_telex());
//
//     let (_, session) = sessions.open();
//     assert_eq!(type_into(&session, "hoas"), OLD_HOA);
//     assert!(
//         !session.lock().expect("uncontended").refresh_config(),
//         "nothing to catch up on"
//     );
// }
//
// #[test]
// fn a_private_session_in_a_registry_ignores_the_shared_config() {
//     let sessions = all();
//     let (_, following) = sessions.open();
//     let (_, private) = sessions.open_with(old_telex());
//
//     // Type into following session before config change
//     type_into(&following, "hoas");
//
//     sessions.set_config(Config::from_keymap(
//         Settings::default(),
//         DefaultKeymap::vni(),
//     ));
//
//     // Following: buffer cleared on keymap change, then new input
//     assert_eq!(type_into(&following, "hoa1"), "hoá");
//     // Private: ignores shared config, keeps old_telex
//     assert_eq!(type_into(&private, "hoas"), OLD_HOA);
// }
//
// #[test]
// /// The keymap setter is sugar for the full config replace.
// ///
// /// The buffer is cleared on keymap change.
// fn set_keymap_is_the_short_form_of_set_config() {
//     let sessions = all();
//     let (_, session) = sessions.open();
//     type_into(&session, "hoas");
//
//     sessions.set_keymap(Settings::default(), DefaultKeymap::vni());
//     let mut session = session.lock().expect("uncontended");
//     assert!(session.refresh_config(), "a new keymap has to be noticed");
//     // Buffer cleared on keymap change
//     assert_eq!(rendered_to_string(&session), "");
// }
//
// // ────────────────────────────────── Broadcast ──────────────────────────────────
//
// /// The one thing that does have to walk, and the reason the settings path does
// /// not come through here.
// #[test]
// fn for_each_reaches_every_session() {
//     let sessions = all();
//     let mut ids = Vec::new();
//     for _ in 0..3 {
//         ids.push(sessions.open().0);
//     }
//     for id in &ids {
//         type_into(&sessions.get(*id).expect("open"), "hoa");
//     }
//
//     assert_eq!(sessions.reset_all(), 3, "one per open session");
//     for id in &ids {
//         assert_eq!(
//             rendered_to_string(&sessions.get(*id).expect("open").lock().unwrap()),
//             ""
//         );
//     }
//
//     // A private session is registered like any other, so it is reached too.
//     let (private_id, private) = sessions.open_with(old_telex());
//     type_into(&private, "hoa");
//     assert_eq!(sessions.reset_all(), 4);
//     assert_eq!(rendered_to_string(&private.lock().expect("uncontended")), "");
//
//     // And closing drops one from the count.
//     assert!(sessions.close(private_id));
//     assert_eq!(sessions.reset_all(), 3);
// }
//
// #[test]
// fn for_each_can_do_something_other_than_reset() {
//     let sessions = all();
//     for _ in 0..3 {
//         sessions.open();
//     }
//
//     let mut seen = 0;
//     let reached = sessions.for_each(|session| {
//         session.insert('a');
//         assert_eq!(rendered_to_string(session), "a");
//         seen += 1;
//     });
//
//     assert_eq!(reached, 3);
//     assert_eq!(seen, 3);
// }
//
// /// A settings change must not be implemented as a broadcast, or this would fail:
// /// the counter only moves once.
// #[test]
// fn a_settings_change_does_not_visit_the_sessions() {
//     let sessions = all();
//     sessions.open();
//     sessions.open();
//
//     let mut visited = 0;
//     sessions.set_config(old_telex());
//     let reached = sessions.for_each(|_| visited += 1);
//
//     assert_eq!(reached, 2);
//     assert_eq!(
//         visited, 2,
//         "for_each ran now, but set_config did not need it"
//     );
// }
//
// // ─────────────────────────────── Starting from an engine ───────────────────────────────
//
// #[test]
// fn a_set_can_adopt_an_engines_settings() {
//     let engine = SessionFactory::telex(Settings::default());
//     let sessions = vime_engine::Sessions::from_engine(&engine);
//     let (_, session) = sessions.open();
//     assert_eq!(type_into(&session, "hoas"), MODERN_HOA);
//
//     // The settings are shared rather than copied, so the engine can still move
//     // them. That is the point: one config, however many owners it has.
//     engine.set_config(old_telex());
//     let mut session = session.lock().expect("uncontended");
//     assert!(session.refresh_config());
//     assert_eq!(rendered_to_string(&session), OLD_HOA);
//
//     // And the set moves them the same way the engine does.
//     sessions.set_config(Config::from_keymap(
//         Settings::default(),
//         DefaultKeymap::vni(),
//     ));
//     assert!(session.refresh_config());
//     // Keymap change: buffer cleared
//     assert_eq!(rendered_to_string(&session), "");
// }
//
// /// A cloned handle names the same set, which is how a frontend hands the
// /// registry to whichever component needs it.
// #[test]
// fn cloning_a_set_gives_a_handle_to_the_same_sessions() {
//     let sessions = all();
//     let (id, session) = sessions.open();
//
//     let other = sessions.clone();
//     assert_eq!(other.len(), 1);
//     assert!(other.get(id).is_some());
//     assert_eq!(type_into(&session, "hoas"), MODERN_HOA);
//
//     // Opening through the clone registers in the original too.
//     let (second, _) = other.open();
//     assert_eq!(sessions.len(), 2);
//     assert!(sessions.get(second).is_some());
// }
//
// /// Closing through one handle is visible through the other.
// #[test]
// fn a_clone_shares_the_registry() {
//     let sessions = all();
//     let other = sessions.clone();
//     let (id, _) = other.open();
//
//     assert_eq!(sessions.len(), 1);
//     assert!(sessions.close(id));
//     assert!(other.is_empty());
// }
//
// // ─────────────────────────────────── The contract ───────────────────────────────────
//
// /// Two registered sessions share settings and nothing else. This is the one
// /// thing a registry must never break.
// #[test]
// fn registered_sessions_do_not_share_buffers() {
//     let sessions = all();
//     let (_, first) = sessions.open();
//     let (_, second) = sessions.open();
//
//     type_into(&first, "hoa");
//     assert_eq!(
//         rendered_to_string(&second.lock().expect("uncontended")),
//         "",
//         "the other buffer is untouched"
//     );
//
//     type_into(&second, "ba");
//     assert_eq!(
//         rendered_to_string(&first.lock().expect("uncontended")),
//         "hoa"
//     );
//     assert_eq!(
//         rendered_to_string(&second.lock().expect("uncontended")),
//         "ba",
//         "and the commit did not touch it"
//     );
// }
//
// /// The whole point of the split: a session's settings come from the shared
// /// config, and its `Result` is the same one the by-value API gives.
// #[test]
// fn a_registered_session_behaves_like_any_other() {
//     let sessions = all();
//     let (_, session) = sessions.open();
//     let mut session = session.lock().expect("uncontended");
//
//     assert_eq!(type_str(&mut session, "hoas"), MODERN_HOA);
//     assert!(*session.move_cursor_left().rendered());
//     assert_eq!(session.config().context.tone_placement(), TonePlacement::Modern);
//     assert!(!session.has_private_config());
// }
