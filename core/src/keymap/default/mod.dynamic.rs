//! KeyMask-based variant of the default keymap.
//!
//! This is a parallel, drop-in alternative to `super::DefaultKeymap`. It replaces the
//! three fixed `u128` bitmasks with a single [`KeyMask`] that selects a representation
//! at build time:
//!
//! * [`KeyMask::Compact`] — for dense layouts (Telex, VNI, ...): a 64-bit window
//!   `[base, base+64)`, probed with a single `shrx`. Cheaper to evaluate than a u128
//!   shift (which compiles to `shrd`+`shrx`+`test`+`cmov` on x86-64).
//! * [`KeyMask::Full`] — for sparse/wild layouts (VIQR, custom): the full 128-bit mask.
//!
//! Note the enum still occupies 32 bytes (16-byte alignment is required by the `u128`
//! payload), so the keymap is 96 bytes instead of the current 48. Runtime probing is
//! unaffected by that footprint.
//!
//! To use it in `crate::keymap`, expose `self::dynamic::DefaultKeymap` alongside (or
//! instead of) `super::DefaultKeymap`.

use crate::phonology::{RootVowel, Shape, Tone};

use super::{config::{Rules, ShapeRule, ToneRule}, telex, viqr, vni, Keymap};

/// A set of ASCII keys, represented as compactly as the key spread allows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyMask {
    /// Tight layout (Telex, VNI, ...): keys are assumed within a 64-key window
    /// starting at `base`. Probe = `mask >> (key - base)` — a single 64-bit shift.
    Compact { mask: u64, base: u8 },
    /// Wide layout (VIQR, unusual custom layouts): full 128-bit ASCII mask.
    Full(u128),
}

impl KeyMask {
    pub const EMPTY: Self = Self::Compact { mask: 0, base: 0 };

    /// Returns `true` if `key` (expected already lowercased) is set in the mask.
    ///
    /// `key` may be any byte (0..=255); `Compact` bounds-checks via the wrapping
    /// subtraction and the `< 64` window test, `Full` via the `< 128` gate.
    #[inline(always)]
    pub const fn contains(&self, key: u8) -> bool {
        match *self {
            Self::Compact { mask, base } => {
                let offset = key.wrapping_sub(base);
                offset < 64 && ((mask >> offset) & 1) != 0
            }
            Self::Full(mask) => key < 128 && ((mask >> key) & 1) != 0,
        }
    }
}

/// Builds the cheapest [`KeyMask`] representation for a set of ASCII keys.
///
/// When the min..max spread is under 64 keys a `Compact` window is used; otherwise
/// the mask falls back to `Full`. All keys are lowercased before being folded in.
///
/// Bits are first collected at their absolute (0..128) positions in a u128, then
/// compacted to a 64-bit window iff it fits — a single pass, no temporary buffers.
macro_rules! build_auto_mask {
    ($items:expr, $count:expr, |$item:ident| $key:expr) => {{
        let n = $count;
        let mut mask = 0u128;
        let mut min = 255u8;
        let mut max = 0u8;
        let mut i = 0;

        while i < n {
            let $item = $items[i];
            let key = $key;
            // Absolute bits only for ASCII; the min/max scan below still records any
            // out-of-range key so the resulting mask degrades to EMPTY.
            if key < 128 {
                mask |= 1u128 << key;
            }
            if key < min {
                min = key;
            }
            if key > max {
                max = key;
            }
            i += 1;
        }

        if mask == 0 || max >= 128 {
            KeyMask::EMPTY
        } else if (max - min) < 64 {
            // Dense window: drop the unused high bits and probe via a single u64 shift.
            KeyMask::Compact {
                mask: (mask >> min) as u64,
                base: min,
            }
        } else {
            // Sparse: keep the full 128-bit ASCII mask.
            KeyMask::Full(mask)
        }
    }};
}

/// Configuration-driven key mapping, backed by [`KeyMask`] lookups.
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
            tone_mask: build_auto_mask!(config.tones, config.tones.len(), |t| t.key.to_ascii_lowercase()),
            shape_mask: build_auto_mask!(config.shapes, config.shapes.len(), |s| s.key.to_ascii_lowercase()),
            stroke_mask: build_auto_mask!(config.strokes, config.strokes.len(), |k| k.to_ascii_lowercase()),
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

    /// Probes `mask` for a character (lowercased), mirroring `super::has_key`.
    ///
    /// Non-ASCII input (`u8::try_from` failure) short-circuits to `false` before the
    /// lowercasing step; the mask probe itself is branch-free apart from the
    /// predictable enum-discriminant test.
    #[inline(always)]
    fn has_key(mask: &KeyMask, input: char) -> bool {
        let Ok(byte) = u8::try_from(input) else { return false; };
        mask.contains(byte.to_ascii_lowercase())
    }
}

impl Keymap for DefaultKeymap<'_> {
    #[inline(always)]
    fn is_tone_key(&self, key: char) -> bool {
        Self::has_key(&self.tone_mask, key)
    }

    #[inline(always)]
    fn is_shape_key(&self, key: char) -> bool {
        Self::has_key(&self.shape_mask, key)
    }

    #[inline(always)]
    fn is_stroke_key(&self, key: char) -> bool {
        Self::has_key(&self.stroke_mask, key)
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