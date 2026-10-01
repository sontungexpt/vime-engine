//! Vietnamese phonotactic validation.
//!
//! A Vietnamese syllable is built from four parts, and this module answers one
//! question: can these four appear together?
//!
//! ```text
//! onset + nucleus + coda + tone
//!  kh     oa       ng     huyền
//! ```
//!
//! Nearly every rule has the same shape — *"is this onset one of these?"*,
//! *"is this vowel one of those?"*. Such questions are written as **membership
//! tests against a bitmask**. Each phonology enum is `#[repr(u8)]` with dense
//! IDs, so one item becomes one bit:
//!
//! ```text
//! vowel_bit(I)  ==  1 << 2  ==  0b0000_0100
//! ```
//!
//! A set of items is several bits joined with `|`; a value belongs to the set
//! when `value_bit & set != 0`. With only 12 vowels, 28 onsets and 9 codas the
//! masks are 16, 32 and 16 bits wide — small enough that the compiler turns them
//! into single register operations.
//!
//! A mask records *which* items are present, never *what order* they are in.
//! The few rules that care about order (`qu` + `u`, a vowel before `ng`)
//! therefore read the first or last vowel directly instead of going through a
//! mask.
//!
//! `Onset::None` and `Coda::None` are ID `0`, so their bit is bit `0` too. No
//! rule mask includes it, and "no onset" / "no coda" is asked with `is_none()`.

use crate::phonology::{BaseVowelId, BaseVowelSlice, Coda, Onset, Tone};

type VowelMask = u16;
type OnsetMask = u32;
type CodaMask = u16;

// `BaseVowelSlice` comes from the phonology/composition layer and is
// intentionally not redefined here.

// ============================================================================
// Bit helpers
// ============================================================================

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

// ============================================================================
// Vowel sets
// ============================================================================
//
// Each constant below is a *set*: "any of these vowels". They are named after
// the rule that uses them rather than after the letters, so a rule and its set
// read together.

// ============================================================================
// Onset sets
// ============================================================================

// ============================================================================
// Coda sets
// ============================================================================

/// Vowels counted as "front": `i`, `e`, `ê`, `y`.
///
/// Used by the two onset rules below, which are really one statement and its
/// opposite — the same set, required by some onsets and banned by others.
pub const FRONT_VOWELS: VowelMask = vowel_bit(BaseVowelId::I)
    | vowel_bit(BaseVowelId::E)
    | vowel_bit(BaseVowelId::ECircumflex)
    | vowel_bit(BaseVowelId::Y);

/// Vowels allowed right before a palatal coda (`ch`, `nh`): `i`, `e`, `ê`,
/// `y`, `a`.
///
/// So `lịch` and `kính` are fine, but `o` + `ch` is not.
pub const PALATAL_CODA_VOWELS: VowelMask = vowel_bit(BaseVowelId::I)
    | vowel_bit(BaseVowelId::E)
    | vowel_bit(BaseVowelId::ECircumflex)
    | vowel_bit(BaseVowelId::Y)
    | vowel_bit(BaseVowelId::A);

/// Vowels that cannot sit directly before `ch`: `u`, `ư`.
///
/// See [`PhonotacticError::RoundedVowelBeforeCh`] — currently unreachable,
/// because the palatal set above already excludes both of them.
pub const ROUNDED_BEFORE_CH: VowelMask = vowel_bit(BaseVowelId::U) | vowel_bit(BaseVowelId::UHorn);

/// `ơ`, used as the whole nucleus — see
/// [`PhonotacticError::OpenVowelCodaMismatch`].
pub const BARE_OPEN_VOWEL: VowelMask = vowel_bit(BaseVowelId::OHorn);

// ============================================================================
// Onset sets
// ============================================================================

/// Onsets that demand a front vowel next: `k`, `gh`, `ngh`.
///
/// Not `kh`. That spelling takes an ordinary vowel, which is why `kho`, `khoa`
/// and `khúc` are ordinary words while `ka` and `ko` are not.
pub const FRONT_REQUIRED_ONSETS: OnsetMask =
    onset_bit(Onset::K) | onset_bit(Onset::Gh) | onset_bit(Onset::Ngh);

/// Onsets that refuse a front vowel next: `c`, `g`, `ng`.
///
/// The mirror of [`FRONT_REQUIRED_ONSETS`], and it only bites when the front
/// vowel is the *first* one — `cái` is a word, because the `i` comes second.
pub const FRONT_FORBIDDEN_ONSETS: OnsetMask =
    onset_bit(Onset::C) | onset_bit(Onset::G) | onset_bit(Onset::Ng);

