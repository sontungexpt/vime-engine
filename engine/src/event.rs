use bitflags::bitflags;

bitflags! {
    /// Canonical modifier (key-state) bitmask for engine events.
    ///
    /// This layout is owned by vime-engine and is independent of any specific
    /// frontend's modifier model. It is a stable part of the public ABI: the
    /// FFI header (`vime_ffi.h`) exposes the same bits, and each frontend
    /// translates its own native modifier state into them. The low bits
    /// prioritize the shortcuts the engine cares about (Ctrl/Alt/Super);
    /// toggle locks and extra modifiers follow.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct KeyState: u32 {
        const CTRL      = 1 << 0;
        const ALT       = 1 << 1;
        const SHIFT     = 1 << 2;
        const SUPER     = 1 << 3;
        const CAPS_LOCK = 1 << 4;
        const NUM_LOCK  = 1 << 5;
        const HYPER     = 1 << 6;
        const META      = 1 << 7;
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
    pub states: KeyState,
}

impl KeyEvent {
    /// Creates an event with no modifier keys held.
    #[inline]
    pub const fn key(key: Key) -> Self {
        Self {
            key,
            states: KeyState::empty(),
        }
    }
}
