//! I. Toggle / revert behaviour: a transform key yields Applied (the
//! tone/shape is set), Reverted (an equal tone/shape toggled off, then the key
//! falls through as a literal into the coda) or NotApplicable (falls through
//! as literal); the expected states below reflect that exactly.
//!
//! Section groups:
//!   1. Tone toggle (same tone twice = flat)
//!   2. Shape toggle (same shape twice = revert)
//!   3. D-stroke toggle (d <-> đ)
//!   4. Uo/ươ cycle with horn key (w)
//!   5. Mixed tone+shape sequences

use super::prelude::*;

pub const CASES: &[Case] = &[
    // ═══════════════════════════════════════════
    // 1. Tone toggle (same tone twice = flat)
    // ═══════════════════════════════════════════
    case!(['a','s'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Acute)),
    case!(['á','f'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Grave)),
    case!(['à','r'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Hook)),
    case!(['ả','x'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Tilde)),
    case!(['ã','j'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Dot)),
    case!(['ả','f'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Grave)),
    case!(['á','z'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Flat)),
    case!(['ạ','z'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Flat)),

    // ═══════════════════════════════════════════
    // 2. Shape toggle (same shape twice = revert)
    // ═══════════════════════════════════════════
    case!(['ô','o'], ExpectedSyllable::vowel(&[(V::O,C::Lower),(V::O,C::Lower)], Tone::Flat)),
    case!(['a','a'], ExpectedSyllable::vowel(&[(V::ACircumflex,C::Lower)], Tone::Flat)),
    case!(['â','w'], ExpectedSyllable::vowel(&[(V::ABreve,C::Lower)], Tone::Flat)),
    case!(['ă','a'], ExpectedSyllable::vowel(&[(V::ACircumflex,C::Lower)], Tone::Flat)),
    case!(['a','w'], ExpectedSyllable::vowel(&[(V::ABreve,C::Lower)], Tone::Flat)),
    case!(['o','o'], ExpectedSyllable::vowel(&[(V::OCircumflex,C::Lower)], Tone::Flat)),

    // ═══════════════════════════════════════════
    // 3. D-stroke toggle (d <-> đ)
    // ═══════════════════════════════════════════
    case!(['a','s','w'], ExpectedSyllable::vowel(&[(V::ABreve,C::Lower)], Tone::Acute)),
    case!(['ă','s'], ExpectedSyllable::vowel(&[(V::ABreve,C::Lower)], Tone::Acute)),
    case!(['a','w','s'], ExpectedSyllable::vowel(&[(V::ABreve,C::Lower)], Tone::Acute)),
    case!(['d','d'], ExpectedSyllable::consonant(Onset::DStroke, &['đ'])),
    case!(['D','D'], ExpectedSyllable::consonant(Onset::DStroke, &['Đ'])),
    case!(['d','d','a'], ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::A,C::Lower)], Tone::Flat)),

    // ═══════════════════════════════════════════
    // 4. Uo/ươ cycle with horn key (w)
    // ═══════════════════════════════════════════
    case!(['u','o','w'], ExpectedSyllable::vowel(&[(V::U,C::Lower),(V::OHorn,C::Lower)], Tone::Flat)),
    case!(['u','o','w','w'], ExpectedSyllable::vowel(&[(V::UHorn,C::Lower),(V::OHorn,C::Lower)], Tone::Flat)),

    // ═══════════════════════════════════════════
    // 5. Mixed tone+shape sequences
    // ═══════════════════════════════════════════
    case!(['s'], ExpectedSyllable::consonant(Onset::S, &['s'])),
    case!(['o','s','r'], ExpectedSyllable::vowel(&[(V::O,C::Lower)], Tone::Hook)),
];