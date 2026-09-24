use super::{config::rules, Rules, ShapeRule, ToneRule};
use crate::phonology::{RootVowel, Shape, Tone};

/// VNI layout: shapes on `6` (circumflex), `7` (breve/horn), `8` (horn),
/// `9` (stroke); tones on `1-5` and `0`.
pub(crate) const CONFIG: &Rules = &rules! {
    tones: [
        ToneRule {
            key: b'1',
            tone: Tone::Acute,
        },
        ToneRule {
            key: b'2',
            tone: Tone::Grave,
        },
        ToneRule {
            key: b'3',
            tone: Tone::Hook,
        },
        ToneRule {
            key: b'4',
            tone: Tone::Tilde,
        },
        ToneRule {
            key: b'5',
            tone: Tone::Dot,
        },
        ToneRule {
            key: b'0',
            tone: Tone::Flat,
        },
    ],
    shapes: [
        ShapeRule {
            key: b'6',
            on: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: b'7',
            on: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeRule {
            key: b'6',
            on: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: b'6',
            on: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: b'7',
            on: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeRule {
            key: b'8',
            on: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    strokes: [b'9'],
};
