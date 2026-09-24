use vime_engine::{Key, KeyEvent, KeyStates};

use crate::types::{VimeKey, VimeKeyEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEventConversionError {
    InvalidUnicodeCharacter(u32),
    InvalidKey(u32),
}

impl TryFrom<VimeKeyEvent> for KeyEvent {
    type Error = KeyEventConversionError;

    #[inline]
    fn try_from(event: VimeKeyEvent) -> std::result::Result<Self, Self::Error> {
        let key = match event.key {
            VimeKey::None => char::from_u32(event.character).map(Key::Character).ok_or(
                KeyEventConversionError::InvalidUnicodeCharacter(event.character),
            )?,
            VimeKey::Backspace => Key::Backspace,
            VimeKey::Delete => Key::Delete,
            VimeKey::Left => Key::Left,
            VimeKey::Right => Key::Right,
            VimeKey::Enter => Key::Enter,
            VimeKey::Escape => Key::Escape,
            VimeKey::Tab => Key::Tab,
            VimeKey::Space => Key::Space,
        #[allow(unreachable_patterns)]
        _ => return Err(KeyEventConversionError::InvalidKey(event.key as u32)),
        };

        Ok(KeyEvent {
            key,
            states: KeyStates::from_bits_truncate(event.states),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(key: VimeKey, character: u32, states: u32) -> VimeKeyEvent {
        VimeKeyEvent {
            key,
            character,
            states,
        }
    }

    #[test]
    fn converts_character_key() {
        let event = event(VimeKey::None, 'v' as u32, 0);

        let result = KeyEvent::try_from(event).unwrap();

        assert_eq!(result.key, Key::Character('v'));
        assert_eq!(result.states, KeyStates::empty());
    }

    #[test]
    fn converts_every_special_key() {
        let cases = [
            (VimeKey::Backspace, Key::Backspace),
            (VimeKey::Delete, Key::Delete),
            (VimeKey::Left, Key::Left),
            (VimeKey::Right, Key::Right),
            (VimeKey::Enter, Key::Enter),
            (VimeKey::Escape, Key::Escape),
            (VimeKey::Tab, Key::Tab),
            (VimeKey::Space, Key::Space),
        ];

        for (vime_key, expected) in cases {
            let result = KeyEvent::try_from(event(vime_key, 0, 0)).unwrap();
            assert_eq!(result.key, expected);
        }
    }

    #[test]
    fn special_key_wins_over_character() {
        // Both fields set: the key takes precedence over the character payload.
        let result = KeyEvent::try_from(event(VimeKey::Backspace, 'v' as u32, 0)).unwrap();
        assert_eq!(result.key, Key::Backspace);
    }

    #[test]
    fn preserves_all_modifier_state_bits() {
        // Canonical VIME bit values (see vime_engine.h / event.rs).
        let cases = [
            (KeyStates::CTRL, 1u32 << 0),
            (KeyStates::ALT, 1u32 << 1),
            (KeyStates::SHIFT, 1u32 << 2),
            (KeyStates::SUPER, 1u32 << 3),
            (KeyStates::CAPS_LOCK, 1u32 << 4),
            (KeyStates::NUM_LOCK, 1u32 << 5),
            (KeyStates::HYPER, 1u32 << 6),
            (KeyStates::META, 1u32 << 7),
        ];

        for (state, vime_bit) in cases {
            let result = KeyEvent::try_from(event(VimeKey::None, 'v' as u32, vime_bit)).unwrap();
            assert!(result.states.contains(state), "missing bit {vime_bit:#x}");
        }
    }

    #[test]
    fn preserves_combined_states() {
        let vime_bits = (1u32 << 0) | (1u32 << 3); // CTRL | SUPER
        let result = KeyEvent::try_from(event(VimeKey::None, 'v' as u32, vime_bits)).unwrap();
        assert!(result.states.contains(KeyStates::CTRL));
        assert!(result.states.contains(KeyStates::SUPER));
    }

    #[test]
    fn keeps_unknown_state_bits() {
        let result = KeyEvent::try_from(event(VimeKey::None, 'v' as u32, 0xFFFF)).unwrap();
        assert_eq!(result.states.bits(), 0xFFFF);
    }

    #[test]
    fn rejects_invalid_unicode_character() {
        let event = event(VimeKey::None, 0x11_0000, 0);

        assert_eq!(
            KeyEvent::try_from(event),
            Err(KeyEventConversionError::InvalidUnicodeCharacter(0x11_0000))
        );
    }
}
