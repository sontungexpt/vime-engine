//! Integration tests for `validate_phonotactics` — Vietnamese syllable spelling rules.
//! Each rule tested with valid words and near-miss rejections. Cases as tuples for readability.

use vime_engine::phonology::{
    validate_phonotactics, BaseVowel, Coda, Onset, PhonotacticError, Tone,
};

use BaseVowel::{ABreve, ACircumflex, ECircumflex, OCircumflex, OHorn, UHorn};
use BaseVowel::{A, E, I, O, U, Y};

/// A syllable the rules accept: `(word, onset, nucleus, tone, coda)`.
type Ok_ = (&'static str, Onset, &'static [BaseVowel], Tone, Coda);

/// A syllable they reject: `Ok_` plus the error that must come back.
type Bad = (
    &'static str,
    Onset,
    &'static [BaseVowel],
    Tone,
    Coda,
    PhonotacticError,
);

/// Asserts every case is accepted, reporting all failures together.
fn check_ok(cases: &[Ok_]) {
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|&(word, onset, nucleus, tone, coda)| {
            match validate_phonotactics(onset, nucleus, tone, coda) {
                Ok(()) => None,
                Err(e) => Some(format!("  {word}: rejected with {e:?}")),
            }
        })
        .collect();

    assert!(
        failures.is_empty(),
        "{} syllable(s) should be accepted but were not:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Asserts every case is rejected with its expected error, reporting all
/// failures together.
fn check_err(cases: &[Bad]) {
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|&(word, onset, nucleus, tone, coda, expected)| {
            match validate_phonotactics(onset, nucleus, tone, coda) {
                Err(e) if e == expected => None,
                Ok(()) => Some(format!("  {word}: accepted, expected {expected:?}")),
                Err(e) => Some(format!("  {word}: expected {expected:?}, got {e:?}")),
            }
        })
        .collect();

    assert!(
        failures.is_empty(),
        "{} syllable(s) were not rejected as expected:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ─────────────────────────────────────────────────── a nucleus is required

/// Nothing can be judged before there is a vowel: `b` on its own is a fragment,
/// not a misspelling.
#[test]
fn an_empty_nucleus_is_incomplete() {
    assert_eq!(
        validate_phonotactics(Onset::B, &[] as &[BaseVowel], Tone::Flat, Coda::None),
        Err(PhonotacticError::IncompleteNucleus)
    );
}

// ────────────────────────────────────── onsets that require a front vowel

/// `k`, `gh` and `ngh` must be followed by `i`, `e`, `ê` or `y`.
#[test]
fn front_required_onsets_accept_front_vowels() {
    check_ok(&[
        ("kêu", Onset::K, &[ECircumflex, U], Tone::Flat, Coda::None),
        ("ký", Onset::K, &[Y], Tone::Acute, Coda::None),
        ("kiên", Onset::K, &[I, ECircumflex], Tone::Flat, Coda::N),
        ("ghe", Onset::Gh, &[E], Tone::Flat, Coda::None),
        ("ghê", Onset::Gh, &[ECircumflex], Tone::Flat, Coda::None),
        ("ghi", Onset::Gh, &[I], Tone::Flat, Coda::None),
        ("nghe", Onset::Ngh, &[E], Tone::Flat, Coda::None),
        ("nghĩ", Onset::Ngh, &[I], Tone::Tilde, Coda::None),
    ]);
}

#[test]
fn front_required_onsets_reject_other_vowels() {
    let want = PhonotacticError::MissingFrontVowel;
    check_err(&[
        ("ka", Onset::K, &[A], Tone::Flat, Coda::None, want),
        ("kô", Onset::K, &[OCircumflex], Tone::Flat, Coda::None, want),
        ("ku", Onset::K, &[U], Tone::Flat, Coda::None, want),
        ("kư", Onset::K, &[UHorn], Tone::Flat, Coda::None, want),
        (
            "ghô",
            Onset::Gh,
            &[OCircumflex],
            Tone::Flat,
            Coda::None,
            want,
        ),
        (
            "nghô",
            Onset::Ngh,
            &[OCircumflex],
            Tone::Flat,
            Coda::None,
            want,
        ),
    ]);
}

/// `kh` is spelled with an `h` and takes an ordinary vowel, so `kho`, `khoa` and
/// `khúc` are ordinary words. Only bare `k` is restricted.
#[test]
fn kh_is_not_a_front_required_onset() {
    check_ok(&[
        ("kho", Onset::Kh, &[O], Tone::Flat, Coda::None),
        ("khoa", Onset::Kh, &[O, A], Tone::Flat, Coda::None),
        ("khúc", Onset::Kh, &[U], Tone::Acute, Coda::C),
        ("khuya", Onset::Kh, &[U, Y, A], Tone::Flat, Coda::None),
    ]);
}

// ───────────────────────────────────── onsets that forbid a front vowel

#[test]
fn front_forbidden_onsets_accept_other_vowels() {
    check_ok(&[
        ("cá", Onset::C, &[A], Tone::Acute, Coda::None),
        ("gà", Onset::G, &[A], Tone::Grave, Coda::None),
        ("ngủ", Onset::Ng, &[U], Tone::Hook, Coda::None),
        ("co", Onset::C, &[O], Tone::Flat, Coda::None),
    ]);
}

#[test]
fn front_forbidden_onsets_reject_front_vowels() {
    let want = PhonotacticError::ForbiddenFrontVowel;
    check_err(&[
        ("ci", Onset::C, &[I], Tone::Flat, Coda::None, want),
        ("cê", Onset::C, &[ECircumflex], Tone::Flat, Coda::None, want),
        ("gê", Onset::G, &[ECircumflex], Tone::Flat, Coda::None, want),
        ("ngi", Onset::Ng, &[I], Tone::Flat, Coda::None, want),
    ]);
}

/// Only the **first** vowel counts. `cái` is a word because its `i` comes
/// second; a rule that scanned the whole nucleus would reject all of these.
#[test]
fn a_front_vowel_in_second_position_is_fine() {
    check_ok(&[
        ("cái", Onset::C, &[A, I], Tone::Dot, Coda::None),
        ("cay", Onset::C, &[A, Y], Tone::Flat, Coda::None),
        ("coi", Onset::C, &[O, I], Tone::Flat, Coda::None),
        ("củi", Onset::C, &[U, I], Tone::Tilde, Coda::None),
        ("gai", Onset::G, &[A, I], Tone::Flat, Coda::None),
    ]);
}

// ───────────────────────────────────────── glides already inside the onset

/// `qu` already carries its `w` and `gi` already carries its `i`, so neither may
/// be doubled.
#[test]
fn qu_and_gi_cannot_double_their_glide() {
    check_ok(&[
        ("qua", Onset::Qu, &[A], Tone::Flat, Coda::None),
        ("quy", Onset::Qu, &[Y], Tone::Flat, Coda::None),
        ("giá", Onset::Gi, &[A], Tone::Acute, Coda::None),
    ]);

    check_err(&[
        (
            "quu",
            Onset::Qu,
            &[U],
            Tone::Flat,
            Coda::None,
            PhonotacticError::GlideAfterQu,
        ),
        (
            "gui",
            Onset::Gi,
            &[I],
            Tone::Flat,
            Coda::None,
            PhonotacticError::GlideAfterGi,
        ),
    ]);
}

// ─────────────────────────────────────────────────────── codas and tone

/// A coda that closes the mouth — `p`, `t`, `c`, `ch` — takes only sắc or nặng.
#[test]
fn checked_codas_need_an_entering_tone() {
    check_ok(&[
        ("học", Onset::H, &[O], Tone::Dot, Coda::C),
        ("lớp", Onset::L, &[OHorn], Tone::Acute, Coda::P),
        ("mát", Onset::M, &[A, ACircumflex], Tone::Dot, Coda::T),
        ("tích", Onset::T, &[I], Tone::Dot, Coda::Ch),
        ("tốt", Onset::T, &[OCircumflex], Tone::Acute, Coda::T),
    ]);

    let want = PhonotacticError::EnteringToneRequired;
    check_err(&[
        ("hoc, no tone", Onset::H, &[O], Tone::Flat, Coda::C, want),
        ("at, no tone", Onset::None, &[A], Tone::Flat, Coda::T, want),
    ]);
}

/// A coda that does not close the mouth is free with any tone.
#[test]
fn open_codas_take_any_tone() {
    check_ok(&[
        ("an", Onset::None, &[A], Tone::Flat, Coda::N),
        ("ông", Onset::None, &[OCircumflex], Tone::Hook, Coda::Ng),
        ("tân", Onset::T, &[A, ACircumflex], Tone::Flat, Coda::N),
    ]);
}

// ───────────────────────────────────────────────────────── palatal codas

/// `ch` and `nh` need one of `i`, `e`, `ê`, `y`, `a` directly in front.
#[test]
fn palatal_codas_need_a_matching_vowel() {
    check_ok(&[
        ("ích", Onset::None, &[I], Tone::Dot, Coda::Ch),
        ("lịch", Onset::L, &[I], Tone::Dot, Coda::Ch),
        ("kính", Onset::K, &[I], Tone::Dot, Coda::Nh),
        ("anh", Onset::None, &[A], Tone::Flat, Coda::Nh),
        ("bạch", Onset::B, &[A], Tone::Dot, Coda::Ch),
    ]);

    let want = PhonotacticError::PalatalCodaVowelMismatch;
    check_err(&[
        ("o then ch", Onset::None, &[O], Tone::Dot, Coda::Ch, want),
        ("u then nh", Onset::None, &[U], Tone::Flat, Coda::Nh, want),
        (
            "ư then ch",
            Onset::None,
            &[UHorn],
            Tone::Dot,
            Coda::Ch,
            want,
        ),
    ]);
}

// ───────────────────────────────────────────────────────── short vowels

/// `ă` and `â` are too short to stand alone, so an open syllable is wrong.
#[test]
fn short_vowels_need_a_coda() {
    let want = PhonotacticError::ShortVowelRequiresCoda;
    check_err(&[
        ("ă alone", Onset::S, &[ABreve], Tone::Flat, Coda::None, want),
        (
            "â alone",
            Onset::T,
            &[ACircumflex],
            Tone::Flat,
            Coda::None,
            want,
        ),
    ]);
}

/// `ă` takes c / m / n / ng outright. `ch` is in the `ă` set too, but no
/// Vietnamese syllable spells `ăch` — `bách`, `mạch` and `sạch` all use plain
/// `a` — and the palatal rule runs first anyway, so `ă` before `ch` is
/// reported as a palatal mismatch (rejected below).
#[test]
fn a_breve_takes_its_codas() {
    check_ok(&[
        ("căn", Onset::C, &[A, ABreve], Tone::Flat, Coda::N),
        ("mắc", Onset::M, &[A, ABreve], Tone::Dot, Coda::C),
        ("ngăn", Onset::None, &[A, ABreve], Tone::Flat, Coda::Ng),
        ("lắm", Onset::L, &[A, ABreve], Tone::Acute, Coda::M),
    ]);

    // `ch` claims `ă` first: the palatal rule fires before the short-vowel
    // rule ever looks at the coda set.
    let shadowed = PhonotacticError::PalatalCodaVowelMismatch;
    check_err(&[(
        "ă then ch",
        Onset::None,
        &[ABreve],
        Tone::Acute,
        Coda::Ch,
        shadowed,
    )]);

    // `p` is not in the `ă` set at all, so this one does report the rule.
    let want = PhonotacticError::ShortVowelCodaMismatch;
    check_err(&[(
        "ă then p",
        Onset::None,
        &[ABreve],
        Tone::Acute,
        Coda::P,
        want,
    )]);
}

/// `â` takes c / m / n / ng / nh / p / t — wider than `ă`, which is why `mất`
/// is a word but `mắp` is not.
#[test]
fn a_circumflex_takes_its_seven_codas() {
    check_ok(&[
        ("cân", Onset::C, &[A, ACircumflex], Tone::Flat, Coda::N),
        ("cấp", Onset::C, &[A, ACircumflex], Tone::Acute, Coda::P),
        ("mất", Onset::M, &[A, ACircumflex], Tone::Dot, Coda::T),
        ("bệnh", Onset::B, &[E], Tone::Dot, Coda::Nh),
    ]);

    // `ch` is the one coda outside the `â` set — but the palatal rule answers
    // first, and `â` is none of `i e ê y a`, so `âch` is reported as a palatal
    // mismatch before the short-vowel rule is reached.
    let want = PhonotacticError::PalatalCodaVowelMismatch;
    check_err(&[(
        "â then ch",
        Onset::None,
        &[ACircumflex],
        Tone::Acute,
        Coda::Ch,
        want,
    )]);
}

/// `âu` and `ây` are diphthongs spelled a + â + vowel. The `â` is inside the
/// nucleus, but the syllable is not short, so it stays open.
#[test]
fn a_circumflex_diphthongs_stay_open() {
    check_ok(&[
        (
            "cau",
            Onset::C,
            &[A, ACircumflex, U],
            Tone::Flat,
            Coda::None,
        ),
        (
            "bay",
            Onset::B,
            &[A, ACircumflex, Y],
            Tone::Flat,
            Coda::None,
        ),
    ]);
}

// ─────────────────────────────────────────────── a lone `ơ` before `c`

/// `ơ` on its own takes no `c`. The language writes `o` + `c` (`học`) or the
/// `uơ` / `ươ` diphthong (`lược`, `ngược`) instead.
#[test]
fn a_lone_open_vowel_refuses_final_c() {
    let want = PhonotacticError::OpenVowelCodaMismatch;
    check_err(&[("ơ then c", Onset::None, &[OHorn], Tone::Dot, Coda::C, want)]);

    check_ok(&[
        // The same syllable with plain `o` is an ordinary word.
        ("học", Onset::H, &[O], Tone::Dot, Coda::C),
        // With `u` in front it is the diphthong, so also fine.
        ("lược", Onset::L, &[U, OHorn], Tone::Dot, Coda::C),
        ("ngược", Onset::Ng, &[U, OHorn], Tone::Dot, Coda::C),
    ]);
}

/// `ơ` before `p` or `t` is fine even on its own — `lớp`, `hớp` and `chớt` are
/// all real words. Only `c` is restricted.
#[test]
fn an_open_vowel_takes_p_and_t() {
    check_ok(&[
        ("lớp", Onset::L, &[OHorn], Tone::Acute, Coda::P),
        ("hớp", Onset::H, &[OHorn], Tone::Acute, Coda::P),
        ("chót", Onset::Ch, &[O], Tone::Acute, Coda::T),
        ("luốt", Onset::L, &[U, OHorn], Tone::Acute, Coda::T),
    ]);
}

// ──────────────────────────────────────────────── vowels before `ng`

/// `i` and `e` do not sit directly before `ng`.
#[test]
fn i_and_e_do_not_precede_ng() {
    check_err(&[
        (
            "i then ng",
            Onset::None,
            &[I],
            Tone::Flat,
            Coda::Ng,
            PhonotacticError::IBeforeNg,
        ),
        (
            "e then ng",
            Onset::None,
            &[E],
            Tone::Flat,
            Coda::Ng,
            PhonotacticError::EBeforeNg,
        ),
    ]);

    check_ok(&[
        ("ông", Onset::None, &[OCircumflex], Tone::Hook, Coda::Ng),
        ("âng", Onset::None, &[A, ACircumflex], Tone::Flat, Coda::Ng),
        ("ưng", Onset::None, &[UHorn], Tone::Flat, Coda::Ng),
    ]);
}

// ────────────────────────────────────────────────── onsetless syllables

/// No onset means no onset rules, so the vowel and coda rules decide alone.
#[test]
fn an_onsetless_syllable_only_obeys_the_coda_rules() {
    check_ok(&[
        ("an", Onset::None, &[A], Tone::Flat, Coda::N),
        ("eo", Onset::None, &[E, O], Tone::Flat, Coda::None),
        ("ươi", Onset::None, &[U, OHorn, I], Tone::Flat, Coda::None),
    ]);

    // Still subject to the coda rules.
    check_err(&[(
        "i then ng",
        Onset::None,
        &[I],
        Tone::Flat,
        Coda::Ng,
        PhonotacticError::IBeforeNg,
    )]);
}

// ─────────────────────────────────────────── a rule that cannot fire today

/// `RoundedVowelBeforeCh` is shadowed by the palatal rule. `u` and `ư` are not
/// in `PALATAL_CODA_VOWELS`, so that rule rejects them first and this one is
/// never reached.
///
/// It is kept so the constraint stays stated on its own terms, and so the
/// variant keeps a reachable producer if the palatal set is ever widened. This
/// test pins the behaviour that actually happens today rather than the one the
/// variant is named after.
#[test]
fn rounded_vowel_before_ch_is_shadowed_by_the_palatal_rule() {
    assert_eq!(
        validate_phonotactics(Onset::None, &[U], Tone::Dot, Coda::Ch),
        Err(PhonotacticError::PalatalCodaVowelMismatch),
    );
    assert_eq!(
        validate_phonotactics(Onset::None, &[UHorn], Tone::Dot, Coda::Ch),
        Err(PhonotacticError::PalatalCodaVowelMismatch),
    );
}

// ───────────────────────────────────────────────── every rule stays covered

/// Guards the suite against a rule being added without a case.
///
/// The count is the check: a new variant in [`PhonotacticError`] that nobody
/// wrote a test for leaves this list short and the assert fires. Naming each
/// variant also means a rename breaks the build here, rather than quietly
/// turning a test into a no-op.
#[test]
fn every_error_variant_is_covered() {
    let covered = [
        PhonotacticError::IncompleteNucleus,
        PhonotacticError::MissingFrontVowel,
        PhonotacticError::ForbiddenFrontVowel,
        PhonotacticError::GlideAfterQu,
        PhonotacticError::GlideAfterGi,
        PhonotacticError::EnteringToneRequired,
        PhonotacticError::PalatalCodaVowelMismatch,
        PhonotacticError::ShortVowelRequiresCoda,
        PhonotacticError::ShortVowelCodaMismatch,
        PhonotacticError::RoundedVowelBeforeCh, // via the shadowing test above
        PhonotacticError::OpenVowelCodaMismatch,
        PhonotacticError::IBeforeNg,
        PhonotacticError::EBeforeNg,
    ];

    const VARIANT_COUNT: usize = 13;
    assert_eq!(
        covered.len(),
        VARIANT_COUNT,
        "PhonotacticError has more variants than this file covers: add a case for the new one"
    );
}
