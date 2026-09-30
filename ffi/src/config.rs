//! The one place a C configuration is read and turned into an engine config.
//!
//! # Why conversion lives alone here
//!
//! Every entry point that accepts a `VimeConfig` routes through
//! [`VimeConfig::read`], so there is exactly one answer to "what does this C
//! struct mean" and no entry point can disagree with another about it.
//!
//! # Why the read is field by field
//!
//! [`VimeConfig`] holds [`VimeInputMethod`] and [`VimeTonePlacement`], which are
//! Rust enums — and reading a Rust enum out of memory that holds a value outside
//! its variants is instant undefined behaviour, no matter how carefully the value
//! is checked afterwards. A C caller is free to put `99` in that field, and there
//! is no way to look before looking.
//!
//! So nothing here ever reads a caller's bytes as a `VimeConfig`. Each field is
//! read at its asserted offset as a plain `u32`, which has no validity
//! constraints, and only then checked against the known values. Reading a
//! field directly as the Rust enum it stands for — or, for a `bool`, as a
//! `bool` — would reintroduce exactly the hazard the enums are being avoided
//! for: those types are only valid for a subset of the bit patterns, and C is
//! free to write something else.
//!
//! [`VimeConfig::read`] returning `None` for an unknown discriminant is what
//! lets every caller reject a bad config with the NULL / `false` the header
//! documents, instead of inventing an enum value.

use vime_engine::phonology::TonePlacement;
use vime_engine::{DefaultKeymap, SessionConfig, Settings};

/// The engine this C ABI is fixed to: the three bundled keymaps, statically
/// dispatched. A C caller cannot supply a keymap, so there is nothing to be
/// generic over, and `Box<dyn Keymap>` would only add a vtable hop to the
/// keystroke path.
pub(crate) type FfiKeymap = DefaultKeymap<'static>;

/// The engine config a C `VimeConfig` converts to.
pub(crate) type FfiSessionConfig = SessionConfig<FfiKeymap>;

/// Constants matching values defined in C Header (`vime_engine.h`)
pub type VimeInputMethod = u32;
pub const VIME_INPUT_METHOD_TELEX: VimeInputMethod = 1;
pub const VIME_INPUT_METHOD_VNI: VimeInputMethod = 2;
pub const VIME_INPUT_METHOD_VIQR: VimeInputMethod = 3;

pub type VimeTonePlacement = u32;
pub const VIME_TONE_PLACEMENT_MODERN: VimeTonePlacement = 1;
pub const VIME_TONE_PLACEMENT_OLD: VimeTonePlacement = 2;

/// Complete configuration used by sessions and factories.
///
/// Mirrors the C `VimeConfig` struct exactly: same fields, same order, same
/// types, same layout (asserted below). The factory holds one shared
/// configuration; a session follows it unless it has taken a private one.
#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct VimeConfig {
    pub input_method: VimeInputMethod,
    pub tone_placement: VimeTonePlacement,
}

/// Byte offsets of the two fields, used both by [`VimeConfig::read`] and by the
/// layout assertions below, so the read and the proof that justifies it cannot
/// drift apart. If the struct ever moves a field, the assertions stop compiling.
const INPUT_METHOD_OFFSET: usize = 0;
const TONE_PLACEMENT_OFFSET: usize = 4;

impl VimeConfig {
    /// The configuration a C caller gets from `VIME_CONFIG_INIT` and from a NULL
    /// pointer: Telex, modern tone placement.
    pub const fn init() -> Self {
        Self {
            input_method: VIME_INPUT_METHOD_TELEX,
            tone_placement: VIME_TONE_PLACEMENT_MODERN,
        }
    }

