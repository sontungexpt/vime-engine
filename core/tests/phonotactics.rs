//! Integration tests for the Vietnamese spelling validator
//! (`PhonotacticValidator` in `core/src/phonology/phonotactics.rs`).
//!
//! Each rule is exercised both positively (real Vietnamese syllables —
//! segmented onset / vowel nucleus / coda / tone) and negatively (typos that
//! violate exactly one constraint):
//!
//! 1. `k`/`gh`/`ngh` need a front vowel (`i/y/e/ê`) right after; `c`/`g`/`ng`
//!    forbid it.
//! 2. `Qu` already carries the glide `u`, so the nucleus may not start with `u`.
//! 3. Stop codas `p/t/c/ch` demand an entering tone (`Sắc`/`Nặng`).
//! 4. Palatal codas `ch/nh` follow only a front vowel or plain `a`.
//! 5. Short vowels `ă/â` need a closing sound, never stand open.

use vime_engine::phonology::{BaseVowel, Coda, Onset, Tone};
use vime_engine::phonology::{DefaultPhonotacticValidator, PhonotacticError, PhonotacticValidator};
use BaseVowel::{ABreve, ACircumflex, ECircumflex, OCircumflex, OHorn, UHorn, A, E, I, O, U, Y};

fn err(onset: Onset, vowels: &[BaseVowel], coda: Coda, tone: Tone, expected: PhonotacticError) {
    assert_eq!(
        DefaultPhonotacticValidator.validate(onset, vowels, coda, tone),
        Err(expected),
        "expected {expected:?}: {onset:?} {vowels:?} {coda:?} {tone:?}"
    );
}

/// One accepted syllable, labelled with its Vietnamese spelling so a failure
/// names the word instead of making the reader decode a tuple.
struct Valid(&'static str, Onset, &'static [BaseVowel], Coda, Tone);

