//! Deletion path: `BuildingSyllable::remove` and its literal helpers.
//!
//! The removal is transactional: if deleting a character would leave the
//! syllable invalid, the whole edit is rolled back.

use super::*;
use crate::{
    composition::syllable::InputEffect,
    phonology::{NucleusState, TonePlacement},
};

impl BuildingSyllable {
    // ─────────────────────────── Remove ───────────────────────────

    // NOTE: UNCHECKED
    #[inline]
    pub fn remove(
        &mut self,
        index: usize,
        tone_placement: TonePlacement,
    ) -> Result<InputEffect, SyllableBuildError> {
        let onset_len = self.onset.len();
        let vowels_len = self.nucleus.len();
        let total_len = onset_len + vowels_len + self.coda.len();

        debug_assert!(index < total_len);

        // Not a hot path; a whole-call transaction is fine.
        self.transaction(|this| {
            // ─────────────────────────── Onset ───────────────────────────

            if index < onset_len {
                if !this.remove_onset(index) {
                    return Err(SyllableBuildError::InvalidOnset);
                }

                // Removing onset may expose `I` and move it into the nucleus.
                this.normalize_i_placement();

                if this.check_nucleus() == NucleusState::Dead {
                    return Err(SyllableBuildError::InvalidNucleus);
                }

                return Ok(InputEffect::StructurallyChanged);
            }

            // ─────────────────────────── Vowel ───────────────────────────

            let vowel_index = index - onset_len;

            if vowel_index < vowels_len {
                if !this.remove_vowel(vowel_index, tone_placement) {
                    return Err(SyllableBuildError::InvalidNucleus);
                }

                // Removing a vowel may expose `I` from the onset.
                this.normalize_i_placement();

                if this.check_nucleus() == NucleusState::Dead {
                    return Err(SyllableBuildError::InvalidNucleus);
                }

                return Ok(InputEffect::StructurallyChanged);
            }

            // ─────────────────────────── Coda ───────────────────────────

            let coda_index = vowel_index - vowels_len;

            if !this.remove_coda(coda_index) {
                return Err(SyllableBuildError::InvalidCoda);
            }

            Ok(InputEffect::StructurallyChanged)
        })
    }

    /// Removes one character from the onset, restoring it if the result is invalid.
    #[inline(always)]
    fn remove_onset(&mut self, index: usize) -> bool {
        debug_assert!(index < self.onset.len());

        self.try_update_onset(
            |onset| onset.remove(index),
            |onset, removed| {
                onset.insert(index, removed);
            },
        )
    }

    /// Removes the vowel at `index`, clearing the tone when it targeted that vowel
    /// or the nucleus becomes empty.
    #[inline(always)]
    fn remove_vowel(&mut self, index: usize, tone_placement: TonePlacement) -> bool {
        debug_assert!(index < self.nucleus.len());

        // Recalculate the tone position after a removal shifts the vowels.
        let tone_pos = tone_placement.vowel_index(&self.nucleus[..], self.coda.is_empty());

        if tone_pos == Some(index) {
            self.tone = Tone::Flat;
        }

        // Do not recompute UO normalization on deletion: `ươo` minus `ơ` leaves an
        // ambiguous `ưo`, so the remaining literal vowels are preserved.

        if index >= self.nucleus.len() {
            return false;
        }

        self.nucleus.remove(index);

        // A tone without a vowel has no semantic target.
        if self.nucleus.is_empty() {
            self.tone = Tone::Flat;
        }

        true
    }

    /// Removes one char from the coda, restoring it if the result is invalid.
    #[inline(always)]
    fn remove_coda(&mut self, index: usize) -> bool {
        debug_assert!(index < self.coda.len());

        self.try_update_coda(
            |coda| coda.remove(index),
            |coda, removed| {
                coda.insert(index, removed);
            },
        )
    }
}