    /// Reads and validates a caller's configuration.
    ///
    /// A NULL pointer means [`VimeConfig::init`], so "no configuration" and
    /// `VIME_CONFIG_INIT` cannot be told apart by a C caller. An unknown enum
    /// discriminant is `None`: the caller reports the NULL / `false` the header
    /// documents for a bad config instead of guessing a value.
    ///
    /// Every field comes back as a plain `u32` and is checked before it becomes
    /// a [`VimeConfig`], which is the only order in which the check is sound —
    /// see the module docs. The accepted values are spelled against the
    /// `VIME_*` constants rather than repeated as literals, so a value changing
    /// in the header cannot leave the check behind.
    ///
    /// Not on the keystroke path: only the four entry points that accept a
    /// `VimeConfig` come through here, so the work is four integer loads and
    /// two range checks, once per configuration change.
    ///
    /// # Safety
    ///
    /// `config` must be NULL or point to a readable, correctly aligned
    /// `VimeConfig` that stays readable for the duration of the call.
    #[inline]
    pub(crate) unsafe fn read(config: *const Self) -> Option<Self> {
        // Checked on the typed pointer, before it is used as a base for offset
        // arithmetic; the cast to `u8` below exists only to apply the offsets.
        if config.is_null() {
            return Some(Self::init());
        }
        let base = config.cast::<u8>();

        // SAFETY: the caller guarantees a readable, correctly aligned
        // `VimeConfig` for the duration of the call. `VimeConfig` is 8 bytes with
        // 4-byte alignment, both asserted below, so each `u32` read is in bounds
        // and aligned. Reading them as integers means no Rust enum is ever
        // constructed from an unvalidated value.
        let input_method = unsafe { base.add(INPUT_METHOD_OFFSET).cast::<u32>().read() };
        let tone_placement = unsafe { base.add(TONE_PLACEMENT_OFFSET).cast::<u32>().read() };

        // Enumerated against the `VIME_*` constants rather than range-checked,
        // so the accepted set is exactly the one `to_ffi_session_config` can map. A
        // range would also admit a value this match does not name, and that
        // value would reach the `unreachable!()` below — a panic on the way out
        // of a function whose whole job is to answer `None` for a bad config.
        if !matches!(
            input_method,
            VIME_INPUT_METHOD_TELEX..=VIME_INPUT_METHOD_VIQR
        ) || !matches!(
            tone_placement,
            VIME_TONE_PLACEMENT_MODERN..=VIME_TONE_PLACEMENT_OLD
        ) {
            return None;
        }

        Some(Self {
            input_method,
            tone_placement,
        })
    }

    /// Converts to the engine's own configuration.
    ///
    /// Consumes a `VimeConfig` that has already been through [`Self::read`], so
    /// the u32 values are known-good and this cannot fail.
    pub(crate) fn to_ffi_session_config(self) -> FfiSessionConfig {
        let keymap = match self.input_method {
            VIME_INPUT_METHOD_TELEX => FfiKeymap::telex(),
            VIME_INPUT_METHOD_VNI => FfiKeymap::vni(),
            VIME_INPUT_METHOD_VIQR => FfiKeymap::viqr(),
            _ => unreachable!(),
        };

        let tone_placement = match self.tone_placement {
            VIME_TONE_PLACEMENT_MODERN => TonePlacement::Modern,
            VIME_TONE_PLACEMENT_OLD => TonePlacement::Old,
            _ => unreachable!(),
        };

        FfiSessionConfig::new(Settings::default(), keymap, tone_placement)
    }
}

