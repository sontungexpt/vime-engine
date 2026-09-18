//! Precomposed Vietnamese vowel lookup tables and codec.

use super::{BaseVowel, Case, Tone};

// All 144 precomposed Vietnamese vowel characters.
//
// Layout: one block of 12 entries per base vowel, in priority-ID order.
// Within each block, the 6 tones come in Lower/Upper order (Lower, Upper,
// Lower, Upper, ...), so each block is 12 entries long. See `encode`.
const ENCODED_VOWELS: [char; 144] = [
    'ơ', 'Ơ', 'ớ', 'Ớ', 'ờ', 'Ờ', 'ở', 'Ở', 'ỡ', 'Ỡ', 'ợ', 'Ợ', // ID 0: OHorn (ơ)
    'ê', 'Ê', 'ế', 'Ế', 'ề', 'Ề', 'ể', 'Ể', 'ễ', 'Ễ', 'ệ', 'Ệ', // ID 1: ECircumflex (ê)
    'ă', 'Ă', 'ắ', 'Ắ', 'ằ', 'Ằ', 'ẳ', 'Ẳ', 'ẵ', 'Ẵ', 'ặ', 'Ặ', // ID 2: ABreve (ă)
    'ô', 'Ô', 'ố', 'Ố', 'ồ', 'Ồ', 'ổ', 'Ổ', 'ỗ', 'Ỗ', 'ộ', 'Ộ', // ID 3: OCircumflex (ô)
    'â', 'Â', 'ấ', 'Ấ', 'ầ', 'Ầ', 'ẩ', 'Ẩ', 'ẫ', 'Ẫ', 'ậ', 'Ậ', // ID 4: ACircumflex (â)
    'ư', 'Ư', 'ứ', 'Ứ', 'ừ', 'Ừ', 'ử', 'Ử', 'ữ', 'Ữ', 'ự', 'Ự', // ID 5: UHorn (ư)
    'a', 'A', 'á', 'Á', 'à', 'À', 'ả', 'Ả', 'ã', 'Ã', 'ạ', 'Ạ', // ID 6: A (a)
    'o', 'O', 'ó', 'Ó', 'ò', 'Ò', 'ỏ', 'Ỏ', 'õ', 'Õ', 'ọ', 'Ọ', // ID 7: O (o)
    'e', 'E', 'é', 'É', 'è', 'È', 'ẻ', 'Ẻ', 'ẽ', 'Ẽ', 'ẹ', 'Ẹ', // ID 8: E (e)
    'i', 'I', 'í', 'Í', 'ì', 'Ì', 'ỉ', 'Ỉ', 'ĩ', 'Ĩ', 'ị', 'Ị', // ID 9: I (i)
    'u', 'U', 'ú', 'Ú', 'ù', 'Ù', 'ủ', 'Ủ', 'ũ', 'Ũ', 'ụ', 'Ụ', // ID 10: U (u)
    'y', 'Y', 'ý', 'Ý', 'ỳ', 'Ỳ', 'ỷ', 'Ỷ', 'ỹ', 'Ỹ', 'ỵ', 'Ỵ', // ID 11: Y (y)
];

/// Encodes a `(base, tone, case)` triple as the single precomposed character.
///
/// The index is: `(block id * 6 tones + tone) * 2 cases + case`. Because
/// `base.id() <= 11`, `tone <= 5` and `case <= 1`, the index is always in
/// `0..=143`, so the lookup below can never go out of bounds.
#[inline(always)]
pub const fn encode_vowel(base: BaseVowel, tone: Tone, case: Case) -> char {
    let idx = ((base.id() * 6 + tone as usize) << 1) | (case as usize);
    ENCODED_VOWELS[idx]
}

