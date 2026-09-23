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

        // Rule 1: Tone keys must be unique
        let mut i = 0;
        while i < tones.len() {
            let key = tones[i].key;
            if key >= 128 {
                panic!("Invalid layout: key must be ASCII");
            }
            // Keys are stored as-is; lowercasing happens at lookup time (has_key)
            let bit = 1u128 << key;

            if (tone_mask & bit) != 0 {
                panic!("Invalid layout: a tone key maps to multiple tones");
            }
            tone_mask |= bit;
            i += 1;
        }

        // Rule 2 & 3: Validate Shape keys
        let mut i = 0;
        while i < shapes.len() {
            let key = shapes[i].key;
            if key >= 128 {
                panic!("Invalid layout: key must be ASCII");
            }
            let bit = 1u128 << key;

            // Rule 2: A shape key must not collide with a tone key
            if (tone_mask & bit) != 0 {
                panic!("Invalid layout: a key maps to both a tone and a shape");
            }

            // Rule 3: One shape key must not assign two shapes to the same RootVowel owner
            let mut j = i + 1;
            while j < shapes.len() {
                if shapes[i].key == shapes[j].key && (shapes[i].on as u16) == (shapes[j].on as u16)
                {
                    panic!("Invalid layout: a shape key applies multiple shapes to one owner");
                }
                j += 1;
            }

            shape_mask |= bit;
            i += 1;
        }

        // Rule 5, 6 & 7: Validate Stroke keys
        let mut i = 0;
        while i < strokes.len() {
            let key = strokes[i];
            if key >= 128 {
                panic!("Invalid layout: key must be ASCII");
            }
            let bit = 1u128 << key;

            // Rule 5: Stroke keys must not repeat
            if (stroke_mask & bit) != 0 {
                panic!("Invalid layout: a stroke key is duplicated");
            }
            // Rule 6: A stroke key must not collide with a tone key
            if (tone_mask & bit) != 0 {
                panic!("Invalid layout: a stroke key maps to both a stroke and a tone");
            }
            // Rule 7: A stroke key must not collide with a shape key
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
