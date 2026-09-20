//! H. Uppercase input.

use super::{case, ExpectedSyllable, TestCase};
use super::{C, V};
use vime_engine::composition::ParseAppendingPhase;
use vime_engine::phonology::{Coda, Onset, Tone};

pub const CASES: &[TestCase] = &[
    case!(['A'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Upper)], Tone::Flat)),
    case!(['E'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::E, C::Upper)], Tone::Flat)),
    case!(['I'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::I, C::Upper)], Tone::Flat)),
    case!(['O'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::O, C::Upper)], Tone::Flat)),
    case!(['U'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Upper)], Tone::Flat)),
    case!(['Y'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::Y, C::Upper)], Tone::Flat)),
    case!(['B', 'A'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::B, &['B'], &[(V::A, C::Upper)], Tone::Flat)),
    case!(['C', 'H', 'A'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Ch, &['C', 'H'], &[(V::A, C::Upper)], Tone::Flat)),
    case!(['D', 'D', 'A'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::DStroke, &['Đ'], &[(V::A, C::Upper)], Tone::Flat)),
    case!(['Q', 'U', 'A'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['Q', 'U'], &[(V::A, C::Upper)], Tone::Flat)),
    case!(['Q', 'U', 'Y'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['Q', 'U'], &[(V::Y, C::Upper)], Tone::Flat)),
    case!(['Q'], ParseAppendingPhase::Onset, ExpectedSyllable::consonant(Onset::None, &['Q'])),
    case!(['Q', 'U'], ParseAppendingPhase::Onset, ExpectedSyllable::consonant(Onset::Qu, &['Q', 'U'])),
    case!(['B', 'A', 'N'], ParseAppendingPhase::Coda, ExpectedSyllable::syllable(Onset::B, &['B'], &[(V::A, C::Upper)], Tone::Flat, Coda::N, &['N'])),
    case!(
        ['C', 'H', 'U', 'Y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ch, &['C', 'H'], &[(V::U, C::Upper), (V::Y, C::Upper)], Tone::Flat)
    ),
    case!(['A', 'W'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Upper)], Tone::Flat)),
    case!(['A', 'A'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Upper)], Tone::Flat)),
    case!(['E', 'E'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Upper)], Tone::Flat)),
    case!(['O', 'O'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Upper)], Tone::Flat)),
    case!(['O', 'W'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Upper)], Tone::Flat)),
    case!(['U', 'W'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Upper)], Tone::Flat)),
    case!(['A', 'W', 'S'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Upper)], Tone::Acute)),
    case!(['A', 'A', 'S'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Upper)], Tone::Acute)),
    case!(['U', 'W', 'S'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Upper)], Tone::Acute)),
    case!(['O', 'W', 'S'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Upper)], Tone::Acute)),
    case!(['A', 'W', 'R'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Upper)], Tone::Hook)),
    case!(['A', 'A', 'X'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Upper)], Tone::Tilde)),
    case!(['O', 'W', 'R'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Upper)], Tone::Hook)),
    case!(['O', 'W', 'X'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OHorn, C::Upper)], Tone::Tilde)),
    case!(['U', 'W', 'F'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Upper)], Tone::Grave)),
    case!(['U', 'W', 'J'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Upper)], Tone::Dot)),
    case!(['A', 'S'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Upper)], Tone::Acute)),
    case!(['A', 'F'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Upper)], Tone::Grave)),
    case!(['A', 'R'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Upper)], Tone::Hook)),
    case!(['A', 'X'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Upper)], Tone::Tilde)),
    case!(['A', 'J'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::A, C::Upper)], Tone::Dot)),
    case!(['E', 'F'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::E, C::Upper)], Tone::Grave)),
    case!(['U', 'S'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::U, C::Upper)], Tone::Acute)),
    case!(['E', 'R'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::E, C::Upper)], Tone::Hook)),
    case!(['O', 'O', 'F'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Upper)], Tone::Grave)),
    case!(['O', 'O', 'X'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Upper)], Tone::Tilde)),
    case!(['Y', 'F'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::Y, C::Upper)], Tone::Grave)),
    case!(['Y', 'S'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::Y, C::Upper)], Tone::Acute)),
    case!(['Ắ'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Upper)], Tone::Acute)),
    case!(['Ằ'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Upper)], Tone::Grave)),
    case!(['Ấ'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Upper)], Tone::Acute)),
    case!(['Ệ'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ECircumflex, C::Upper)], Tone::Dot)),
    case!(['Đ'], ParseAppendingPhase::Onset, ExpectedSyllable::consonant(Onset::DStroke, &['Đ'])),
    case!(['Ầ'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Upper)], Tone::Grave)),
    case!(['Ị'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::I, C::Upper)], Tone::Dot)),
    case!(['Ử'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::UHorn, C::Upper)], Tone::Hook)),
    case!(['Ỗ'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::OCircumflex, C::Upper)], Tone::Tilde)),
    case!(['Ắ', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Upper)], Tone::Grave)),
    case!(['Ấ', 'f'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ACircumflex, C::Upper)], Tone::Grave)),
    case!(['Ạ', 'w'], ParseAppendingPhase::Vowel, ExpectedSyllable::vowel(&[(V::ABreve, C::Upper)], Tone::Dot)),
    case!(['D', 'D', 'I'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::DStroke, &['Đ'], &[(V::I, C::Upper)], Tone::Flat)),
    case!(['B', 'A', 'Y'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::B, &['B'], &[(V::A, C::Upper), (V::Y, C::Upper)], Tone::Flat)),
    case!(['T', 'R', 'A'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Tr, &['T', 'R'], &[(V::A, C::Upper)], Tone::Flat)),
    case!(['N', 'H', 'A'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Nh, &['N', 'H'], &[(V::A, C::Upper)], Tone::Flat)),
    case!(['Y', 'E', 'E', 'U'], ParseAppendingPhas ExpectedSyllable::vowel(&[(V::Y, C::Upper), (V::ECircumflex, C::Upper), (V::U, C::Upper)], Tone::Flat)),
    case!(
        ['N', 'G', 'U', 'O', 'W', 'I', 'F'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Ng, &['N', 'G'], &[(V::UHorn, C::Upper), (V::OHorn, C::Upper), (V::I, C::Upper)], Tone::Grave)
    ),
    case!(
        ['T', 'R', 'U', 'O', 'W', 'N', 'G'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::Tr, &['T', 'R'], &[(V::UHorn, C::Upper), (V::OHorn, C::Upper)], Tone::Flat, Coda::Ng, &['N', 'G'])
    ),
    case!(
        ['V', 'I', 'E', 'E', 'T', 'S'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::V, &['V'], &[(V::I, C::Upper), (V::ECircumflex, C::Upper)], Tone::Acute, Coda::T, &['T'])
    ),
    case!(
        ['D', 'D', 'U', 'O', 'W', 'C', 'J'],
        ParseAppendingPhase::Coda,
        ExpectedSyllable::syllable(Onset::DStroke, &['Đ'], &[(V::UHorn, C::Upper), (V::OHorn, C::Upper)], Tone::Dot, Coda::C, &['C'])
    ),
    case!(['Q', 'U', 'A', 'S'], ParseAppendingPhase::Vowel, ExpectedSyllable::onset_vowel(Onset::Qu, &['Q', 'U'], &[(V::A, C::Upper)], Tone::Acute)),
    case!(
        ['K', 'H', 'O', 'O', 'I'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::Kh, &['K', 'H'], &[(V::OCircumflex, C::Upper), (V::I, C::Upper)], Tone::Flat)
    ),
    case!(
        ['M', 'A', 'A', 'Y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::M, &['M'], &[(V::ACircumflex, C::Upper), (V::Y, C::Upper)], Tone::Flat)
    ),
    case!(
        ['C', 'A', 'A', 'Y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::C, &['C'], &[(V::ACircumflex, C::Upper), (V::Y, C::Upper)], Tone::Flat)
    ),
    case!(
        ['D', 'D', 'A', 'A', 'S', 'Y'],
        ParseAppendingPhase::Vowel,
        ExpectedSyllable::onset_vowel(Onset::DStroke, &['Đ'], &[(V::ACircumflex, C::Upper), (V::Y, C::Upper)], Tone::Acute)
    ),
];