impl Default for VimeConfig {
    #[inline]
    fn default() -> Self {
        Self::init()
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

    // C: { u32; u32; } -> two 4-byte integers.
    assert!(size_of::<VimeConfig>() == 8);
    assert!(align_of::<VimeConfig>() == 4);
    assert!(offset_of!(VimeConfig, input_method) == INPUT_METHOD_OFFSET);
    assert!(offset_of!(VimeConfig, tone_placement) == TONE_PLACEMENT_OFFSET);

    // `VimeConfig::read` walks the caller's bytes one integer at a time, so those
    // reads need somewhere to land: the whole struct, and 4-byte alignment for
    // the two `u32`s.
    assert!(size_of::<VimeConfig>() >= TONE_PLACEMENT_OFFSET + size_of::<u32>());
    assert!(align_of::<VimeConfig>() >= size_of::<u32>());

    // Raw constants must match the header values.
    assert!(VIME_INPUT_METHOD_TELEX == 1);
    assert!(VIME_INPUT_METHOD_VNI == 2);
    assert!(VIME_INPUT_METHOD_VIQR == 3);
    assert!(VIME_TONE_PLACEMENT_MODERN == 1);
    assert!(VIME_TONE_PLACEMENT_OLD == 2);
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The bytes a C caller would have written for `(input_method,
    /// tone_placement)`, held in a buffer aligned the way a real `VimeConfig *`
    /// is, so the `u32` reads in `read` are aligned as they would be in anger.
    #[repr(C, align(4))]
    #[derive(Clone, Copy)]
    struct Bytes([u8; 8]);

    impl Bytes {
        fn new(input_method: u32, tone_placement: u32) -> Self {
            let mut bytes = [0; 8];
            bytes[INPUT_METHOD_OFFSET..][..4].copy_from_slice(&input_method.to_ne_bytes());
            bytes[TONE_PLACEMENT_OFFSET..][..4].copy_from_slice(&tone_placement.to_ne_bytes());
            Self(bytes)
        }

        /// Hands over the bytes the way a C caller would.
        fn as_config(&self) -> *const VimeConfig {
            (self as *const Bytes).cast()
        }
    }

    /// Every discriminant the header declares is accepted, read the way a C
    /// caller delivers it — from bytes, not from a typed value.
    ///
    /// This drives [`VimeConfig::read`] rather than restating its predicate. The
    /// previous version asserted `matches!(1u32, 1..=3)`, which proves only that
    /// `1` is in `1..=3`: a second copy of the check, beside the check, free to
    /// drift from it.
    #[test]
    fn every_header_discriminant_is_accepted() {
        for method in [
            VIME_INPUT_METHOD_TELEX,
            VIME_INPUT_METHOD_VNI,
            VIME_INPUT_METHOD_VIQR,
        ] {
            for tone in [VIME_TONE_PLACEMENT_MODERN, VIME_TONE_PLACEMENT_OLD] {
                let raw = Bytes::new(method, tone);
                // SAFETY: `Bytes` is the same size as `VimeConfig` and aligned the
                // same way (both asserted above), so this is exactly the pointer a C
                // caller would hand over.
                let read = unsafe { VimeConfig::read(raw.as_config()) };
                assert_eq!(
                    read,
                    Some(VimeConfig {
                        input_method: method,
                        tone_placement: tone,
                    }),
                    "input_method {method}, tone_placement {tone}"
                );
            }
        }
    }

    /// A C caller is not bound by the header, so 0 and 4 must both be refused
    /// rather than treated as some default variant.
    ///
    /// Each value is tried in both fields, because the two are read from
    /// different offsets and a check that only guarded one would still pass this
    /// loop. The extremes matter most: `u32::MAX` is the value most likely to
    /// wrap an `as`-based conversion into something that looks declared.
    #[test]
    fn out_of_range_discriminants_are_refused() {
        for raw in [0u32, 4, 5, u32::MAX, u32::MAX - 1] {
            let in_input_method = Bytes::new(raw, VIME_TONE_PLACEMENT_MODERN);
            let in_tone_placement = Bytes::new(VIME_INPUT_METHOD_TELEX, raw);

            for (field, bytes) in [
                ("input_method", &in_input_method),
                ("tone_placement", &in_tone_placement),
            ] {
                // SAFETY: as above.
                let read = unsafe { VimeConfig::read(bytes.as_config()) };
                assert_eq!(read, None, "{raw} in {field} must be refused");
            }
        }
    }

    #[test]
    fn a_null_pointer_reads_as_the_header_init() {
        // SAFETY: NULL is one of the two forms `read` accepts.
        let read = unsafe { VimeConfig::read(core::ptr::null()) };
        assert_eq!(read, Some(VimeConfig::init()));
        assert_eq!(
            read.unwrap().to_ffi_session_config(),
            VimeConfig::default().to_ffi_session_config()
        );
    }

    #[test]
    fn a_good_config_survives_the_round_trip() {
        for method in [
            VIME_INPUT_METHOD_TELEX,
            VIME_INPUT_METHOD_VNI,
            VIME_INPUT_METHOD_VIQR,
        ] {
            for tone in [VIME_TONE_PLACEMENT_MODERN, VIME_TONE_PLACEMENT_OLD] {
                let config = VimeConfig {
                    input_method: method,
                    tone_placement: tone,
                };
                // SAFETY: the pointer is to a live value of this type.
                let read = unsafe { VimeConfig::read(&config) };
                assert_eq!(read, Some(config));

                let engine = config.to_ffi_session_config();
                let expected_tone = match tone {
                    VIME_TONE_PLACEMENT_MODERN => TonePlacement::Modern,
                    VIME_TONE_PLACEMENT_OLD => TonePlacement::Old,
                    _ => unreachable!(),
                };
                assert_eq!(engine.tone_placement(), expected_tone);
            }
        }
    }

    #[test]
    fn a_bad_discriminant_rejects_the_whole_config() {
        // The bytes `VimeConfig::init` would produce, with one discriminant
        // replaced by a value no variant claims.
        for raw in [Bytes::new(99, 1), Bytes::new(1, 99)] {
            // SAFETY: `Bytes` is the same size as `VimeConfig` and aligned the
            // same way (both asserted above), so this is exactly the pointer a C
            // caller would hand over.
            let read = unsafe { VimeConfig::read(raw.as_config()) };
            assert_eq!(read, None);
        }
    }
}
