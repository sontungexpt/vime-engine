use std::str::FromStr;

// ------------------- Error -------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnsetParseError;

impl std::fmt::Display for OnsetParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid Vietnamese Onset")
    }
}

impl std::error::Error for OnsetParseError {}

/// Syllable Onset — the initial consonant cluster of a Vietnamese syllable.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Onset {
    #[default]
    None = 0,
    B,

    C,
    Ch,

    D,
    DStroke,

    G,
    Gh,
    Gi,

    H,

    K,
    Kh,

    L,

    M,

    N,
    Nh,
    Ng,
    Ngh,

    P,
    Ph,

    // Special case need considered when implement logic
    Qu,

    R,

    S,

    T,
    Th,
    Tr,

    V,

    X,
}

impl Onset {
    pub const MAX_LEN: usize = 3;
    pub const COUNT: usize = 28 as usize;

    #[inline(always)]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    #[inline(always)]
    pub const fn from_id(id: usize) -> Result<Self, OnsetParseError> {
        // Safety: real discriminants are contiguous from 0 through MAX_ID.
        if id < Self::COUNT {
            Ok(unsafe { std::mem::transmute::<u8, Self>(id as u8) })
        } else {
            Err(OnsetParseError)
        }
    }

    #[inline(always)]
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, OnsetParseError> {
        match bytes {
            [] => Ok(Self::None),

            // One-byte onset (A-Z, a-z).
            &[b] => match b | 0x20 {
                b'b' => Ok(Self::B),
                b'c' => Ok(Self::C),
                b'd' => Ok(Self::D),
                b'g' => Ok(Self::G),
                b'h' => Ok(Self::H),
                b'k' => Ok(Self::K),
                b'l' => Ok(Self::L),
                b'm' => Ok(Self::M),
                b'n' => Ok(Self::N),
                b'p' => Ok(Self::P),
                b'r' => Ok(Self::R),
                b's' => Ok(Self::S),
                b't' => Ok(Self::T),
                b'v' => Ok(Self::V),
                b'x' => Ok(Self::X),
                _ => Err(OnsetParseError),
            },

            // UTF-8 for 'đ' / 'Đ'.
            [0xC4, 0x91 | 0x90] => Ok(Self::DStroke),

            // Two-byte ASCII onsets.
            &[b1, b2] => match [b1 | 0x20, b2 | 0x20] {
                [b'c', b'h'] => Ok(Self::Ch),
                [b'g', b'h'] => Ok(Self::Gh),
                [b'g', b'i'] => Ok(Self::Gi),
                [b'k', b'h'] => Ok(Self::Kh),
                [b'n', b'h'] => Ok(Self::Nh),
                [b'n', b'g'] => Ok(Self::Ng),
                [b'p', b'h'] => Ok(Self::Ph),
                [b'q', b'u'] => Ok(Self::Qu),
                [b't', b'h'] => Ok(Self::Th),
                [b't', b'r'] => Ok(Self::Tr),
                _ => Err(OnsetParseError),
            },

            // Three-byte onset ("ngh").
            &[b1, b2, b3] => match [b1 | 0x20, b2 | 0x20, b3 | 0x20] {
                [b'n', b'g', b'h'] => Ok(Self::Ngh),
                _ => Err(OnsetParseError),
            },

            _ => Err(OnsetParseError),
        }
    }

    #[inline(always)]
    pub const fn from_chars(chars: &[char]) -> Result<Self, OnsetParseError> {
        match chars {
            [] => Ok(Self::None),
            &['đ'] | &['Đ'] => Ok(Self::DStroke),
            &[character] if character.is_ascii() => Self::from_bytes(&[character as u8]),
            &[first, second] if first.is_ascii() && second.is_ascii() => {
                Self::from_bytes(&[first as u8, second as u8])
            }
            &[first, second, third]
                if first.is_ascii() && second.is_ascii() && third.is_ascii() =>
            {
                Self::from_bytes(&[first as u8, second as u8, third as u8])
            }
            _ => Err(OnsetParseError),
        }
    }

    #[inline(always)]
    pub const fn is_possible_first_char(ch: char) -> bool {
        matches!(ch, 'Đ' | 'đ')
            || matches!(
                ch.to_ascii_lowercase(),
                'b' | 'c'
                    | 'd'
                    | 'g'
                    | 'h'
                    | 'k'
                    | 'l'
                    | 'm'
                    | 'n'
                    | 'p'
                    | 'q'
                    | 'r'
                    | 's'
                    | 't'
                    | 'v'
                    | 'x'
            )
    }

    #[inline(always)]
    pub const fn is_possible_char(ch: char) -> bool {
        Self::is_possible_first_char(ch) || matches!(ch.to_ascii_lowercase(), 'i' | 'u')
    }
}

impl FromStr for Onset {
    type Err = OnsetParseError;

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(s.as_bytes())
    }
}
