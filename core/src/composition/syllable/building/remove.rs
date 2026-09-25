//! Deletion path: `BuildingSyllable::remove` and its literal helpers.
//!
//! Every helper is atomic on its own: it applies one mutation and restores it
//! if the resulting structure is invalid, so any rejected removal leaves the
//! syllable unchanged.

use super::*;
use crate::{composition::syllable::InputEffect, phonology::TonePlacement};

impl BuildingSyllable {
    // ─────────────────────────── Remove ───────────────────────────

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

        // ─────────────────────────── Onset ───────────────────────────

        if index < onset_len {
            if self.onset_kind == Onset::Gi {
                if self.remove_gi_onset(index) {
                    return Ok(InputEffect::StructurallyChanged);
                }
                return Err(SyllableBuildError::InvalidNucleus);
            }

            if !self.remove_onset(index) {
                return Err(SyllableBuildError::InvalidOnset);
            }

            return Ok(InputEffect::StructurallyChanged);
        }

        // ─────────────────────────── Vowel ───────────────────────────

        let vowel_index = index - onset_len;

        if vowel_index < vowels_len {
            if !self.remove_vowel(vowel_index, tone_placement) {
                return Err(SyllableBuildError::InvalidNucleus);
            }

            return Ok(InputEffect::StructurallyChanged);
        }

        // ─────────────────────────── Coda ───────────────────────────

        let coda_index = vowel_index - vowels_len;

        if !self.remove_coda(coda_index) {
            return Err(SyllableBuildError::InvalidCoda);
        }

        Ok(InputEffect::StructurallyChanged)
    }

    /// Removes one character from the onset, restoring it if the result is invalid.
    #[inline]
    fn remove_onset(&mut self, onset_index: usize) -> bool {
        debug_assert!(onset_index < self.onset.len());

        self.try_update_onset(
            |onset| onset.remove(onset_index),
            |onset, removed| _ = onset.insert(onset_index, removed),
        )
    }

    #[inline]
    fn remove_gi_onset(&mut self, onset_index: usize) -> bool {
        debug_assert!(onset_index < 2);

        // Remove i
        if onset_index == 1 {
            self.onset.pop();
            self.onset_kind = Onset::G;
            return true;
        }

        // remove G -> i becomes vowels
        let i = self.onset[1];
        if self.nucleus.len() >= NUCLEUS_MAX_LEN {
            return false;
        }
        if !self.try_update_nucleus(
            |nucleus| nucleus.insert(0, ExtendedBaseVowel::with_case(BaseVowel::I, i == 'I')),
            |nucleus, _| {
                nucleus.remove(0);
            },
        ) {
            return false;
        }

        self.onset.clear();
        self.onset_kind = Onset::None;

        true
    }

    /// Removes the vowel at `index`, clearing the tone when it targeted that vowel
    /// or the nucleus becomes empty.
    #[inline]
    fn remove_vowel(&mut self, vowel_index: usize, tone_placement: TonePlacement) -> bool {
        let len = self.nucleus.len();
        debug_assert!(vowel_index < len);

        // Nucleus can not be empty when coda is existed
        if len == 1 && !self.coda.is_empty() {
            return false;
        }

        // Never exist because if we has at least one vowel i always becomes onset
        // if len == 2 && self.onset_kind == Onset::G && self.nucleus[0].get() == BaseVowel::I {
        // }

        let tone_pos = if self.tone.is_some() {
            self.tone_vowel_index(tone_placement)
        } else {
            None
        };

        if !self.try_update_nucleus(
            |nucleus| nucleus.remove(vowel_index),
            |nucleus, old| {
                nucleus.insert(vowel_index, old);
            },
        ) {
            return false;
        }

        // Removing the vowel carrying the tone removes the tone as well.
        if tone_pos == Some(vowel_index) {
            self.tone = Tone::Flat;
        }

        true
    }

    /// Removes one char from the coda, restoring it if the result is invalid.
    #[inline]
    fn remove_coda(&mut self, index: usize) -> bool {
        debug_assert!(index < self.coda.len());

        self.try_update_coda(
            |coda| coda.remove(index),
            |coda, removed| _ = coda.insert(index, removed),
        )
    }
}
