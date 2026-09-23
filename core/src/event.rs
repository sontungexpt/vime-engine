#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(transparent)]
pub struct KeyStates(u32);

impl KeyStates {
    pub const CTRL: Self = Self(1 << 0);
    pub const ALT: Self = Self(1 << 1);
    pub const SHIFT: Self = Self(1 << 2);
    pub const SUPER: Self = Self(1 << 3);
    pub const CAPS_LOCK: Self = Self(1 << 4);
    pub const NUM_LOCK: Self = Self(1 << 5);
    pub const HYPER: Self = Self(1 << 6);
    pub const META: Self = Self(1 << 7);

    /// The bitmask with no modifiers set.
    #[inline(always)]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// The raw `u32` bitmask.
    #[inline(always)]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Builds a state from a raw `u32`, keeping any unknown bits.
    #[inline(always)]
    pub const fn from_bits_truncate(bits: u32) -> Self {
        Self(bits)
    }

    /// Whether all bits of `other` are set in `self`.
    #[inline(always)]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Whether `self` and `other` share any bit.
    #[inline(always)]
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for KeyStates {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOrAssign for KeyStates {
    #[inline(always)]
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

/// A physical key press, independent of modifier state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Character(char),
    Backspace,
    Delete,
    Left,
    Right,
    Enter,
    Escape,
    Tab,
    Space,
}

/// A fully described keyboard event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyEvent {
    pub key: Key,
    pub states: KeyStates,
}

impl KeyEvent {
    /// Creates an event with no modifier keys held.
    #[inline]
    pub const fn key(key: Key) -> Self {
        Self {
            key,
            states: KeyStates::empty(),
        }
    }
}
