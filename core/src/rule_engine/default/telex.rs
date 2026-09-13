use super::{ShapeRule, ToneRule, TypingRules};
use crate::{RootVowel, Shape, Tone};

/// Telex layout: shapes on `a/e/o` (circumflex), `w` (breve/horn), `d`
/// (stroke); tones on `s/f/r/x/j/z`.
pub(crate) const CONFIG: &TypingRules = &TypingRules::new(
    &[
        ToneRule {
            key: 's',
            tone: Tone::Acute,
        },
        ToneRule {
            key: 'f',
            tone: Tone::Grave,
        },
        ToneRule {
            key: 'r',
            tone: Tone::Hook,
        },
        ToneRule {
            key: 'x',
            tone: Tone::Tilde,
        },
        ToneRule {
            key: 'j',
            tone: Tone::Dot,
        },
        ToneRule {
            key: 'z',
            tone: Tone::Flat,
        },
    ],
    &[
        ShapeRule {
            key: 'a',
            on: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: 'w',
            on: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeRule {
            key: 'e',
            on: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: 'o',
            on: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: 'w',
            on: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeRule {
            key: 'w',
            on: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    &['d'],
);
