//! Rendered output: what the user actually sees. The push corpus checks
//! internal state only, so these tests cover the render round trip, the
//! precomposed codec, and the modern/old tone-placement pairs the corpus
//! model cannot express.

use super::corpus::{
    gi, onsets, precomposed, syllables, telex_shapes, telex_tones, toggles, tones_shapes,
    uo_sequences, uppercase, Case, Outcome,
};

use crate::keymap::{DefaultKeymap, Keymap};
use crate::phonology::{decode_vowel, is_vowel, Tone, TonePlacement};
use crate::syllable::building::BuildingSyllable;

/// Every Telex corpus slice, borrowed from the shared data files. A const
/// table rather than a `Vec`: rebuilding ~850 elements per test showed up in
/// the profile for no benefit.
const TELEX_SLICES: &[&[Case]] = &[
    onsets::CASES,
    telex_tones::CASES,
    telex_shapes::CASES,
    tones_shapes::CASES,
    uo_sequences::CASES,
    precomposed::CASES,
    uppercase::CASES,
    toggles::CASES,
    syllables::CASES,
    gi::CASES,
];

/// Both tone-placement schemes, in the order the tests assert them.
const PLACEMENTS: [TonePlacement; 2] = [TonePlacement::Modern, TonePlacement::Old];

/// The live cases of [`TELEX_SLICES`]; [`Outcome::Dead`] cases roll back, so
/// their syllable is not a rendering of the input and has no output to check.
fn renderable_cases() -> impl Iterator<Item = &'static Case> {
    TELEX_SLICES
        .iter()
        .copied()
        .flatten()
        .filter(|case| !matches!(case.outcome, Outcome::Dead(_)))
}

fn push_all<KM: Keymap>(keymap: &KM, input: &[char]) -> BuildingSyllable {
    let mut builder = BuildingSyllable::default();
    for &ch in input {
        builder
            .push(keymap, ch)
            .unwrap_or_else(|e| panic!("push({ch:?}) failed for {input:?}: {e:?}"));
    }
    builder
}

/// One placement case: a push order plus the text each scheme must render.
struct PlacementCase {
    input: &'static [char],
    modern: &'static str,
    old: &'static str,
}

fn render(builder: &BuildingSyllable, placement: TonePlacement) -> String {
    builder.to_chars(placement).iter().collect()
}

fn rendered<KM: Keymap>(input: &[char], keymap: &KM, placement: TonePlacement) -> String {
    render(&push_all(keymap, input), placement)
}

/// A rendered tone mark must survive a round trip through the keymap.
/// Scoped to non-flat tones: the ASCII keymap is not injective (`ooo` renders
/// as `"oo"` and re-parses as `ô`), so flat shapes are left to the corpus and
/// the codec test covers the characters themselves.
#[test]
fn toned_corpus_cases_survive_a_render_round_trip() {
    let telex = DefaultKeymap::telex();
    let mut checked = 0usize;
    let mut failures = Vec::new();

    for case in renderable_cases() {
        // Built once and rendered twice; an earlier version re-pushed per placement.
        let builder = push_all(&telex, case.input);
        if builder.tone() == Tone::Flat {
            continue;
        }

        for placement in PLACEMENTS {
            let first = render(&builder, placement);
            let reparsed = first.chars().collect::<Vec<char>>();
            let second = rendered(&reparsed, &telex, placement);
            if first != second {
                failures.push(format!(
                    "input={:?} {placement:?}\n  first:  {first:?}\n  second: {second:?}",
                    case.input
                ));
            }
            checked += 1;
        }
    }

    assert!(
        checked >= 300,
        "expected the round trip to cover every toned case; got {checked}"
    );
    if !failures.is_empty() {
        panic!(
            "{} tone(s) lost in a render round trip:\n\n{}",
            failures.len(),
            failures.join("\n\n")
        );
    }
}

/// Every rendered character must decode back to itself, so the precomposed
/// table and the placement index cannot drift apart.
#[test]
fn rendered_vowels_round_trip_through_the_codec() {
    let telex = DefaultKeymap::telex();
    let mut vowels = 0usize;

    for case in renderable_cases() {
        // Pushed once, rendered under both schemes.
        let builder = push_all(&telex, case.input);
        for placement in PLACEMENTS {
            for ch in builder.to_chars(placement).iter().copied() {
                if !is_vowel(ch) {
                    continue;
                }
                let decoded = decode_vowel(ch)
                    .unwrap_or_else(|| panic!("rendered vowel {ch:?} is not decodable"));
                assert_eq!(
                    decoded.to_char(),
                    ch,
                    "codec mismatch for {ch:?} from {:?} {placement:?}",
                    case.input
                );
                vowels += 1;
            }
        }
    }

    assert!(vowels >= 700, "expected many rendered vowels; got {vowels}");
}

