//! Shared configuration state: one `Config` every following session reads.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use super::config::Config;
use crate::keymap::Keymap;

/// A `resolved` value that cannot be a real generation, so a fresh session is
/// forced to resolve once. Generations count up from `0` and the counter is
/// bumped by a config change, so this is unreachable in practice.
pub(crate) const UNRESOLVED: u64 = u64::MAX;

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
    /// shared config sees them from its next config-dependent operation.
    ///
    /// Returns the new generation.
    #[inline]
    pub fn replace(&self, config: Config<KM>) -> u64 {
        self.edit(|current| *current = config)
    }

    /// Mutates the shared settings in place, for a caller that has to read
    /// before it writes.
    ///
    /// A [`Config`]'s fields are private, so a partial change is made by
    /// mutating the one value inside the lock rather than by building a second
    /// config and handing it to [`Self::replace`]. The lock is held across the
    /// whole closure, so the generation published at the end always describes
    /// the value the closure left behind.
    ///
    /// Returns the new generation.
    #[inline]
    pub fn update<F>(&self, update: F) -> u64
    where
        F: FnOnce(&mut Config<KM>),
    {
        self.edit(update)
    }

    /// Runs `edit` against the locked config, then publishes a new generation.
    ///
    /// The single place a config is written, so the ordering the readers rely
    /// on is stated once: the value lands before the counter is published, so a
    /// session that observes the new generation is guaranteed to read the new
    /// value rather than the old one.
    ///
    /// The lock is dropped before the counter is bumped — the guard is scoped to
    /// the block — so a reader that has already seen the new generation is
    /// never made to wait behind this writer.
    #[inline(always)]
    fn edit<F>(&self, edit: F) -> u64
    where
        F: FnOnce(&mut Config<KM>),
    {
        {
            let mut config = self
                .state
                .current
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            edit(&mut config);
        }

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
