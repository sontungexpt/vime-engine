//! Session configuration and shared configuration state.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::composition::syllable::SyllableContext;
use crate::keymap::Keymap;
use crate::phonology::TonePlacement;

/// A `resolved` value that cannot be a real generation, so a fresh session is
/// forced to resolve once. Generations count up from `0` and the counter is
/// bumped by a config change, so this is unreachable in practice.
pub(crate) const UNRESOLVED: u64 = u64::MAX;

/// Small engine/product settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {
    // Empty Settings now
}

impl Default for Settings {
    fn default() -> Self {
        Self {}
    }
}

/// Complete configuration needed by a session: settings + parse/render context.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config<KM: Keymap> {
    /// SessionFactory-level settings.
    pub settings: Settings,
    /// The parse and render context.
    pub context: SyllableContext<KM>,
}

impl<KM: Keymap> Config<KM> {
    /// Pairs engine [`Settings`] with a parse context.
    #[inline]
    pub fn new(settings: Settings, context: SyllableContext<KM>) -> Self {
        Self { settings, context }
    }

    /// Pairs engine [`Settings`] with a keymap, using the modern
    /// tone-placement convention.
    #[inline]
    pub fn from_keymap(settings: Settings, keymap: KM) -> Self {
        Self::new(
            settings,
            SyllableContext::new(keymap, TonePlacement::Modern),
        )
    }
}

/// Internal mutable state shared across all sessions following the same config.
struct SharedState<KM: Keymap> {
    /// Bumped by every `replace`. Read on the keystroke path, so it is an
    /// atomic rather than a field behind the lock.
    generation: AtomicU64,
    current: RwLock<Config<KM>>,
}

/// Settings shared by every session that has not taken a private config.
///
/// Cloning is a refcount bump, so each session can hold one and still observe a
/// later [`SharedConfig::replace`].
pub struct SharedConfig<KM: Keymap> {
    state: Arc<SharedState<KM>>,
}

impl<KM: Keymap> Clone for SharedConfig<KM> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl<KM: Keymap> SharedConfig<KM> {
    /// Shares `config` with every session that does not override it.
    #[inline]
    pub fn new(config: Config<KM>) -> Self {
        Self {
            state: Arc::new(SharedState {
                generation: AtomicU64::new(0),
                current: RwLock::new(config),
            }),
        }
    }

    /// The generation of the current settings. Cheap; call it to find out
    /// whether anything moved since a previous read.
    #[inline]
    pub fn generation(&self) -> u64 {
        self.state.generation.load(Ordering::Acquire)
    }

    /// Replaces the shared settings, so that every session following the
    /// shared config sees them from its next [`Session::refresh_config`].
    ///
    /// Returns the new generation.
    pub fn replace(&self, config: Config<KM>) -> u64 {
        // The value lands before the counter is published, so a session that
        // observes the new generation is guaranteed to read the new value
        // rather than the old one.
        *self
            .state
            .current
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = config;
        self.state.generation.fetch_add(1, Ordering::Release) + 1
    }
}

impl<KM: Keymap + Clone> SharedConfig<KM> {
    /// The settings in force right now.
    ///
    /// Takes the lock, so this is for a caller that is not on the keystroke
    /// path. A session does not use it: it reads [`SharedConfig::generation`]
    /// first and only looks at the value when something has actually moved.
    pub fn snapshot(&self) -> Config<KM> {
        // A poisoned lock means some other thread panicked mid-replace. The
        // value inside is still a whole `Config<KM>`, because the write is a
        // single assignment, so recovering beats pushing the panic out to every
        // keystroke of every session.
        self.state
            .current
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

impl<KM: Keymap> std::fmt::Debug for SharedConfig<KM>
where
    KM: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedConfig")
            .field("generation", &self.generation())
            .finish_non_exhaustive()
    }
}