/// Direct lookup table (LUT) for the precomposed Vietnamese Unicode block (U+1EA0..=U+1EF9).
///
/// Maps character offsets directly to `(BaseVowel, Tone, Case)` with $O(1)$ constant-time performance.
/// Offset formula: `(code - 0x1EA0) as usize`
const DECODED_VIETNAMESE_BLOCK_LUT: [(BaseVowel, Tone, Case); 90] = [
    // 0x1EA0 - 0x1EA1 (Ạ, ạ)
    (BaseVowel::A, Tone::Dot, Case::Upper),
    (BaseVowel::A, Tone::Dot, Case::Lower),
    // 0x1EA2 - 0x1EA3 (Ả, ả)
    (BaseVowel::A, Tone::Hook, Case::Upper),
    (BaseVowel::A, Tone::Hook, Case::Lower),
    // 0x1EA4 - 0x1EA5 (Ấ, ấ)
    (BaseVowel::ACircumflex, Tone::Acute, Case::Upper),
    (BaseVowel::ACircumflex, Tone::Acute, Case::Lower),
    // 0x1EA6 - 0x1EA7 (Ầ, ầ)
    (BaseVowel::ACircumflex, Tone::Grave, Case::Upper),
    (BaseVowel::ACircumflex, Tone::Grave, Case::Lower),
    // 0x1EA8 - 0x1EA9 (Ẩ, ẩ)
    (BaseVowel::ACircumflex, Tone::Hook, Case::Upper),
    (BaseVowel::ACircumflex, Tone::Hook, Case::Lower),
    // 0x1EAA - 0x1EAB (Ẫ, ẫ)
    (BaseVowel::ACircumflex, Tone::Tilde, Case::Upper),
    (BaseVowel::ACircumflex, Tone::Tilde, Case::Lower),
    // 0x1EAC - 0x1EAD (Ậ, ậ)
    (BaseVowel::ACircumflex, Tone::Dot, Case::Upper),
    (BaseVowel::ACircumflex, Tone::Dot, Case::Lower),
    // 0x1EAE - 0x1EAF (Ắ, ắ)
    (BaseVowel::ABreve, Tone::Acute, Case::Upper),
    (BaseVowel::ABreve, Tone::Acute, Case::Lower),
    // 0x1EB0 - 0x1EB1 (Ằ, ằ)
    (BaseVowel::ABreve, Tone::Grave, Case::Upper),
    (BaseVowel::ABreve, Tone::Grave, Case::Lower),
    // 0x1EB2 - 0x1EB3 (Ẳ, ẳ)
    (BaseVowel::ABreve, Tone::Hook, Case::Upper),
    (BaseVowel::ABreve, Tone::Hook, Case::Lower),
    // 0x1EB4 - 0x1EB5 (Ẵ, ẵ)
    (BaseVowel::ABreve, Tone::Tilde, Case::Upper),
    (BaseVowel::ABreve, Tone::Tilde, Case::Lower),
    // 0x1EB6 - 0x1EB7 (Ặ, ặ)
    (BaseVowel::ABreve, Tone::Dot, Case::Upper),
    (BaseVowel::ABreve, Tone::Dot, Case::Lower),
    // 0x1EB8 - 0x1EB9 (Ẹ, ẹ)
    (BaseVowel::E, Tone::Dot, Case::Upper),
    (BaseVowel::E, Tone::Dot, Case::Lower),
    // 0x1EBA - 0x1EBB (Ẻ, ẻ)
    (BaseVowel::E, Tone::Hook, Case::Upper),
    (BaseVowel::E, Tone::Hook, Case::Lower),
    // 0x1EBC - 0x1EBD (Ẽ, ẽ)
    (BaseVowel::E, Tone::Tilde, Case::Upper),
    (BaseVowel::E, Tone::Tilde, Case::Lower),
    // 0x1EBE - 0x1EBF (Ế, ế)
    (BaseVowel::ECircumflex, Tone::Acute, Case::Upper),
    (BaseVowel::ECircumflex, Tone::Acute, Case::Lower),
    // 0x1EC0 - 0x1EC1 (Ề, ề)
    (BaseVowel::ECircumflex, Tone::Grave, Case::Upper),
    (BaseVowel::ECircumflex, Tone::Grave, Case::Lower),
    // 0x1EC2 - 0x1EC3 (Ể, ể)
    (BaseVowel::ECircumflex, Tone::Hook, Case::Upper),
    (BaseVowel::ECircumflex, Tone::Hook, Case::Lower),
    // 0x1EC4 - 0x1EC5 (Ễ, ễ)
    (BaseVowel::ECircumflex, Tone::Tilde, Case::Upper),
    (BaseVowel::ECircumflex, Tone::Tilde, Case::Lower),
    // 0x1EC6 - 0x1EC7 (Ệ, ệ)
    (BaseVowel::ECircumflex, Tone::Dot, Case::Upper),
    (BaseVowel::ECircumflex, Tone::Dot, Case::Lower),
    // 0x1EC8 - 0x1EC9 (Ỉ, ỉ)
    (BaseVowel::I, Tone::Hook, Case::Upper),
    (BaseVowel::I, Tone::Hook, Case::Lower),
    // 0x1ECA - 0x1ECB (Ị, ị)
    (BaseVowel::I, Tone::Dot, Case::Upper),
    (BaseVowel::I, Tone::Dot, Case::Lower),
    // 0x1ECC - 0x1ECD (Ọ, ọ)
    (BaseVowel::O, Tone::Dot, Case::Upper),
    (BaseVowel::O, Tone::Dot, Case::Lower),
    // 0x1ECE - 0x1ECF (Ỏ, ỏ)
    (BaseVowel::O, Tone::Hook, Case::Upper),
    (BaseVowel::O, Tone::Hook, Case::Lower),
    // 0x1ED0 - 0x1ED1 (Ố, ố)
    (BaseVowel::OCircumflex, Tone::Acute, Case::Upper),
    (BaseVowel::OCircumflex, Tone::Acute, Case::Lower),
    // 0x1ED2 - 0x1ED3 (Ồ, ồ)
    (BaseVowel::OCircumflex, Tone::Grave, Case::Upper),
    (BaseVowel::OCircumflex, Tone::Grave, Case::Lower),
    // 0x1ED4 - 0x1ED5 (Ổ, ổ)
    (BaseVowel::OCircumflex, Tone::Hook, Case::Upper),
    (BaseVowel::OCircumflex, Tone::Hook, Case::Lower),
    // 0x1ED6 - 0x1ED7 (Ỗ, ỗ)
    (BaseVowel::OCircumflex, Tone::Tilde, Case::Upper),
    (BaseVowel::OCircumflex, Tone::Tilde, Case::Lower),
    // 0x1ED8 - 0x1ED9 (Ộ, ộ)
    (BaseVowel::OCircumflex, Tone::Dot, Case::Upper),
    (BaseVowel::OCircumflex, Tone::Dot, Case::Lower),
    // 0x1EDA - 0x1EDB (Ớ, ớ)
    (BaseVowel::OHorn, Tone::Acute, Case::Upper),
    (BaseVowel::OHorn, Tone::Acute, Case::Lower),
    // 0x1EDC - 0x1EDD (Ờ, ờ)
    (BaseVowel::OHorn, Tone::Grave, Case::Upper),
    (BaseVowel::OHorn, Tone::Grave, Case::Lower),
    // 0x1EDE - 0x1EDF (Ở, ở)
    (BaseVowel::OHorn, Tone::Hook, Case::Upper),
    (BaseVowel::OHorn, Tone::Hook, Case::Lower),
    // 0x1EE0 - 0x1EE1 (Ỡ, ỡ)
    (BaseVowel::OHorn, Tone::Tilde, Case::Upper),
    (BaseVowel::OHorn, Tone::Tilde, Case::Lower),
    // 0x1EE2 - 0x1EE3 (Ợ, ợ)
    (BaseVowel::OHorn, Tone::Dot, Case::Upper),
    (BaseVowel::OHorn, Tone::Dot, Case::Lower),
    // 0x1EE4 - 0x1EE5 (Ụ, ụ)
    (BaseVowel::U, Tone::Dot, Case::Upper),
    (BaseVowel::U, Tone::Dot, Case::Lower),
    // 0x1EE6 - 0x1EE7 (Ủ, ủ)
    (BaseVowel::U, Tone::Hook, Case::Upper),
    (BaseVowel::U, Tone::Hook, Case::Lower),
    // 0x1EE8 - 0x1EE9 (Ứ, ứ)
    (BaseVowel::UHorn, Tone::Acute, Case::Upper),
    (BaseVowel::UHorn, Tone::Acute, Case::Lower),
    // 0x1EEA - 0x1EEB (Ừ, ừ)
    (BaseVowel::UHorn, Tone::Grave, Case::Upper),
    (BaseVowel::UHorn, Tone::Grave, Case::Lower),
    // 0x1EEC - 0x1EED (Ử, sử)
    (BaseVowel::UHorn, Tone::Hook, Case::Upper),
    (BaseVowel::UHorn, Tone::Hook, Case::Lower),
    // 0x1EEE - 0x1EEF (Ữ, ữ)
    (BaseVowel::UHorn, Tone::Tilde, Case::Upper),
    (BaseVowel::UHorn, Tone::Tilde, Case::Lower),
    // 0x1EF0 - 0x1EF1 (Ự, ự)
    (BaseVowel::UHorn, Tone::Dot, Case::Upper),
    (BaseVowel::UHorn, Tone::Dot, Case::Lower),
    // 0x1EF2 - 0x1EF3 (Ỳ, ỳ)
    (BaseVowel::Y, Tone::Grave, Case::Upper),
    (BaseVowel::Y, Tone::Grave, Case::Lower),
    // 0x1EF4 - 0x1EF5 (Ỵ, ỵ)
    (BaseVowel::Y, Tone::Dot, Case::Upper),
    (BaseVowel::Y, Tone::Dot, Case::Lower),
    // 0x1EF6 - 0x1EF7 (Ỷ, ỷ)
    (BaseVowel::Y, Tone::Hook, Case::Upper),
    (BaseVowel::Y, Tone::Hook, Case::Lower),
    // 0x1EF8 - 0x1EF9 (Ỹ, ỹ)
    (BaseVowel::Y, Tone::Tilde, Case::Upper),
    (BaseVowel::Y, Tone::Tilde, Case::Lower),
];

