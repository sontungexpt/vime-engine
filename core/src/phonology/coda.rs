use std::mem::transmute;
use std::str::FromStr;

// ------------------- Error -------------------

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
    pub const MAX_CODA_LEN: usize = 2;
    pub const COUNT: usize = 9;
    pub const MAX_ID: usize = Self::COUNT - 1;

    /// Returns the coda for the given ID, or `Err` if out of range.
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
        matches!(
            ch as u32 | 0x20,
            0x63 // c
            | 0x6D // m
            | 0x6E // n
            | 0x70 // p
            | 0x74 // t
        )
    }

    #[inline(always)]
    pub const fn is_possible_char(ch: char) -> bool {
        Self::is_possible_first_char(ch)
            || matches!(
                ch as u32 | 0x20,
                | 0x67 // g
                | 0x68 // h
            )
    }

    #[inline(always)]
    pub const fn from_byte(byte: u8) -> Result<Self, CodaParseError> {
        match byte | 0x20 {
            b'c' => Ok(Self::C),
            b'm' => Ok(Self::M),
            b'n' => Ok(Self::N),
            b'p' => Ok(Self::P),
            b't' => Ok(Self::T),
            _ => Err(CodaParseError),
        }
    }

    #[inline(always)]
    pub const fn from_two_bytes(first: u8, second: u8) -> Result<Self, CodaParseError> {
        match (first | 0x20, second | 0x20) {
            (b'c', b'h') => Ok(Self::Ch),
            (b'n', b'g') => Ok(Self::Ng),
            (b'n', b'h') => Ok(Self::Nh),
            _ => Err(CodaParseError),
        }
    }

    /// Returns the coda for the given ASCII bytes, or `Err` for an invalid cluster.
    #[inline(always)]
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, CodaParseError> {
        match bytes {
            [] => Ok(Self::None),
            &[byte] => Self::from_byte(byte),
            &[first, second] => Self::from_two_bytes(first, second),
            _ => Err(CodaParseError),
        }
    }

    /// Returns the coda for the given ASCII characters, or `Err` for an
    /// invalid cluster.
    #[inline(always)]
    pub const fn from_chars(chars: &[char]) -> Result<Self, CodaParseError> {
        match chars {
            [] => Ok(Self::None),
            &[c] if c.is_ascii() => Self::from_byte(c as u8),
            &[c0, c1] if c0.is_ascii() && c1.is_ascii() => Self::from_two_bytes(c0 as u8, c1 as u8),
            _ => Err(CodaParseError),
        }
    }
}

// Standard FromStr implementation for idiomatic parsing via .parse::<Coda>()
impl FromStr for Coda {
    type Err = CodaParseError;

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(s.as_bytes())
    }
}
