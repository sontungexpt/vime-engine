//! Exhaustive checks on `nucleus_state`: pins population counts (Valid/InComplete/Dead)
//! over all 1..=3 vowel sequences (1884 total) and verifies >3 vowels are Dead.

use vime_engine::phonology::{nucleus_state, BaseVowel, NucleusState, NUCLEUS_MAX_LEN};

/// Every base vowel, in declaration order. Adding a variant means adding a row
/// here, and the population counts below will move — which is the point.
const ALL_VOWELS: &[BaseVowel] = &[
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

/// Every nucleus of length 1..=3: 12 + 144 + 1728 = 1884 sequences, in that
/// order.
///
/// Yields `(buffer, len)` rather than a `Vec` per sequence. The state table
/// only ever reads the slice, so the old shape — building 1884 throwaway
/// `Vec<BaseVowel>` to hand it one — allocated once per case for nothing. The
/// buffer is a fixed array, so this walks the same 1884 sequences in the same
/// order without touching the allocator at all.
fn all_nuclei() -> impl Iterator<Item = ([BaseVowel; NUCLEUS_MAX_LEN], usize)> {
    let one = ALL_VOWELS.iter().map(|&a| ([a; NUCLEUS_MAX_LEN], 1));
    let two = ALL_VOWELS
        .iter()
        .flat_map(|&a| ALL_VOWELS.iter().map(move |&b| ([a, b, b], 2)));
    let three = ALL_VOWELS.iter().flat_map(|&a| {
        ALL_VOWELS
            .iter()
            .flat_map(move |&b| ALL_VOWELS.iter().map(move |&c| ([a, b, c], 3)))
    });
    one.chain(two).chain(three)
}

/// The state distribution is pinned, so a table refactor that silently
/// reclassifies a family cannot pass unnoticed.
#[test]
fn nucleus_state_population_is_pinned() {
    let mut total = 0;
    let mut valid = 0;
    let mut incomplete = 0;
    let mut dead = 0;
    for (nucleus, len) in all_nuclei() {
        total += 1;
        match nucleus_state(&nucleus[..len]) {
            NucleusState::Valid => valid += 1,
            NucleusState::InComplete => incomplete += 1,
            NucleusState::Dead => dead += 1,
        }
    }

    assert_eq!(
        (total, valid, incomplete, dead),
        (1884, 56, 17, 1811),
        "nucleus table drifted: 12 + 12^2 + 12^3 = 1884 sequences, \
         of which 56 are real nuclei and 17 are real-but-unfinished"
    );
}

/// A single vowel is always a usable nucleus, and the empty one is always
/// merely incomplete — never `Dead`, or the builder would reject a word at its
/// first keystroke.
#[test]
fn every_single_vowel_is_valid() {
    for &vowel in ALL_VOWELS {
        assert_eq!(
            nucleus_state(&[vowel]),
            NucleusState::Valid,
            "{vowel:?} should be a complete nucleus on its own"
        );
    }
    assert_eq!(nucleus_state(&[] as &[BaseVowel]), NucleusState::InComplete);
}

/// `NUCLEUS_MAX_LEN` is 3, so a longer nucleus is not a nucleus at all.
///
/// This is the one bound the test suite did not previously pin: the benches
/// only ever generated sequences of length 1..=3, so the `_ => Dead` arm of
/// the table was never exercised.
#[test]
fn nuclei_longer_than_the_maximum_are_dead() {
    assert_eq!(NUCLEUS_MAX_LEN, 3, "this test assumes a max length of 3");

    for len in NUCLEUS_MAX_LEN + 1..=NUCLEUS_MAX_LEN + 3 {
        // Vary the tail so the test cannot pass on a repeated-suffix quirk.
        let mut nucleus = vec![BaseVowel::A; len];
        nucleus[3] = BaseVowel::I;
        assert_eq!(
            nucleus_state(nucleus.as_slice()),
            NucleusState::Dead,
            "a {len}-vowel nucleus is past NUCLEUS_MAX_LEN, so it must be dead: {nucleus:?}"
        );
    }
}
