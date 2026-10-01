use crate::phonology::{BaseVowel, BaseVowelSlice, Coda, Onset, Vowel, NUCLEUS_MAX_LEN};
use crate::util::vec::ArrayVec;

// ─────────────────────────── Transform ───────────────────────────

/// Effect of applying a transform key (shape/tone mark) to the syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformTarget {
    DStroke,
    Nucleus(usize),
    UoNucleus,
    LazyTone,
}

/// Describes how the logical rendered buffer changed.
///
/// Indices address `onset ++ nucleus ++ coda`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditEffect {
    StructurallyChanged,

    // Usize here is the smallest index affected by the transform.
    Transformed {
        target: TransformTarget,
        reverted: bool,
    },
}

/// Effect of applying a transform key (shape/tone mark) to the syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TransformResult {
    /// Applied a new mark to the syllable (e.g. `a` + `w` -> `ă`).
    Applied(TransformTarget),
    /// Undid an existing mark back to base (e.g. `ă` + `w` -> `a`).
    Reverted(TransformTarget),
    /// The key cannot transform the current state; pass through as a literal char.
    NotApplicable,
}

pub(super) type OnsetChars = ArrayVec<char, { Onset::MAX_LEN }>;
pub(super) type CodaChars = ArrayVec<char, { Coda::MAX_LEN }>;

/// A lightweight helper wrapper pairing toneless nucleus vowels with their cached state.
///
/// This is a private/internal builder helper for `BuildingSyllable`. Its main roles are:
/// 1. Co-locating the vowel storage and its [`NucleusState`] cache to keep them synchronized.
/// 2. Enforcing the toneless invariant: every mutation method automatically strips incoming vowel tones via `.without_tone()`.
///
/// The syllable's actual tone remains exclusively managed by `BuildingSyllable`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct FlatNucleus {
    vowels: ArrayVec<Vowel, NUCLEUS_MAX_LEN>,
}

impl FlatNucleus {
    /// Borrow the underlying toneless vowels.
    #[inline(always)]
    pub fn vowels(&self) -> &[Vowel] {
        &self.vowels
    }

    /// Returns the number of vowels currently in the nucleus.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.vowels.len()
    }

    /// Returns `true` if the nucleus contains no vowels.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.vowels.is_empty()
    }

    /// Returns an iterator over the internal toneless vowels.
    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &Vowel> {
        self.vowels.iter()
    }

    /// Appends a vowel, automatically stripping its tone to maintain the toneless invariant.
    #[inline(always)]
    pub fn push(&mut self, vowel: Vowel) {
        self.vowels.push(vowel.without_tone());
    }

    /// Removes and returns the last vowel.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<Vowel> {
        self.vowels.pop()
    }

    /// Inserts a vowel at `index`, automatically stripping its tone to maintain the toneless invariant.
    #[inline(always)]
    pub fn insert(&mut self, index: usize, vowel: Vowel) {
        self.vowels.insert(index, vowel.without_tone());
    }

    /// Removes and returns the vowel at `index`.
    #[inline(always)]
    pub fn remove(&mut self, index: usize) -> Vowel {
        self.vowels.remove(index)
    }

    /// Resets the nucleus to empty and resets state to incomplete.
    #[inline]
    pub fn clear(&mut self) {
        self.vowels.clear();
    }
}

impl core::ops::Index<usize> for FlatNucleus {
    type Output = Vowel;

    #[inline(always)]
    fn index(&self, index: usize) -> &Vowel {
        &self.vowels[index]
    }
}

impl core::ops::IndexMut<usize> for FlatNucleus {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Vowel {
        &mut self.vowels[index]
    }
}

impl BaseVowelSlice for FlatNucleus {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    unsafe fn at_unchecked(&self, index: usize) -> BaseVowel {
        unsafe { self.vowels.get_unchecked(index).base() }
    }
}

impl Default for FlatNucleus {
    fn default() -> Self {
        Self {
            vowels: ArrayVec::new(),
        }
    }
}
