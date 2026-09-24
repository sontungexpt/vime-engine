//! Configuration-driven default keymaps, backed by compact [`KeyMask`] lookups.
//!
//! `[KeyMask::Compact]` — dense layouts (Telex, VNI): a 64-bit window `[base, base+64)`,
//! probed with a single `shrx`. `[KeyMask::Full]` — sparse layouts (VIQR, custom): a
//! split 128-bit mask (low 64-bit for ASCII 0..63, high for 64..127). `KeyMask` is
//! `#[repr(u8)]` + 64-bit payloads = exactly 16 bytes; `DefaultKeymap` fits one
//! 64-byte L1 cache line.

mod config;
mod telex;
mod viqr;
mod vni;

use crate::phonology::{RootVowel, Shape, Tone};

use super::Keymap;
pub use config::{Rules, ShapeRule, ToneRule};

/// A set of ASCII keys, represented as compactly as the key spread allows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum KeyMask {
    /// Dense layouts (Telex, VNI): keys within a 64-key window starting at `base`.
    Compact { mask: u64, base: u8 } = 0,
    /// Sparse layouts (VIQR, custom): full split 128-bit ASCII mask.
    Full { low: u64, high: u64 } = 1,
}

impl KeyMask {
    pub const EMPTY: Self = Self::Compact { mask: 0, base: 0 };

    /// Tests whether `key` is present in the mask.
    ///
    /// Compact uses `wrapping_sub` to reject out-of-range and non-ASCII keys without
    /// an explicit ASCII check; Full checks the halves (a failed `key < 128` → false).
    #[inline(always)]
    pub const fn contains(&self, key: char) -> bool {
        let k = key.to_ascii_lowercase() as u32;

        match *self {
            Self::Compact { mask, base } => {
                let offset = k.wrapping_sub(base as u32);
                offset < 64 && ((mask >> offset) & 1) != 0
            }
            Self::Full { low, high } => {
                if k < 64 {
                    ((low >> k) & 1) != 0
                } else if k < 128 {
                    ((high >> (k - 64)) & 1) != 0
                } else {
                    false
                }
            }
        }
    }
}

/// Builds the cheapest [`KeyMask`] for a set of ASCII keys.
///
/// Spread `min..max < 64` → [`KeyMask::Compact`]; otherwise [`KeyMask::Full`].
/// Keys are lowercased when folded in.
macro_rules! build_mask {
    ($items:expr, $count:expr, |$item:ident| $key:expr) => {{
        let n = $count;
        let mut min = 255u8;
        let mut max = 0u8;
        let mut i = 0;

        while i < n {
            let $item = &$items[i];
            let key = $key.to_ascii_lowercase();
            if key < min {
                min = key;
            }
            if key > max {
                max = key;
            }
            i += 1;
        }

        if n == 0 || max >= 128 {
            KeyMask::EMPTY
        } else if (max - min) < 64 {
            // Dense window: probe via a single u64 shift.
            let mut mask = 0u64;
            i = 0;
            while i < n {
                let $item = &$items[i];
                let key = $key.to_ascii_lowercase();
                mask |= 1u64 << (key - min);
                i += 1;
            }
            KeyMask::Compact { mask, base: min }
        } else {
            // Sparse: split 128-bit ASCII mask (low: 0..63, high: 64..127).
            let mut low = 0u64;
            let mut high = 0u64;
            i = 0;
            while i < n {
                let $item = &$items[i];
                let key = $key.to_ascii_lowercase();
                if key < 64 {
                    low |= 1u64 << key;
                } else if key < 128 {
                    high |= 1u64 << (key - 64);
                }
                i += 1;
            }
            KeyMask::Full { low, high }
        }
    }};
}

/// Configuration-driven key mapping backed by [`KeyMask`] lookups, declared via [`Rules`].
///
/// ASCII keys only — Unicode keys need a custom [`Keymap`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DefaultKeymap<'a> {
    rules: &'a Rules<'a>,
    tone_mask: KeyMask,
    shape_mask: KeyMask,
    stroke_mask: KeyMask,
}

impl<'a> DefaultKeymap<'a> {
    /// Creates a key mapping from a declarative configuration.
    pub const fn new(config: &'a Rules<'a>) -> Self {
        Self {
            rules: config,
            tone_mask: build_mask!(config.tones, config.tones.len(), |t| t.key),
            shape_mask: build_mask!(config.shapes, config.shapes.len(), |s| s.key),
            stroke_mask: build_mask!(config.strokes, config.strokes.len(), |k| *k),
        }
    }

    /// The Telex input method.
    #[inline(always)]
    pub const fn telex() -> Self {
        Self::new(telex::CONFIG)
    }

    /// The VNI input method.
    #[inline(always)]
    pub const fn vni() -> Self {
        Self::new(vni::CONFIG)
    }

    /// The VIQR input method.
    #[inline(always)]
    pub const fn viqr() -> Self {
        Self::new(viqr::CONFIG)
    }

    /// The underlying configuration.
    #[inline(always)]
    pub const fn rules(&self) -> &Rules<'a> {
        self.rules
    }
}

impl Keymap for DefaultKeymap<'_> {
    #[inline(always)]
    fn is_tone_key(&self, key: char) -> bool {
        self.tone_mask.contains(key)
    }

    #[inline(always)]
    fn is_shape_key(&self, key: char) -> bool {
        self.shape_mask.contains(key)
    }

    #[inline(always)]
    fn is_stroke_key(&self, key: char) -> bool {
        self.stroke_mask.contains(key)
    }

    #[inline(always)]
    fn decode_tone(&self, key: char) -> Option<Tone> {
        let lower = (u8::try_from(key).ok()?).to_ascii_lowercase();
        let tones = self.rules.tones;
        let len = tones.len();
        let mut i = 0;

        while i < len {
            if tones[i].key == lower {
                return Some(tones[i].tone);
            }
            i += 1;
        }

        None
    }

    #[inline(always)]
    fn decode_shape(&self, key: char, target: RootVowel) -> Option<Shape> {
        let lower = (u8::try_from(key).ok()?).to_ascii_lowercase();
        let shapes = self.rules.shapes;
        let len = shapes.len();
        let mut i = 0;

        while i < len {
            if shapes[i].key == lower && shapes[i].on == target {
                return Some(shapes[i].shape);
            }
            i += 1;
        }

        None
    }
}