/// Decodes a precomposed Vietnamese vowel character into `(base, tone, case)`.
///
/// Returns `None` if the character is not a Vietnamese vowel.
///
/// # How it works
///
/// This is the fastest decoder variant measured: instead of a giant branch-heavy
/// 144-arm match table, characters are split into four compact, non-overlapping
/// code-point regions to maximize branch-prediction and cache locality:
///
/// 1. **ASCII** (`0x00..=0x7F`) — `a e i o u y` in both cases (12 arms).
/// 2. **Latin-1 Supplement** (`0x80..=0xFF`) — `â ê ô á à ã...` scattered in Latin-1 (32 arms).
/// 3. **Latin Extended** (`0x0100..=0x01B0`) — Narrowed bounds for `ă ĩ ũ ơ ư` (10 arms).
/// 4. **Vietnamese Block** (`0x1EA0..=0x1EF9`) — Continuous 90-element precomposed block
///    mapped via zero-overhead $O(1)$ direct array index (`code - 0x1EA0`).
#[inline(always)]
pub const fn decode_vowel(character: char) -> Option<(BaseVowel, Tone, Case)> {
    let code = character as u32;

    match code {
        // 1. ASCII Block (Fast path - Keystrokes)
        0x00..=0x7F => match character {
            'a' => Some((BaseVowel::A, Tone::Flat, Case::Lower)),
            'A' => Some((BaseVowel::A, Tone::Flat, Case::Upper)),
            'o' => Some((BaseVowel::O, Tone::Flat, Case::Lower)),
            'O' => Some((BaseVowel::O, Tone::Flat, Case::Upper)),
            'e' => Some((BaseVowel::E, Tone::Flat, Case::Lower)),
            'E' => Some((BaseVowel::E, Tone::Flat, Case::Upper)),
            'i' => Some((BaseVowel::I, Tone::Flat, Case::Lower)),
            'I' => Some((BaseVowel::I, Tone::Flat, Case::Upper)),
            'u' => Some((BaseVowel::U, Tone::Flat, Case::Lower)),
            'U' => Some((BaseVowel::U, Tone::Flat, Case::Upper)),
            'y' => Some((BaseVowel::Y, Tone::Flat, Case::Lower)),
            'Y' => Some((BaseVowel::Y, Tone::Flat, Case::Upper)),
            _ => None,
        },

        // 2. Latin-1 Supplement (U+00C0..U+00FF)
        0x80..=0xFF => match character {
            'ê' => Some((BaseVowel::ECircumflex, Tone::Flat, Case::Lower)),
            'Ê' => Some((BaseVowel::ECircumflex, Tone::Flat, Case::Upper)),
            'ô' => Some((BaseVowel::OCircumflex, Tone::Flat, Case::Lower)),
            'Ô' => Some((BaseVowel::OCircumflex, Tone::Flat, Case::Upper)),
            'â' => Some((BaseVowel::ACircumflex, Tone::Flat, Case::Lower)),
            'Â' => Some((BaseVowel::ACircumflex, Tone::Flat, Case::Upper)),
            'á' => Some((BaseVowel::A, Tone::Acute, Case::Lower)),
            'Á' => Some((BaseVowel::A, Tone::Acute, Case::Upper)),
            'à' => Some((BaseVowel::A, Tone::Grave, Case::Lower)),
            'À' => Some((BaseVowel::A, Tone::Grave, Case::Upper)),
            'ã' => Some((BaseVowel::A, Tone::Tilde, Case::Lower)),
            'Ã' => Some((BaseVowel::A, Tone::Tilde, Case::Upper)),
            'ó' => Some((BaseVowel::O, Tone::Acute, Case::Lower)),
            'Ó' => Some((BaseVowel::O, Tone::Acute, Case::Upper)),
            'ò' => Some((BaseVowel::O, Tone::Grave, Case::Lower)),
            'Ò' => Some((BaseVowel::O, Tone::Grave, Case::Upper)),
            'õ' => Some((BaseVowel::O, Tone::Tilde, Case::Lower)),
            'Õ' => Some((BaseVowel::O, Tone::Tilde, Case::Upper)),
            'é' => Some((BaseVowel::E, Tone::Acute, Case::Lower)),
            'É' => Some((BaseVowel::E, Tone::Acute, Case::Upper)),
            'è' => Some((BaseVowel::E, Tone::Grave, Case::Lower)),
            'È' => Some((BaseVowel::E, Tone::Grave, Case::Upper)),
            'í' => Some((BaseVowel::I, Tone::Acute, Case::Lower)),
            'Í' => Some((BaseVowel::I, Tone::Acute, Case::Upper)),
            'ì' => Some((BaseVowel::I, Tone::Grave, Case::Lower)),
            'Ì' => Some((BaseVowel::I, Tone::Grave, Case::Upper)),
            'ú' => Some((BaseVowel::U, Tone::Acute, Case::Lower)),
            'Ú' => Some((BaseVowel::U, Tone::Acute, Case::Upper)),
            'ù' => Some((BaseVowel::U, Tone::Grave, Case::Lower)),
            'Ù' => Some((BaseVowel::U, Tone::Grave, Case::Upper)),
            'ý' => Some((BaseVowel::Y, Tone::Acute, Case::Lower)),
            'Ý' => Some((BaseVowel::Y, Tone::Acute, Case::Upper)),
            _ => None,
        },

        // 3. Latin Extended
        0x0100..=0x01B0 => match character {
            'ơ' => Some((BaseVowel::OHorn, Tone::Flat, Case::Lower)),
            'Ơ' => Some((BaseVowel::OHorn, Tone::Flat, Case::Upper)),
            'ă' => Some((BaseVowel::ABreve, Tone::Flat, Case::Lower)),
            'Ă' => Some((BaseVowel::ABreve, Tone::Flat, Case::Upper)),
            'ư' => Some((BaseVowel::UHorn, Tone::Flat, Case::Lower)),
            'Ư' => Some((BaseVowel::UHorn, Tone::Flat, Case::Upper)),
            'ĩ' => Some((BaseVowel::I, Tone::Tilde, Case::Lower)),
            'Ĩ' => Some((BaseVowel::I, Tone::Tilde, Case::Upper)),
            'ũ' => Some((BaseVowel::U, Tone::Tilde, Case::Lower)),
            'Ũ' => Some((BaseVowel::U, Tone::Tilde, Case::Upper)),
            _ => None,
        },

        // 4. Vietnamese block (U+1EA0..U+1EF9) -> Direct Indexing Table!
        0x1EA0..=0x1EF9 => {
            let offset = (code - 0x1EA0) as usize;
            Some(DECODED_VIETNAMESE_BLOCK_LUT[offset])
        }

        _ => None,
    }
}
