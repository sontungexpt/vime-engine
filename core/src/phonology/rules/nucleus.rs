use super::super::vowel::BaseVowel;

/// Whether a vowel nucleus is a known Vietnamese sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NucleusState {
    /// The sequence can never form a valid Vietnamese nucleus.
    Dead,
    /// The sequence is a complete, valid nucleus.
    Valid,
    /// The sequence is not yet complete but may become valid.
    InComplete,
}

impl NucleusState {
    pub fn check(vowels: &[BaseVowel]) -> Self {
        use BaseVowel::*;
        use NucleusState::*;

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
            [I, E, U] => InComplete,      // ieu
            [I, ECircumflex, U] => Valid, // iêu
            [I, U] => Valid,              // iu

            [Y, E] => InComplete,         // ye
            [Y, ECircumflex] => Valid,    // yê
            [Y, E, U] => InComplete,      // yeu
            [Y, ECircumflex, U] => Valid, // yêu

            // ─────────────────── e family ───────────────────
            [E, O] => Valid,           // eo
            [E, U] => InComplete,      // eu
            [ECircumflex, U] => Valid, // êu

            // ─────────────────── o family ───────────────────
            [O, A] => Valid,           // oa
            [O, ABreve] => Valid,      // oă
            [O, A, I] => Valid,        // oai
            [O, A, O] => Valid,        // oao
            [O, A, U] => Valid,        // oau
            [O, A, Y] => Valid,        // oay
            [O, E] => Valid,           // oe
            [O, E, O] => Valid,        // oeo
            [O, I] => Valid,           // oi
            [OCircumflex, I] => Valid, // ôi
            [OHorn, I] => Valid,       // ơi
            [O, O] => InComplete,      // oo

            // ─────────────────── u + y family ───────────────────
            [U, Y] => Valid,              // uy
            [U, Y, U] => Valid,           // uyu
            [U, Y, A] => Valid,           // uya
            [U, Y, E] => InComplete,      // uye
            [U, Y, ECircumflex] => Valid, // uyê

            // ─────────────────── u + a family ───────────────────
            [U, A] => Valid,              // ua
            [U, A, O] => Valid,           // uao
            [U, ACircumflex] => Valid,    // uâ
            [U, ACircumflex, Y] => Valid, // uây

            // ─────────────────── u + o transactional family ───────────────────
            [U, O] => InComplete,      // uo
            [U, OHorn] => Valid,       // uơ
            [U, OCircumflex] => Valid, // uô

            [U, O, I] => InComplete,      // uoi
            [U, OCircumflex, I] => Valid, // uôi
            [U, OHorn, I] => InComplete,  // uơi

            [U, O, U] => InComplete,     // uou
            [U, OHorn, U] => InComplete, // uơu

            // ─────────────────── u + e family ───────────────────
            [U, E] => InComplete,      // ue
            [U, ECircumflex] => Valid, // uê
            [U, I] => Valid,           // ui

            // ─────────────────── ư + o transactional family ───────────────────
            [UHorn, O] => InComplete, // ưo
            [UHorn, OHorn] => Valid,  // ươ

            [UHorn, O, I] => InComplete, // ưoi
            [UHorn, OHorn, I] => Valid,  // ươi

            [UHorn, O, U] => InComplete, // ưou
            [UHorn, OHorn, U] => Valid,  // ươu

            // ─────────────────── ư family ───────────────────
            [UHorn, A] => Valid,  // ưa
            [UHorn, I] => Valid,  // ưi
            [U, U] => InComplete, // uu
            [UHorn, U] => Valid,  // ưu

            // ─────────────────── Invalid ───────────────────
            _ => Dead,
        }
    }
}
