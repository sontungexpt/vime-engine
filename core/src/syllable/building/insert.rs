//! Cursor insertion path: `BuildingSyllable::insert` and its literal helpers.
//! `index` addresses `onset ++ nucleus ++ coda`; appending delegates to
//! [`push`], and a transform at a vowel sees only vowels to its left.

use super::*;
use crate::{
    keymap::Keymap,
    phonology::{BaseVowel, Onset, Tone, Vowel, NUCLEUS_MAX_LEN},
};

impl BuildingSyllable {
    // ─────────────────────────── Insert ───────────────────────────

    pub fn insert<KM: Keymap>(
        &mut self,
        keymap: &KM,
        index: usize,
        key: char,
    ) -> Result<TransformEffect, SyllableBuildError> {
        let onset_len = self.onset.len();
        let vowels_len = self.nucleus.len();
        // Absolute indices at the boundaries between syllable parts.
        let vowel_boundary = onset_len + vowels_len;
        let total_len = vowel_boundary + self.coda.len();

        // Appending follows the same path as push.
        if index == total_len {
            return self.push(keymap, key);
        }

        assert!(
            index < total_len,
            "insertion index out of bounds: index={index}, len={total_len}"
        );

        // ─────────────────────────── Onset ───────────────────────────

        if index <= onset_len {
            // Give the d/đ stroke transform priority over literal insertion.
            let effect = self.try_toggle_d_stroke(keymap, key);

            if matches!(effect, TransformEffect::Applied(_)) {
                return Ok(effect);
            }

            // Try the key as an onset character first.
            if self.insert_onset(index, key) {
                return Ok(effect);
            }

            // Only the position after the onset can start the nucleus.
            if index != onset_len {
                return Err(SyllableBuildError::InvalidOnset);
            }

            if let Some(vowel) = Vowel::from_char(key) {
                if self.insert_vowel(0, vowel) {
                    return Ok(effect);
                }
                return Err(SyllableBuildError::InvalidNucleus);
            }

            return Err(SyllableBuildError::InvalidOnset);
        }

        // ─────────────────────────── Vowel ───────────────────────────

        if index <= vowel_boundary {
            let vowel_index = index - onset_len;

            // At a vowel position, try transforms before literal insertion.
            let effect = self.try_transform(keymap, key, Some(vowel_index));
            if matches!(effect, TransformEffect::Applied(_)) {
                return Ok(effect);
            }

            if let Some(decoded) = Vowel::from_char(key) {
                if self.insert_vowel(vowel_index, decoded) {
                    return Ok(effect);
                }
                return Err(SyllableBuildError::InvalidNucleus);
            }

            // Only the position after the nucleus can start the coda.
            if vowel_index != vowels_len {
                return Err(SyllableBuildError::InvalidNucleus);
            }

            if self.insert_coda(0, key) {
                self.normalize_uo_horn();
                return Ok(effect);
            }
            return Err(SyllableBuildError::InvalidCoda);
        }

        // ─────────────────────────── Coda ───────────────────────────
        let effect = self.try_transform(keymap, key, None);
        if matches!(effect, TransformEffect::Applied(_)) {
            return Ok(effect);
        }

        let coda_index = index - vowel_boundary;

        if self.insert_coda(coda_index, key) {
            return Ok(effect);
        }

        Err(SyllableBuildError::InvalidCoda)
    }

    /// Inserts a literal char into the onset (no vowel fallback).
    #[inline]
    fn insert_onset(&mut self, onset_index: usize, key: char) -> bool {
        debug_assert!(
            onset_index <= self.onset.len(),
            "onset insertion index out of bounds: index={onset_index}, len={}",
            self.onset.len()
        );

        // Keep `i` in the nucleus after `g` so a later vowel can resolve `gi` + V;
        // `nucleus_state` is set directly because this path bypasses `try_update_nucleus`.
        if is_i_ignore_case(key) {
            // `i` follows the one-character G onset, before any nucleus vowel.
            let should_move_i_to_nucleus =
                self.onset_kind == Onset::G && onset_index == 1 && self.nucleus.is_empty();

            if should_move_i_to_nucleus {
                self.nucleus
                    .push(Vowel::new(BaseVowel::I, Tone::Flat, key == 'I'));

                // `i` is the only vowel in the nucleus, so it must be valid.
                self.nucleus_state = NucleusState::Valid;

                return true;
            }

            // Leave `i` for the nucleus in every other case.
            return false;
        }

        if self.onset.len() < Onset::MAX_LEN {
            return self.try_update_onset(
                |onset| onset.insert(onset_index, key),
                |onset, _| _ = onset.remove(onset_index),
            );
        }

        false
    }

    /// Inserts a literal vowel into the nucleus; transforms are already handled by `insert()`.
    #[inline]
    fn insert_vowel(&mut self, vowel_index: usize, vowel: Vowel) -> bool {
        let len = self.nucleus.len();

        debug_assert!(
            vowel_index <= len,
            "vowel insertion index {vowel_index} exceeds nucleus length {len}"
        );

        if len >= NUCLEUS_MAX_LEN {
            return false;
        }

        // Let push handle appending, including insertion of the first vowel.
        if vowel_index == len {
            return self.push_vowel(vowel);
        }

        // Appends returned above, so the nucleus is non-empty here.

        // Two pre-toned vowels conflict (`á` + `ắ`); a pre-toned and an
        // unmarked vowel can be combined (`á` + `a`).
        let new_tone = match (self.tone, vowel.tone()) {
            (Tone::Flat, incoming) => incoming,

            (current, Tone::Flat) => current,

            (_, _) => return false,
        };

        // Inserting `i` before a G onset's vowels attaches it to the onset, forming `gi`.
        let should_form_gi =
            vowel_index == 0 && self.onset_kind == Onset::G && vowel.base() == BaseVowel::I;

        if should_form_gi {
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
        debug_assert!(
            coda_index <= self.coda.len(),
            "coda insertion index out of bounds: index={coda_index}, len={}",
            self.coda.len()
        );

        if self.coda.len() < Coda::MAX_LEN {
            return self.try_update_coda(
                |coda| coda.insert(coda_index, input),
                |coda, _| _ = coda.remove(coda_index),
            );
        }

        false
    }
}
