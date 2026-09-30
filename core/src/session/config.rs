//! Session configuration: what a session parses and renders under.
//!
//! Two types, one per ownership model:
//!
//! - [`SessionConfig`] — one immutable set of settings, owned by whoever holds
//!   it. A session that has taken a private config holds exactly this.
//! - [`SharedSessionConfig`] — the same value behind a generation counter, so a
//!   replacement reaches every following session without anyone having to notify
//!   them.

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

/// Complete configuration needed by a session: settings, keymap and the
/// tone-placement convention.
///
/// The keymap decodes transform keys and the tone-placement scheme picks the
/// nucleus vowel that carries the tone mark. Neither is buffered input, so both
/// sit beside the settings rather than inside the parse state.
///
/// The fields are private and reached through the accessors below, so a config
/// is only ever built whole — by [`Self::new`], [`Self::from_keymap`], or a
/// mutation through `SharedSessionConfig::update`, which holds the only handle that
/// can change one in place.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SessionConfig<KM: Keymap> {
    /// SessionFactory-level settings.
    settings: Settings,

    /// Keymap used for parsing input.
    keymap: KM,

    /// Convention used when placing tones during rendering.
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

    /// Pairs engine [`Settings`] with a keymap, using the modern
    /// tone-placement convention.
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

    /// Replaces the keymap, leaving the rest of the config alone.
    ///
    /// Reachable only from inside the crate, and in practice only through
    /// `SharedSessionConfig::update`, which holds the lock and bumps the generation
    /// around it.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn set_keymap(&mut self, keymap: KM) {
        self.keymap = keymap;
    }

    /// Replaces the tone-placement convention, leaving the rest of the config
    /// alone.
    ///
    /// Reachable only from inside the crate, and in practice only through
    /// `SharedSessionConfig::update`.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn set_tone_placement(&mut self, tone_placement: TonePlacement) {
        self.tone_placement = tone_placement;
    }
}

// ─────────────────────── the shared half ───────────────────────
//
// Everything above is one immutable value; everything below is that same value
// held once on behalf of every session, with a counter for noticing it changed.

/// The marker a session records before it has resolved a generation of its own:
/// the shared config must be re-read before the cached one is trusted.
///
/// Generations count up from `0` and only move when a config is written, so
/// `u64::MAX` is not a real one. A fresh session therefore carries this and is
/// forced to resolve exactly once, which is what stops a replacement racing
/// with construction from leaving it holding settings from a generation it never
/// named. Unreachable in practice.
pub(crate) const UNRESOLVED_GENERATION: u64 = u64::MAX;

/// Internal mutable state shared across all sessions following the same config.
struct SharedState<KM: Keymap> {
    /// Bumped by every `replace`. Read on the keystroke path, so it is an
    /// atomic rather than a field behind the lock.
    generation: AtomicU64,
    current: RwLock<SessionConfig<KM>>,
}

/// Settings shared by every session that has not taken a private config.
///
/// Cloning is a refcount bump, so each session can hold one and still observe a
/// later [`SharedSessionConfig::replace`].
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
    pub fn replace(&self, config: SessionConfig<KM>) -> u64 {
        self.edit(|current| *current = config)
    }

    /// Mutates the shared settings in place, for a caller that has to read
    /// before it writes.
    ///
    /// A [`SessionConfig`]'s fields are private, so a partial change is made by
    /// mutating the one value inside the lock rather than by building a second
    /// config and handing it to [`Self::replace`]. The lock is held across the
    /// whole closure, so the generation published at the end always describes
    /// the value the closure left behind.
    ///
    /// Returns the new generation.
    #[inline]
    pub fn update<F>(&self, update: F) -> u64
    where
        F: FnOnce(&mut SessionConfig<KM>),
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
    /// The settings in force right now.
    ///
    /// Takes the lock, so this is for a caller that is not on the keystroke
    /// path. A session does not use it: it reads [`SharedSessionConfig::generation`]
    /// first and only looks at the value when something has actually moved.
    pub fn snapshot(&self) -> SessionConfig<KM> {
        // A poisoned lock means some other thread panicked mid-replace. The
        // value inside is still a whole `SessionConfig<KM>`, because the write is a
        // single assignment, so recovering beats pushing the panic out to every
        // keystroke of every session.
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
