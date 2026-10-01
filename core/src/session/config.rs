//! What a session parses and renders under: one immutable `SessionConfig`, or
//! that same value behind a generation counter in `SharedSessionConfig`, so a
//! replacement reaches following sessions without notification.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::keymap::Keymap;
use crate::phonology::TonePlacement;

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

/// A session's settings, keymap, and tone-placement convention. Fields are
/// private, so a config is built whole: [`Self::new`], [`Self::from_keymap`],
/// or a mutation through `SharedSessionConfig::update`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SessionConfig<KM: Keymap> {
    settings: Settings,

    keymap: KM,

    tone_placement: TonePlacement,
}

impl<KM: Keymap> SessionConfig<KM> {
    /// Pairs engine [`Settings`] with a keymap and a tone-placement scheme.
    #[inline]
    pub const fn new(settings: Settings, keymap: KM, tone_placement: TonePlacement) -> Self {
        Self {
            settings,
            keymap,
            tone_placement,
        }
    }

    /// Pairs engine [`Settings`] with a keymap on the modern tone-placement
    /// convention.
    #[inline]
    pub fn from_keymap(settings: Settings, keymap: KM) -> Self {
        Self::new(settings, keymap, TonePlacement::Modern)
    }

    /// The session-level settings.
    #[inline]
    pub const fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The keymap input is parsed under.
    #[inline]
    pub const fn keymap(&self) -> &KM {
        &self.keymap
    }

    /// The tone-placement convention used when rendering.
    #[inline]
    pub const fn tone_placement(&self) -> TonePlacement {
        self.tone_placement
    }

    /// Replaces the keymap, leaving the rest alone. Reached only through
    /// `SharedSessionConfig::update`, which locks and bumps the generation.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn set_keymap(&mut self, keymap: KM) {
        self.keymap = keymap;
    }

    /// Replaces the tone-placement convention, leaving the rest alone. Reached
    /// only through `SharedSessionConfig::update`.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn set_tone_placement(&mut self, tone_placement: TonePlacement) {
        self.tone_placement = tone_placement;
    }
}

// ─────────────────────── the shared half ───────────────────────
// One immutable value held on behalf of every session, plus a counter that
// says when it changed.

/// Marker carried until a session resolves a generation of its own, so the
/// shared config is read before the cache is trusted. Never a real generation
/// (they count from `0`): it closes the race where a replacement lands while a
/// session is being constructed.
pub(crate) const UNRESOLVED_GENERATION: u64 = u64::MAX;

/// State shared by every session that follows this config.
struct SharedState<KM: Keymap> {
    /// Bumped by every write; read on the keystroke path, so it is an atomic
    /// rather than a field behind the lock.
    generation: AtomicU64,
    current: RwLock<SessionConfig<KM>>,
}

/// Settings shared by every session that has not taken a private config.
/// Cloning is a refcount bump, so each session can hold one and still observe
/// a later [`SharedSessionConfig::replace`].
pub struct SharedSessionConfig<KM: Keymap> {
    state: Arc<SharedState<KM>>,
}

impl<KM: Keymap> Clone for SharedSessionConfig<KM> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl<KM: Keymap> SharedSessionConfig<KM> {
    /// Shares `config` with every session that does not override it.
    #[inline]
    pub fn new(config: SessionConfig<KM>) -> Self {
        Self {
            state: Arc::new(SharedState {
                generation: AtomicU64::new(0),
                current: RwLock::new(config),
            }),
        }
    }

    /// The current settings' generation, in one atomic load. Compare against an
    /// earlier read to see whether anything moved.
    #[inline]
    pub fn generation(&self) -> u64 {
        self.state.generation.load(Ordering::Acquire)
    }

    /// Replaces the shared settings, so every following session sees them from
    /// its next config-dependent operation. Returns the new generation.
    #[inline]
    pub fn replace(&self, config: SessionConfig<KM>) -> u64 {
        self.edit(|current| *current = config)
    }

    /// Mutates the shared settings in place, for a caller that has to read
    /// before it writes. The lock spans the whole closure, so the generation
    /// published at the end describes the value it left behind. Returns the new
    /// generation.
    #[inline]
    pub fn update<F>(&self, update: F) -> u64
    where
        F: FnOnce(&mut SessionConfig<KM>),
    {
        self.edit(update)
    }

    /// Runs `edit` on the locked config, then publishes a new generation — the
    /// one place a config is written. The value lands before the counter, so a
    /// reader that sees the new generation reads the new value; the lock drops
    /// before the bump, so that reader never waits behind this writer.
    #[inline(always)]
    fn edit<F>(&self, edit: F) -> u64
    where
        F: FnOnce(&mut SessionConfig<KM>),
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

impl<KM: Keymap + Clone> SharedSessionConfig<KM> {
    /// The settings in force right now. Takes the lock, so not for the
    /// keystroke path: a session reads [`SharedSessionConfig::generation`] first
    /// and only looks at the value once something moved.
    pub fn snapshot(&self) -> SessionConfig<KM> {
        // A poisoned lock means a writer panicked mid-replace; the write is one
        // assignment, so the value is still a whole config and recovering beats
        // pushing the panic out to every keystroke.
        self.state
            .current
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

impl<KM: Keymap> std::fmt::Debug for SharedSessionConfig<KM>
where
    KM: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedSessionConfig")
            .field("generation", &self.generation())
            .finish_non_exhaustive()
    }
}
