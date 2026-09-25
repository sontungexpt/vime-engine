//! Cursor insertion path: `BuildingSyllable::insert` and its literal helpers.
//!
//! The index is absolute over `onset ++ vowels ++ coda`. Inserting at the end
//! delegates to [`push`](super::push), and the cursor-bounded transforms only
//! scan the vowels strictly left of the insertion point.

use super::*;
use crate::{
    composition::syllable::InputEffect,
    keymap::Keymap,
    phonology::{decode_vowel, BaseVowel, ExtendedBaseVowel, Onset, Tone, NUCLEUS_MAX_LEN},
};

impl BuildingSyllable {
    // ─────────────────────────── Insert ───────────────────────────

    #[inline(always)]
    pub fn insert<KM: Keymap>(
        &mut self,
        keymap: &KM,
        index: usize,
        key: char,
    ) -> Result<InputEffect, SyllableBuildError> {
        let onset_len = self.onset.len();
        let vowels_len = self.nucleus.len();
        let vowel_boundary = onset_len + vowels_len;
        let total_len = vowel_boundary + self.coda.len();

        // Inserting at the end is equivalent to pushing.
        if index == total_len {
            return self.push(keymap, key);
        }

        assert!(index < total_len, "insertion index out of bounds");

        // ─────────────────────────── Onset ───────────────────────────

        if index <= onset_len {
            // The d/đ stroke is toggled before literal insertion.
            if self.try_toggle_d_stroke(keymap, key) == TransformResult::Applied {
                return Ok(InputEffect::Transformed);
            }

            // The onset consumes the input.
            if self.insert_onset(index, key) {
                return Ok(InputEffect::StructurallyChanged);
            }

            // Only the last onset boundary may fall through to a vowel.
            if index != onset_len {
                return Err(SyllableBuildError::InvalidOnset);
            }

            if let Some(decoded) = decode_vowel(key) {
                if self.insert_vowel(0, decoded) {
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableBuildError::InvalidNucleus);
            }

            return Err(SyllableBuildError::InvalidOnset);
        }

        // ─────────────────────────── Vowel ───────────────────────────

        if index <= vowel_boundary {
            let vowel_index = index - onset_len;

            // Transforms are handled before literal insertion.
            if self.try_transform(keymap, key, Some(vowel_index)) == TransformResult::Applied {
                return Ok(InputEffect::Transformed);
            }

            if let Some(decoded) = decode_vowel(key) {
                if self.insert_vowel(vowel_index, decoded) {
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableBuildError::InvalidNucleus);
            }

            // Only the last vowels boundary may fall through to a coda.
            if vowel_index != vowels_len {
                return Err(SyllableBuildError::InvalidNucleus);
            }

            if self.insert_coda(0, key) {
                self.normalize_uo_horn();
                return Ok(InputEffect::StructurallyChanged);
            }
            return Err(SyllableBuildError::InvalidCoda);
        }

        // ─────────────────────────── Coda ───────────────────────────

        if self.try_transform(keymap, key, None) == TransformResult::Applied {
            return Ok(InputEffect::Transformed);
        }

        let coda_index = index - vowel_boundary;

        if self.insert_coda(coda_index, key) {
            return Ok(InputEffect::StructurallyChanged);
        }

        Err(SyllableBuildError::InvalidCoda)
    }

    /// Inserts a literal char into the onset (no vowel fallback).
    #[inline(always)]
    fn insert_onset(&mut self, index: usize, key: char) -> bool {
        // `g` + `i` with no vowel yet: keep the `i` as a nucleus vowel instead
        // of an onset char, mirroring the push path. A lone `g i` must stay the
        // ambiguous prefix `g` + `i`, so that a following vowel can still
        // promote the `i` into the onset (`gi` + V). The nucleus is marked
        // `InComplete` explicitly because this path bypasses
        // `try_update_nucleus`, which would otherwise cache the state.
        if is_i_ignore_case(key) {
            if self.onset_kind == Onset::G
                && index == 1 // Because we know g is only one char so index == 1 means push to onset
                && self.nucleus.is_empty()
            {
                self.nucleus
                    .push(ExtendedBaseVowel::with_case(BaseVowel::I, key == 'I'));
                self.nucleus_state = NucleusState::InComplete;

                return true;
            }

            // No onset contains i excepted gi
            return false;
        }

        if self.onset.len() >= Onset::MAX_LEN {
            return false;
        }
        self.try_update_onset(
            |onset| onset.insert(index, key),
            |onset, _| _ = onset.remove(index),
        )
    }

    /// Inserts a literal vowel into the nucleus; transforms are already handled
    /// by `insert()`.
    #[inline]
    fn insert_vowel(
        &mut self,
        vowel_index: usize,
        (vowel, tone): (ExtendedBaseVowel, Tone),
    ) -> bool {
        let len = self.nucleus.len();
        debug_assert!(vowel_index <= len);

        if len >= NUCLEUS_MAX_LEN {
            return false;
        }

        // Like push
        if vowel_index == len {
            return self.push_vowel((vowel, tone));
        }

        // From here the nucleus always has at least one vowel:
        //   * appending at the end (`vowel_index == len`) already returned via
        //     `push_vowel`, which handles the empty first-vowel case;
        //   * the onset path inserts at index 0, but with an empty nucleus that is the
        //     same as appending (also handled above), so it only reaches here when a
        //     vowel already exists;
        //   * the vowel path only runs when `index > onset_len`, so `vowel_index >= 1`,
        //     and here it is also `< len`, meaning at least two vowels.

        // Pre-toned vowels clash with an existing non-flat tone (`á` + `ắ`
        // renders `áắ`), while `á` + `a` is valid.
        let new_tone = match (self.tone, tone) {
            // No tone yet -> adopt the incoming tone.
            (Tone::Flat, incoming) => incoming,

            // No incoming tone -> keep the syllable tone.
            (current, Tone::Flat) => current,

            // Two non-flat tones conflict.
            (_, _) => return false,
        };

        // Special case push i at the start of nucleus
        // A vowel after `G + I` moves `I` into the onset, forming `Gi + V`.
        if vowel_index == 0
            // We already know that the vowels is not empty so i must be belong to onset
            && self.onset_kind == Onset::G
            && vowel.get() == BaseVowel::I
        {
            self.onset.push(if vowel.is_upper() { 'I' } else { 'i' });
            self.onset_kind = Onset::Gi;
            self.tone = new_tone;
            return true;
        }

        if !self.try_update_nucleus(
            |nucleus| nucleus.insert(vowel_index, vowel),
            |nucleus, _| _ = nucleus.remove(vowel_index),
        ) {
            return false;
        }

        // Adopt the tone only after the nucleus accepted the vowel.
        self.tone = new_tone;
        true
    }

    /// Inserts a literal char into the coda.
    #[inline(always)]
    fn insert_coda(&mut self, coda_index: usize, input: char) -> bool {
        debug_assert!(coda_index <= self.coda.len());

        if self.coda.len() >= Coda::MAX_LEN {
            return false;
        }
        self.try_update_coda(
            |coda| coda.insert(coda_index, input),
            |coda, _| _ = coda.remove(coda_index),
        )
    }
}
