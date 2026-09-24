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

use vime_engine::phonology::{
    DefaultPhonotacticValidator, PhonotacticValidator, ValidationError,
};
use vime_engine::phonology::{BaseVowel, Coda, Onset, Tone};
use BaseVowel::{ABreve, ACircumflex, ECircumflex, OCircumflex, OHorn, UHorn, A, E, I, O, U, Y};

fn ok(onset: Onset, vowels: &[BaseVowel], coda: Coda, tone: Tone) {
    assert_eq!(
        DefaultPhonotacticValidator.validate(onset, vowels, coda, tone),
        Ok(()),
        "expected valid: {onset:?} {vowels:?} {coda:?} {tone:?}"
    );
}

fn err(onset: Onset, vowels: &[BaseVowel], coda: Coda, tone: Tone, expected: ValidationError) {
    assert_eq!(
        DefaultPhonotacticValidator.validate(onset, vowels, coda, tone),
        Err(expected),
        "expected {expected:?}: {onset:?} {vowels:?} {coda:?} {tone:?}"
    );
}

// ──────────────────────────────────────────────────────────── valid syllables

#[test]
fn accepts_common_syllables() {
    // c/g/ng with non-front nuclei ("cây", gà, ngủ, ...) — rule 1 negative side
    ok(Onset::C, &[A], Coda::None, Tone::Flat); // ca
    ok(Onset::C, &[O, I], Coda::None, Tone::Flat); // coi
    ok(Onset::C, &[OCircumflex], Coda::Ng, Tone::Dot); // cộng
    ok(Onset::C, &[U, A], Coda::None, Tone::Flat); // cua
    ok(Onset::C, &[A, I], Coda::None, Tone::Flat); // cai
    ok(Onset::C, &[A], Coda::Nh, Tone::Flat); // canh
    ok(Onset::C, &[UHorn, U], Coda::None, Tone::Acute); // cứu
    ok(Onset::C, &[ACircumflex, Y], Coda::None, Tone::Flat); // cây
    ok(Onset::G, &[A], Coda::None, Tone::Grave); // gà
    ok(Onset::G, &[ACircumflex, U], Coda::None, Tone::Acute); // gấu
    ok(Onset::G, &[OHorn], Coda::None, Tone::Grave); // gờ
    ok(Onset::Ng, &[U], Coda::None, Tone::Hook); // ngủ
    ok(Onset::Ng, &[ACircumflex, Y], Coda::None, Tone::Grave); // ngày
    ok(Onset::Ng, &[UHorn, OHorn, I], Coda::None, Tone::Grave); // người
    ok(Onset::Ng, &[O], Coda::T, Tone::Dot); // ngọt
    ok(Onset::Ng, &[A], Coda::None, Tone::Flat); // nga

    // k/gh/ngh with front nuclei — rule 1 positive side
    ok(Onset::K, &[ECircumflex, U], Coda::None, Tone::Flat); // kêu
    ok(Onset::K, &[E, O], Coda::None, Tone::Dot); // kẹo
    ok(Onset::K, &[Y], Coda::None, Tone::Acute); // ký
    ok(Onset::K, &[I, ECircumflex], Coda::N, Tone::Flat); // kiên
    ok(Onset::Gh, &[E], Coda::None, Tone::Flat); // ghe
    ok(Onset::Gh, &[ECircumflex], Coda::None, Tone::Flat); // ghê
    ok(Onset::Gh, &[I], Coda::None, Tone::Flat); // ghi
    ok(Onset::Ngh, &[E], Coda::None, Tone::Flat); // nghe
    ok(Onset::Ngh, &[I], Coda::None, Tone::Tilde); // nghĩ
    ok(Onset::Ngh, &[I, ECircumflex], Coda::Ng, Tone::Flat); // nghiêng

    // Qu + non-u nuclei — rule 2
    ok(Onset::Qu, &[A], Coda::None, Tone::Grave); // quà
    ok(Onset::Qu, &[ECircumflex], Coda::None, Tone::Flat); // quê
    ok(Onset::Qu, &[E], Coda::N, Tone::Flat); // quen
    ok(Onset::Qu, &[ACircumflex], Coda::N, Tone::Flat); // quân
    ok(Onset::Qu, &[ACircumflex], Coda::N, Tone::Grave); // quần
    ok(Onset::Qu, &[OCircumflex], Coda::C, Tone::Acute); // quốc
    ok(Onset::Qu, &[Y], Coda::None, Tone::Acute); // quý
    ok(Onset::Qu, &[Y, ECircumflex], Coda::T, Tone::Acute); // quyết
    ok(Onset::Qu, &[Y, ECircumflex], Coda::N, Tone::Grave); // quyền
    ok(Onset::Qu, &[Y], Coda::Nh, Tone::Grave); // quỳnh
    ok(Onset::Qu, &[A], Coda::T, Tone::Dot); // quạt
    ok(Onset::Qu, &[ABreve], Coda::Ng, Tone::Flat); // quăng

    // stop codas carrying an entering tone — rule 3
    ok(Onset::C, &[A], Coda::P, Tone::Acute); // cáp
    ok(Onset::C, &[A], Coda::P, Tone::Dot); // cạp
    ok(Onset::C, &[A], Coda::C, Tone::Acute); // các
    ok(Onset::None, &[OCircumflex], Coda::C, Tone::Acute); // ốc
    ok(Onset::N, &[ECircumflex], Coda::P, Tone::Dot); // nếp
    ok(Onset::B, &[ECircumflex], Coda::P, Tone::Dot); // bếp
    ok(Onset::DStroke, &[E], Coda::P, Tone::Dot); // đẹp
    ok(Onset::DStroke, &[O], Coda::C, Tone::Dot); // đọc
    ok(Onset::DStroke, &[U, OCircumflex], Coda::C, Tone::Dot); // được
    ok(Onset::M, &[UHorn], Coda::C, Tone::Dot); // mực
    ok(Onset::S, &[UHorn], Coda::C, Tone::Acute); // sức
    ok(Onset::T, &[O], Coda::T, Tone::Acute); // tốt
    ok(Onset::B, &[I, ECircumflex], Coda::T, Tone::Acute); // biết
    ok(Onset::Th, &[ACircumflex], Coda::T, Tone::Dot); // thật
    ok(Onset::M, &[A], Coda::T, Tone::Acute); // mát
    ok(Onset::D, &[ACircumflex], Coda::T, Tone::Dot); // đất
    ok(Onset::V, &[I, ECircumflex], Coda::T, Tone::Dot); // việt
    ok(Onset::C, &[UHorn, OHorn], Coda::C, Tone::Dot); // cược
    ok(Onset::M, &[U, OHorn], Coda::N, Tone::Acute); // muốn

    // palatal codas after a front vowel or plain a — rule 4
    ok(Onset::None, &[A], Coda::Nh, Tone::Flat); // anh
    ok(Onset::None, &[A], Coda::Ch, Tone::Acute); // ách
    ok(Onset::C, &[A], Coda::Nh, Tone::Flat); // canh
    ok(Onset::X, &[I], Coda::Nh, Tone::Flat); // xinh
    ok(Onset::M, &[I], Coda::Nh, Tone::Grave); // mình
    ok(Onset::S, &[A], Coda::Ch, Tone::Acute); // sách
    ok(Onset::M, &[A], Coda::Ch, Tone::Dot); // mạch
    ok(Onset::None, &[ECircumflex], Coda::Ch, Tone::Acute); // ếch
    ok(Onset::None, &[I], Coda::Ch, Tone::Acute); // ích
    ok(Onset::B, &[ECircumflex], Coda::Nh, Tone::Dot); // bệnh
    ok(Onset::L, &[ECircumflex], Coda::Nh, Tone::Dot); // lệnh
    ok(Onset::H, &[U, Y], Coda::Nh, Tone::Flat); // huynh
    ok(Onset::H, &[O, A], Coda::Ch, Tone::Dot); // hoạch
    ok(Onset::H, &[O, A], Coda::Nh, Tone::Flat); // hoành
    ok(Onset::None, &[O, A], Coda::Nh, Tone::Flat); // oanh
    ok(Onset::K, &[I], Coda::Nh, Tone::Acute); // kính

    // short vowels with a closing sound — rule 5
    ok(Onset::None, &[ABreve], Coda::N, Tone::Flat); // ăn
    ok(Onset::S, &[ABreve], Coda::N, Tone::Flat); // săn
    ok(Onset::M, &[ABreve], Coda::N, Tone::Dot); // mặn
    ok(Onset::T, &[ABreve], Coda::M, Tone::Acute); // tắm
    ok(Onset::H, &[ABreve], Coda::N, Tone::Acute); // hắn
    ok(Onset::C, &[ACircumflex], Coda::N, Tone::Flat); // cân
    ok(Onset::S, &[ACircumflex], Coda::N, Tone::Flat); // sân
    ok(Onset::N, &[ACircumflex, U], Coda::None, Tone::Acute); // nấu
    ok(Onset::T, &[ACircumflex], Coda::T, Tone::Dot); // tất
    ok(Onset::DStroke, &[ACircumflex], Coda::T, Tone::Dot); // đất

    // assorted real syllables across other onsets
    ok(Onset::None, &[Y, ECircumflex, U], Coda::None, Tone::Flat); // yêu
    ok(Onset::None, &[A, I], Coda::None, Tone::Flat); // ai
    ok(Onset::None, &[A, Y], Coda::None, Tone::Flat); // ay
    ok(Onset::None, &[A, O], Coda::None, Tone::Flat); // ao
    ok(Onset::None, &[O, A], Coda::None, Tone::Flat); // oa
    ok(Onset::None, &[O, A, I], Coda::None, Tone::Flat); // oai
    ok(Onset::None, &[U, Y], Coda::None, Tone::Flat); // uy
    ok(Onset::None, &[U, Y, ECircumflex], Coda::N, Tone::Flat); // uyên
    ok(Onset::None, &[U, OCircumflex, I], Coda::None, Tone::Flat); // uôi
    ok(Onset::None, &[UHorn, U], Coda::None, Tone::Flat); // ưu
    ok(Onset::None, &[UHorn, I], Coda::None, Tone::Flat); // ơi
    ok(Onset::None, &[UHorn], Coda::None, Tone::Tilde); // ở
    ok(Onset::Ch, &[UHorn], Coda::None, Tone::Dot); // chợ
    ok(Onset::Th, &[UHorn], Coda::None, Tone::Flat); // thơ
    ok(Onset::Ch, &[UHorn], Coda::Ng, Tone::Tilde); // những
    ok(Onset::Ch, &[U, OCircumflex], Coda::Ng, Tone::Grave); // chuồng
    ok(Onset::Ch, &[I], Coda::M, Tone::Flat); // chim
    ok(Onset::Ph, &[O], Coda::Ng, Tone::Flat); // phong
    ok(Onset::Kh, &[U, Y, A], Coda::None, Tone::Flat); // khuya
    ok(Onset::Tr, &[OHorn, I], Coda::None, Tone::Grave); // trời
    ok(Onset::Gi, &[UHorn], Coda::None, Tone::Tilde); // giữ
    ok(Onset::Gi, &[A], Coda::None, Tone::Flat); // gia
    ok(Onset::Gi, &[ECircumflex], Coda::Ng, Tone::Flat); // giêng
    ok(Onset::Th, &[U, ECircumflex], Coda::None, Tone::Flat); // thuê
    ok(Onset::X, &[O, A, Y], Coda::None, Tone::Flat); // xoay
    ok(Onset::N, &[U, OCircumflex, I], Coda::None, Tone::Flat); // nuôi
    ok(Onset::T, &[U, Y, ECircumflex], Coda::N, Tone::Dot); // tuyện
}

