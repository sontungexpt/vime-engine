//! Checks which onset, nucleus, coda and tone combinations form a legal syllable.
//!
//! Nearly every rule is a bitmask membership test: each phonology enum is
//! `#[repr(u8)]` with dense IDs, so one item is one bit and a set is
//! `mask & bit != 0`. A mask records no order, so the few order-caring rules
//! (`qu` + `u`, a vowel before `ng`) read the first or last vowel directly.

use crate::phonology::{BaseVowelId, BaseVowelSlice, Coda, Onset, Tone};

type VowelMask = u16;
type OnsetMask = u32;
type CodaMask = u16;

// `BaseVowelSlice` comes from the composition layer; it is not redefined here.

// ---- Bit helpers ----

/// One item, as one bit. See the module docs for why.
#[inline(always)]
const fn vowel_bit(vowel: BaseVowelId) -> VowelMask {
    1u16 << vowel.id()
}

/// One onset, as one bit.
#[inline(always)]
const fn onset_bit(onset: Onset) -> OnsetMask {
    1u32 << onset.id()
}

/// One coda, as one bit.
#[inline(always)]
const fn coda_bit(coda: Coda) -> CodaMask {
    1u16 << coda.id()
}

// ---- Vowel sets ----
//
// Each constant below is one set of vowels, named after the rule that uses it
// rather than after its letters, so a rule and its set read together.

// Mask bits follow `BaseVowelId`: bit 0 is `y`, bit 11 is `ơ`.

// Each set is spelled `vowel_bit(a) | vowel_bit(b) | ...`.

/// Vowels counted as "front": `i`, `e`, `ê`, `y`.
///
/// Both onset rules below test it: some onsets require it, others ban it.
pub const FRONT_VOWELS: VowelMask = vowel_bit(BaseVowelId::I)
    | vowel_bit(BaseVowelId::E)
    | vowel_bit(BaseVowelId::ECircumflex)
    | vowel_bit(BaseVowelId::Y);

/// Vowels allowed right before a palatal coda (`ch`, `nh`): `i`, `e`, `ê`, `y`,
/// `a` — so `lịch` and `kính` are fine but `o` + `ch` is not.
pub const PALATAL_CODA_VOWELS: VowelMask = vowel_bit(BaseVowelId::I)
    | vowel_bit(BaseVowelId::E)
    | vowel_bit(BaseVowelId::ECircumflex)
    | vowel_bit(BaseVowelId::Y)
    | vowel_bit(BaseVowelId::A);

/// Vowels that cannot sit directly before `ch`: `u`, `ư`. Unreachable today:
/// [`PhonotacticError::RoundedVowelBeforeCh`] fires only after the palatal set
/// above has already excluded both.
pub const ROUNDED_BEFORE_CH: VowelMask = vowel_bit(BaseVowelId::U) | vowel_bit(BaseVowelId::UHorn);

/// `ơ`, used as the whole nucleus — see [`PhonotacticError::OpenVowelCodaMismatch`].
pub const BARE_OPEN_VOWEL: VowelMask = vowel_bit(BaseVowelId::OHorn);

// ---- Onset sets ----

/// Onsets that demand a front vowel next: `k`, `gh`, `ngh` — but not `kh`, so
/// `kho`, `khoa` and `khúc` are fine while `ka` and `ko` are not.
pub const FRONT_REQUIRED_ONSETS: OnsetMask =
    onset_bit(Onset::K) | onset_bit(Onset::Gh) | onset_bit(Onset::Ngh);

/// Onsets that refuse a front vowel next: `c`, `g`, `ng`. The mirror of
/// [`FRONT_REQUIRED_ONSETS`]; only the *first* vowel counts, so `cái` is fine.
pub const FRONT_FORBIDDEN_ONSETS: OnsetMask =
    onset_bit(Onset::C) | onset_bit(Onset::G) | onset_bit(Onset::Ng);

// ---- Coda sets ----

/// Codas allowed after `ă`: `c`, `ch`, `m`, `n`, `ng` — as in `căn`, `lắm`,
/// `mặt`, `ăn`, `ngăn`.
pub const A_BREVE_CODAS: CodaMask = coda_bit(Coda::C)
    | coda_bit(Coda::Ch)
    | coda_bit(Coda::M)
    | coda_bit(Coda::N)
    | coda_bit(Coda::Ng);

/// Codas allowed after `â`: `c`, `m`, `n`, `ng`, `nh`, `p`, `t` — as in `cấp`,
/// `ấm`, `ân`, `tất`, `bệnh`. Wider than [`A_BREVE_CODAS`] in `p`/`t`/`nh`,
/// narrower in `ch`.
pub const A_CIRCUMFLEX_CODAS: CodaMask = coda_bit(Coda::C)
    | coda_bit(Coda::M)
    | coda_bit(Coda::N)
    | coda_bit(Coda::Ng)
    | coda_bit(Coda::Nh)
    | coda_bit(Coda::P)
    | coda_bit(Coda::T);

