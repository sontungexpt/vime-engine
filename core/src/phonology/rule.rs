use super::vowel::BaseVowel;

/// Whether a vowel nucleus is a known Vietnamese sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NucleusStatus {
    /// The sequence can never form a valid Vietnamese nucleus.
    Dead,
    /// The sequence is a complete, valid nucleus.
    Valid,
    /// The sequence is not yet complete but may become valid.
    InComplete,
}

/// Checks a vowel nucleus (1–3 vowels) against the Vietnamese rule table.
#[inline]
pub const fn check_nucleus_validity(vowels: &[BaseVowel]) -> NucleusStatus {
    use BaseVowel::*;
    use NucleusStatus::*;

    match vowels {
        // ─────────────────── Single vowels ───────────────────
        [A] => Valid,           // a
        [ABreve] => Valid,      // ă
        [ACircumflex] => Valid, // â

        [E] => Valid,           // e
        [ECircumflex] => Valid, // ê

        [I] => Valid, // i
        [Y] => Valid, // y

        [O] => Valid,           // o
        [OCircumflex] => Valid, // ô
        [OHorn] => Valid,       // ơ

        [U] => Valid,     // u
        [UHorn] => Valid, // ư

        // ─────────────────── a family ───────────────────
        [A, I] => Valid,           // ai
        [A, O] => Valid,           // ao
        [A, U] => Valid,           // au
        [A, Y] => Valid,           // ay
        [ACircumflex, U] => Valid, // âu
        [ACircumflex, Y] => Valid, // ây

        // ─────────────────── i / y family ───────────────────
        [I, A] => Valid,              // ia
        [I, E] => InComplete,         // ie
        [I, ECircumflex] => Valid,    // iê
        [I, ECircumflex, U] => Valid, // iêu

        [Y, E] => InComplete,         // ye
        [Y, ECircumflex] => Valid,    // yê
        [Y, E, U] => InComplete,      // yeu
        [Y, ECircumflex, U] => Valid, // yêu

        // ─────────────────── e family ───────────────────
        [E, O] => Valid,           // eo
        [E, U] => InComplete,      // eu
        [ECircumflex, U] => Valid, // êu

        // ─────────────────── o family ───────────────────
        [O, A] => Valid,    // oa
        [O, A, I] => Valid, // oai
        [O, A, O] => Valid, // oao
        [O, E] => Valid,    // oe

        [O, ABreve] => Valid, // oa

        // ─────────────────── u + y family ───────────────────
        [U, Y] => Valid,              // uy
        [U, Y, E] => InComplete,      // uye
        [U, Y, ECircumflex] => Valid, // uyê

        // ─────────────────── u + a family ───────────────────
        [U, A] => Valid, // ua

        // ─────────────────── u + o transactional family ───────────────────
        [U, O] => InComplete,      // uo
        [U, OHorn] => Valid,       // uơ
        [U, OCircumflex] => Valid, // uô

        [U, O, I] => InComplete,      // uoi
        [U, OCircumflex, I] => Valid, // uôi

        // ─────────────────── u + e family ───────────────────
        [U, E] => InComplete,      // ue
        [U, ECircumflex] => Valid, // uê

        // ─────────────────── ư + o transactional family ───────────────────
        [UHorn, O] => InComplete, // ưo
        [UHorn, OHorn] => Valid,  // ươ

        [UHorn, OHorn, I] => Valid, // ươi
        [UHorn, OHorn, U] => Valid, // ươu

        // ─────────────────── ư family ───────────────────
        [UHorn, A] => Valid, // ưa
        [UHorn, I] => Valid, // ưi
        [UHorn, U] => Valid, // ưu

        // ─────────────────── Invalid ───────────────────
        _ => Dead,
    }
}
