//! The one place a C configuration is read and turned into an engine config.
//!
//! # Why conversion lives alone here
//!
//! Every entry point that accepts a `VimeConfig` routes through
//! [`VimeConfig::read`], so there is exactly one answer to "what does this C
//! struct mean" and no entry point can disagree with another about it.
//!
//! # Why the raw view exists
//!
//! [`VimeConfig`] holds [`VimeInputMethod`] and [`VimeTonePlacement`], which are
//! Rust enums — and reading a Rust enum out of memory that holds a value outside
//! its variants is instant undefined behaviour, no matter how carefully the
//! value is checked afterwards. A C caller is free to put `99` in that field.
//!
//! So incoming bytes are read through [`RawVimeConfig`], which spells both
//! fields as `u32`, and only ever turned into an enum after [`VimeInputMethod::from_raw`]
//! / [`VimeTonePlacement::from_raw`] have matched it against a known variant.
//! The two views are asserted to be layout-identical below, so the read sees
//! exactly the bytes the caller wrote.
//!
//! [`VimeConfig::read`] returning `None` for an unknown discriminant is what
//! lets every caller reject a bad config with the NULL / `false` the header
//! documents, instead of inventing an enum value.

use vime_engine::composition::syllable::SyllableContext;
use vime_engine::phonology::TonePlacement;
use vime_engine::{Config, DefaultKeymap, Settings};

/// The engine this C ABI is fixed to: the three bundled keymaps, statically
/// dispatched. A C caller cannot supply a keymap, so there is nothing to be
/// generic over, and `Box<dyn Keymap>` would only add a vtable hop to the
/// keystroke path.
pub(crate) type Keymap = DefaultKeymap<'static>;

/// The engine config a C `VimeConfig` converts to.
pub(crate) type EngineConfig = Config<Keymap>;

/// Complete configuration used by sessions and factories.
///
/// Mirrors the C `VimeConfig` struct exactly: same fields, same order, same
/// types, same layout (asserted below). The factory holds one shared
/// configuration; a session follows it unless it has taken a private one.
#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct VimeConfig {
    pub auto_restore_english: bool,
    pub input_method: VimeInputMethod,
    pub tone_placement: VimeTonePlacement,
}

/// The C `VimeConfig` as raw bytes, for reading a caller's copy.
///
/// `u32` where [`VimeConfig`] has an enum, so that an out-of-range discriminant
/// is a value to check rather than undefined behaviour to suffer. See the module
/// docs.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct RawVimeConfig {
    auto_restore_english: bool,
    input_method: u32,
    tone_placement: u32,
}

impl VimeConfig {
    /// The configuration a C caller gets from `VIME_CONFIG_INIT` and from a NULL
    /// pointer: Telex, modern tone placement, English auto-restore on.
    pub const fn init() -> Self {
        Self {
            auto_restore_english: true,
            input_method: VimeInputMethod::Telex,
            tone_placement: VimeTonePlacement::Modern,
        }
    }

    /// Reads and validates a caller's configuration.
    ///
    /// A NULL pointer means [`VimeConfig::init`], so "no configuration" and
    /// `VIME_CONFIG_INIT` cannot be told apart by a C caller. An unknown enum
    /// discriminant is `None`: the caller reports the NULL / `false` the header
    /// documents for a bad config instead of guessing a value.
    ///
    /// # Safety
    ///
    /// `config` must be NULL or point to a readable, correctly aligned
    /// `VimeConfig` that stays readable for the duration of the call.
    pub(crate) unsafe fn read(config: *const Self) -> Option<Self> {
        // SAFETY: the caller guarantees the pointer is NULL or readable. The
        // read goes through `RawVimeConfig`, so an unknown discriminant is a
        // `u32` rather than an invalid enum, and both enum fields are validated
        // below before they are used as enums.
        let Some(raw) = (unsafe { config.cast::<RawVimeConfig>().as_ref() }).copied() else {
            return Some(Self::init());
        };
        Some(Self {
            auto_restore_english: raw.auto_restore_english,
            input_method: VimeInputMethod::from_raw(raw.input_method)?,
            tone_placement: VimeTonePlacement::from_raw(raw.tone_placement)?,
        })
    }

    /// Converts to the engine's own configuration.
    ///
    /// Consumes a `VimeConfig` that has already been through [`Self::read`], so
    /// the enums are known-good and this cannot fail.
    pub(crate) fn to_engine_config(self) -> EngineConfig {
        EngineConfig::new(
            Settings {
                auto_restore_english: self.auto_restore_english,
            },
            SyllableContext::new(
                self.input_method.to_keymap(),
                self.tone_placement.to_tone_placement(),
            ),
        )
    }
}

impl Default for VimeConfig {
    #[inline]
    fn default() -> Self {
        Self::init()
    }
}

/// The input method a session parses under.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimeInputMethod {
    #[default]
    Telex = 1,
    Vni = 2,
    Viqr = 3,
}

impl VimeInputMethod {
    /// Matches a C value against the known variants, rejecting anything else.
    #[inline]
    pub(crate) const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            1 => Some(Self::Telex),
            2 => Some(Self::Vni),
            3 => Some(Self::Viqr),
            _ => None,
        }
    }

    /// The keymap this method names.
    #[inline]
    const fn to_keymap(self) -> Keymap {
        match self {
            Self::Telex => Keymap::telex(),
            Self::Vni => Keymap::vni(),
            Self::Viqr => Keymap::viqr(),
        }
    }
}

/// Which vowel a tone mark is written on.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimeTonePlacement {
    #[default]
    Modern = 1,
    Old = 2,
}

