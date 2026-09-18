/// Base ASCII vowel letter independent of shape, tone, and case.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u16)]
pub enum RootVowel {
    A = 0,
    E = 1,
    I = 2,
    O = 3,
    U = 4,
    Y = 5,
}

/// A diacritic shape that attaches to a base vowel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u16)]
pub enum Shape {
    /// No diacritic.
    #[default]
    None = 0,
    /// The circumflex `^` (â, ô, ê).
    Circumflex = 1,
    /// The breve `˘` (ă).
    Breve = 2,
    /// The horn `"` (ư, ơ).
    Horn = 3,
}

/// A Vietnamese lexical tone applied to the vowel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u16)]
pub enum Tone {
    /// No tone.
    #[default]
    Flat = 0,
    Acute = 1,
    Grave = 2,
    Hook = 3,
    Tilde = 4,
    Dot = 5,
}

impl Tone {
    /// Out-of-range indices map to [`Tone::Flat`] instead of panicking.
    #[inline(always)]
    pub const fn from_id(id: usize) -> Self {
        if id > 5 {
            return Self::Flat;
        }
        unsafe { std::mem::transmute::<u16, Self>(id as u16) }
    }
}

/// Letter case of a vowel (affects the rendered character).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u16)]
pub enum Case {
    Lower = 0,
    Upper = 1,
}

/// The 12 base vowels of Vietnamese, declared in tone-placement priority order.
///
/// # Bit Layout (`u16`)
/// `[7 bits Reserved | 4 bits Priority ID | 2 bits Shape | 3 bits Root]`
///
/// - **Bits 5..=8**: Tone priority ID (`0..=11`); higher means higher priority.
/// - **Bits 3..=4**: [`Shape`] (`0..=3`).
/// - **Bits 0..=2**: [`RootVowel`] (`0..=5`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u16)]
#[rustfmt::skip]
pub enum BaseVowel {
    // ── Priority 0..=2: Khép / Bán nguyên âm (thấp nhất) ──
    Y           = (0 << 5)  | ((Shape::None as u16) << 3)        | RootVowel::Y as u16,
    U           = (1 << 5)  | ((Shape::None as u16) << 3)        | RootVowel::U as u16,
    I           = (2 << 5)  | ((Shape::None as u16) << 3)        | RootVowel::I as u16,

    // ── Priority 3..=5: Mở đơn trơn (e, o, a) ──
    E           = (3 << 5)  | ((Shape::None as u16) << 3)        | RootVowel::E as u16,
    O           = (4 << 5)  | ((Shape::None as u16) << 3)        | RootVowel::O as u16,
    A           = (5 << 5)  | ((Shape::None as u16) << 3)        | RootVowel::A as u16,

    // ── Priority 6..=9: Có phụ hiệu (ư, â, ô, ă) ──
    UHorn       = (6 << 5)  | ((Shape::Horn as u16) << 3)        | RootVowel::U as u16,
    ACircumflex = (7 << 5)  | ((Shape::Circumflex as u16) << 3)  | RootVowel::A as u16,
    OCircumflex = (8 << 5)  | ((Shape::Circumflex as u16) << 3)  | RootVowel::O as u16,
    ABreve      = (9 << 5)  | ((Shape::Breve as u16) << 3)       | RootVowel::A as u16,

    // ── Priority 10..=11: Cao nhất (ê, ơ) ──
    // Trong vần chứa 'ơ' hoặc 'ê' (như ươ, ơi, iê, uê), dấu thanh LUÔN rơi vào Ơ / Ê.
    ECircumflex = (10 << 5) | ((Shape::Circumflex as u16) << 3)  | RootVowel::E as u16,
    OHorn       = (11 << 5) | ((Shape::Horn as u16) << 3)        | RootVowel::O as u16,
}

impl BaseVowel {
    // ─────────────── Cardinality ───────────────
    pub const COUNT: u8 = 12;
    pub const MAX_ID: u8 = Self::COUNT - 1;

    // ─────────────── Bit-field layout ───────────────
    // Packed `u16`: [Reserved | Priority ID | Shape | Root].
    // Field widths match the enum discriminants above; the two that shape
    // the layout are `RootVowel::Y` (0b101 → 3 bits) and `Shape::Horn`
    // (0b11 → 2 bits).

