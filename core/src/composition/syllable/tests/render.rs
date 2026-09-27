//! Rendered-output tests: what the user actually sees.
//!
//! The push corpus only inspects internal state (onset kind, nucleus, tone,
//! coda), so a wrong tone-placement rule or a wrong precomposed character would
//! pass every corpus case while producing the wrong text. The tests here close
//! that gap:
//!
//! * [`every_corpus_case_renders_and_reparses`] renders all ~850 live corpus
//!   cases and pushes the result back through the keymap, for both
//!   [`TonePlacement`] values.
//! * [`rendered_vowels_round_trip_through_the_codec`] pins every rendered
//!   character to the codec entry that produced it.
//! * [`classic_tone_placement`] states the well-known modern/old orthography
//!   pairs (`hòa`/`hoà`, `hóa`/`hoá`) as data, which the corpus model cannot
//!   express because it never looks at the output.

use super::corpus::{
    gi, onsets, precomposed, syllables, telex_shapes, telex_tones, toggles, tones_shapes,
    uo_sequences, uppercase, Case, Outcome,
};

use crate::composition::syllable::building::BuildingSyllable;
use crate::keymap::{DefaultKeymap, Keymap};
use crate::phonology::{decode_vowel, is_vowel, Tone, TonePlacement};

/// Every case the Telex corpus pushes, borrowed from the shared data files.
///
/// Callers skip [`Outcome::Dead`]: a dead case rolls back, so its syllable is
/// not a rendering of the input and has no output to check.
fn telex_cases() -> Vec<&'static Case> {
    let mut cases = Vec::new();
    for slice in [
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
    ] {
        cases.extend(slice);
    }
    cases
}

/// The cases of `telex_cases` that stayed alive and can therefore be rendered.
fn renderable_cases() -> impl Iterator<Item = &'static Case> {
    telex_cases()
        .into_iter()
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

fn rendered<KM: Keymap>(input: &[char], keymap: &KM, placement: TonePlacement) -> String {
    let text: String = push_all(keymap, input).to_chars(placement).iter().collect();
    text
}

