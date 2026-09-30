//! Session configuration: what a session parses and renders under.

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
/// mutation through `SharedConfig::update`, which holds the only handle that
/// can change one in place.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config<KM: Keymap> {
    /// SessionFactory-level settings.
    settings: Settings,

    /// Keymap used for parsing input.
    keymap: KM,

    /// Convention used when placing tones during rendering.
    tone_placement: TonePlacement,
}

impl<KM: Keymap> Config<KM> {
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
    /// `SharedConfig::update`, which holds the lock and bumps the generation
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
    /// `SharedConfig::update`.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn set_tone_placement(&mut self, tone_placement: TonePlacement) {
        self.tone_placement = tone_placement;
    }
}
