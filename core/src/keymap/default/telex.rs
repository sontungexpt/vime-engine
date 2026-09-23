use super::{Rules, ShapeRule, ToneRule};
use crate::phonology::{RootVowel, Shape, Tone};

/// Telex layout: shapes on `a/e/o` (circumflex), `w` (breve/horn), `d`
/// (stroke); tones on `s/f/r/x/j/z`.
pub(crate) const CONFIG: &Rules = &Rules::new(
    &[
        ToneRule {
            key: b's',
            tone: Tone::Acute,
        },
        ToneRule {
            key: b'f',
            tone: Tone::Grave,
        },
        ToneRule {
            key: b'r',
            tone: Tone::Hook,
        },
        ToneRule {
            key: b'x',
            tone: Tone::Tilde,
        },
        ToneRule {
            key: b'j',
            tone: Tone::Dot,
        },
        ToneRule {
            key: b'z',
            tone: Tone::Flat,
        },
    ],
    &[
        ShapeRule {
            key: b'a',
            on: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: b'w',
            on: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeRule {
            key: b'e',
            on: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: b'o',
            on: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: b'w',
            on: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeRule {
            key: b'w',
            on: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    &[b'd'],
);