// ============================================================================
// Coda sets
// ============================================================================

/// Codas allowed after `ă`: `c`, `ch`, `m`, `n`, `ng` — as in `căn`, `lắm`,
/// `mặt`, `ăn`, `ngăn`.
pub const A_BREVE_CODAS: CodaMask = coda_bit(Coda::C)
    | coda_bit(Coda::Ch)
    | coda_bit(Coda::M)
    | coda_bit(Coda::N)
    | coda_bit(Coda::Ng);

/// Codas allowed after `â`: `c`, `m`, `n`, `ng`, `nh`, `p`, `t` — as in `cấp`,
/// `ấm`, `ân`, `tất`, `bệnh`.
///
/// Wider than [`A_BREVE_CODAS`] in `p`/`t`/`nh`, narrower in `ch`.
pub const A_CIRCUMFLEX_CODAS: CodaMask = coda_bit(Coda::C)
    | coda_bit(Coda::M)
    | coda_bit(Coda::N)
    | coda_bit(Coda::Ng)
    | coda_bit(Coda::Nh)
    | coda_bit(Coda::P)
    | coda_bit(Coda::T);

/// Codas that are also vowel constraints: `ch`, `nh`.
///
/// These are the palatals — they need a matching vowel in front.
pub const PALATAL_CODAS: CodaMask = coda_bit(Coda::Ch) | coda_bit(Coda::Nh);

/// Codas that "close" the mouth, so the tone is limited: `p`, `t`, `c`, `ch`.
///
/// These take sắc or nặng only, which is why `học` and `lớp` are fine but
/// `hoc` without a tone is not.
pub const TONE_RESTRICTED_CODAS: CodaMask =
    coda_bit(Coda::P) | coda_bit(Coda::T) | coda_bit(Coda::C) | coda_bit(Coda::Ch);

// ============================================================================
// Errors
// ============================================================================

/// Describes why a syllable violates a phonotactic rule.
///
/// Every variant is a real constraint of Vietnamese, named so a caller can say
/// what went wrong without knowing the rule number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhonotacticError {
    /// There is no vowel yet, so the syllable cannot be judged.
    IncompleteNucleus,

    /// The onset needs a front vowel (`i`, `e`, `ê`, `y`) and did not get one:
    /// `ka`, `ghô`, `nghô`.
    MissingFrontVowel,

    /// The onset cannot take a front vowel: `ci`, `gê`, `ngi`.
    ///
    /// Only the *first* vowel counts — `cái` is fine, because its `i` is second.
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

    /// `ă` and `â` are too short to stand alone, so they need a coda:
    /// `să`, `tâ`.
    ShortVowelRequiresCoda,

    /// The coda is not one this short vowel allows — `ă` takes
    /// `c ch m n ng`, `â` takes `c m n ng nh p t`.
    ShortVowelCodaMismatch,

    /// `u` or `ư` cannot sit directly before `ch`.
    ///
    /// Unreachable today: [`PALATAL_CODA_VOWELS`] already rejects both before
    /// this is reached. Kept so the rule stays stated on its own terms.
    RoundedVowelBeforeCh,

    /// A nucleus of just `ơ` cannot take final `c`.
    ///
    /// Not `ơ` in a diphthong — `lược` and `ngược` are fine, because the nucleus
    /// is `uơ`/`ươ`. And `p`/`t` are fine either way: `lớp`, `hớp`, `chớt`,
    /// `lượt` are all words.
    OpenVowelCodaMismatch,

    /// `i` cannot sit directly before `ng`.
    IBeforeNg,

    /// `e` cannot sit directly before `ng`.
    EBeforeNg,
}

// ============================================================================
// Validation
// ============================================================================

/// Validates one Vietnamese syllable.
///
/// Returns the first rule the syllable breaks, or `Ok(())` when it is
/// well-formed. `nucleus` is read through [`BaseVowelSlice`], so the caller can
/// pass a slice, an array, or the builder's own inline buffer.
///
/// One pass over the nucleus collects everything the rules need: which vowels
/// are present (as a mask), the first one, and the last one. Nothing is
/// allocated.
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

    // `len > 0`, so index 0 is guaranteed to be valid.
    //
    // SAFETY: established by the check immediately above.
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

// ============================================================================
// Onset validation
// ============================================================================

