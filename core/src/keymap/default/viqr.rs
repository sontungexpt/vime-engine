use super::{config::rules, Rules, ShapeRule, ToneRule};
use crate::phonology::{RootVowel, Shape, Tone};

/// VIQR layout: shapes on `^` (circumflex), `(` (breve), `+` (horn), `d`
/// (stroke); tones on `` ` `` `?` `~` `'` `.` and `z`.
pub(crate) const CONFIG: &Rules = &rules! {
    tones: [
        ToneRule {
            key: b'`',
            tone: Tone::Grave,
        },
        ToneRule {
            key: b'?',
            tone: Tone::Hook,
        },
        ToneRule {
            key: b'~',
            tone: Tone::Tilde,
        },
        ToneRule {
            key: b'\'',
            tone: Tone::Acute,
        },
        ToneRule {
            key: b'.',
            tone: Tone::Dot,
        },
        ToneRule {
            key: b'z',
            tone: Tone::Flat,
        },
    ],
    shapes: [
        ShapeRule {
            key: b'^',
            on: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: b'^',
            on: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: b'^',
            on: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: b'(',
            on: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeRule {
            key: b'+',
            on: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeRule {
            key: b'+',
            on: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    strokes: [b'd'],
};
