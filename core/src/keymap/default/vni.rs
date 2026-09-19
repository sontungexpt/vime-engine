use super::{Rules, ShapeRule, ToneRule};
use crate::{RootVowel, Shape, Tone};

/// VNI layout: shapes on `6` (circumflex), `7` (breve/horn), `8` (horn),
/// `9` (stroke); tones on `1-5` and `0`.
pub(crate) const CONFIG: &Rules = &Rules::new(
    &[
        ToneRule {
            key: '1',
            tone: Tone::Acute,
        },
        ToneRule {
            key: '2',
            tone: Tone::Grave,
        },
        ToneRule {
            key: '3',
            tone: Tone::Hook,
        },
        ToneRule {
            key: '4',
            tone: Tone::Tilde,
        },
        ToneRule {
            key: '5',
            tone: Tone::Dot,
        },
        ToneRule {
            key: '0',
            tone: Tone::Flat,
        },
    ],
    &[
        ShapeRule {
            key: '6',
            on: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: '7',
            on: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeRule {
            key: '6',
            on: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: '6',
            on: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: '7',
            on: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeRule {
            key: '8',
            on: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    &['9'],
);