/// The placement rule decides which vowel carries the tone: the textbook cases
/// the corpus model cannot express, because it never looks at the output.
#[test]
fn classic_tone_placement() {
    let telex = DefaultKeymap::telex();

    // This Telex variant swaps two keys: `f` is grave and `r` is hook.
    const GRAVE: char = 'f';
    const ACUTE: char = 's';

    let cases = [
        // A shaped vowel always wins ("thuế", "cuối"); here the shape key is the letter itself.
        PlacementCase {
            input: &['t', 'h', 'u', 'e', 'e', ACUTE],
            modern: "thuế",
            old: "thuế",
        },
        // An unshaped triphthong puts the mark on the middle vowel.
        PlacementCase {
            input: &['h', 'o', 'a', 'i', GRAVE],
            modern: "hoài",
            old: "hoài",
        },
        // `uơ` needs a doubled horn key in this keymap to become `ươ`.
        PlacementCase {
            input: &['c', 'u', 'o', 'w', 'w', 'i', ACUTE],
            modern: "cưới",
            old: "cưới",
        },
        // A shaped vowel outranks the open-diphthong rule: `cuốn` marks `ô`.
        PlacementCase {
            input: &['c', 'u', 'o', 'o', 'n', ACUTE],
            modern: "cuốn",
            old: "cuốn",
        },
        // The coda never carries the tone: `anh` + acute -> "ánh".
        PlacementCase {
            input: &['a', 'n', 'h', ACUTE],
            modern: "ánh",
            old: "ánh",
        },
        // A closed `oa` marks the second vowel in both modes, as real orthography does ("hoán").
        PlacementCase {
            input: &['h', 'o', 'a', 'n', ACUTE],
            modern: "hoán",
            old: "hoán",
        },
        // Additional cases for better coverage
        // Single vowel: tone always on that vowel
        PlacementCase {
            input: &['t', 'a', ACUTE],
            modern: "tá",
            old: "tá",
        },
        // Two vowels with shape: shaped vowel wins (tone on ô)
        PlacementCase {
            input: &['t', 'r', 'u', 'o', 'w', 'n', 'g', ACUTE],
            modern: "trướng",
            old: "trướng",
        },
        // uy triphthong (grave tone: modern on y, old on u)
        PlacementCase {
            input: &['t', 'h', 'u', 'y', GRAVE],
            modern: "thuỳ",
            old: "thùy",
        },
        // ia triphthong
        PlacementCase {
            input: &['g', 'i', 'a', ACUTE],
            modern: "giá",
            old: "giá",
        },
        // ua diphthong (closed)
        PlacementCase {
            input: &['q', 'u', 'a', 'n', ACUTE],
            modern: "quán",
            old: "quán",
        },
    ];

    for case in cases {
        let builder = push_all(&telex, case.input);
        assert_eq!(
            render(&builder, TonePlacement::Modern),
            case.modern,
            "modern placement for {:?}",
            case.input
        );
        assert_eq!(
            render(&builder, TonePlacement::Old),
            case.old,
            "old placement for {:?}",
            case.input
        );
    }
}

/// `write_to` and `to_chars` must agree, in order, for both schemes across the
/// corpus: the render logic exists twice, and this is what keeps the two in step.
#[test]
fn write_to_matches_to_chars_across_the_corpus() {
    let telex = DefaultKeymap::telex();

    for placement in PLACEMENTS {
        for case in renderable_cases() {
            // The keymap is built once for the whole sweep, not per case.
            let builder = push_all(&telex, case.input);

            let mut written = String::new();
            builder.write_to(placement, &mut written);
            let buffered: String = builder.to_chars(placement).iter().collect();

            assert_eq!(
                written, buffered,
                "write_to != to_chars for {:?} under {placement:?}",
                case.input
            );
        }
    }
}

/// `write_to` appends, so it must leave anything already in the buffer alone.
#[test]
fn write_to_appends_rather_than_replaces() {
    let builder = push_all(&DefaultKeymap::telex(), &['n', 'g', 'u', 'y', 'e', 'n']);
    let expected = render(&builder, TonePlacement::Modern);

    let mut out = String::from("prefix|");
    builder.write_to(TonePlacement::Modern, &mut out);
    assert_eq!(out, format!("prefix|{expected}"));
}

/// Additional placement tests for shaped vowels
#[test]
fn tone_placement_on_shaped_vowels() {
    let telex = DefaultKeymap::telex();
    const ACUTE: char = 's';

    // Shaped vowels always win regardless of position
    let cases = [
        // Circumflex on first vowel of triphthong
        PlacementCase {
            input: &['t', 'r', 'u', 'o', 'w', 'n', 'g', ACUTE],
            modern: "trướng",
            old: "trướng",
        },
        // Horn on second vowel of uơ -> ươ
        PlacementCase {
            input: &['c', 'u', 'o', 'w', 'w', 'i', ACUTE],
            modern: "cưới",
            old: "cưới",
        },
    ];

    for case in cases {
        let builder = push_all(&telex, case.input);
        assert_eq!(
            render(&builder, TonePlacement::Modern),
            case.modern,
            "modern shaped for {:?}",
            case.input
        );
        assert_eq!(
            render(&builder, TonePlacement::Old),
            case.old,
            "old shaped for {:?}",
            case.input
        );
    }
}

#[test]
fn tone_placement_with_coda() {
    let telex = DefaultKeymap::telex();
    const ACUTE: char = 's';
    const GRAVE: char = 'f';

    // Coda never carries the tone
    let cases = [
        PlacementCase {
            input: &['a', 'n', 'h', ACUTE],
            modern: "ánh",
            old: "ánh",
        },
        PlacementCase {
            input: &['t', 'r', 'u', 'o', 'w', 'n', 'g', ACUTE],
            modern: "trướng",
            old: "trướng",
        },
        PlacementCase {
            input: &['t', 'h', 'a', 'n', GRAVE],
            modern: "thàn",
            old: "thàn",
        },
        PlacementCase {
            input: &['n', 'g', 'u', 'y', 'e', 'n', ACUTE],
            modern: "nguýen",
            old: "nguýen",
        },
    ];

    for case in cases {
        let builder = push_all(&telex, case.input);
        assert_eq!(
            render(&builder, TonePlacement::Modern),
            case.modern,
            "modern coda for {:?}",
            case.input
        );
        assert_eq!(
            render(&builder, TonePlacement::Old),
            case.old,
            "old coda for {:?}",
            case.input
        );
    }
}