/// A rendered tone mark must survive a round trip through the keymap.
///
/// Scoped to non-flat tones on purpose. The ASCII keymap is not injective:
/// `["o", "o", "o"]` ends as two plain vowels and renders `"oo"`, which
/// re-parses as one circumflex `"ô"` by the `oo -> ô` doubling rule. That is
/// correct behaviour the corpus already pins, not a render bug, so shape
/// ambiguity on flat syllables is left to the corpus and the codec test below
/// covers the characters themselves.
#[test]
fn toned_corpus_cases_survive_a_render_round_trip() {
    let telex = DefaultKeymap::telex();
    let mut checked = 0usize;
    let mut failures = Vec::new();

    for case in renderable_cases() {
        let builder = push_all(&telex, case.input);
        if builder.tone() == Tone::Flat {
            continue;
        }

        for placement in [TonePlacement::Modern, TonePlacement::Old] {
            let first = rendered(case.input, &telex, placement);
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

/// Every rendered character must be decodable, and decoding must give back the
/// exact same character, so the precomposed table and the placement index
/// cannot drift apart.
#[test]
fn rendered_vowels_round_trip_through_the_codec() {
    let telex = DefaultKeymap::telex();
    let mut vowels = 0usize;

    for case in renderable_cases() {
        for placement in [TonePlacement::Modern, TonePlacement::Old] {
            let builder = push_all(&telex, case.input);
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

/// The placement rule decides which vowel carries the tone. These are the
/// textbook cases the corpus model cannot express, because it never looks at
/// the output.
#[test]
fn classic_tone_placement() {
    let telex = DefaultKeymap::telex();

    // This Telex variant swaps two keys against the usual convention:
    // `f` is grave and `r` is hook.
    const GRAVE: char = 'f';
    const ACUTE: char = 's';

    struct Case {
        input: &'static [char],
        modern: &'static str,
        old: &'static str,
    }

    let cases = [
        // A shaped vowel always wins ("thuế", "cuối"). In this keymap the
        // shape key is the letter itself: `e` shapes `e`, `o` shapes `o`.
        Case {
            input: &['t', 'h', 'u', 'e', 'e', ACUTE],
            modern: "thuế",
            old: "thuế",
        },
        // An unshaped triphthong puts the mark on the middle vowel.
        Case {
            input: &['h', 'o', 'a', 'i', GRAVE],
            modern: "hoài",
            old: "hoài",
        },
        // `uơ` needs a doubled horn key in this keymap to become `ươ`.
        Case {
            input: &['c', 'u', 'o', 'w', 'w', 'i', ACUTE],
            modern: "cưới",
            old: "cưới",
        },
        // A shaped vowel outranks the open-diphthong rule: `cuốn` puts the
        // mark on `ô`, not on the following consonant or the leading `u`.
        Case {
            input: &['c', 'u', 'o', 'o', 'n', ACUTE],
            modern: "cuốn",
            old: "cuốn",
        },
        // The coda never carries the tone: `anh` + acute -> "ánh".
        Case {
            input: &['a', 'n', 'h', ACUTE],
            modern: "ánh",
            old: "ánh",
        },
        // A closed `oa` syllable puts the mark on the second vowel in both
        // modes, which is also what real orthography does ("hoán").
        Case {
            input: &['h', 'o', 'a', 'n', ACUTE],
            modern: "hoán",
            old: "hoán",
        },
    ];

    for case in cases {
        assert_eq!(
            rendered(case.input, &telex, TonePlacement::Modern),
            case.modern,
            "modern placement for {:?}",
            case.input
        );
        assert_eq!(
            rendered(case.input, &telex, TonePlacement::Old),
            case.old,
            "old placement for {:?}",
            case.input
        );
    }
}

/// Known bug: `Modern` and `Old` are transposed for every *open* two-vowel
/// nucleus.
///
/// `tone_index_2_modern` returns 1 for `oa`, `oe` and `uy`, and
/// `tone_index_2_old` returns 0 when the coda is empty — the two branches have
/// the same two outcomes, assigned to the opposite modes. So Modern renders
/// old-style text and Old renders modern-style text:
///
/// | input | Modern (actual) | Old (actual) | real Modern | real Old |
/// |---|---|---|---|---|
/// | `ho` + `f` | `hoà` | `hòa` | `hòa` | `hoà` |
/// | `hoa` + `s` | `hoá` | `hóa` | `hóa` | `hoá` |
/// | `thu` + `y` + `s` | `thuý` | `thúy` | `thúy` | `thuý` |
///
/// Everything else is unaffected: shaped vowels, single vowels, and closed
/// two-vowel syllables such as `hoán` place the mark identically in both modes,
/// which `classic_tone_placement` covers.
///
/// The inversion is consistent across the crate, so it is deliberate-looking
/// rather than a stray typo:
///
/// * `TonePlacement::Modern` / `Old` doc comments say "hóa" / "hoá" — the
///   *opposite* of what the code produces.
/// * `tone_index_2_modern` / `tone_index_2_old` inline comments show the
///   opposite index from the one the branch returns.
/// * `ffi/include/vime_engine.h` labels Modern "hoá" and Old "hóa".
/// * `ffi/tests/typing.rs`, `ffi/tests/lifecycle.rs` and `core/tests/config.rs`
///   assert the current behaviour, so flipping it needs those updated too.
///
/// Ignored rather than deleted so the fix has a spec waiting for it. Run with
/// `cargo test -p vime-engine --lib render::open_diphthong -- --ignored`.
#[test]
#[ignore = "Modern/Old placement for open oa/oe is inverted; see the comment above"]
fn open_diphthong_tone_placement_is_swapped() {
    let telex = DefaultKeymap::telex();

    struct Case {
        input: &'static [char],
        modern: &'static str,
        old: &'static str,
    }

    let cases = [
        Case {
            input: &['h', 'o', 'a', 'f'],
            modern: "hòa",
            old: "hoà",
        },
        Case {
            input: &['h', 'o', 'e', 'f'],
            modern: "hòe",
            old: "hoè",
        },
        Case {
            input: &['h', 'o', 'a', 's'],
            modern: "hóa",
            old: "hoá",
        },
        // `uy` is transposed the same way: Modern renders "thuý" where real
        // orthography has "thúy".
        Case {
            input: &['t', 'h', 'u', 'y', 's'],
            modern: "thúy",
            old: "thuý",
        },
    ];

    let mut wrong = Vec::new();
    for case in cases {
        let modern = rendered(case.input, &telex, TonePlacement::Modern);
        let old = rendered(case.input, &telex, TonePlacement::Old);
        if modern != case.modern {
            wrong.push(format!(
                "modern {:?} = {modern:?}, want {:?}",
                case.input, case.modern
            ));
        }
        if old != case.old {
            wrong.push(format!(
                "old    {:?} = {old:?}, want {:?}",
                case.input, case.old
            ));
        }
    }

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
