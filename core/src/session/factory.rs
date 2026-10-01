//! Sessions minted from one shared configuration.
//!
//! Every session follows the factory's settings until it is given a private
//! [`SessionConfig`]. A change to the shared config reaches the others at their
//! next config-dependent operation, so there is nothing to call on each.
//!
//! ```
//! use vime_engine::{SessionConfig, DefaultKeymap, SessionFactory, Settings};
//!
//! let factory = SessionFactory::from_keymap(Settings::default(), DefaultKeymap::telex());
//!
//! let mut first = factory.new_session();
//! let mut second = factory.new_session();
//!
//! for session in [&mut first, &mut second] {
//!     session.insert('a');
//! }
//!
//! // Both were typed under Telex, so both see this.
//! factory.set_config(SessionConfig::from_keymap(
//!     Settings::default(),
//!     DefaultKeymap::vni(),
//! ));
//!
//! for session in [&mut first, &mut second] {
//!     session.insert('b');
//! }
//! ```
use crate::keymap::{DefaultKeymap, Keymap};
use crate::session::{Session, SessionConfig, Settings, SharedSessionConfig};

/// Mints sessions that share one configuration.
pub struct SessionFactory<KM: Keymap> {
    config: SharedSessionConfig<KM>,
}

impl<KM: Keymap> SessionFactory<KM> {
    /// Creates a factory whose sessions all start from `config`.
    #[inline]
    pub fn new(config: SessionConfig<KM>) -> Self {
        Self {
            config: SharedSessionConfig::new(config),
        }
    }

    /// Creates a factory whose sessions all start from `settings` and `keymap`,
    /// using the modern tone-placement convention.
    #[inline]
    pub fn from_keymap(settings: Settings, keymap: KM) -> Self {
        Self::new(SessionConfig::from_keymap(settings, keymap))
    }

    /// The shared config as it stands now. Takes the lock.
    #[inline]
    pub fn config(&self) -> SessionConfig<KM> {
        self.config.snapshot()
    }

    /// Mutates the shared config in place, returning the new generation.
    ///
    /// For a partial change: [`SessionConfig`]'s fields are private, so one is
    /// changed without rebuilding the whole value. Prefer [`Self::set_config`] when
    /// a whole config is in hand.
    #[inline]
    pub fn update_config(&self, update: impl FnOnce(&mut SessionConfig<KM>)) -> u64 {
        self.config.update(update)
    }

    /// Replaces the shared config, returning the new generation.
    ///
    /// A session holding a private config is not one of "every session" and keeps
    /// its own settings.
    #[inline]
    pub fn set_config(&self, config: SessionConfig<KM>) -> u64 {
        self.config.replace(config)
    }
}

impl<KM: Keymap + PartialEq> SessionFactory<KM> {
    /// Creates an empty session that follows the shared config.
    #[inline]
    pub fn new_session(&self) -> Session<KM> {
        Session::new(self.config.clone())
    }

    /// Creates a session with its own settings, unaffected by later shared changes.
    ///
    /// It still holds this factory's shared config, so
    /// [`Session::clear_private_config`] returns it to the shared config *as it
    /// stands then*.
    #[inline]
    pub fn new_session_with(&self, config: SessionConfig<KM>) -> Session<KM> {
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
