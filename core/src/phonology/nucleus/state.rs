//! The Vietnamese nucleus rule table: whether a sequence of base vowels is a
//! known syllable nucleus, and if so whether it is complete.

use super::super::BaseVowel;

use super::slice::BaseVowelSlice;

pub const NUCLEUS_MAX_LEN: usize = 3;

pub trait NucleusStateResolver {
    fn resolve_state(&self) -> NucleusState;
}

impl<T: BaseVowelSlice + ?Sized> NucleusStateResolver for T {
    #[inline(always)]
    fn resolve_state(&self) -> NucleusState {
        nucleus_state(self)
    }
}

/// Whether a vowel nucleus is a known Vietnamese sequence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum NucleusState {
    /// The sequence can never form a valid Vietnamese nucleus.
    Dead = 0,
    /// The sequence is a complete, valid nucleus.
    Valid = 1,
    /// The sequence is not yet complete but may become valid.
    #[default]
    InComplete = 2,
}

impl NucleusState {
    #[inline(always)]
    pub fn is_dead(self) -> bool {
        matches!(self, Self::Dead)
    }

    #[inline(always)]
    pub fn is_incomplete(self) -> bool {
        matches!(self, Self::InComplete)
    }

    #[inline(always)]
    pub fn is_valid(self) -> bool {
        matches!(self, Self::Valid)
    }
}

macro_rules! state_match {
    // Public entry point.

    ($vowels:ident; $($rules:tt)*) => {
        state_match! {
            @collect
            $vowels
            []
            []
            $($rules)*
        }
    };

    // Empty-nucleus rule.

    (@collect
        $vowels:ident
        [$($two:tt)*]
        [$($three:tt)*]

        [] => $state:ident,
        $($rest:tt)*
    ) => {
        state_match! {
            @collect
            $vowels
            [$($two)*]
            [$($three)*]
            $($rest)*
        }
    };

    // Single-vowel rule.

    (@collect
        $vowels:ident
        [$($two:tt)*]
        [$($three:tt)*]

        [$a:ident] => $state:ident,
        $($rest:tt)*
    ) => {
        state_match! {
            @collect
            $vowels
            [$($two)*]
            [$($three)*]
            $($rest)*
        }
    };

    // Two-vowel rules.

    (@collect
        $vowels:ident
        [$($two:tt)*]
        [$($three:tt)*]

        [$a:ident, $b:ident] => $state:ident,
        $($rest:tt)*
    ) => {
        state_match! {
            @collect
            $vowels
            [
                $($two)*
                ($a, $b) => $state,
            ]
            [$($three)*]
            $($rest)*
        }
    };

    // Three-vowel rules.

    (@collect
        $vowels:ident
        [$($two:tt)*]
        [$($three:tt)*]

        [$a:ident, $b:ident, $c:ident] => $state:ident,
        $($rest:tt)*
    ) => {
        state_match! {
            @collect
            $vowels
            [$($two)*]
            [
                $($three)*
                ($a, $b, $c) => $state,
            ]
            $($rest)*
        }
    };

    // End: emit the generated expression.

    (@collect
        $vowels:ident
        [$($two:tt)*]
        [$($three:tt)*]
    ) => {
        match $vowels.len() {
            // Empty nucleus.
            0 => InComplete,

            // Every single vowel is valid; the `[X] => Valid` rules in the table
            // are documentation only, so no match is generated for arity 1.
            1 => Valid,

            // Two-vowel nucleus.
            2 => match unsafe { ($vowels.at_unchecked(0), $vowels.at_unchecked(1)) } {
                $($two)*
                _ => Dead,
            },

            // Three-vowel nucleus.
            3 => match unsafe {
                (
                    $vowels.at_unchecked(0),
                    $vowels.at_unchecked(1),
                    $vowels.at_unchecked(2),
                )
            } {
                $($three)*
                _ => Dead,
            },

            // NUCLEUS_MAX_LEN = 3.
            _ => Dead,
        }
    };
}

/// Looks up whether a vowel nucleus is a known Vietnamese sequence.
///
/// Reads `vowels` in place with no copy; a `[Vowel]` slice is unpacked by the
/// [`BaseVowelSlice`] impl.
#[inline(always)]
pub fn nucleus_state<Slice>(vowels: &Slice) -> NucleusState
where
    Slice: BaseVowelSlice + ?Sized,
{
    use BaseVowel::*;
    use NucleusState::*;

    // The macro buckets this flat table by arity into `match vowels.len()`.
    state_match! {
        vowels;

        [] => InComplete,

        // ─────────────────── Single vowels ───────────────────
        // Kept for documentation; `1 => Valid` covers them all.
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