    // ── field widths ──
    /// Width of the [`RootVowel`] field — 3 bits cover `RootVowel::A..=Y` (0..=5).
    const ROOT_BITS: u32 = 3;
    /// Width of the [`Shape`] field — 2 bits cover `Shape::None..=Horn` (0..=3).
    const SHAPE_BITS: u32 = 2;

    // ── field shifts ──
    /// Shift of the [`Shape`] field (sits directly above the root field).
    const SHAPE_SHIFT: u32 = Self::ROOT_BITS;
    /// Shift of the Priority ID field (sits directly above the shape field).
    const ID_SHIFT: u32 = Self::SHAPE_SHIFT + Self::SHAPE_BITS;

    // ── field masks ──
    /// Bitmask for the [`RootVowel`] field.
    const ROOT_MASK: u16 = (1u16 << Self::ROOT_BITS) - 1;
    /// Bitmask for the [`Shape`] field.
    const SHAPE_MASK: u16 = (1u16 << Self::SHAPE_BITS) - 1;

    /// Full positioned bitmask for the [`Shape`] field (`0b11_000` / `0x18`).
    const SHAPE_BITMASK: u16 = Self::SHAPE_MASK << Self::SHAPE_SHIFT;

    // ─────────────── Lookup table ───────────────
    /// All base vowels in priority-ID order; the index equals the priority ID
    /// (higher ID = higher priority).
    const VARIANTS_BY_ID: [BaseVowel; Self::COUNT as usize] = [
        BaseVowel::Y,
        BaseVowel::U,
        BaseVowel::I,
        BaseVowel::E,
        BaseVowel::O,
        BaseVowel::A,
        BaseVowel::UHorn,
        BaseVowel::ACircumflex,
        BaseVowel::OCircumflex,
        BaseVowel::ABreve,
        BaseVowel::ECircumflex,
        BaseVowel::OHorn,
    ];

    /// Returns the tone-placement priority ID (`0..=11`); higher = higher priority.
    #[inline(always)]
    pub const fn id(self) -> usize {
        (self as u16 >> Self::ID_SHIFT) as usize
    }

    /// Returns the base vowel for the given priority ID, or `Err` if out of range.
    #[inline(always)]
    pub const fn from_id(id: usize) -> Result<Self, ()> {
        if id < Self::COUNT as usize {
            Ok(Self::VARIANTS_BY_ID[id])
        } else {
            Err(())
        }
    }

    /// Returns the base vowel for the given priority ID without bounds-checking.
    ///
    /// # Safety
    ///
    /// `id` must be in `0..Self::COUNT`.
    #[inline(always)]
    pub unsafe fn from_id_unchecked(id: usize) -> Self {
        *Self::VARIANTS_BY_ID.get_unchecked(id)
    }

    /// The [`BaseVowel`] for a root letter and shape, if that combination
    /// exists. Returns `None` for [`Shape::Stroke`] (a consonant stroke, never
    /// a vowel) and for shapes Vietnamese does not attach to that root.
    #[inline(always)]
    pub const fn from_parts(root: RootVowel, shape: Shape) -> Result<Self, ()> {
        match (root, shape) {
            (RootVowel::O, Shape::None) => Ok(BaseVowel::O),
            (RootVowel::O, Shape::Horn) => Ok(BaseVowel::OHorn),
            (RootVowel::O, Shape::Circumflex) => Ok(BaseVowel::OCircumflex),
            (RootVowel::E, Shape::None) => Ok(BaseVowel::E),
            (RootVowel::E, Shape::Circumflex) => Ok(BaseVowel::ECircumflex),
            (RootVowel::A, Shape::None) => Ok(BaseVowel::A),
            (RootVowel::A, Shape::Breve) => Ok(BaseVowel::ABreve),
            (RootVowel::A, Shape::Circumflex) => Ok(BaseVowel::ACircumflex),
            (RootVowel::U, Shape::None) => Ok(BaseVowel::U),
            (RootVowel::U, Shape::Horn) => Ok(BaseVowel::UHorn),
            (RootVowel::I, Shape::None) => Ok(BaseVowel::I),
            (RootVowel::Y, Shape::None) => Ok(BaseVowel::Y),
            _ => Err(()),
        }
    }

