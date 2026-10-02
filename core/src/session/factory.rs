use crate::keymap::{DefaultKeymap, Keymap};
use crate::session::{Session, SessionConfig, Settings, SharedSessionConfig};

/// Factory that mints sessions sharing one configuration. Sessions follow the
/// factory's config until given a private [`SessionConfig`]; shared changes
/// propagate at the next config-dependent operation.
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

    /// Creates a factory from `settings` and `keymap` on the modern
    /// tone-placement convention.
    #[inline]
    pub fn from_keymap(settings: Settings, keymap: KM) -> Self {
        Self::new(SessionConfig::from_keymap(settings, keymap))
    }

    /// Current shared config (takes the lock).
    #[inline]
    pub fn config(&self) -> SessionConfig<KM> {
        self.config.snapshot()
    }

    /// Mutates the shared config in place, returning the new generation.
    /// Prefer [`Self::set_config`] for whole-config replacement.
    #[inline]
    pub fn update_config(&self, update: impl FnOnce(&mut SessionConfig<KM>)) -> u64 {
        self.config.update(update)
    }

    /// Replaces the shared config, returning the new generation. Sessions with
    /// private configs keep their own settings.
    #[inline]
    pub fn set_config(&self, config: SessionConfig<KM>) -> u64 {
        self.config.replace(config)
    }
}

impl<KM: Keymap + PartialEq> SessionFactory<KM> {
    /// Creates an empty session following the shared config.
    #[inline]
    pub fn new_session(&self) -> Session<KM> {
        Session::new(self.config.clone())
    }

    /// Creates a session with its own settings, unaffected by later shared
    /// changes. Holds this factory's shared config, so
    /// [`Session::clear_private_config`] returns it to that config *as it stands
    /// then*.
    #[inline]
    pub fn new_session_with(&self, config: SessionConfig<KM>) -> Session<KM> {
        Session::with_config_on_shared(self.config.clone(), config)
    }
}

impl SessionFactory<DefaultKeymap<'static>> {
    /// Creates a factory starting from `settings` and the Telex keymap.
    #[inline]
    pub fn telex(settings: Settings) -> Self {
        Self::from_keymap(settings, DefaultKeymap::telex())
    }

    /// Creates a factory starting from `settings` and the VNI keymap.
    #[inline]
    pub fn vni(settings: Settings) -> Self {
        Self::from_keymap(settings, DefaultKeymap::vni())
    }

    /// Creates a factory starting from `settings` and the VIQR keymap.
    #[inline]
    pub fn viqr(settings: Settings) -> Self {
        Self::from_keymap(settings, DefaultKeymap::viqr())
    }
}
