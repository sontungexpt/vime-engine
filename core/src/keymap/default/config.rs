use crate::phonology::{RootVowel, Shape, Tone};

/// A keyboard key mapped to a Vietnamese tone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToneRule {
    pub key: u8, // Only ASCII keys are supported
    pub tone: Tone,
}

/// A keyboard key mapped to a Vietnamese vowel shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShapeRule {
    pub key: u8, // Only ASCII keys are supported
    pub on: RootVowel,
    pub shape: Shape,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rules<'a> {
    pub tones: &'a [ToneRule],
    pub shapes: &'a [ShapeRule],
    pub strokes: &'a [u8], // Only ASCII keys are supported
}

impl<'a> Rules<'a> {
    pub const fn new(tones: &'a [ToneRule], shapes: &'a [ShapeRule], strokes: &'a [u8]) -> Self {
        let mut tone_mask: u128 = 0;
        let mut shape_mask: u128 = 0;
        let mut stroke_mask: u128 = 0;

        // Tone keys: ASCII, each mapping to at most one tone.
        let mut i = 0;
        while i < tones.len() {
            let key = tones[i].key;

            if key >= 128 {
                panic!("Invalid layout: key must be ASCII");
            }

            let bit = 1u128 << key;

            if (tone_mask & bit) != 0 {
                panic!("Invalid layout: a tone key maps to multiple tones");
            }

            tone_mask |= bit;
            i += 1;
        }

        // Shape keys: ASCII, no collision with a tone key, and at most one
        // shape per vowel.
        let mut i = 0;
        while i < shapes.len() {
            let key = shapes[i].key;

            if key >= 128 {
                panic!("Invalid layout: key must be ASCII");
            }

            let bit = 1u128 << key;

            if (tone_mask & bit) != 0 {
                panic!("Invalid layout: a key maps to both a tone and a shape");
            }

            let mut j = i + 1;
            while j < shapes.len() {
                if shapes[i].key == shapes[j].key && shapes[i].on as u16 == shapes[j].on as u16 {
                    panic!("Invalid layout: a shape key applies multiple shapes to one owner");
                }

                j += 1;
            }

            shape_mask |= bit;
            i += 1;
        }

        // Stroke keys: ASCII, unique, and colliding with neither tone nor
        // shape keys.
        let mut i = 0;
        while i < strokes.len() {
            let key = strokes[i];

            if key >= 128 {
                panic!("Invalid layout: key must be ASCII");
            }

            let bit = 1u128 << key;

            if (stroke_mask & bit) != 0 {
                panic!("Invalid layout: a stroke key is duplicated");
            }

            if (tone_mask & bit) != 0 {
                panic!("Invalid layout: a stroke key maps to both a stroke and a tone");
            }

            if (shape_mask & bit) != 0 {
                panic!("Invalid layout: a stroke key maps to both a stroke and a shape");
            }

            stroke_mask |= bit;
            i += 1;
        }

        Self {
            tones,
            shapes,
            strokes,
        }
    }
}

/// Builds a [`Rules`] for the layouts in `keymap::default`.
macro_rules! rules {
    (
        tones: [ $( $tone:expr ),* $(,)? ],
        shapes: [ $( $shape:expr ),* $(,)? ],
        strokes: [ $( $stroke:expr ),* $(,)? ] $(,)?
    ) => {
        Rules::new(
            &[ $( $tone ),* ],
            &[ $( $shape ),* ],
            &[ $( $stroke ),* ],
        )
    };
}

pub(super) use rules;
