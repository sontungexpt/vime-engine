use crate::{keymap::Keymap, phonology::TonePlacement};

/// The configuration a syllable is parsed and rendered under.
///
/// The [`Keymap`] decoding transform keys, and the [`TonePlacement`] scheme
/// picking the nucleus vowel that carries the tone mark. Neither is buffered
/// input, so both are kept apart from the state, the only part that changes
/// per keystroke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyllableContext<KM: Keymap> {
    keymap: KM,
    tone_placement: TonePlacement,
}

impl<KM: Keymap> SyllableContext<KM> {
    /// Creates a context from a keymap and a tone-placement scheme.
    #[inline]
    pub const fn new(keymap: KM, tone_placement: TonePlacement) -> Self {
        Self {
            keymap,
            tone_placement,
        }
    }

    /// The active keymap.
    #[inline(always)]
    pub const fn keymap(&self) -> &KM {
        &self.keymap
    }

    /// The tone-placement scheme used when rendering.
    #[inline(always)]
    pub const fn tone_placement(&self) -> TonePlacement {
        self.tone_placement
    }
}