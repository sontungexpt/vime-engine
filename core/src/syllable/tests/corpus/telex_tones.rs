//! B. Telex tone keys.
//!
//! Section groups:
//!   1. Single vowel + tone (all 5 tones × 6 vowels)
//!   2. Precomposed tone + tone key (revert to flat)
//!   3. Onset + vowel + tone
//!   4. Vowel + coda + tone
//!   5. Full syllable (onset + vowel + coda + tone)
//!   6. Diphthong + tone
//!   5. Tone toggle/revert (same tone twice = flat)
//!   6. Tone sequence (different tones on same vowel)

use super::prelude::*;

pub const CASES: &[Case] = &[
    // ═══════════════════════════════════════════
    // 1. Single vowel + tone (all 5 tones × 6 vowels)
    // ═══════════════════════════════════════════
    // Acute (s)
    case!(['a','s'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Acute)),
    case!(['e','s'], ExpectedSyllable::vowel(&[(V::E,C::Lower)], Tone::Acute)),
    case!(['i','s'], ExpectedSyllable::vowel(&[(V::I,C::Lower)], Tone::Acute)),
    case!(['o','s'], ExpectedSyllable::vowel(&[(V::O,C::Lower)], Tone::Acute)),
    case!(['u','s'], ExpectedSyllable::vowel(&[(V::U,C::Lower)], Tone::Acute)),
    case!(['y','s'], ExpectedSyllable::vowel(&[(V::Y,C::Lower)], Tone::Acute)),
    // Grave (f)
    case!(['a','f'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Grave)),
    case!(['e','f'], ExpectedSyllable::vowel(&[(V::E,C::Lower)], Tone::Grave)),
    case!(['i','f'], ExpectedSyllable::vowel(&[(V::I,C::Lower)], Tone::Grave)),
    case!(['o','f'], ExpectedSyllable::vowel(&[(V::O,C::Lower)], Tone::Grave)),
    case!(['u','f'], ExpectedSyllable::vowel(&[(V::U,C::Lower)], Tone::Grave)),
    case!(['y','f'], ExpectedSyllable::vowel(&[(V::Y,C::Lower)], Tone::Grave)),
    // Hook (r)
    case!(['a','r'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Hook)),
    case!(['e','r'], ExpectedSyllable::vowel(&[(V::E,C::Lower)], Tone::Hook)),
    case!(['i','r'], ExpectedSyllable::vowel(&[(V::I,C::Lower)], Tone::Hook)),
    case!(['o','r'], ExpectedSyllable::vowel(&[(V::O,C::Lower)], Tone::Hook)),
    case!(['u','r'], ExpectedSyllable::vowel(&[(V::U,C::Lower)], Tone::Hook)),
    case!(['y','r'], ExpectedSyllable::vowel(&[(V::Y,C::Lower)], Tone::Hook)),
    // Tilde (x)
    case!(['a','x'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Tilde)),
    case!(['e','x'], ExpectedSyllable::vowel(&[(V::E,C::Lower)], Tone::Tilde)),
    case!(['i','x'], ExpectedSyllable::vowel(&[(V::I,C::Lower)], Tone::Tilde)),
    case!(['o','x'], ExpectedSyllable::vowel(&[(V::O,C::Lower)], Tone::Tilde)),
    case!(['u','x'], ExpectedSyllable::vowel(&[(V::U,C::Lower)], Tone::Tilde)),
    case!(['y','x'], ExpectedSyllable::vowel(&[(V::Y,C::Lower)], Tone::Tilde)),
    // Dot (j)
    case!(['a','j'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Dot)),
    case!(['e','j'], ExpectedSyllable::vowel(&[(V::E,C::Lower)], Tone::Dot)),
    case!(['i','j'], ExpectedSyllable::vowel(&[(V::I,C::Lower)], Tone::Dot)),
    case!(['o','j'], ExpectedSyllable::vowel(&[(V::O,C::Lower)], Tone::Dot)),
    case!(['u','j'], ExpectedSyllable::vowel(&[(V::U,C::Lower)], Tone::Dot)),
    case!(['y','j'], ExpectedSyllable::vowel(&[(V::Y,C::Lower)], Tone::Dot)),

    // ═══════════════════════════════════════════
    // 2. Precomposed tone + tone key (revert to flat)
    // ═══════════════════════════════════════════
    case!(['á','z'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Flat)),
    case!(['à','z'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Flat)),

    // ═══════════════════════════════════════════
    // 3. Onset + vowel + tone
    // ═══════════════════════════════════════════
    case!(['b','a','s'], ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::A,C::Lower)], Tone::Acute)),
    case!(['c','h','a','f'], ExpectedSyllable::onset_vowel(Onset::Ch, &['c','h'], &[(V::A,C::Lower)], Tone::Grave)),
    case!(['g','i','a','s'], ExpectedSyllable::onset_vowel(Onset::Gi, &['g','i'], &[(V::A,C::Lower)], Tone::Acute)),

    // ═══════════════════════════════════════════
    // 4. Vowel + coda + tone
    // ═══════════════════════════════════════════
    case!(['a','n','s'], ExpectedSyllable::vowel_coda(&[(V::A,C::Lower)], Tone::Acute, Coda::N, &['n'])),
    case!(['b','a','c','j'], ExpectedSyllable::full(Onset::B, &['b'], &[(V::A,C::Lower)], Tone::Dot, Coda::C, &['c'])),
    case!(['t','o','a','n','j'], ExpectedSyllable::full(Onset::T, &['t'], &[(V::O,C::Lower),(V::A,C::Lower)], Tone::Dot, Coda::N, &['n'])),
    case!(['a','m','f'], ExpectedSyllable::vowel_coda(&[(V::A,C::Lower)], Tone::Grave, Coda::M, &['m'])),
    case!(['a','p','j'], ExpectedSyllable::vowel_coda(&[(V::A,C::Lower)], Tone::Dot, Coda::P, &['p'])),
    case!(['a','t','j'], ExpectedSyllable::vowel_coda(&[(V::A,C::Lower)], Tone::Dot, Coda::T, &['t'])),

    // ═══════════════════════════════════════════
    // 5. Diphthong + tone
    // ═══════════════════════════════════════════
    case!(['a','i','s'], ExpectedSyllable::vowel(&[(V::A,C::Lower),(V::I,C::Lower)], Tone::Acute)),
    case!(['o','i','s'], ExpectedSyllable::vowel(&[(V::O,C::Lower),(V::I,C::Lower)], Tone::Acute)),
    case!(['u','i','s'], ExpectedSyllable::vowel(&[(V::U,C::Lower),(V::I,C::Lower)], Tone::Acute)),
    case!(['a','u','s'], ExpectedSyllable::vowel(&[(V::A,C::Lower),(V::U,C::Lower)], Tone::Acute)),
    case!(['a','y','s'], ExpectedSyllable::vowel(&[(V::A,C::Lower),(V::Y,C::Lower)], Tone::Acute)),
    case!(['u','a','s'], ExpectedSyllable::vowel(&[(V::U,C::Lower),(V::A,C::Lower)], Tone::Acute)),
    case!(['i','a','s'], ExpectedSyllable::vowel(&[(V::I,C::Lower),(V::A,C::Lower)], Tone::Acute)),
    case!(['o','i','r'], ExpectedSyllable::vowel(&[(V::O,C::Lower),(V::I,C::Lower)], Tone::Hook)),
    case!(['a','y','r'], ExpectedSyllable::vowel(&[(V::A,C::Lower),(V::Y,C::Lower)], Tone::Hook)),
    case!(['a','u','r'], ExpectedSyllable::vowel(&[(V::A,C::Lower),(V::U,C::Lower)], Tone::Hook)),
    case!(['a','i','x'], ExpectedSyllable::vowel(&[(V::A,C::Lower),(V::I,C::Lower)], Tone::Tilde)),
    case!(['i','a','x'], ExpectedSyllable::vowel(&[(V::I,C::Lower),(V::A,C::Lower)], Tone::Tilde)),
    case!(['u','a','j'], ExpectedSyllable::vowel(&[(V::U,C::Lower),(V::A,C::Lower)], Tone::Dot)),
    case!(['b','u','i','j'], ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::U,C::Lower),(V::I,C::Lower)], Tone::Dot)),

    // ═══════════════════════════════════════════
    // 6. Tone toggle/revert (same tone twice = flat)
    // ═══════════════════════════════════════════
    case!(['a','s','f'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Grave)),
    case!(['a','f','r'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Hook)),
    case!(['e','s','r'], ExpectedSyllable::vowel(&[(V::E,C::Lower)], Tone::Hook)),
    case!(['o','f','s'], ExpectedSyllable::vowel(&[(V::O,C::Lower)], Tone::Acute)),
    case!(['u','s','j'], ExpectedSyllable::vowel(&[(V::U,C::Lower)], Tone::Dot)),
    case!(['y','x'], ExpectedSyllable::vowel(&[(V::Y,C::Lower)], Tone::Tilde)),
    case!(['y','j'], ExpectedSyllable::vowel(&[(V::Y,C::Lower)], Tone::Dot)),

    // ═══════════════════════════════════════════
    // 7. Full syllable (onset + vowel + coda + tone)
    // ═══════════════════════════════════════════
    case!(['p','h','a','s'], ExpectedSyllable::onset_vowel(Onset::Ph, &['p','h'], &[(V::A,C::Lower)], Tone::Acute)),
    case!(['t','r','o','j'], ExpectedSyllable::onset_vowel(Onset::Tr, &['t','r'], &[(V::O,C::Lower)], Tone::Dot)),
    case!(['k','h','a','f'], ExpectedSyllable::onset_vowel(Onset::Kh, &['k','h'], &[(V::A,C::Lower)], Tone::Grave)),
    case!(['b','a','n','h','s'], ExpectedSyllable::full(Onset::B, &['b'], &[(V::A,C::Lower)], Tone::Acute, Coda::Nh, &['n','h'])),
    case!(['o','a','r'], ExpectedSyllable::vowel(&[(V::O,C::Lower),(V::A,C::Lower)], Tone::Hook)),
    case!(['o','e','j'], ExpectedSyllable::vowel(&[(V::O,C::Lower),(V::E,C::Lower)], Tone::Dot)),
    case!(['x','o','a','f'], ExpectedSyllable::onset_vowel(Onset::X, &['x'], &[(V::O,C::Lower),(V::A,C::Lower)], Tone::Grave)),
    case!(['e','u','f'], ExpectedSyllable::vowel(&[(V::E,C::Lower),(V::U,C::Lower)], Tone::Grave)),
    case!(['a','o','j'], ExpectedSyllable::vowel(&[(V::A,C::Lower),(V::O,C::Lower)], Tone::Dot)),
    case!(['i','e','u','f'], ExpectedSyllable::vowel(&[(V::I,C::Lower),(V::E,C::Lower),(V::U,C::Lower)], Tone::Grave)),
    case!(['b','a','y','s'], ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::A,C::Lower),(V::Y,C::Lower)], Tone::Acute)),
    case!(['c','h','o','a','y','j'], ExpectedSyllable::onset_vowel(Onset::Ch, &['c','h'], &[(V::O,C::Lower),(V::A,C::Lower),(V::Y,C::Lower)], Tone::Dot)),
    case!(['m','ư','a','r'], ExpectedSyllable::onset_vowel(Onset::M, &['m'], &[(V::UHorn,C::Lower),(V::A,C::Lower)], Tone::Hook)),
    case!(['n','ư','a','f'], ExpectedSyllable::onset_vowel(Onset::N, &['n'], &[(V::UHorn,C::Lower),(V::A,C::Lower)], Tone::Grave)),
    case!(['o','a','n','f'], ExpectedSyllable::vowel_coda(&[(V::O,C::Lower),(V::A,C::Lower)], Tone::Grave, Coda::N, &['n'])),
    case!(['i','ê','u','s'], ExpectedSyllable::vowel(&[(V::I,C::Lower),(V::ECircumflex,C::Lower),(V::U,C::Lower)], Tone::Acute)),
    case!(['ư','ơ','n','s'], ExpectedSyllable::vowel_coda(&[(V::UHorn,C::Lower),(V::OHorn,C::Lower)], Tone::Acute, Coda::N, &['n'])),
    case!(['b','o','a','y','s'], ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::O,C::Lower),(V::A,C::Lower),(V::Y,C::Lower)], Tone::Acute)),
    case!(['t','i','u','f'], ExpectedSyllable::onset_vowel(Onset::T, &['t'], &[(V::I,C::Lower),(V::U,C::Lower)], Tone::Grave)),
    case!(['q','u','e','e'], ExpectedSyllable::onset_vowel(Onset::Qu, &['q','u'], &[(V::ECircumflex,C::Lower)], Tone::Flat)),

    // ═══════════════════════════════════════════
    // 8. Tone sequence (different tones on same vowel)
    // ═══════════════════════════════════════════
    case!(['a','s','f'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Grave)),
    case!(['a','f','r'], ExpectedSyllable::vowel(&[(V::A,C::Lower)], Tone::Hook)),
    case!(['e','s','r'], ExpectedSyllable::vowel(&[(V::E,C::Lower)], Tone::Hook)),
    case!(['o','f','s'], ExpectedSyllable::vowel(&[(V::O,C::Lower)], Tone::Acute)),
    case!(['u','s','j'], ExpectedSyllable::vowel(&[(V::U,C::Lower)], Tone::Dot)),
    case!(['y','x'], ExpectedSyllable::vowel(&[(V::Y,C::Lower)], Tone::Tilde)),
    case!(['y','j'], ExpectedSyllable::vowel(&[(V::Y,C::Lower)], Tone::Dot)),
    case!(['c','h','a','w','n','s'], ExpectedSyllable::full(Onset::Ch, &['c','h'], &[(V::ABreve,C::Lower)], Tone::Acute, Coda::N, &['n'])),
    case!(['m','a','a','n','s'], ExpectedSyllable::full(Onset::M, &['m'], &[(V::ACircumflex,C::Lower)], Tone::Acute, Coda::N, &['n'])),
    case!(['k','h','a','a','s'], ExpectedSyllable::onset_vowel(Onset::Kh, &['k','h'], &[(V::ACircumflex,C::Lower)], Tone::Acute)),
    case!(['a','a','n','s'], ExpectedSyllable::vowel_coda(&[(V::ACircumflex,C::Lower)], Tone::Acute, Coda::N, &['n'])),
    case!(['e','e','n','j'], ExpectedSyllable::vowel_coda(&[(V::ECircumflex,C::Lower)], Tone::Dot, Coda::N, &['n'])),
    case!(['o','w','n','f'], ExpectedSyllable::vowel_coda(&[(V::OHorn,C::Lower)], Tone::Grave, Coda::N, &['n'])),
    case!(['ấ','w'], ExpectedSyllable::vowel(&[(V::ABreve,C::Lower)], Tone::Acute)),
    case!(['e','e','u','f'], ExpectedSyllable::vowel(&[(V::ECircumflex,C::Lower),(V::U,C::Lower)], Tone::Grave)),
    case!(['o','o','i','f'], ExpectedSyllable::vowel(&[(V::OCircumflex,C::Lower),(V::I,C::Lower)], Tone::Grave)),
    case!(['u','w','a','r'], ExpectedSyllable::vowel(&[(V::UHorn,C::Lower),(V::A,C::Lower)], Tone::Hook)),
    case!(['b','a','w','n','s'], ExpectedSyllable::full(Onset::B, &['b'], &[(V::ABreve,C::Lower)], Tone::Acute, Coda::N, &['n'])),
    case!(['b','a','a','n','f'], ExpectedSyllable::full(Onset::B, &['b'], &[(V::ACircumflex,C::Lower)], Tone::Grave, Coda::N, &['n'])),
    case!(['o','a','w','t','s'], ExpectedSyllable::vowel_coda(&[(V::O,C::Lower),(V::ABreve,C::Lower)], Tone::Acute, Coda::T, &['t'])),
    case!(['l','o','a','w','t','s'], ExpectedSyllable::full(Onset::L, &['l'], &[(V::O,C::Lower),(V::ABreve,C::Lower)], Tone::Acute, Coda::T, &['t'])),
    case!(['t','h','o','o','i','s'], ExpectedSyllable::onset_vowel(Onset::Th, &['t','h'], &[(V::OCircumflex,C::Lower),(V::I,C::Lower)], Tone::Acute)),
    case!(['d','d','a','a','u','r'], ExpectedSyllable::onset_vowel(Onset::DStroke, &['đ'], &[(V::ACircumflex,C::Lower),(V::U,C::Lower)], Tone::Hook)),
    case!(['c','u','o','o','n','j'], ExpectedSyllable::full(Onset::C, &['c'], &[(V::U,C::Lower),(V::OCircumflex,C::Lower)], Tone::Dot, Coda::N, &['n'])),
    case!(['b','a','a','y','s'], ExpectedSyllable::onset_vowel(Onset::B, &['b'], &[(V::ACircumflex,C::Lower),(V::Y,C::Lower)], Tone::Acute)),
];