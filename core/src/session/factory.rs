//! Session factory: holds shared config and creates sessions.
//!
//! A session factory owns the shared configuration and creates sessions from it.
//! Sessions follow the shared config unless given a private one.
//! [`SessionFactory::set_config`] changes the settings that all subsequently
//! created sessions (and existing ones following the shared config) will use.
//!
//! ```
//! use vime_engine::{Config, DefaultKeymap, SessionFactory, Key, KeyEvent, KeyStates, Settings};
//!
//! let factory = SessionFactory::from_keymap(Settings::default(), DefaultKeymap::telex());
//!
//! let mut first = factory.new_session();
//! let mut second = factory.new_session();
//!
//! // Both were typed under Telex, so both see this.
//! factory.set_config(Config::from_keymap(
//!     Settings::default(),
//!     DefaultKeymap::vni(),
//! ));
//!
//! for session in [&mut first, &mut second] {
//!     session.process_key(KeyEvent { key: Key::Character('a'), states: KeyStates::empty() });
//! }
//! ```
use crate::keymap::{DefaultKeymap, Keymap};
use crate::session::{Config, Session, Settings, SharedConfig};

/// Creates sessions from a shared config.
///
/// Use [`SessionFactory::set_config`] to change the settings that all
/// subsequently created sessions (and existing ones following the shared config)
/// will use.
pub struct SessionFactory<KM: Keymap> {
    config: SharedConfig<KM>,
}

impl<KM: Keymap> SessionFactory<KM> {
    /// Creates a factory whose sessions all start from `config`.
    #[inline]
    pub fn new(config: Config<KM>) -> Self {
        Self {
            config: SharedConfig::new(config),
        }
    }

    /// Creates a factory whose sessions all start from `settings` and `keymap`,
    /// using the modern tone-placement convention.
    #[inline]
    pub fn from_keymap(settings: Settings, keymap: KM) -> Self {
        Self::new(Config::from_keymap(settings, keymap))
    }

    /// The config every session follows unless it has taken a private one.
    #[inline]
    pub fn config(&self) -> Config<KM> {
        self.config.snapshot()
    }

    /// Replaces the config shared by every session, and returns the new
    /// generation.
    ///
    /// Sessions pick the change up at their next [`Session::process_key`] or
    /// [`Session::refresh_config`], so there is nothing to call on each of them.
    /// A session holding a private config is not one of "every session" and keeps
    /// its own settings.
    #[inline]
    pub fn set_config(&self, config: Config<KM>) -> u64 {
        self.config.replace(config)
    }
}

impl<KM: Keymap + PartialEq> SessionFactory<KM> {
    /// Creates an empty session that follows the shared config.
    #[inline]
    pub fn new_session(&self) -> Session<KM> {
        Session::new(self.config.clone())
    }

    /// Creates an empty session with its own settings, which do not change when
    /// the shared config does.
    ///
    /// The session still knows this factory, so
    /// [`Session::clear_private_config`] returns it to the shared config as it
    /// stands by then.
    #[inline]
    pub fn new_session_with(&self, config: Config<KM>) -> Session<KM> {
        Session::with_config_on_shared(self.config.clone(), config)
    }
}

impl SessionFactory<DefaultKeymap<'static>> {
    // ------------------------------------------------------ convenience ctor

    /// Creates a factory whose sessions start from the given settings and
    /// the Telex keymap.
    #[inline]
    pub fn telex(settings: Settings) -> Self {
        Self::from_keymap(settings, DefaultKeymap::telex())
    }

    /// Creates a factory whose sessions start from the given settings and
    /// the VNI keymap.
    #[inline]
    pub fn vni(settings: Settings) -> Self {
        Self::from_keymap(settings, DefaultKeymap::vni())
    }

    /// Creates a factory whose sessions start from the given settings and
    /// the VIQR keymap.
    #[inline]
    pub fn viqr(settings: Settings) -> Self {
        Self::from_keymap(settings, DefaultKeymap::viqr())
    }
}
