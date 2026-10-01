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

/// The syllable's coda: its final consonant (`p t c ch m n ng nh`) or `None`.
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
    pub const fn id(self) -> u8 {
        self as u8
    }

    /// The `Coda` with this id, or `None` if `id` is out of range. `from_id(id())`
    /// round-trips, as the `coda_discriminants_are_contiguous` test pins.
    #[inline(always)]
    pub const fn from_id(id: u8) -> Option<Self> {
        // SAFETY: variants are contiguous `0..COUNT`, so the guard admits only
        // real discriminants — that is what the test named above pins.
        if id < Self::COUNT as u8 {
            Some(unsafe { transmute(id as u8) })
        } else {
            None
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

    /// Parses a coda from ASCII bytes; the `char` and `str` parsers delegate here.
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

    /// Parses a coda from a `char` slice; non-ASCII input is rejected.
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
