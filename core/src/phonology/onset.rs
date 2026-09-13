use std::str::FromStr;

/// Syllable Onset — the initial consonant cluster of a Vietnamese syllable.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Onset {
    #[default]
    None = 0,
    B,
    C,
    Ch,
    D,
    Đ,
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
    pub const MAX_ONSET_LEN: usize = 3;
    pub const COUNT: usize = 28 as usize;
    pub const MAX_ID: usize = Self::COUNT - 1;

    #[inline(always)]
    pub const fn from_id(id: usize) -> Result<Self, ()> {
        // Safety: real discriminants are contiguous from 0 through MAX_ID.
        if id < Self::COUNT {
            Ok(unsafe { std::mem::transmute::<u8, Self>(id as u8) })
        } else {
            Err(())
        }
    }

    #[inline(always)]
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        match bytes {
            [] => Ok(Self::None),

            // One-byte onset (A-Z, a-z).
            &[byte] => match byte | 0x20 {
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
                _ => Err(()),
            },

            // UTF-8 for 'đ' / 'Đ'.
            [0xC4, 0x91 | 0x90] => Ok(Self::Đ),

            // Two-byte ASCII onsets.
            &[first, second] => match [first | 0x20, second | 0x20] {
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
                _ => Err(()),
            },

            // Three-byte onset ("ngh").
            &[first, second, third] => match [first | 0x20, second | 0x20, third | 0x20] {
                [b'n', b'g', b'h'] => Ok(Self::Ngh),
                _ => Err(()),
            },

            _ => Err(()),
        }
    }

    #[inline(always)]
    pub const fn from_chars(chars: &[char]) -> Result<Self, ()> {
        match chars {
            [] => Ok(Self::None),
            &['đ'] | &['Đ'] => Ok(Self::Đ),
            &[character] if character.is_ascii() => Self::from_bytes(&[character as u8]),
            &[first, second] if first.is_ascii() && second.is_ascii() => {
                Self::from_bytes(&[first as u8, second as u8])
            }
            &[first, second, third]
                if first.is_ascii() && second.is_ascii() && third.is_ascii() =>
            {
                Self::from_bytes(&[first as u8, second as u8, third as u8])
            }
            _ => Err(()),
        }
    }
}

impl FromStr for Onset {
    type Err = ();

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(s.as_bytes())
    }
}