/// The palatal codas `ch`, `nh`; they also constrain the vowel in front.
pub const PALATAL_CODAS: CodaMask = coda_bit(Coda::Ch) | coda_bit(Coda::Nh);

/// Codas that close the mouth, so the tone is limited: `p`, `t`, `c`, `ch` take
/// sắc or nặng only — `học` and `lớp` are fine, `hoc` is not.
pub const TONE_RESTRICTED_CODAS: CodaMask =
    coda_bit(Coda::P) | coda_bit(Coda::T) | coda_bit(Coda::C) | coda_bit(Coda::Ch);

// ---- Errors ----

/// Why a syllable breaks a phonotactic rule; each variant names the rule it
/// broke, so a caller can report it without knowing rule numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PhonotacticError {
    /// There is no vowel yet, so the syllable cannot be judged.
    IncompleteNucleus,

    /// The onset needs a front vowel (`i`, `e`, `ê`, `y`) and got `ka`, `ghô`, `nghô`.
    MissingFrontVowel,

    /// The onset cannot take a front vowel: `ci`, `gê`, `ngi`. Only the *first*
    /// vowel counts, so `cái` is fine.
    ForbiddenFrontVowel,

    /// `qu` already carries a `w`, so a following `u` would double it: `quu`.
    GlideAfterQu,

    /// `gi` already carries an `i`, so a following `i` would double it.
    GlideAfterGi,

    /// A coda that closes the mouth (`p`, `t`, `c`, `ch`) appeared without sắc
    /// or nặng: `hoc` where `học` was meant.
    EnteringToneRequired,

    /// `ch` or `nh` is not preceded by one of `i`, `e`, `ê`, `y`, `a`.
    PalatalCodaVowelMismatch,

    /// `ă` and `â` are too short to stand alone, so they need a coda: `să`, `tâ`.
    ShortVowelRequiresCoda,

    /// The coda is not one this short vowel allows — `ă` takes
    /// `c ch m n ng`, `â` takes `c m n ng nh p t`.
    ShortVowelCodaMismatch,

    /// `u` or `ư` cannot sit directly before `ch`. Unreachable today:
    /// `PALATAL_CODA_VOWELS` rejects both first; kept so the rule still stands
    /// on its own terms.
    RoundedVowelBeforeCh,

    /// A nucleus of just `ơ` cannot take final `c`. Not `ơ` in a diphthong —
    /// `lược` and `ngược` are fine, because their nucleus is `uơ`/`ươ`. `p`/`t`
    /// are fine either way: `lớp`, `hớp`, `chớt`, `lượt` are all words.
    OpenVowelCodaMismatch,

    /// `i` cannot sit directly before `ng`.
    IBeforeNg,

    /// `e` cannot sit directly before `ng`.
    EBeforeNg,
}

// ---- Validation ----

/// Validates one Vietnamese syllable, returning the first rule it breaks.
///
/// `nucleus` is read through [`BaseVowelSlice`], so a slice, an array or the
/// builder's inline buffer all work. One pass collects the vowel mask plus the
/// first and last vowel; nothing is allocated.
#[inline(always)]
pub fn validate_phonotactics<N: BaseVowelSlice + ?Sized>(
    onset: Onset,
    nucleus: &N,
    tone: Tone,
    coda: Coda,
) -> Result<(), PhonotacticError> {
    let len = nucleus.len();

    if len == 0 {
        return Err(PhonotacticError::IncompleteNucleus);
    }

    // SAFETY: `len > 0` from the check above, so index 0 is in bounds.
    let first = unsafe { nucleus.at_unchecked(0) };
    let first_id = first.id();

    let mut vowel_mask = vowel_bit(first_id);
    let mut last_id = first_id;

    // The first vowel was already read above, so start at index 1.
    let mut index = 1;

    while index < len {
        // SAFETY: `index < len` is guaranteed by the loop condition.
        let vowel = unsafe { nucleus.at_unchecked(index) };
        let vowel_id = vowel.id();

        vowel_mask |= vowel_bit(vowel_id);
        last_id = vowel_id;

        index += 1;
    }

    validate_onset(onset, first_id)?;
    validate_coda(vowel_mask, last_id, tone, coda)
}

// ---- Onset validation ----

