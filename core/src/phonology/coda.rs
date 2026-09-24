use std::mem::transmute;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodaParseError;

impl std::fmt::Display for CodaParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid Vietnamese Coda")
    }
}

impl std::error::Error for CodaParseError {}

/// Syllable Coda — the final consonant cluster of a Vietnamese syllable.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Coda {
    #[default]
    None = 0,
    P,
    T,
    C,
    Ch,
    M,
    N,
    Ng,
    Nh,
}

impl Coda {
    pub const MAX_LEN: usize = 2;
    pub const COUNT: usize = 9;

    #[inline(always)]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    #[inline(always)]
    pub const fn from_id(id: usize) -> Result<Self, CodaParseError> {
        if id < Self::COUNT {
            Ok(unsafe { transmute::<u8, Self>(id as u8) })
        } else {
            Err(CodaParseError)
        }
    }

    #[inline(always)]
    pub const fn is_possible_first_char(ch: char) -> bool {
        matches!(ch.to_ascii_lowercase(), 'c' | 'm' | 'n' | 'p' | 't')
    }

    #[inline(always)]
    pub const fn is_possible_char(ch: char) -> bool {
        Self::is_possible_first_char(ch) || matches!(ch.to_ascii_lowercase(), 'g' | 'h')
    }

    /// Primary const parser for ASCII byte slices.
    #[inline(always)]
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, CodaParseError> {
        match bytes {
            [] => Ok(Self::None),
            &[b] => match b | 0x20 {
                b'c' => Ok(Self::C),
                b'm' => Ok(Self::M),
                b'n' => Ok(Self::N),
                b'p' => Ok(Self::P),
                b't' => Ok(Self::T),
                _ => Err(CodaParseError),
            },
            &[b0, b1] => match [b0 | 0x20, b1 | 0x20] {
                [b'c', b'h'] => Ok(Self::Ch),
                [b'n', b'g'] => Ok(Self::Ng),
                [b'n', b'h'] => Ok(Self::Nh),
                _ => Err(CodaParseError),
            },
            _ => Err(CodaParseError),
        }
    }

    /// Direct parser from a character slice.
    #[inline(always)]
    pub const fn from_chars(chars: &[char]) -> Result<Self, CodaParseError> {
        match chars {
            [] => Ok(Self::None),
            &[c] if c.is_ascii() => Self::from_bytes(&[c as u8]),
            &[c0, c1] if c0.is_ascii() && c1.is_ascii() => Self::from_bytes(&[c0 as u8, c1 as u8]),
            _ => Err(CodaParseError),
        }
    }
}

impl FromStr for Coda {
    type Err = CodaParseError;

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(s.as_bytes())
    }
}
