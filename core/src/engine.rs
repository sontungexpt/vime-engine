//! The engine: a shared configuration, and the sessions created from it.

use crate::config::Config;
use crate::keymap::{DefaultKeymap, Keymap};
use crate::session::{Session, SessionConfig, SharedConfig};

/// Holds the [`SharedConfig`] that sessions are created from, and mints them.
///
/// This is not a typing buffer. A keystroke belongs to a [`Session`], and an
/// application with one buffer needs one session, while an application with
/// several — a window per conversation, a per-app setting, a test fixture per
/// case — creates one session per buffer. Nothing here changes with the number:
/// creating a single session and creating a hundred are the same call.
///
/// The engine exists to own the settings those sessions have in common. Holding
/// them here rather than in each session is what lets one
/// [`Engine::set_config`] reach every session at once; see [`SharedConfig`] for
/// how a session notices.
///
/// ```
/// use vime_engine::{Config, DefaultKeymap, Engine, Key, KeyEvent, KeyStates, SessionConfig};
///
/// let engine = Engine::from_keymap(Config::default(), DefaultKeymap::telex());
///
/// let mut first = engine.new_session();
/// let mut second = engine.new_session();
///
/// // Both were typed under Telex, so both see this.
/// engine.set_config(SessionConfig::from_keymap(
///     Config::default(),
///     DefaultKeymap::vni(),
/// ));
///
/// for session in [&mut first, &mut second] {
///     session.process_key(KeyEvent { key: Key::Character('a'), states: KeyStates::empty() });
/// }
/// ```
pub struct Engine<KM: Keymap> {
    shared: SharedConfig<KM>,
}

impl<KM: Keymap> Engine<KM>
where
    KM: Clone + PartialEq,
{
    /// Creates an engine whose sessions all start from `config`.
    #[inline]
    pub fn new(config: SessionConfig<KM>) -> Self {
        Self {
            shared: SharedConfig::new(config),
        }
    }

    /// Creates an engine whose sessions all start from `config` and `keymap`,
    /// using the modern tone-placement convention.
    #[inline]
    pub fn from_keymap(config: Config, keymap: KM) -> Self {
        Self::new(SessionConfig::from_keymap(config, keymap))
    }

    /// The config every session follows unless it has taken a private one.
    #[inline]
    pub fn shared(&self) -> &SharedConfig<KM> {
        &self.shared
    }

    /// Replaces the config shared by every session, and returns the new
    /// generation.
    ///
    /// Sessions pick the change up at their next [`Session::process_key`] or
    /// [`Session::refresh`], so there is nothing to call on each of them. A
    /// session holding a private config is not one of "every session" and keeps
    /// its own settings.
    #[inline]
    pub fn set_config(&self, config: SessionConfig<KM>) -> u64 {
        self.shared.replace(config)
    }

    /// Creates an empty session that follows the shared config.
    #[inline]
    pub fn new_session(&self) -> Session<KM> {
        Session::new(self.shared.clone())
    }

    /// Creates an empty session with its own settings, which do not change when
    /// the shared config does.
    ///
    /// The session still knows this engine, so
    /// [`Session::clear_private_config`] returns it to the shared config as it
    /// stands by then.
    #[inline]
    pub fn new_session_with(&self, config: SessionConfig<KM>) -> Session<KM> {
        Session::with_config_on(self.shared.clone(), config)
    }
}

impl Engine<DefaultKeymap<'static>> {
    // ------------------------------------------------------ convenience ctor

    /// Creates an engine whose sessions start from the given configuration and
    /// the Telex keymap.
    #[inline]
    pub fn telex(config: Config) -> Self {
        Self::from_keymap(config, DefaultKeymap::telex())
    }

    /// Creates an engine whose sessions start from the given configuration and
    /// the VNI keymap.
    #[inline]
    pub fn vni(config: Config) -> Self {
        Self::from_keymap(config, DefaultKeymap::vni())
    }

    /// Creates an engine whose sessions start from the given configuration and
    /// the VIQR keymap.
    #[inline]
    pub fn viqr(config: Config) -> Self {
        Self::from_keymap(config, DefaultKeymap::viqr())
    }
}
