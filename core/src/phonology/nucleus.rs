use super::vowel::BaseVowel;

pub const NUCLEUS_MAX_LEN: usize = 3;

/// Whether a vowel nucleus is a known Vietnamese sequence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NucleusState {
    /// The sequence can never form a valid Vietnamese nucleus.
    Dead,
    /// The sequence is a complete, valid nucleus.
    Valid,

    /// The sequence is not yet complete but may become valid.
    #[default]
    InComplete,
}

macro_rules! nucleus_rules {
    // ============================================================
    // Public entry point
    // ============================================================

    ($vowels:ident; $($rules:tt)*) => {
        nucleus_rules! {
            @collect
            $vowels
            []
            []
            $($rules)*
        }
    };

    // ============================================================
    // Empty nucleus
    // ============================================================

    (@collect
        $vowels:ident
        [$($two:tt)*]
        [$($three:tt)*]

        [] => $state:ident,
        $($rest:tt)*
    ) => {
        nucleus_rules! {
            @collect
            $vowels
            [$($two)*]
            [$($three)*]
            $($rest)*
        }
    };

    // ============================================================
    // Single vowel
    // ============================================================

    (@collect
        $vowels:ident
        [$($two:tt)*]
        [$($three:tt)*]

        [$a:ident] => $state:ident,
        $($rest:tt)*
    ) => {
        nucleus_rules! {
            @collect
            $vowels
            [$($two)*]
            [$($three)*]
            $($rest)*
        }
    };

    // ============================================================
    // Two vowels
    // ============================================================

    (@collect
        $vowels:ident
        [$($two:tt)*]
        [$($three:tt)*]

        [$a:ident, $b:ident] => $state:ident,
        $($rest:tt)*
    ) => {
        nucleus_rules! {
            @collect
            $vowels
            [$($two)* [$a, $b] => $state,]
            [$($three)*]
            $($rest)*
        }
    };

    // ============================================================
    // Three vowels
    // ============================================================

    (@collect
        $vowels:ident
        [$($two:tt)*]
        [$($three:tt)*]

        [$a:ident, $b:ident, $c:ident] => $state:ident,
        $($rest:tt)*
    ) => {
        nucleus_rules! {
            @collect
            $vowels
            [$($two)*]
            [$($three)* [$a, $b, $c] => $state,]
            $($rest)*
        }
    };

    // ============================================================
    // End: generate expression
    // ============================================================

    (@collect
        $vowels:ident
        [$($two:tt)*]
        [$($three:tt)*]
    ) => {
        match $vowels.len() {
            // [ ] => InComplete
            0 => InComplete,

            // [A], [E], [I], ... => Valid
            //
            // Single-vowel rules are intentionally kept in the rule table
            // as documentation, but do not need individual match arms.
            1 => Valid,

            // Two-vowel rules are collected from the rule table above.
            2 => match $vowels {
                $($two)*
                _ => Dead,
            },

            // Three-vowel rules are collected from the rule table above.
            3 => match $vowels {
                $($three)*
                _ => Dead,
            },

            // NUCLEUS_MAX_LEN = 3.
            _ => Dead,
        }
    };
}

impl NucleusState {
    #[inline(always)]
    pub fn is_dead(self) -> bool {
        matches!(self, Self::Dead)
    }

    #[inline(always)]
    pub fn check(vowels: &[BaseVowel]) -> Self {
        use BaseVowel::*;
        use NucleusState::*;

        nucleus_rules! {
            vowels;

            [] => InComplete,

            // ─────────────────── Single vowels ───────────────────
            // Kept explicitly for readability/documentation.
            // All single vowels are handled uniformly by `1 => Valid`.
            [A] => Valid,
            [ABreve] => Valid,
            [ACircumflex] => Valid,

            [E] => Valid,
            [ECircumflex] => Valid,

            [I] => Valid,
            [Y] => Valid,

            [O] => Valid,
            [OCircumflex] => Valid,
            [OHorn] => Valid,

            [U] => Valid,
            [UHorn] => Valid,

            // ─────────────────── a family ───────────────────
            [A, I] => Valid,
            [A, O] => Valid,
            [A, U] => Valid,
            [A, Y] => Valid,
            [ACircumflex, U] => Valid,
            [ACircumflex, Y] => Valid,

            // ─────────────────── i / y family ───────────────────
            [I, A] => Valid,
            [I, E] => InComplete,
            [I, ECircumflex] => Valid,
            [I, E, U] => InComplete,
            [I, ECircumflex, U] => Valid,
            [I, U] => Valid,

            [Y, E] => InComplete,
            [Y, ECircumflex] => Valid,
            [Y, E, U] => InComplete,
            [Y, ECircumflex, U] => Valid,

            // ─────────────────── e family ───────────────────
            [E, O] => Valid,
            [E, U] => InComplete,
            [ECircumflex, U] => Valid,

            // ─────────────────── o family ───────────────────
            [O, A] => Valid,
            [O, ABreve] => Valid,
            [O, A, I] => Valid,
            [O, A, O] => Valid,
            [O, A, U] => Valid,
            [O, A, Y] => Valid,
            [O, E] => Valid,
            [O, E, O] => Valid,
            [O, I] => Valid,
            [OCircumflex, I] => Valid,
            [OHorn, I] => Valid,
            [O, O] => InComplete,

            // ─────────────────── u + y family ───────────────────
            [U, Y] => Valid,
            [U, Y, U] => Valid,
            [U, Y, A] => Valid,
            [U, Y, E] => InComplete,
            [U, Y, ECircumflex] => Valid,

            // ─────────────────── u + a family ───────────────────
            [U, A] => Valid,
            [U, A, O] => Valid,
            [U, ACircumflex] => Valid,
            [U, ACircumflex, Y] => Valid,

            // ─────────────────── u + o family ───────────────────
            [U, O] => InComplete,
            [U, OHorn] => Valid,
            [U, OCircumflex] => Valid,

            [U, O, I] => InComplete,
            [U, OCircumflex, I] => Valid,
            [U, OHorn, I] => InComplete,

            [U, O, U] => InComplete,
            [U, OHorn, U] => InComplete,

            // ─────────────────── u + e family ───────────────────
            [U, E] => InComplete,
            [U, ECircumflex] => Valid,
            [U, I] => Valid,

            // ─────────────────── ư + o family ───────────────────
            [UHorn, O] => InComplete,
            [UHorn, OHorn] => Valid,

            [UHorn, O, I] => InComplete,
            [UHorn, OHorn, I] => Valid,

            [UHorn, O, U] => InComplete,
            [UHorn, OHorn, U] => Valid,

            // ─────────────────── ư family ───────────────────
            [UHorn, A] => Valid,
            [UHorn, I] => Valid,
            [U, U] => InComplete,
            [UHorn, U] => Valid,
        }
    }
}
