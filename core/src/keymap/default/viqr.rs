use super::{Rules, ShapeRule, ToneRule};
use crate::phonology::{RootVowel, Shape, Tone};

/// VIQr layout: shapes on `^` (circumflex), `(` (breve), `+` (horn), `d`
/// (stroke); tones on `` ` `` `?` `~` `'` `.` and `z`.
pub(crate) const CONFIG: &Rules = &Rules::new(
    &[
        ToneRule {
            key: '`',
            tone: Tone::Grave,
        },
        ToneRule {
            key: '?',
            tone: Tone::Hook,
        },
        ToneRule {
            key: '~',
            tone: Tone::Tilde,
        },
        ToneRule {
            key: '\'',
            tone: Tone::Acute,
        },
        ToneRule {
            key: '.',
            tone: Tone::Dot,
        },
        ToneRule {
            key: 'z',
            tone: Tone::Flat,
        },
    ],
    &[
        ShapeRule {
            key: '^',
            on: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: '^',
            on: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: '^',
            on: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: '(',
            on: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeRule {
            key: '+',
            on: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeRule {
            key: '+',
            on: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    &['d'],
);