    /// Returns the base vowel with the given root and no shape.
    #[inline(always)]
    pub const fn from_root(root: RootVowel) -> Self {
        match root {
            RootVowel::A => BaseVowel::A,
            RootVowel::E => BaseVowel::E,
            RootVowel::I => BaseVowel::I,
            RootVowel::O => BaseVowel::O,
            RootVowel::U => BaseVowel::U,
            RootVowel::Y => BaseVowel::Y,
        }
    }

    /// Returns the [`Shape`] component of this vowel.
    #[inline(always)]
    pub const fn shape(self) -> Shape {
        let raw = (self as u16 >> Self::SHAPE_SHIFT) & Self::SHAPE_MASK;
        unsafe { std::mem::transmute(raw) }
    }

    /// Returns `true` when this vowel has a structural diacritic shape (ă, â, ê, ô, ơ, ư).
    #[inline(always)]
    pub const fn is_shaped(self) -> bool {
        (self as u16 & Self::SHAPE_BITMASK) != 0
    }

    /// Returns `true` when this vowel has no structural shape.
    ///
    /// The unaccented vowels occupy the lowest priority IDs (and thus the
    /// lowest raw values), so anything at or below `A` is unaccented.
    #[inline(always)]
    pub const fn is_plain(self) -> bool {
        !self.is_shaped()
    }

    /// Returns the [`RootVowel`] component of this vowel.
    #[inline(always)]
    pub const fn root(self) -> RootVowel {
        let raw = self as u16 & Self::ROOT_MASK;
        unsafe { std::mem::transmute(raw) }
    }

    /// Keeps the root vowel but swaps the shape for a new one (e.g. ă → â).
    ///
    /// Returns the canonical [`BaseVowel`] for the root and shape; `Err` when
    /// Vietnamese has no letter for that combination (e.g. `A` + [`Shape::Horn`]).
    #[inline(always)]
    pub const fn replace_shape(self, shape: Shape) -> Result<Self, ()> {
        Self::from_parts(self.root(), shape)
    }

    /// Returns the vowel with the shape removed (shape becomes [`Shape::None`]).
    #[inline(always)]
    pub const fn remove_shape(self) -> Self {
        match self {
            BaseVowel::OHorn | BaseVowel::OCircumflex => BaseVowel::O,
            BaseVowel::ACircumflex | BaseVowel::ABreve => BaseVowel::A,
            BaseVowel::ECircumflex => BaseVowel::E,
            BaseVowel::UHorn => BaseVowel::U,
            _ => self,
        }
    }
}

mod tables;

pub use tables::{decode_vowel, encode_vowel};

/// Whether `ch` is a Vietnamese vowel, without decoding its parts.
///
/// Optimized with compact range checks and lookup masks.
#[inline(always)]
pub const fn is_vowel(ch: char) -> bool {
    let code = ch as u32;

    // 1. ASCII: A..Z and a..z.
    let ascii_shift = code.wrapping_sub(65);
    if ascii_shift <= 57 {
        const ASCII_VOWEL_MASK: u64 = 0x0110411101104111;
        return ((ASCII_VOWEL_MASK >> ascii_shift) & 1) != 0;
    }
    // 2. Vietnamese Extended: every code point in this range is a vowel.
    else if code >= 7840 && code <= 7929 {
        return true;
    }
    // 3. Latin-1 Supplement.
    else if code >= 192 && code <= 253 {
        let shift = code - 192;
        const LATIN1_VOWEL_MASK: u64 = 0x263C370F_263C370F;
        return ((LATIN1_VOWEL_MASK >> shift) & 1) != 0;
    }
    // 4. Latin Extended.
    else if code >= 258 && code <= 432 {
        return matches!(
            code - 258,
            0 | 1 |     // Ă ă
              38 | 39 |   // Ĩ ĩ
              102 | 103 | // Ũ ũ
              158 | 159 | // Ơ ơ
              173 | 174 // Ư ư
        );
    }

    false
}
