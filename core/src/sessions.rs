// //! A registry of open sessions, for callers that cannot own them outright.
// //!
// //! A registered session is an `Arc<Mutex<Session>>`: sharing a mutable buffer
// //! needs interior mutability. The keystroke path types through the caller's
// //! own `Arc` and never touches the registry lock; a settings change is picked
// //! up by each session on its own next keystroke, so nothing walks the list.
//
// use std::sync::{Arc, Mutex};
//
// use crate::keymap::Keymap;
// use crate::session::{Config, Session, Settings, SharedConfig};
//
// /// Names one registered session. Ids are never reused, so a stale id is
// /// always "not found" rather than silently reaching another buffer.
// #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
// pub struct SessionId(u64);
//
// impl SessionId {
//     /// The number behind this id, for frontends that store ids in a
//     /// C-facing structure.
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
// /// Keep the returned [`SessionRef`](type@SessionRef) — that is what a caller types through.
// pub struct Sessions<KM: Keymap> {
//     shared: SharedConfig<KM>,
//     /// Behind an `Arc` so that cloning `Sessions` yields a handle to the
//     /// *same* set, not a copy.
//     open: Arc<Mutex<Entries<KM>>>,
// }
//
// struct Entries<KM: Keymap> {
//     /// The next id to hand out; ids are never reused.
//     next_id: u64,
//     /// Open sessions, in the order they were opened.
//     live: Vec<(SessionId, SessionRef<KM>)>,
// }
//
// impl<KM: Keymap> Clone for Sessions<KM> {
//     /// Clones a handle to the same set of sessions, not a copy: a session
//     /// opened through either handle is visible to both.
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
//     /// Opens an empty set that shares an existing engine's settings, so a
//     /// later [`SessionFactory::set_config`] reaches these sessions too.
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
//     /// Opens a session following the shared config, returning its id and the
//     /// handle to type through.
//     pub fn open(&self) -> (SessionId, SessionRef<KM>) {
//         let session = SessionRef::new(Mutex::new(Session::new(self.shared.clone())));
//         let mut open = self.lock();
//         let id = SessionId(open.next_id);
//         open.next_id += 1;
//         open.live.push((id, Arc::clone(&session)));
//         (id, session)
//     }
//
//     /// Opens a session with settings of its own, which later shared-config
//     /// changes do not touch.
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
//     /// Closes `id`, dropping the registry's own reference; the session lives
//     /// on while any caller holds its [`SessionRef`], so closing mid-keystroke
//     /// is safe. Returns whether `id` was open.
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
//     /// Replaces the config every session here follows, returning the new
//     /// generation. Sessions are not walked: each picks the change up on its own
//     /// next keystroke or [`Session::refresh`].
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
//     /// The highest id handed out, or `None` if nothing was ever opened. Prefer
//     /// this over the count when persisting an id: the count drops as sessions
//     /// close. The next [`Sessions::open`] uses `id + 1`.
//     pub fn next(&self) -> Option<SessionId> {
//         self.next_id().checked_sub(1).map(SessionId)
//     }
//
//     /// The id the next [`Sessions::open`] hands out, or `0`. Unbounded, so
//     /// cloning a handle needs no `KM` bounds.
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
//     /// Applies `visit` to every open session, returning how many were reached.
//     ///
//     /// The one operation that walks the list; a settings change does not come
//     /// through here. Each lock is held only for its own call, so a visitor that
//     /// types into a session would deadlock — take the work instead.
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
//             // A poisoned lock only means a caller panicked mid-visit; the
//             // session is still whole, so take the buffer and carry on.
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