/// The onset rules.
///
/// Every rule here is about the **first** nucleus vowel only — which is why they
/// cannot be written against the whole-nucleus mask.
#[inline(always)]
fn validate_onset(onset: Onset, first: BaseVowelId) -> Result<(), PhonotacticError> {
    // No onset, no onset rules.
    if onset.is_none() {
        return Ok(());
    }

    // The onset as one bit, so it can be tested against the sets below.
    let o_bit = onset_bit(onset);

    // The first vowel as one bit, tested against the front set. Using the whole
    // nucleus here would be wrong twice over: it would call `cai` illegal
    // because its `i` is front, and it would let `k` + `ai` through only by
    // accident of the other rule.
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

// ============================================================================
// Coda validation
// ============================================================================

/// The coda rules.
///
/// These are all about the **last** nucleus vowel, for the same reason the onset
/// rules are about the first: a mask cannot say where a vowel sits.
#[inline(always)]
fn validate_coda(
    nucleus: VowelMask,
    last: BaseVowelId,
    tone: Tone,
    coda: Coda,
) -> Result<(), PhonotacticError> {
    // No coda yet — the normal state while typing. Nearly every vowel is happy
    // in an open syllable, so there is nothing to check.
    //
    // The exception is `ă` and `â`, which are too short to stand alone.
    //
    // Checked on the last vowel, not on membership: `âu` and `ây` are spelled
    // a + â + u, so `â` is in the nucleus mask even though the syllable is not
    // short. Testing membership would reject `câu`, `bây` and every other open
    // `â` diphthong.
    if coda.is_none() {
        if last == BaseVowelId::ABreve || last == BaseVowelId::ACircumflex {
            return Err(PhonotacticError::ShortVowelRequiresCoda);
        }

        return Ok(());
    }

    // A real coda exists from here on.
    let c_bit = coda_bit(coda);

    // `p`, `t`, `c`, `ch` close the mouth, so they only take sắc or nặng.
    // `học` and `lớp` are fine; `hoc` without a tone is not.
    if c_bit & TONE_RESTRICTED_CODAS != 0 && !matches!(tone, Tone::Acute | Tone::Dot) {
        return Err(PhonotacticError::EnteringToneRequired);
    }

    // The last vowel as one bit, so the set tests below read the same way.
    let last_vowel_bit = vowel_bit(last);

    // `ch` and `nh` need i / e / ê / y / a in front — `lịch`, `kính`.
    if c_bit & PALATAL_CODAS != 0 && last_vowel_bit & PALATAL_CODA_VOWELS == 0 {
        return Err(PhonotacticError::PalatalCodaVowelMismatch);
    }

    // `u` and `ư` cannot sit directly before `ch`.
    //
    // Unreachable today: the palatal test above has already rejected both, so
    // this never runs. Kept because it states a real rule on its own terms — if
    // `PALATAL_CODA_VOWELS` is ever widened, this still holds the line, and
    // `RoundedVowelBeforeCh` stays a variant an input can actually produce.
    if coda == Coda::Ch && last_vowel_bit & ROUNDED_BEFORE_CH != 0 {
        return Err(PhonotacticError::RoundedVowelBeforeCh);
    }

    // A nucleus of just `ơ` cannot take final `c`.
    //
    // `nucleus == BARE_OPEN_VOWEL` means the mask holds one bit and that bit is
    // `ơ` — the whole nucleus is the single vowel. The diphthongs `lược`,
    // `ngược`, `dược` all end in an `ơ` shape too, so testing the last letter
    // alone would reject them; they are fine because `uơ`/`ươ` sets more bits.
    // And `p` / `t` are fine in every case: `lớp`, `hớp`, `chớt`, `lượt` are
    // all words, so only `c` is restricted.
    if nucleus == BARE_OPEN_VOWEL && coda == Coda::C {
        return Err(PhonotacticError::OpenVowelCodaMismatch);
    }

    // `i` and `e` cannot sit directly before `ng`. Two separate rules, so two
    // variants — a direct `==` is clearer than a two-item mask.
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

// ============================================================================
// Compile-time invariants
// ============================================================================
//
// These assertions ensure that the chosen integer types remain large enough
// if the phonology enums are extended in the future.
//
// Without these checks, adding enough enum variants could make `1 << id`
// exceed the mask width.

const _: () = {
    assert!(BaseVowelId::COUNT <= VowelMask::BITS as usize);
    assert!(Onset::COUNT <= OnsetMask::BITS as usize);
    assert!(Coda::COUNT <= CodaMask::BITS as usize);
};
