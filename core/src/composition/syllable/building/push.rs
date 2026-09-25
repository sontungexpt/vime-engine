//! Append path: `BuildingSyllable::push` and the literal helpers it drives.
//!
//! Pushing always acts at the end of the syllable. While the onset is still
//! being parsed (no vowel, no coda), keys are offered to the onset first; once
//! a vowel or coda exists the key is parsed as a transform, a vowel, or a coda.

use super::*;
use crate::{
    composition::syllable::InputEffect,
    keymap::Keymap,
    phonology::{decode_vowel, BaseVowel, ExtendedBaseVowel, Onset, Tone, NUCLEUS_MAX_LEN},
};

impl BuildingSyllable {
    // ─────────────────────────── Push ───────────────────────────

    #[inline(always)]
    pub fn push<KM: Keymap>(
        &mut self,
        keymap: &KM,
        key: char,
    ) -> Result<InputEffect, SyllableBuildError> {
        // ─────────────────────────── Onset ───────────────────────────
        //
        // No vowel and no coda: still parsing the onset. A stroke key can
        // modify an existing D/Đ before being read as a literal character.
        if self.coda.is_empty() && self.nucleus.is_empty() {
            if self.try_toggle_d_stroke(keymap, key) == TransformResult::Applied {
                return Ok(InputEffect::Transformed);
            }

            if self.push_onset(key) {
                return Ok(InputEffect::StructurallyChanged);
            }

            // A lone Q must be followed by U to be valid.
            if self.onset.len() == 1 && is_q_ignore_case(self.onset[0]) {
                return Err(SyllableBuildError::InvalidOnset);
            }

            // The input may start the vowel nucleus.
            let Some(decoded) = decode_vowel(key) else {
                return Err(SyllableBuildError::InvalidOnset);
            };
            if !self.push_vowel(decoded) {
                return Err(SyllableBuildError::InvalidNucleus);
            }
            return Ok(InputEffect::StructurallyChanged);
        }

        // ─────────────────────────── Nucleus ───────────────────────────
        //
        // Vowels and/or a coda exist: offer the transform keys (tone / shape /
        // stroke) once, then parse the input as a literal vowel or coda.
        if self.try_transform(keymap, key, None) == TransformResult::Applied {
            return Ok(InputEffect::Transformed);
        }

        if self.coda.is_empty() {
            // Vowel literal...
            let Some(decoded) = decode_vowel(key) else {
                // ...otherwise fall back to a coda.
                if self.push_coda(key) {
                    // Fold a leftover `uơ` / `ưo` prefix into `ươ` once a coda lands.
                    self.normalize_uo_horn();
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableBuildError::InvalidCoda);
            };
            // The nucleus may still reject the decoded vowel.
            if !self.push_vowel(decoded) {
                return Err(SyllableBuildError::InvalidNucleus);
            }
            self.normalize_uo_horn();
            return Ok(InputEffect::StructurallyChanged);
        }

        // Coda literal.
        if self.push_coda(key) {
            return Ok(InputEffect::StructurallyChanged);
        }
        Err(SyllableBuildError::InvalidCoda)
    }

    /// Consumes a literal character into the onset; returns `false` to fall
    /// back to the vowel parser. `q` is a transitional prefix waiting for `u`;
    /// `i` is left for the nucleus so `gi` stays ambiguous until another vowel.
    #[inline]
    fn push_onset(&mut self, key: char) -> bool {
        if is_i_ignore_case(key) {
            return false;
        }

        if self.onset.is_empty() && is_q_ignore_case(key) {
            self.onset.push(key);
            return true;
        }

        if self.onset.len() >= Onset::MAX_LEN {
            return false;
        }

        self.try_update_onset(
            |onset| onset.push(key),
            |onset, _| {
                onset.pop();
            },
        )
    }

    /// Adds a decoded vowel to the nucleus (max 3 vowels); returns `false`
    /// when the tone conflicts or the resulting nucleus is invalid.
    ///
    /// `pub(super)`: `insert` delegates to the append path when the insert
    /// lands behind the end of the nucleus.
    #[inline]
    pub(super) fn push_vowel(&mut self, (vowel, tone): (ExtendedBaseVowel, Tone)) -> bool {
        let len = self.nucleus.len();

        if len >= NUCLEUS_MAX_LEN {
            return false;
        }

        // First vowel; adopt its tone directly.
        if len == 0 {
            self.nucleus.push(vowel);
            self.tone = tone;
            return true;
        }

        // Pre-toned vowels clash with a non-flat syllable tone (`á` + `ắ`), while
        // `á` + `a` is valid.
        let new_tone = match (self.tone, tone) {
            // No tone yet -> adopt the incoming tone.
            (Tone::Flat, incoming) => incoming,

            // No incoming tone -> keep the syllable tone.
            (current, Tone::Flat) => current,

            // Two non-flat tones conflict.
            (_, _) => return false,
        };

        // A vowel after `g i` moves the `i` into the onset, forming `gi` + V.
        //
        // `G + I + V` -> `Gi + V`.
        if len == 1 && self.onset_kind == Onset::G && self.nucleus[0].get() == BaseVowel::I {
            let i = self.nucleus.pop().expect("nucleus contains i");

            self.onset.push(if i.is_upper() { 'I' } else { 'i' });
            self.onset_kind = Onset::Gi;

            // Only one vowel for now adopt its directly
            self.nucleus.push(vowel);
            self.tone = new_tone;

            return true;
        }

        if !self.try_update_nucleus(
            |nucleus| nucleus.push(vowel),
            |nucleus, _| {
                nucleus.pop();
            },
        ) {
            return false;
        }

        self.tone = new_tone;
        true
    }

    /// Consumes a literal char into the coda; returns `false` if the coda does
    /// not accept the input.
    #[inline]
    fn push_coda(&mut self, key: char) -> bool {
        if self.coda.len() >= Coda::MAX_LEN {
            return false;
        }
        self.try_update_coda(
            |coda| coda.push(key),
            |coda, _| {
                coda.pop();
            },
        )
    }
}