/// Runs every accepted case and reports all failures together, rather than
/// stopping at the first of ~110.
fn assert_all_valid(cases: &[Valid]) {
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|Valid(name, onset, vowels, coda, tone)| {
            let got = DefaultPhonotacticValidator.validate(*onset, *vowels, *coda, *tone);
            (got != Ok(())).then(|| {
                format!("{name}: expected Ok, got {got:?} ({onset:?} {vowels:?} {coda:?} {tone:?})")
            })
        })
        .collect();

    assert!(
        failures.is_empty(),
        "{} invalid syllable(s):\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

// ──────────────────────────────────────────────────────────── valid syllables

#[test]
fn accepts_common_syllables() {
    assert_all_valid(&[
        // c/g/ng with non-front nuclei ("cây", gà, ngủ, ...) — rule 1 negative side
        Valid("ca", Onset::C, &[A], Coda::None, Tone::Flat),
        Valid("coi", Onset::C, &[O, I], Coda::None, Tone::Flat),
        Valid("cộng", Onset::C, &[OCircumflex], Coda::Ng, Tone::Dot),
        Valid("cua", Onset::C, &[U, A], Coda::None, Tone::Flat),
        Valid("cai", Onset::C, &[A, I], Coda::None, Tone::Flat),
        Valid("canh", Onset::C, &[A], Coda::Nh, Tone::Flat),
        Valid("cứu", Onset::C, &[UHorn, U], Coda::None, Tone::Acute),
        Valid("cây", Onset::C, &[ACircumflex, Y], Coda::None, Tone::Flat),
        Valid("gà", Onset::G, &[A], Coda::None, Tone::Grave),
        Valid("gấu", Onset::G, &[ACircumflex, U], Coda::None, Tone::Acute),
        Valid("gờ", Onset::G, &[OHorn], Coda::None, Tone::Grave),
        Valid("ngủ", Onset::Ng, &[U], Coda::None, Tone::Hook),
        Valid(
            "ngày",
            Onset::Ng,
            &[ACircumflex, Y],
            Coda::None,
            Tone::Grave,
        ),
        Valid(
            "người",
            Onset::Ng,
            &[UHorn, OHorn, I],
            Coda::None,
            Tone::Grave,
        ),
        Valid("ngọt", Onset::Ng, &[O], Coda::T, Tone::Dot),
        Valid("nga", Onset::Ng, &[A], Coda::None, Tone::Flat),
        // k/gh/ngh with front nuclei — rule 1 positive side
        Valid("kêu", Onset::K, &[ECircumflex, U], Coda::None, Tone::Flat),
        Valid("kẹo", Onset::K, &[E, O], Coda::None, Tone::Dot),
        Valid("ký", Onset::K, &[Y], Coda::None, Tone::Acute),
        Valid("kiên", Onset::K, &[I, ECircumflex], Coda::N, Tone::Flat),
        Valid("ghe", Onset::Gh, &[E], Coda::None, Tone::Flat),
        Valid("ghê", Onset::Gh, &[ECircumflex], Coda::None, Tone::Flat),
        Valid("ghi", Onset::Gh, &[I], Coda::None, Tone::Flat),
        Valid("nghe", Onset::Ngh, &[E], Coda::None, Tone::Flat),
        Valid("nghĩ", Onset::Ngh, &[I], Coda::None, Tone::Tilde),
        Valid(
            "nghiêng",
            Onset::Ngh,
            &[I, ECircumflex],
            Coda::Ng,
            Tone::Flat,
        ),
        // Qu + non-u nuclei — rule 2
        Valid("quà", Onset::Qu, &[A], Coda::None, Tone::Grave),
        Valid("quê", Onset::Qu, &[ECircumflex], Coda::None, Tone::Flat),
        Valid("quen", Onset::Qu, &[E], Coda::N, Tone::Flat),
        Valid("quân", Onset::Qu, &[ACircumflex], Coda::N, Tone::Flat),
        Valid("quần", Onset::Qu, &[ACircumflex], Coda::N, Tone::Grave),
        Valid("quốc", Onset::Qu, &[OCircumflex], Coda::C, Tone::Acute),
        Valid("quý", Onset::Qu, &[Y], Coda::None, Tone::Acute),
        Valid("quyết", Onset::Qu, &[Y, ECircumflex], Coda::T, Tone::Acute),
        Valid("quyền", Onset::Qu, &[Y, ECircumflex], Coda::N, Tone::Grave),
        Valid("quỳnh", Onset::Qu, &[Y], Coda::Nh, Tone::Grave),
        Valid("quạt", Onset::Qu, &[A], Coda::T, Tone::Dot),
        Valid("quăng", Onset::Qu, &[ABreve], Coda::Ng, Tone::Flat),
        // stop codas carrying an entering tone — rule 3
        Valid("cáp", Onset::C, &[A], Coda::P, Tone::Acute),
        Valid("cạp", Onset::C, &[A], Coda::P, Tone::Dot),
        Valid("các", Onset::C, &[A], Coda::C, Tone::Acute),
        Valid("ốc", Onset::None, &[OCircumflex], Coda::C, Tone::Acute),
        Valid("nếp", Onset::N, &[ECircumflex], Coda::P, Tone::Dot),
        Valid("bếp", Onset::B, &[ECircumflex], Coda::P, Tone::Dot),
        Valid("đẹp", Onset::DStroke, &[E], Coda::P, Tone::Dot),
        Valid("đọc", Onset::DStroke, &[O], Coda::C, Tone::Dot),
        Valid(
            "được",
            Onset::DStroke,
            &[U, OCircumflex],
            Coda::C,
            Tone::Dot,
        ),
        Valid("mực", Onset::M, &[UHorn], Coda::C, Tone::Dot),
        Valid("sức", Onset::S, &[UHorn], Coda::C, Tone::Acute),
        Valid("tốt", Onset::T, &[O], Coda::T, Tone::Acute),
        Valid("biết", Onset::B, &[I, ECircumflex], Coda::T, Tone::Acute),
        Valid("thật", Onset::Th, &[ACircumflex], Coda::T, Tone::Dot),
        Valid("mát", Onset::M, &[A], Coda::T, Tone::Acute),
        Valid("đất", Onset::D, &[ACircumflex], Coda::T, Tone::Dot),
        Valid("việt", Onset::V, &[I, ECircumflex], Coda::T, Tone::Dot),
        Valid("cược", Onset::C, &[UHorn, OHorn], Coda::C, Tone::Dot),
        Valid("muốn", Onset::M, &[U, OHorn], Coda::N, Tone::Acute),
        // palatal codas after a front vowel or plain a — rule 4
        Valid("anh", Onset::None, &[A], Coda::Nh, Tone::Flat),
        Valid("ách", Onset::None, &[A], Coda::Ch, Tone::Acute),
        Valid("canh", Onset::C, &[A], Coda::Nh, Tone::Flat),
        Valid("xinh", Onset::X, &[I], Coda::Nh, Tone::Flat),
        Valid("mình", Onset::M, &[I], Coda::Nh, Tone::Grave),
        Valid("sách", Onset::S, &[A], Coda::Ch, Tone::Acute),
        Valid("mạch", Onset::M, &[A], Coda::Ch, Tone::Dot),
        Valid("ếch", Onset::None, &[ECircumflex], Coda::Ch, Tone::Acute),
        Valid("ích", Onset::None, &[I], Coda::Ch, Tone::Acute),
        Valid("bệnh", Onset::B, &[ECircumflex], Coda::Nh, Tone::Dot),
        Valid("lệnh", Onset::L, &[ECircumflex], Coda::Nh, Tone::Dot),
        Valid("huynh", Onset::H, &[U, Y], Coda::Nh, Tone::Flat),
        Valid("hoạch", Onset::H, &[O, A], Coda::Ch, Tone::Dot),
        Valid("hoành", Onset::H, &[O, A], Coda::Nh, Tone::Flat),
        Valid("oanh", Onset::None, &[O, A], Coda::Nh, Tone::Flat),
        Valid("kính", Onset::K, &[I], Coda::Nh, Tone::Acute),
        // short vowels with a closing sound — rule 5
        Valid("ăn", Onset::None, &[ABreve], Coda::N, Tone::Flat),
        Valid("săn", Onset::S, &[ABreve], Coda::N, Tone::Flat),
        Valid("mặn", Onset::M, &[ABreve], Coda::N, Tone::Dot),
        Valid("tắm", Onset::T, &[ABreve], Coda::M, Tone::Acute),
        Valid("hắn", Onset::H, &[ABreve], Coda::N, Tone::Acute),
        Valid("cân", Onset::C, &[ACircumflex], Coda::N, Tone::Flat),
        Valid("sân", Onset::S, &[ACircumflex], Coda::N, Tone::Flat),
        Valid("nấu", Onset::N, &[ACircumflex, U], Coda::None, Tone::Acute),
        Valid("tất", Onset::T, &[ACircumflex], Coda::T, Tone::Dot),
        Valid("đất", Onset::DStroke, &[ACircumflex], Coda::T, Tone::Dot),
        // assorted real syllables across other onsets
        Valid(
            "yêu",
            Onset::None,
            &[Y, ECircumflex, U],
            Coda::None,
            Tone::Flat,
        ),
        Valid("ai", Onset::None, &[A, I], Coda::None, Tone::Flat),
        Valid("ay", Onset::None, &[A, Y], Coda::None, Tone::Flat),
        Valid("ao", Onset::None, &[A, O], Coda::None, Tone::Flat),
        Valid("oa", Onset::None, &[O, A], Coda::None, Tone::Flat),
        Valid("oai", Onset::None, &[O, A, I], Coda::None, Tone::Flat),
        Valid("uy", Onset::None, &[U, Y], Coda::None, Tone::Flat),
        Valid(
            "uyên",
            Onset::None,
            &[U, Y, ECircumflex],
            Coda::N,
            Tone::Flat,
        ),
        Valid(
            "uôi",
            Onset::None,
            &[U, OCircumflex, I],
            Coda::None,
            Tone::Flat,
        ),
        Valid("ưu", Onset::None, &[UHorn, U], Coda::None, Tone::Flat),
        Valid("ơi", Onset::None, &[UHorn, I], Coda::None, Tone::Flat),
        Valid("ở", Onset::None, &[UHorn], Coda::None, Tone::Tilde),
        Valid("chợ", Onset::Ch, &[UHorn], Coda::None, Tone::Dot),
        Valid("thơ", Onset::Th, &[UHorn], Coda::None, Tone::Flat),
        Valid("những", Onset::Ch, &[UHorn], Coda::Ng, Tone::Tilde),
        Valid(
            "chuồng",
            Onset::Ch,
            &[U, OCircumflex],
            Coda::Ng,
            Tone::Grave,
        ),
        Valid("chim", Onset::Ch, &[I], Coda::M, Tone::Flat),
        Valid("phong", Onset::Ph, &[O], Coda::Ng, Tone::Flat),
        Valid("khuya", Onset::Kh, &[U, Y, A], Coda::None, Tone::Flat),
        Valid("trời", Onset::Tr, &[OHorn, I], Coda::None, Tone::Grave),
        Valid("giữ", Onset::Gi, &[UHorn], Coda::None, Tone::Tilde),
        Valid("gia", Onset::Gi, &[A], Coda::None, Tone::Flat),
        Valid("giêng", Onset::Gi, &[ECircumflex], Coda::Ng, Tone::Flat),
        Valid("thuê", Onset::Th, &[U, ECircumflex], Coda::None, Tone::Flat),
        Valid("xoay", Onset::X, &[O, A, Y], Coda::None, Tone::Flat),
        Valid(
            "nuôi",
            Onset::N,
            &[U, OCircumflex, I],
            Coda::None,
            Tone::Flat,
        ),
        Valid("tuyệt", Onset::T, &[U, Y, ECircumflex], Coda::T, Tone::Dot),
    ]);
}

// ──────────────────────────────────────────────── rule 1: k/gh/ngh (front-only)

#[test]
fn k_gh_ngh_require_leading_front_vowel() {
    let e = PhonotacticError::MissingFrontVowel;
    err(Onset::K, &[A], Coda::None, Tone::Flat, e); // ka
    err(Onset::K, &[O], Coda::None, Tone::Flat, e); // ko
    err(Onset::K, &[OCircumflex], Coda::None, Tone::Flat, e); // kô
    err(Onset::K, &[U], Coda::None, Tone::Flat, e); // ku
    err(Onset::K, &[UHorn], Coda::None, Tone::Flat, e); // kư
    err(Onset::K, &[ABreve], Coda::N, Tone::Flat, e); // kăn
    err(Onset::Gh, &[A], Coda::None, Tone::Flat, e); // gha
    err(Onset::Gh, &[U], Coda::None, Tone::Flat, e); // ghu
    err(Onset::Gh, &[OCircumflex], Coda::None, Tone::Flat, e); // ghô
    err(Onset::Ngh, &[A], Coda::None, Tone::Flat, e); // ngha
    err(Onset::Ngh, &[U], Coda::None, Tone::Flat, e); // nghu
    err(Onset::Ngh, &[O], Coda::None, Tone::Flat, e); // ngho
}

#[test]
fn c_g_ng_forbid_leading_front_vowel() {
    let e = PhonotacticError::ForbiddenFrontVowel;
    err(Onset::C, &[I], Coda::None, Tone::Flat, e); // ci
    err(Onset::C, &[I], Coda::Nh, Tone::Flat, e); // cinh
    err(Onset::C, &[E], Coda::None, Tone::Flat, e); // ce
    err(Onset::C, &[E, O], Coda::None, Tone::Flat, e); // ceo
    err(Onset::C, &[ECircumflex], Coda::None, Tone::Flat, e); // cê
    err(Onset::G, &[E], Coda::None, Tone::Flat, e); // ge
    err(Onset::G, &[ECircumflex], Coda::None, Tone::Flat, e); // gê
    err(Onset::Ng, &[E], Coda::None, Tone::Flat, e); // nge
    err(Onset::Ng, &[ECircumflex], Coda::None, Tone::Flat, e); // ngê
    err(Onset::Ng, &[I], Coda::None, Tone::Flat, e); // ngi
}

// ──────────────────────────────────────────── rule 2: Qu carries the glide "u"

#[test]
fn qu_initial_glide_leaves_no_second_u() {
    let e = PhonotacticError::GlideAfterQu;
    err(Onset::Qu, &[U], Coda::None, Tone::Flat, e); // quu
    err(Onset::Qu, &[U], Coda::T, Tone::Acute, e); // quút
    err(Onset::Qu, &[U, A], Coda::None, Tone::Flat, e); // quua
    err(Onset::Qu, &[U, O], Coda::None, Tone::Flat, e); // quuo
    err(Onset::Qu, &[U, Y], Coda::Nh, Tone::Flat, e); // quuy + nh
}

// ──────────────────────────────────────────── rule 3: stop codas need entering tone

#[test]
fn stop_coda_requires_entering_tone() {
    let e = PhonotacticError::EnteringToneRequired;
    err(Onset::C, &[A], Coda::P, Tone::Flat, e); // cap
    err(Onset::T, &[A], Coda::P, Tone::Flat, e); // tap
    err(Onset::T, &[A], Coda::T, Tone::Flat, e); // tat
    err(Onset::N, &[A], Coda::C, Tone::Flat, e); // nac
    err(Onset::H, &[O], Coda::P, Tone::Flat, e); // hop
    err(Onset::None, &[A], Coda::T, Tone::Flat, e); // at
    err(Onset::None, &[E], Coda::C, Tone::Flat, e); // ec
    err(Onset::None, &[ACircumflex], Coda::C, Tone::Flat, e); // âc
    err(Onset::None, &[A], Coda::Ch, Tone::Flat, e); // ach
    err(Onset::None, &[I], Coda::Ch, Tone::Flat, e); // ich
    err(Onset::T, &[I, ECircumflex], Coda::T, Tone::Flat, e); // tiêt
}

// ─────────────────────────────── rule 4: palatal codas need a front vowel or plain a

#[test]
fn palatal_coda_needs_front_vowel_or_plain_a() {
    let e = PhonotacticError::PalatalCodaVowelMismatch;
    err(Onset::None, &[O], Coda::Nh, Tone::Flat, e); // onh
    err(Onset::None, &[OCircumflex], Coda::Nh, Tone::Flat, e); // ônh
    err(Onset::None, &[U], Coda::Nh, Tone::Flat, e); // unh
    err(Onset::None, &[UHorn], Coda::Nh, Tone::Flat, e); // ơnh
    err(Onset::C, &[ABreve], Coda::Nh, Tone::Flat, e); // cănh
    err(Onset::C, &[ACircumflex], Coda::Nh, Tone::Flat, e); // cânh
}

// ──────────────────────────────── rule 5: short vowels always need a closing sound

#[test]
fn short_vowel_needs_a_coda() {
    let e = PhonotacticError::ShortVowelRequiresCoda;
    err(Onset::S, &[ABreve], Coda::None, Tone::Flat, e); // să
    err(Onset::T, &[ACircumflex], Coda::None, Tone::Flat, e); // tâ
    err(Onset::C, &[ABreve], Coda::None, Tone::Flat, e); // că
    err(Onset::C, &[ACircumflex], Coda::None, Tone::Flat, e); // câ
    err(Onset::G, &[ABreve], Coda::None, Tone::Flat, e); // gă
    err(Onset::None, &[ACircumflex], Coda::None, Tone::Flat, e); // â
}
