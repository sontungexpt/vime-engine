//! Append path: `BuildingSyllable::push`, which always acts at the end of the
//! syllable: before the nucleus or coda starts, a key may extend the onset or
//! begin the nucleus; afterwards it may transform, or extend the nucleus or coda.

use super::*;
use crate::{
    keymap::Keymap,
    phonology::{BaseVowel, Onset, Tone, Vowel, NUCLEUS_MAX_LEN},
};

impl BuildingSyllable {
    // ─────────────────────────── Push ───────────────────────────

    pub fn push<KM: Keymap>(
        &mut self,
        keymap: &KM,
        key: char,
    ) -> Result<TransformEffect, SyllableBuildError> {
        // ─────────────────────────── Onset ───────────────────────────
        // No nucleus or coda yet: try the D/Đ stroke, then the onset, then the first vowel.
        if self.coda.is_empty() && self.nucleus.is_empty() {
            let effect = self.try_toggle_d_stroke(keymap, key);

            if matches!(effect, TransformEffect::Applied(_)) {
                return Ok(effect);
            }

            if self.push_onset(key) {
                return Ok(effect);
            }

            // A pending Q can only continue as QU.
            if self.onset.len() == 1 && is_q_ignore_case(self.onset[0]) {
                return Err(SyllableBuildError::InvalidOnset);
            }

            let Some(vowel) = Vowel::from_char(key) else {
                return Err(SyllableBuildError::InvalidOnset);
            };

            if !self.push_vowel(vowel) {
                return Err(SyllableBuildError::InvalidNucleus);
            }

            return Ok(effect);
        }

        // ─────────────────────────── Nucleus ───────────────────────────
        // After the first vowel, transforms get first chance at each key.
        let effect = self.try_transform(keymap, key, None);
        if matches!(effect, TransformEffect::Applied(_)) {
            return Ok(effect);
        }

        if self.coda.is_empty() {
            // With no coda yet, vowels extend the nucleus; other keys may start the coda.
            let Some(vowel) = Vowel::from_char(key) else {
                if self.push_coda(key) {
                    // Once a coda starts, normalize `uơ` / `ưo` to `ươ`.
                    self.normalize_uo_horn();
                    return Ok(effect);
                }

                return Err(SyllableBuildError::InvalidCoda);
            };

            if !self.push_vowel(vowel) {
                return Err(SyllableBuildError::InvalidNucleus);
            }

            self.normalize_uo_horn();
            return Ok(effect);
        }

        // A coda has started, so only another coda character can follow.
        if self.push_coda(key) {
            return Ok(effect);
        }

        Err(SyllableBuildError::InvalidCoda)
    }

    /// Adds a literal character to the onset. Returns `false` when the key
    /// should be considered for the nucleus. `q` waits for `u`, while `i` is
    /// left for the nucleus to keep `gi` ambiguous until another vowel arrives.
    #[inline]
    fn push_onset(&mut self, key: char) -> bool {
        if is_i_ignore_case(key) {
            return false;
        }

        if self.onset.is_empty() && is_q_ignore_case(key) {
            self.onset.push(key);
            return true;
        }

        if self.onset.len() < Onset::MAX_LEN {
            return self.try_update_onset(
                |onset| onset.push(key),
                |onset, _| {
                    onset.pop();
                },
            );
        }

        false
    }

    /// Adds a decoded vowel to the nucleus (max 3); `false` on a tone conflict
    /// or an invalid nucleus. `pub(super)` because `insert` delegates here for
    /// an insertion at the end of the nucleus.
    #[inline]
    pub(super) fn push_vowel(&mut self, vowel: Vowel) -> bool {
        let tone = vowel.tone();
        let len = self.nucleus.len();

        if len >= NUCLEUS_MAX_LEN {
            return false;
        }

        // The first vowel sets the syllable tone.
        if len == 0 {
            self.nucleus.push(vowel);
            // Nucleus with a single vowel is always valid.
            self.nucleus_state = NucleusState::Valid;

            self.tone = tone;
            return true;
        }

        // A non-flat vowel conflicts with an existing non-flat tone (`á` + `ắ`),
        // but an unmarked vowel can follow one (`á` + `a`).
        let new_tone = match (self.tone, tone) {
            (Tone::Flat, incoming) => incoming,

            (current, Tone::Flat) => current,

            (_, _) => return false,
        };

        // When a vowel follows `g i`, move `i` into the onset: `G + I + V` -> `Gi + V`.
        let should_form_gi =
            len == 1 && self.onset_kind == Onset::G && self.nucleus[0].base() == BaseVowel::I;

        if should_form_gi {
            let i = self.nucleus.pop().expect("nucleus contains i");

            self.onset.push(if i.is_upper() { 'I' } else { 'i' });
            self.onset_kind = Onset::Gi;

            // The new nucleus has one vowel, so adopt its tone directly.
            self.nucleus.push(vowel);
            // The nucleus is now valid, as it has one vowel.
            self.nucleus_state = NucleusState::Valid;
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

    /// Adds a literal character to the coda, returning `false` if it is invalid.
    #[inline]
    fn push_coda(&mut self, key: char) -> bool {
        if self.coda.len() < Coda::MAX_LEN {
            return self.try_update_coda(
                |coda| coda.push(key),
                |coda, _| {
                    coda.pop();
                },
            );
        }
        false
    }
}
