use crate::{RootVowel, Shape, Tone};

/// A keyboard key mapped to a Vietnamese tone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToneRule {
    pub key: char,
    pub tone: Tone,
}

/// A keyboard key mapped to a Vietnamese vowel shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShapeRule {
    pub key: char,
    pub on: RootVowel,
    pub shape: Shape,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TypingRules<'a> {
    pub tones: &'a [ToneRule],
    pub shapes: &'a [ShapeRule],
    pub strokes: &'a [char],
}

impl<'a> TypingRules<'a> {
    /// Creates a config, validating the key maps against these rules:
    ///
    /// 1. **Tone key uniqueness** — a tone key must be unique across all
    ///    tone definitions (e.g. `'s'` cannot map to both Acute and Grave).
    /// 2. **Tone & Shape mutual exclusivity** — a key mapped to a tone cannot
    ///    also be mapped to a shape, and vice versa.
    /// 3. **Shape key uniqueness per owner** — a shape key must be unique for
    ///    a given `RootVowel` (e.g. `'w'` cannot map to both Breve and
    ///    Circumflex under `A`).
    /// 4. **Multi-owner shape key allowance** — a shape key MAY be reused
    ///    across different owners (e.g. `'w'` shared between `Vowel(A)`,
    ///    `Vowel(O)`, and `Vowel(U)` is valid).
    /// 5. **Stroke key uniqueness** — a stroke key must be unique across all
    ///    stroke keys.
    /// 6. **Stroke & Tone mutual exclusivity** — a stroke key cannot also be a
    ///    tone key, and vice versa.
    /// 7. **Stroke & Shape mutual exclusivity** — a stroke key cannot also be
    ///    a shape key, and vice versa.
    ///
    /// Violations panic, so when invoked in a constant context a bad layout
    /// fails to compile.
    pub const fn new(tones: &'a [ToneRule], shapes: &'a [ShapeRule], strokes: &'a [char]) -> Self {
        // Rule 1: a tone key cannot map to two tones.
        let mut i = 0;
        while i < tones.len() {
            let mut j = i + 1;
            while j < tones.len() {
                if tones[i].key == tones[j].key {
                    panic!("Invalid layout: a tone key maps to multiple tones");
                }
                j += 1;
            }
            i += 1;
        }

        // Rule 2: a key cannot be both a tone and a shape.
        let mut i = 0;
        while i < tones.len() {
            let mut j = 0;
            while j < shapes.len() {
                if tones[i].key == shapes[j].key {
                    panic!("Invalid layout: a key maps to both a tone and a shape");
                }
                j += 1;
            }
            i += 1;
        }

        // Rules 3 & 4: the same shape key cannot apply two shapes to one
        // owner, but reuse across different owners is allowed.
        let mut i = 0;
        while i < shapes.len() {
            let mut j = i + 1;
            while j < shapes.len() {
                if shapes[i].key == shapes[j].key && (shapes[i].on as u16) == (shapes[j].on as u16)
                {
                    panic!("Invalid layout: a shape key applies multiple shapes to one owner");
                }
                j += 1;
            }
            i += 1;
        }

        // Rule 5: a stroke key cannot be duplicated.
        let mut i = 0;
        while i < strokes.len() {
            let mut j = i + 1;
            while j < strokes.len() {
                if strokes[i] == strokes[j] {
                    panic!("Invalid layout: a stroke key is duplicated");
                }
                j += 1;
            }
            i += 1;
        }

        // Rule 6: a stroke key cannot be a tone key.
        let mut i = 0;
        while i < strokes.len() {
            let mut j = 0;
            while j < tones.len() {
                if strokes[i] == tones[j].key {
                    panic!("Invalid layout: a stroke key maps to both a stroke and a tone");
                }
                j += 1;
            }
            i += 1;
        }

        // Rule 7: a stroke key cannot be a shape key.
        let mut i = 0;
        while i < strokes.len() {
            let mut j = 0;
            while j < shapes.len() {
                if strokes[i] == shapes[j].key {
                    panic!("Invalid layout: a stroke key maps to both a stroke and a shape");
                }
                j += 1;
            }
            i += 1;
        }

        Self {
            tones,
            shapes,
            strokes,
        }
    }
}