impl VimeTonePlacement {
    /// Matches a C value against the known variants, rejecting anything else.
    #[inline]
    pub(crate) const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            1 => Some(Self::Modern),
            2 => Some(Self::Old),
            _ => None,
        }
    }

    /// The engine convention this names.
    #[inline]
    const fn to_tone_placement(self) -> TonePlacement {
        match self {
            Self::Modern => TonePlacement::Modern,
            Self::Old => TonePlacement::Old,
        }
    }
}

// ─────────────────────────── ABI layout locks ───────────────────────────
//
// `#[repr(C)]` makes the layout *predictable*, not *checked*. These assertions
// are the check: they fail the build rather than letting a field move under a
// C caller that was compiled against a different idea of the struct.
//
// The C sizes assumed are the mainstream ones: `bool` is one byte, an enum whose
// values all fit in `int` is four bytes with four-byte alignment, and pointers
// and `size_t` are eight bytes on a 64-bit target. `tests/abi.rs` re-checks these
// at run time, and `tests/c_abi.rs` compiles a real C program against the header
// so the assumption is verified by the C compiler rather than only asserted here.

const _: () = {
    use core::mem::{align_of, offset_of, size_of};

    // C: { bool; enum; enum; } -> 1 byte, 3 bytes of padding, then two 4-byte
    // enums. `bool` is 0/1 in both languages, so no translation is needed.
    assert!(size_of::<VimeConfig>() == 12);
    assert!(align_of::<VimeConfig>() == 4);
    assert!(offset_of!(VimeConfig, auto_restore_english) == 0);
    assert!(offset_of!(VimeConfig, input_method) == 4);
    assert!(offset_of!(VimeConfig, tone_placement) == 8);

    // The raw view must be able to stand in for the typed one field for field.
    assert!(size_of::<RawVimeConfig>() == size_of::<VimeConfig>());
    assert!(align_of::<RawVimeConfig>() == align_of::<VimeConfig>());
    assert!(
        offset_of!(RawVimeConfig, auto_restore_english)
            == offset_of!(VimeConfig, auto_restore_english)
    );
    assert!(offset_of!(RawVimeConfig, input_method) == offset_of!(VimeConfig, input_method));
    assert!(offset_of!(RawVimeConfig, tone_placement) == offset_of!(VimeConfig, tone_placement));

    // A C enum is `int`-sized, so the discriminants have to be 4 bytes wide and
    // have to keep the values the header writes.
    assert!(size_of::<VimeInputMethod>() == 4);
    assert!(size_of::<VimeTonePlacement>() == 4);
    assert!(VimeInputMethod::Telex as u32 == 1);
    assert!(VimeInputMethod::Vni as u32 == 2);
    assert!(VimeInputMethod::Viqr as u32 == 3);
    assert!(VimeTonePlacement::Modern as u32 == 1);
    assert!(VimeTonePlacement::Old as u32 == 2);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_header_discriminant_is_accepted() {
        for raw in 1..=3 {
            assert!(VimeInputMethod::from_raw(raw).is_some(), "{raw}");
        }
        for raw in 1..=2 {
            assert!(VimeTonePlacement::from_raw(raw).is_some(), "{raw}");
        }
    }

    /// A C caller is not bound by the header, so 0 and 4 must both be refused
    /// rather than treated as some default variant.
    #[test]
    fn out_of_range_discriminants_are_refused() {
        for raw in [0u32, 4, 5, u32::MAX, u32::MAX - 1] {
            assert_eq!(VimeInputMethod::from_raw(raw), None, "{raw}");
            assert_eq!(VimeTonePlacement::from_raw(raw), None, "{raw}");
        }
    }

    #[test]
    fn a_null_pointer_reads_as_the_header_init() {
        // SAFETY: NULL is one of the two forms `read` accepts.
        let read = unsafe { VimeConfig::read(core::ptr::null()) };
        assert_eq!(read, Some(VimeConfig::init()));
        assert_eq!(
            read.unwrap().to_engine_config(),
            VimeConfig::default().to_engine_config()
        );
    }

    #[test]
    fn a_good_config_survives_the_round_trip() {
        for method in [
            VimeInputMethod::Telex,
            VimeInputMethod::Vni,
            VimeInputMethod::Viqr,
        ] {
            for tone in [VimeTonePlacement::Modern, VimeTonePlacement::Old] {
                for flag in [true, false] {
                    let config = VimeConfig {
                        auto_restore_english: flag,
                        input_method: method,
                        tone_placement: tone,
                    };
                    // SAFETY: the pointer is to a live value of this type.
                    let read = unsafe { VimeConfig::read(&config) };
                    assert_eq!(read, Some(config));

                    let engine = config.to_engine_config();
                    assert_eq!(engine.settings.auto_restore_english, flag);
                    assert_eq!(engine.context.tone_placement(), tone.to_tone_placement());
                }
            }
        }
    }

    #[test]
    fn a_bad_discriminant_rejects_the_whole_config() {
        // Same bytes `VimeConfig::init` would produce, with one discriminant
        // replaced by a value no variant claims.
        for raw in [
            RawVimeConfig {
                auto_restore_english: true,
                input_method: 99,
                tone_placement: 1,
            },
            RawVimeConfig {
                auto_restore_english: true,
                input_method: 1,
                tone_placement: 99,
            },
        ] {
            // SAFETY: `RawVimeConfig` is layout-identical to `VimeConfig` (asserted
            // above), so these are exactly the bytes a C caller would have written.
            let read = unsafe { VimeConfig::read((&raw as *const RawVimeConfig).cast()) };
            assert_eq!(read, None);
        }
    }
}