/// The onset rules, all about the **first** nucleus vowel — a whole-nucleus mask
/// cannot say which vowel sits first.
#[inline(always)]
fn validate_onset(onset: Onset, first: BaseVowelId) -> Result<(), PhonotacticError> {
    // No onset, no onset rules.
    if onset.is_none() {
        return Ok(());
    }

    let o_bit = onset_bit(onset);

    // Only the first vowel counts: testing the whole nucleus would reject `cái`
    // and still let `k` + `ai` through by accident of the next rule.
    let front_vowel_bit = vowel_bit(first) & FRONT_VOWELS;

    // `k`, `gh`, `ngh` need i / e / ê / y. (`kh` deliberately does not.)
    if o_bit & FRONT_REQUIRED_ONSETS != 0 && front_vowel_bit == 0 {
        return Err(PhonotacticError::MissingFrontVowel);
    }

    // `c`, `g`, `ng` refuse i / e / ê / y.
    if o_bit & FRONT_FORBIDDEN_ONSETS != 0 && front_vowel_bit != 0 {
        return Err(PhonotacticError::ForbiddenFrontVowel);
    }

    // `qu` already carries its `w`, so `u` on top of it would double the glide.
    if onset == Onset::Qu && first == BaseVowelId::U {
        return Err(PhonotacticError::GlideAfterQu);
    }

    // `gi` already carries its `i`.
    if onset == Onset::Gi && first == BaseVowelId::I {
        return Err(PhonotacticError::GlideAfterGi);
    }

    Ok(())
}

// ---- Coda validation ----

/// The coda rules, all about the **last** nucleus vowel — a mask cannot say
/// where a vowel sits.
#[inline(always)]
fn validate_coda(
    nucleus: VowelMask,
    last: BaseVowelId,
    tone: Tone,
    coda: Coda,
) -> Result<(), PhonotacticError> {
    // Open syllable: only `ă`/`â` as the *last* vowel is too short to stand
    // alone — testing membership would wrongly reject `câu` and `ây`.
    if coda.is_none() {
        if last == BaseVowelId::ABreve || last == BaseVowelId::ACircumflex {
            return Err(PhonotacticError::ShortVowelRequiresCoda);
        }

        return Ok(());
    }

    let c_bit = coda_bit(coda);

    // `p`, `t`, `c`, `ch` take sắc or nặng only — `hoc` where `học` was meant.
    if c_bit & TONE_RESTRICTED_CODAS != 0 && !matches!(tone, Tone::Acute | Tone::Dot) {
        return Err(PhonotacticError::EnteringToneRequired);
    }

    let last_vowel_bit = vowel_bit(last);

    // `ch` and `nh` need i / e / ê / y / a in front — `lịch`, `kính`.
    if c_bit & PALATAL_CODAS != 0 && last_vowel_bit & PALATAL_CODA_VOWELS == 0 {
        return Err(PhonotacticError::PalatalCodaVowelMismatch);
    }

    // Unreachable today — the palatal test above already rejects both; kept so
    // a widened `PALATAL_CODA_VOWELS` could not silently allow `uch`, `ưch`.
    if coda == Coda::Ch && last_vowel_bit & ROUNDED_BEFORE_CH != 0 {
        return Err(PhonotacticError::RoundedVowelBeforeCh);
    }

    // `BARE_OPEN_VOWEL` means the nucleus is exactly `ơ`, so `lược`/`ngược` stay
    // valid (they set more bits); only `c` is restricted, not `p`/`t`.
    if nucleus == BARE_OPEN_VOWEL && coda == Coda::C {
        return Err(PhonotacticError::OpenVowelCodaMismatch);
    }

    // `i`/`e` before `ng`: two rules, two variants — a direct `==` beats a mask.
    if coda == Coda::Ng {
        match last {
            BaseVowelId::I => return Err(PhonotacticError::IBeforeNg),
            BaseVowelId::E => return Err(PhonotacticError::EBeforeNg),
            _ => {}
        }
    }

    // `ă` takes c / ch / m / n / ng — `căn`, `lắm`, `mặt`, `ăn`, `ngăn`.
    if last == BaseVowelId::ABreve && c_bit & A_BREVE_CODAS == 0 {
        return Err(PhonotacticError::ShortVowelCodaMismatch);
    }

    // `â` takes c / m / n / ng / nh / p / t — `cấp`, `tất`, `bệnh`.
    if last == BaseVowelId::ACircumflex && c_bit & A_CIRCUMFLEX_CODAS == 0 {
        return Err(PhonotacticError::ShortVowelCodaMismatch);
    }

    Ok(())
}

// ---- Compile-time invariants ----
// The mask types must stay wide enough if the phonology enums grow a variant;
// without these, `1 << id` could overflow the mask type.

const _: () = {
    assert!(BaseVowelId::COUNT <= VowelMask::BITS as usize);
    assert!(Onset::COUNT <= OnsetMask::BITS as usize);
    assert!(Coda::COUNT <= CodaMask::BITS as usize);
};
