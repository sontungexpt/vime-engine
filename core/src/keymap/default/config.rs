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

        // Validate tone keys:
        // - Must be ASCII.
        // - Each key must map to at most one tone.
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

        // Validate shape keys:
        // - Must be ASCII.
        // - Must not collide with a tone key.
        // - A key may not assign multiple shapes to the same vowel.
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

        // Validate stroke keys:
        // - Must be ASCII.
        // - Must be unique.
        // - Must not collide with a tone key.
        // - Must not collide with a shape key.
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

/// Local declarative macro for constructing [`Rules`] within `keymap::default`.
macro_rules! rules {
    (
        tones: [ $( $tone:expr ),* $(,)? ],
        shapes: [ $( $shape:expr ),* $(,)? ],
        strokes: [ $( $stroke:expr ),* $(,)? ] $(,)?
    ) => {
        $crate::keymap::default::Rules::new(
            &[ $( $tone ),* ],
            &[ $( $shape ),* ],
            &[ $( $stroke ),* ],
        )
    };
}

pub(super) use rules;