// ──────────────────────────────────────────────── rule 1: k/gh/ngh (front-only)

#[test]
fn k_gh_ngh_require_leading_front_vowel() {
    let e = ValidationError::MissingFrontVowel;
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
    let e = ValidationError::ForbiddenFrontVowel;
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
    let e = ValidationError::GlideAfterQu;
    err(Onset::Qu, &[U], Coda::None, Tone::Flat, e); // quu
    err(Onset::Qu, &[U], Coda::T, Tone::Acute, e); // quút
    err(Onset::Qu, &[U, A], Coda::None, Tone::Flat, e); // quua
    err(Onset::Qu, &[U, O], Coda::None, Tone::Flat, e); // quuo
    err(Onset::Qu, &[U, Y], Coda::Nh, Tone::Flat, e); // quuy + nh
}

// ──────────────────────────────────────────── rule 3: stop codas need entering tone

#[test]
fn stop_coda_requires_entering_tone() {
    let e = ValidationError::EnteringToneRequired;
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
    let e = ValidationError::PalatalCodaVowelMismatch;
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
    let e = ValidationError::CodaRequiredForShortVowel;
    err(Onset::S, &[ABreve], Coda::None, Tone::Flat, e); // să
    err(Onset::T, &[ACircumflex], Coda::None, Tone::Flat, e); // tâ
    err(Onset::C, &[ABreve], Coda::None, Tone::Flat, e); // că
    err(Onset::C, &[ACircumflex], Coda::None, Tone::Flat, e); // câ
    err(Onset::G, &[ABreve], Coda::None, Tone::Flat, e); // gă
    err(Onset::None, &[ACircumflex], Coda::None, Tone::Flat, e); // â
}
