use std::mem::transmute;
use std::str::FromStr;

/// Syllable Coda — the final consonant cluster of a Vietnamese syllable.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Coda {
    #[default]
    None = 0,
    C = 1,
    Ch = 2,
    M = 3,
    N = 4,
    Ng = 5,
    Nh = 6,
    P = 7,
    T = 8,
}

impl Coda {
    pub const MAX_CODA_LEN: usize = 2;
    pub const COUNT: usize = 9;
    pub const MAX_ID: usize = Self::COUNT - 1;

    /// Returns the coda for the given ID, or `Err` if out of range.
    #[inline(always)]
    pub const fn from_id(id: usize) -> Result<Self, ()> {
        if id < Self::COUNT {
            Ok(unsafe { transmute::<u8, Self>(id as u8) })
        } else {
            Err(())
        }
    }

    /// Returns the coda for the given ASCII bytes, or `Err` for an invalid cluster.
    #[inline(always)]
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        match bytes {
            [] => Ok(Self::None),

            // Single-byte codas ("c", "m", "n", "p", "t")
            &[byte] => match byte | 0x20 {
                b'c' => Ok(Self::C),
                b'm' => Ok(Self::M),
                b'n' => Ok(Self::N),
                b'p' => Ok(Self::P),
                b't' => Ok(Self::T),
                _ => Err(()),
            },

            // Two-byte codas ("ch", "ng", "nh")
            &[first, second] => match [first | 0x20, second | 0x20] {
                [b'c', b'h'] => Ok(Self::Ch), // b"ch"
                [b'n', b'g'] => Ok(Self::Ng), // b"ng"
                [b'n', b'h'] => Ok(Self::Nh), // b"nh"
                _ => Err(()),
            },

            _ => Err(()),
        }
    }

    /// Returns the coda for the given ASCII characters, or `Err` for an
    /// invalid cluster.
    #[inline(always)]
    pub const fn from_chars(chars: &[char]) -> Result<Self, ()> {
        match chars {
            [] => Ok(Self::None),
            &[c] if c.is_ascii() => Self::from_bytes(&[c as u8]),
            &[c0, c1] if c0.is_ascii() && c1.is_ascii() => Self::from_bytes(&[c0 as u8, c1 as u8]),
            _ => Err(()),
        }
    }
}

// Standard FromStr implementation for idiomatic parsing via .parse::<Coda>()
impl FromStr for Coda {
    type Err = ();

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(s.as_bytes())
    }
}
