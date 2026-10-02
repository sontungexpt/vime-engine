use vime_engine::phonology::TonePlacement;
use vime_engine::{DefaultKeymap, SessionConfig, Settings};

/// Keymap used by the C ABI.
pub(crate) type FfiKeymap = DefaultKeymap<'static>;

/// Session configuration used by the C ABI.
pub(crate) type FfiSessionConfig = SessionConfig<FfiKeymap>;

/// Input method identifier for the C ABI.
pub type VimeInputMethod = u32;
pub const VIME_INPUT_METHOD_TELEX: VimeInputMethod = 1;
pub const VIME_INPUT_METHOD_VNI: VimeInputMethod = 2;
pub const VIME_INPUT_METHOD_VIQR: VimeInputMethod = 3;

/// Tone placement scheme for the C ABI.
pub type VimeTonePlacement = u32;
pub const VIME_TONE_PLACEMENT_MODERN: VimeTonePlacement = 1;
pub const VIME_TONE_PLACEMENT_OLD: VimeTonePlacement = 2;

/// C-compatible configuration (8 bytes, 4-byte aligned).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VimeConfig {
    pub input_method: VimeInputMethod,
    pub tone_placement: VimeTonePlacement,
}

impl Default for VimeConfig {
    #[inline]
    fn default() -> Self {
        Self {
            input_method: VIME_INPUT_METHOD_TELEX,
            tone_placement: VIME_TONE_PLACEMENT_MODERN,
        }
    }
}

const INPUT_METHOD_OFFSET: usize = 0;
const TONE_PLACEMENT_OFFSET: usize = 4;

impl VimeConfig {
    /// Reads and validates a C configuration from a pointer.
    ///
    /// # Safety
    /// `config` must be non-null and point to a readable, correctly aligned
    /// `VimeConfig` for the duration of the call.
    #[inline]
    pub(crate) unsafe fn read(config: *const Self) -> Option<Self> {
        let base = config.cast::<u8>();

        let input_method = unsafe { base.add(INPUT_METHOD_OFFSET).cast::<u32>().read() };
        let tone_placement = unsafe { base.add(TONE_PLACEMENT_OFFSET).cast::<u32>().read() };

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

    /// Converts a validated C configuration to an engine configuration.
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

    pub fn from_ffi_session_config(config: FfiSessionConfig) -> Self {
        Self {
            input_method: if config.keymap().is_vni() {
                VIME_INPUT_METHOD_VNI
            } else if config.keymap().is_viqr() {
                VIME_INPUT_METHOD_VIQR
            } else {
                VIME_INPUT_METHOD_TELEX
            },
            tone_placement: if config.tone_placement() == TonePlacement::Old {
                VIME_TONE_PLACEMENT_OLD
            } else {
                VIME_TONE_PLACEMENT_MODERN
            },
        }
    }
}

// ABI layout guarantees.
const _: () = {
    use core::mem::{align_of, offset_of, size_of};

    assert!(size_of::<VimeConfig>() == 8);
    assert!(align_of::<VimeConfig>() == 4);
    assert!(offset_of!(VimeConfig, input_method) == INPUT_METHOD_OFFSET);
    assert!(offset_of!(VimeConfig, tone_placement) == TONE_PLACEMENT_OFFSET);

    assert!(size_of::<VimeConfig>() >= TONE_PLACEMENT_OFFSET + size_of::<u32>());
    assert!(align_of::<VimeConfig>() >= size_of::<u32>());

    assert!(VIME_INPUT_METHOD_TELEX == 1);
    assert!(VIME_INPUT_METHOD_VNI == 2);
    assert!(VIME_INPUT_METHOD_VIQR == 3);
    assert!(VIME_TONE_PLACEMENT_MODERN == 1);
    assert!(VIME_TONE_PLACEMENT_OLD == 2);
};

#[cfg(test)]
mod tests {
    use super::*;

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

        fn as_config(&self) -> *const VimeConfig {
            (self as *const Self).cast()
        }
    }

    #[test]
    fn accepts_all_values() {
        for input_method in [
            VIME_INPUT_METHOD_TELEX,
            VIME_INPUT_METHOD_VNI,
            VIME_INPUT_METHOD_VIQR,
        ] {
            for tone_placement in [VIME_TONE_PLACEMENT_MODERN, VIME_TONE_PLACEMENT_OLD] {
                let raw = Bytes::new(input_method, tone_placement);
                let config = unsafe { VimeConfig::read(raw.as_config()) };

                assert_eq!(
                    config,
                    Some(VimeConfig {
                        input_method,
                        tone_placement,
                    })
                );
            }
        }
    }

    #[test]
    fn rejects_invalid_values() {
        for value in [0, 4, 5, u32::MAX, u32::MAX - 1] {
            let input_method = Bytes::new(value, VIME_TONE_PLACEMENT_MODERN);
            let tone_placement = Bytes::new(VIME_INPUT_METHOD_TELEX, value);

            assert_eq!(unsafe { VimeConfig::read(input_method.as_config()) }, None);
            assert_eq!(
                unsafe { VimeConfig::read(tone_placement.as_config()) },
                None
            );
        }
    }

    #[test]
    fn reads_valid_config() {
        for input_method in [
            VIME_INPUT_METHOD_TELEX,
            VIME_INPUT_METHOD_VNI,
            VIME_INPUT_METHOD_VIQR,
        ] {
            for tone_placement in [VIME_TONE_PLACEMENT_MODERN, VIME_TONE_PLACEMENT_OLD] {
                let config = VimeConfig {
                    input_method,
                    tone_placement,
                };
                assert_eq!(unsafe { VimeConfig::read(&config) }, Some(config));
            }
        }
    }

    #[test]
    fn converts_tone_placement() {
        for (raw, expected) in [
            (VIME_TONE_PLACEMENT_MODERN, TonePlacement::Modern),
            (VIME_TONE_PLACEMENT_OLD, TonePlacement::Old),
        ] {
            let config = VimeConfig {
                input_method: VIME_INPUT_METHOD_TELEX,
                tone_placement: raw,
            };

            assert_eq!(config.to_ffi_session_config().tone_placement(), expected);
        }
    }
}
