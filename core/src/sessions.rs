// //! A registry of open sessions.
// //!
// //! [`SessionFactory`] makes sessions but does not keep them: it hands each one out and
// //! forgets it, which is the right shape when the thing you already own — an fcitx
// //! `InputContext`, a window, a test case — *is* the session. This module is for
// //! when something else should hold the sessions instead.
// //!
// //! # Why sharing a session needs a lock
// //!
// //! Two owners of a buffer is `Arc<T>` plus `&mut T`, and `Arc<T>` only ever
// //! hands out `&T`. There is no way around that: sharing a mutable buffer
// //! requires interior mutability. So a registered session is an
// //! `Arc<Mutex<Session>>`, and the registry and the caller hold one each.
// //!
// //! What that costs is worth being exact about, because it is the whole design:
// //!
// //! - The **keystroke path never touches the registry lock.** A caller types
// //!   through its own `Arc`, so one input context can never block another, and
// //!   the registry's lock is uncontended by construction no matter how many
// //!   sessions are open.
// //! - The per-session lock is uncontended whenever a session has the single
// //!   owner it is meant to have, which is the normal case.
// //! - The alternative — handing sessions out by value, as [`SessionFactory`] does —
// //!   needs no lock at all, and is still the better choice when the caller can
// //!   own the session outright. This is the trade made when it cannot.
// //!
// //! # Settings do not come through here
// //!
// //! A settings change is a [`SharedConfig::replace`], which every session picks
// //! up on its own next keystroke. The registry does not walk its sessions to push
// //! it, so adding or dropping a session can never make one miss an update, and
// //! the cost of a settings change does not grow with the number of sessions.
// //!
// //! A caller that genuinely needs to reach every buffer at once is what
// //! [`Sessions::for_each`] is for.
//
// use std::sync::{Arc, Mutex};
//
// use crate::keymap::Keymap;
// use crate::session::{Config, Session, Settings, SharedConfig};
//
// /// Names one registered session.
// ///
// /// Ids are not reused: a closed id stays closed rather than going on to name
// /// whatever session is opened next, so a stale id is always "not found" instead
// /// of silently reaching a different buffer.
// #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
// pub struct SessionId(u64);
//
// impl SessionId {
//     /// The number behind this id.
//     ///
//     /// Exposed for a frontend that stores ids in a C-facing structure; it is
//     /// not needed to use a session, which is held directly.
//     #[inline]
//     pub const fn get(self) -> u64 {
//         self.0
//     }
// }
//
// /// A registered session, owned by the registry and by its caller.
// pub type SessionRef<KM> = Arc<Mutex<Session<KM>>>;
//
// /// A set of open sessions, with the settings they share by default.
// ///
// /// Create with [`Sessions::new`], or with [`Sessions::from_engine`] to keep the
// /// settings of an existing [`SessionFactory`]. Keep the returned
// /// [`SessionRef`](type@SessionRef) — that is the thing a caller types through.
// pub struct Sessions<KM: Keymap> {
//     shared: SharedConfig<KM>,
//     /// Behind an `Arc` so that cloning a `Sessions` gives a handle to the *same*
//     /// set rather than an empty lookalike, which is the whole reason to clone
//     /// one.
//     open: Arc<Mutex<Entries<KM>>>,
// }
//
// struct Entries<KM: Keymap> {
//     /// The next id to hand out. Ids are never reused, so a closed id stays
//     /// closed for good.
//     next_id: u64,
//     /// Open sessions, in the order they were opened.
//     live: Vec<(SessionId, SessionRef<KM>)>,
// }
//
// impl<KM: Keymap> Clone for Sessions<KM> {
//     /// Clones a handle to the same set of sessions.
//     ///
//     /// A second name for the same registry, not a copy of it: a session opened
//     /// through either is visible to both. This is how a frontend hands the
//     /// registry to whichever component needs it.
//     #[inline]
//     fn clone(&self) -> Self {
//         Self {
//             shared: self.shared.clone(),
//             open: Arc::clone(&self.open),
//         }
//     }
// }
//
// impl<KM: Keymap> Sessions<KM>
// where
//     KM: Clone + PartialEq,
// {
//     /// Opens an empty set whose sessions share `config`.
//     pub fn new(config: Config<KM>) -> Self {
//         Self {
//             shared: SharedConfig::new(config),
//             open: Arc::new(Mutex::new(Entries {
//                 next_id: 0,
//                 live: Vec::new(),
//             })),
//         }
//     }
//
//     /// Opens an empty set whose sessions follow an existing engine's settings.
//     ///
//     /// The settings are *shared*, not copied, so a later
//     /// [`SessionFactory::set_config`](crate::SessionFactory::set_config) reaches these sessions
//     /// too. Two owners of one [`SharedConfig`] is the normal case rather than a
//     /// special one; only the registry itself is this set's own.
//     pub fn from_engine(engine: &crate::SessionFactory<KM>) -> Self {
//         Self {
//             shared: engine.config().clone(),
//             open: Arc::new(Mutex::new(Entries {
//                 next_id: 0,
//                 live: Vec::new(),
//             })),
//         }
//     }
//
//     // ------------------------------------------------------------- sessions
//
//     /// Opens a session that follows the shared config, and returns the id that
//     /// names it here along with the handle to type through.
//     pub fn open(&self) -> (SessionId, SessionRef<KM>) {
//         let session = SessionRef::new(Mutex::new(Session::new(self.shared.clone())));
//         let mut open = self.lock();
//         let id = SessionId(open.next_id);
//         open.next_id += 1;
//         open.live.push((id, Arc::clone(&session)));
//         (id, session)
//     }
//
//     /// Opens a session with its own settings, which do not change when the
//     /// shared config does.
//     pub fn open_with(&self, config: Config<KM>) -> (SessionId, SessionRef<KM>) {
//         let session = SessionRef::new(Mutex::new(Session::with_config_on_shared(
//             self.shared.clone(),
//             config,
//         )));
//         let mut open = self.lock();
//         let id = SessionId(open.next_id);
//         open.next_id += 1;
//         open.live.push((id, Arc::clone(&session)));
//         (id, session)
//     }
//
//     /// The session registered under `id`, or `None` if it is not open.
//     pub fn get(&self, id: SessionId) -> Option<SessionRef<KM>> {
//         self.lock()
//             .live
//             .iter()
//             .find(|(open_id, _)| *open_id == id)
//             .map(|(_, session)| Arc::clone(session))
//     }
//
//     /// Closes `id`, dropping the registry's own reference.
//     ///
//     /// The session itself lives on for as long as a caller still holds its
//     /// [`SessionRef`], which is what makes closing safe to do while a keystroke
//     /// is in flight. Returns whether `id` was open.
//     pub fn close(&self, id: SessionId) -> bool {
//         let mut open = self.lock();
//         let Some(at) = open.live.iter().position(|(open_id, _)| *open_id == id) else {
//             return false;
//         };
//         open.live.remove(at);
//         true
//     }
// }
//
// impl<KM: Keymap> Sessions<KM> {
//     // ---------------------------------------------------------------- state
//
//     /// How many sessions are open here.
//     pub fn len(&self) -> usize {
//         self.lock().live.len()
//     }
//
//     /// Whether no session is open here.
//     pub fn is_empty(&self) -> bool {
//         self.lock().live.is_empty()
//     }
//
//     /// The ids of the open sessions, in the order they were opened.
//     pub fn ids(&self) -> Vec<SessionId> {
//         self.lock().live.iter().map(|(id, _)| *id).collect()
//     }
//
//     // ------------------------------------------------------------- settings
//
//     /// The config every session here follows unless it has taken a private one.
//     #[inline]
//     pub fn shared(&self) -> &SharedConfig<KM> {
//         &self.shared
//     }
//
//     /// Replaces the config shared by every session here, and returns the new
//     /// generation.
//     ///
//     /// This does not walk the sessions. Each one picks the change up on its own
//     /// next keystroke or [`Session::refresh`], so a session opened a moment
//     /// later and one that has been idle for an hour end up in the same place,
//     /// and neither can be left behind.
//     #[inline]
//     pub fn set_config(&self, config: Config<KM>) -> u64 {
//         self.shared.replace(config)
//     }
//
//     /// Replaces the config shared by every session here, from an engine
//     /// [`Config`] and a keymap.
//     #[inline]
//     pub fn set_keymap(&self, config: Settings, keymap: KM) -> u64 {
//         self.set_config(Config::from_keymap(config, keymap))
//     }
//
//     // ------------------------------------------------------------------ ids
//
//     /// Reads the highest id this set has handed out, or `None` if it never
//     /// opened a session.
//     ///
//     /// The next [`Sessions::open`] uses `id + 1`. A caller that has to persist
//     /// an id across runs wants this rather than the count, which drops as
//     /// sessions close.
//     pub fn next(&self) -> Option<SessionId> {
//         self.next_id().checked_sub(1).map(SessionId)
//     }
//
//     /// The id the next [`Sessions::open`] will hand out, or `0` if none has
//     /// been. Unbounded, because cloning a handle must not need the `KM` bounds
//     /// that opening a session does.
//     fn next_id(&self) -> u64 {
//         self.lock().next_id
//     }
// }
//
// impl<KM: Keymap> Sessions<KM> {
//     /// The registry lock, poisoned or not.
//     fn lock(&self) -> std::sync::MutexGuard<'_, Entries<KM>> {
//         self.open
//             .lock()
//             .unwrap_or_else(|poisoned| poisoned.into_inner())
//     }
// }
//
// impl<KM: Keymap> Sessions<KM>
// where
//     KM: Clone + PartialEq,
// {
//     // ------------------------------------------------------------ broadcast
//
//     /// Applies `visit` to every open session, and returns how many were
//     /// reached.
//     ///
//     /// The one operation that does have to walk the list. A settings change does
//     /// not come through here; this is for the things a shared config cannot
//     /// express, like clearing every buffer.
//     ///
//     /// Sessions are visited one at a time and each lock is held only for the
//     /// duration of the call, so a visitor that types into a session would
//     /// deadlock — take the work instead.
//     pub fn for_each<F>(&self, mut visit: F) -> usize
//     where
//         F: FnMut(&mut Session<KM>),
//     {
//         let live: Vec<SessionRef<KM>> = self
//             .lock()
//             .live
//             .iter()
//             .map(|(_, s)| Arc::clone(s))
//             .collect();
//         for session in &live {
//             // A poisoned lock means a caller panicked while holding the
//             // session. The session is still a whole `Session`, and a bulk
//             // operation is not worth escalating a panic into, so take the
//             // buffer and carry on.
//             let mut session = session
//                 .lock()
//                 .unwrap_or_else(|poisoned| poisoned.into_inner());
//             visit(&mut session);
//         }
//         live.len()
//     }
//
//     /// Clears every open buffer.
//     pub fn reset_all(&self) -> usize {
//         self.for_each(|session| {
//             session.reset();
//         })
//     }
// }
//
// impl<KM: Keymap> std::fmt::Debug for Sessions<KM> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Sessions")
//             .field("len", &self.len())
//             .finish_non_exhaustive()
//     }
// }
