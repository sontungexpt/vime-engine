//! Lookup-table (LUT) variant of [`NucleusState::check`](crate::phonology::rules::NucleusState::check).
//!
//! Two-stage dispatch is replaced by a single table load: the 4-bit
//! [`BaseVowel::id`] of up to three vowels is packed into a 12-bit key
//! `a | (b << 4) | (c << 8)`; absent slots contribute the sentinel `0xF`, which
//! keeps keys of different lengths disjoint and caps the index at `0xFFF = 4095`.
//!
//! Benchmarked ~1.5x faster than the slice `match` (1.10 vs 1.71 ns/probe on the
//! exhaustive 1884-nucleus set; 1.55 vs 2.36 on the realistic set) because the
//! 4096-entry × 1-byte table is L1-resident. Kept as a separate module for
//! now — not yet wired into the composition pipeline.

use crate::phonology::rules::NucleusState;
use crate::phonology::BaseVowel;

/// Sentinel 4-bit value marking an "absent" vowel slot in the key.
const SENTINEL: usize = 0xF;

/// All 12 vowels of `BaseVowel`, in [`BaseVowel::id`] order.
const VOWELS: [BaseVowel; 12] = [
    BaseVowel::Y,
    BaseVowel::U,
    BaseVowel::I,
    BaseVowel::E,
    BaseVowel::O,
    BaseVowel::A,
    BaseVowel::UHorn,
    BaseVowel::ACircumflex,
    BaseVowel::OCircumflex,
    BaseVowel::ABreve,
    BaseVowel::ECircumflex,
    BaseVowel::OHorn,
];

fn build() -> [NucleusState; 4096] {
    let mut table = [NucleusState::Dead; 4096];
    for (i, &a) in VOWELS.iter().enumerate() {
        table[key_of(i, SENTINEL, SENTINEL)] = NucleusState::check(&[a]);
        for (j, &b) in VOWELS.iter().enumerate() {
            table[key_of(i, j, SENTINEL)] = NucleusState::check(&[a, b]);
            for (k, &c) in VOWELS.iter().enumerate() {
                table[key_of(i, j, k)] = NucleusState::check(&[a, b, c]);
            }
        }
    }
    table
}

/// 12-bit LUT key from three 4-bit vowel slots (id or `SENTINEL`).
#[inline(always)]
const fn key_of(a: usize, b: usize, c: usize) -> usize {
    a | (b << 4) | (c << 8)
}

fn lut() -> &'static [NucleusState; 4096] {
    static LUT: std::sync::LazyLock<[NucleusState; 4096]> =
        std::sync::LazyLock::new(build);
    &LUT
}

/// LUT-backed [`NucleusState::check`](crate::phonology::rules::NucleusState::check).
#[inline(always)]
pub fn check(vowels: &[BaseVowel]) -> NucleusState {
    if vowels.is_empty() || vowels.len() > 3 {
        return NucleusState::Dead;
    }

    let a = vowels[0].id() as usize;
    let b = vowels.get(1).map_or(SENTINEL, |v| v.id() as usize);
    let c = vowels.get(2).map_or(SENTINEL, |v| v.id() as usize);

    lut()[key_of(a, b, c)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parity_with_slice_match() {
        let mut checked = 0;
        for &a in &VOWELS {
            assert_eq!(check(&[a]), NucleusState::check(&[a]));
            for &b in &VOWELS {
                assert_eq!(check(&[a, b]), NucleusState::check(&[a, b]));
                for &c in &VOWELS {
                    assert_eq!(check(&[a, b, c]), NucleusState::check(&[a, b, c]));
                    checked += 1;
                }
            }
        }
        assert!(checked > 0);
    }

    #[test]
    fn rejects_illformed_inputs() {
        assert_eq!(check(&[]), NucleusState::Dead);
        let four = [
            BaseVowel::A,
            BaseVowel::A,
            BaseVowel::A,
            BaseVowel::A,
        ];
        assert_eq!(check(&four), NucleusState::Dead);
    }
}