//! Vietnamese vowels: the [`BaseVowel`] model and the precomposed-character
//! codec (`encode_vowel` / `decode_vowel` / [`is_vowel`]).

use crate::phonology::Cased;

/// Base ASCII vowel letter independent of shape, tone, and case.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
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
#[repr(u8)]
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
#[repr(u8)]
pub enum Tone {
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
        unsafe { std::mem::transmute::<u8, Self>(id as u8) }
    }
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
    const ROOT_WIDTH: u32 = 3;
    /// Width of the [`Shape`] field — 2 bits cover `Shape::None..=Horn` (0..=3).
    const SHAPE_WIDTH: u32 = 2;

    // ── field shifts ──
    /// Shift of the [`Shape`] field (sits directly above the root field).
    const SHAPE_OFFSET: u32 = Self::ROOT_WIDTH;
    /// Shift of the Priority ID field (sits directly above the shape field).
    const ID_OFFSET: u32 = Self::SHAPE_OFFSET + Self::SHAPE_WIDTH;

    // ── field masks ──
    /// Bitmask for the [`RootVowel`] field.
    const ROOT_MASK: u16 = (1u16 << Self::ROOT_WIDTH) - 1;
    /// Bitmask for the [`Shape`] field.
    const SHAPE_MASK: u16 = (1u16 << Self::SHAPE_WIDTH) - 1;

    /// Fully positioned mask for the [`Shape`] field (`0b11000` / `0x18`).
    const SHAPE_MASK_FULL: u16 = Self::SHAPE_MASK << Self::SHAPE_OFFSET;

    /// ─────────────── Lookup table ───────────────
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
        (self as u16 >> Self::ID_OFFSET) as usize
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
        let raw = (self as u16 >> Self::SHAPE_OFFSET) & Self::SHAPE_MASK;
        // Safety: SHAPE_MASK limits `raw` to 0..=3, matching Shape's u8 discriminants.
        unsafe { std::mem::transmute::<u8, Shape>(raw as u8) }
    }

    /// Returns `true` when this vowel has a structural diacritic shape (ă, â, ê, ô, ơ, ư).
    #[inline(always)]
    pub const fn is_shaped(self) -> bool {
        (self as u16 & Self::SHAPE_MASK_FULL) != 0
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
        // Safety: ROOT_MASK limits `raw` to 0..=5, matching RootVowel u8 discriminants.
        unsafe { std::mem::transmute::<u8, RootVowel>(raw as u8) }
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

pub type CasedBaseVowel = Cased<BaseVowel>;

impl CasedBaseVowel {
    #[inline(always)]
    pub const fn to_char(self) -> char {
        encode_vowel(self.value, Tone::Flat, self.uppercase)
    }

    #[inline(always)]
    pub const fn to_char_tone(self, tone: Tone) -> char {
        encode_vowel(self.value, tone, self.uppercase)
    }
}
// All 144 precomposed Vietnamese vowel characters.
//
// Layout: one block of 12 entries per base vowel, in priority-ID order.
// Within each block, the 6 tones come in Lower/Upper order (Lower, Upper,
// Lower, Upper, ...), so each block is 12 entries long. See `encode`.
const ENCODED_VOWELS: [char; 144] = [
    'y', 'Y', 'ý', 'Ý', 'ỳ', 'Ỳ', 'ỷ', 'Ỷ', 'ỹ', 'Ỹ', 'ỵ', 'Ỵ', // ID 0: Y (y)
    'u', 'U', 'ú', 'Ú', 'ù', 'Ù', 'ủ', 'Ủ', 'ũ', 'Ũ', 'ụ', 'Ụ', // ID 1: U (u)
    'i', 'I', 'í', 'Í', 'ì', 'Ì', 'ỉ', 'Ỉ', 'ĩ', 'Ĩ', 'ị', 'Ị', // ID 2: I (i)
    'e', 'E', 'é', 'É', 'è', 'È', 'ẻ', 'Ẻ', 'ẽ', 'Ẽ', 'ẹ', 'Ẹ', // ID 3: E (e)
    'o', 'O', 'ó', 'Ó', 'ò', 'Ò', 'ỏ', 'Ỏ', 'õ', 'Õ', 'ọ', 'Ọ', // ID 4: O (o)
    'a', 'A', 'á', 'Á', 'à', 'À', 'ả', 'Ả', 'ã', 'Ã', 'ạ', 'Ạ', // ID 5: A (a)
    'ư', 'Ư', 'ứ', 'Ứ', 'ừ', 'Ừ', 'ử', 'Ử', 'ữ', 'Ữ', 'ự', 'Ự', // ID 6: UHorn (ư)
    'â', 'Â', 'ấ', 'Ấ', 'ầ', 'Ầ', 'ẩ', 'Ẩ', 'ẫ', 'Ẫ', 'ậ', 'Ậ', // ID 7: ACircumflex (â)
    'ô', 'Ô', 'ố', 'Ố', 'ồ', 'Ồ', 'ổ', 'Ổ', 'ỗ', 'Ỗ', 'ộ', 'Ộ', // ID 8: OCircumflex (ô)
    'ă', 'Ă', 'ắ', 'Ắ', 'ằ', 'Ằ', 'ẳ', 'Ẳ', 'ẵ', 'Ẵ', 'ặ', 'Ặ', // ID 9: ABreve (ă)
    'ê', 'Ê', 'ế', 'Ế', 'ề', 'Ề', 'ể', 'Ể', 'ễ', 'Ễ', 'ệ', 'Ệ', // ID 10: ECircumflex (ê)
    'ơ', 'Ơ', 'ớ', 'Ớ', 'ờ', 'Ờ', 'ở', 'Ở', 'ỡ', 'Ỡ', 'ợ', 'Ợ', // ID 11: OHorn (ơ)
];

/// Encodes a `(base, tone, uppercase)` triple as the single precomposed character.
///
/// The index is: `(block id * 6 tones + tone) * 2 cases + uppercase`. Because
/// `base.id() <= 11`, `tone <= 5` and `uppercase <= 1`, the index is always in
/// `0..=143`, so the lookup below can never go out of bounds.
#[inline(always)]
pub const fn encode_vowel(base: BaseVowel, tone: Tone, uppercase: bool) -> char {
    let idx = ((base.id() * 6 + tone as usize) << 1) | (uppercase as usize);
    ENCODED_VOWELS[idx]
}

/// Direct lookup table (LUT) for the precomposed Vietnamese Unicode block (U+1EA0..=U+1EF9).
///
/// Maps character offsets directly to `(CasedBaseVowel, Tone)` with $O(1)$ constant-time performance.
/// Offset formula: `(code - 0x1EA0) as usize`
const DECODED_VIETNAMESE_BLOCK_LUT: [(CasedBaseVowel, Tone); 90] = [
    // 0x1EA0 - 0x1EA1 (Ạ, ạ)
    (CasedBaseVowel::upper(BaseVowel::A), Tone::Dot),
    (CasedBaseVowel::lower(BaseVowel::A), Tone::Dot),
    // 0x1EA2 - 0x1EA3 (Ả, ả)
    (CasedBaseVowel::upper(BaseVowel::A), Tone::Hook),
    (CasedBaseVowel::lower(BaseVowel::A), Tone::Hook),
    // 0x1EA4 - 0x1EA5 (Ấ, ấ)
    (CasedBaseVowel::upper(BaseVowel::ACircumflex), Tone::Acute),
    (CasedBaseVowel::lower(BaseVowel::ACircumflex), Tone::Acute),
    // 0x1EA6 - 0x1EA7 (Ầ, ầ)
    (CasedBaseVowel::upper(BaseVowel::ACircumflex), Tone::Grave),
    (CasedBaseVowel::lower(BaseVowel::ACircumflex), Tone::Grave),
    // 0x1EA8 - 0x1EA9 (Ẩ, ẩ)
    (CasedBaseVowel::upper(BaseVowel::ACircumflex), Tone::Hook),
    (CasedBaseVowel::lower(BaseVowel::ACircumflex), Tone::Hook),
    // 0x1EAA - 0x1EAB (Ẫ, ẫ)
    (CasedBaseVowel::upper(BaseVowel::ACircumflex), Tone::Tilde),
    (CasedBaseVowel::lower(BaseVowel::ACircumflex), Tone::Tilde),
    // 0x1EAC - 0x1EAD (Ậ, ậ)
    (CasedBaseVowel::upper(BaseVowel::ACircumflex), Tone::Dot),
    (CasedBaseVowel::lower(BaseVowel::ACircumflex), Tone::Dot),
    // 0x1EAE - 0x1EAF (Ắ, ắ)
    (CasedBaseVowel::upper(BaseVowel::ABreve), Tone::Acute),
    (CasedBaseVowel::lower(BaseVowel::ABreve), Tone::Acute),
    // 0x1EB0 - 0x1EB1 (Ằ, ằ)
    (CasedBaseVowel::upper(BaseVowel::ABreve), Tone::Grave),
    (CasedBaseVowel::lower(BaseVowel::ABreve), Tone::Grave),
    // 0x1EB2 - 0x1EB3 (Ẳ, ẳ)
    (CasedBaseVowel::upper(BaseVowel::ABreve), Tone::Hook),
    (CasedBaseVowel::lower(BaseVowel::ABreve), Tone::Hook),
    // 0x1EB4 - 0x1EB5 (Ẵ, ẵ)
    (CasedBaseVowel::upper(BaseVowel::ABreve), Tone::Tilde),
    (CasedBaseVowel::lower(BaseVowel::ABreve), Tone::Tilde),
    // 0x1EB6 - 0x1EB7 (Ặ, ặ)
    (CasedBaseVowel::upper(BaseVowel::ABreve), Tone::Dot),
    (CasedBaseVowel::lower(BaseVowel::ABreve), Tone::Dot),
    // 0x1EB8 - 0x1EB9 (Ẹ, ẹ)
    (CasedBaseVowel::upper(BaseVowel::E), Tone::Dot),
    (CasedBaseVowel::lower(BaseVowel::E), Tone::Dot),
    // 0x1EBA - 0x1EBB (Ẻ, ẻ)
    (CasedBaseVowel::upper(BaseVowel::E), Tone::Hook),
    (CasedBaseVowel::lower(BaseVowel::E), Tone::Hook),
    // 0x1EBC - 0x1EBD (Ẽ, ẽ)
    (CasedBaseVowel::upper(BaseVowel::E), Tone::Tilde),
    (CasedBaseVowel::lower(BaseVowel::E), Tone::Tilde),
    // 0x1EBE - 0x1EBF (Ế, ế)
    (CasedBaseVowel::upper(BaseVowel::ECircumflex), Tone::Acute),
    (CasedBaseVowel::lower(BaseVowel::ECircumflex), Tone::Acute),
    // 0x1EC0 - 0x1EC1 (Ề, ề)
    (CasedBaseVowel::upper(BaseVowel::ECircumflex), Tone::Grave),
    (CasedBaseVowel::lower(BaseVowel::ECircumflex), Tone::Grave),
    // 0x1EC2 - 0x1EC3 (Ể, ể)
    (CasedBaseVowel::upper(BaseVowel::ECircumflex), Tone::Hook),
    (CasedBaseVowel::lower(BaseVowel::ECircumflex), Tone::Hook),
    // 0x1EC4 - 0x1EC5 (Ễ, ễ)
    (CasedBaseVowel::upper(BaseVowel::ECircumflex), Tone::Tilde),
    (CasedBaseVowel::lower(BaseVowel::ECircumflex), Tone::Tilde),
    // 0x1EC6 - 0x1EC7 (Ệ, ệ)
    (CasedBaseVowel::upper(BaseVowel::ECircumflex), Tone::Dot),
    (CasedBaseVowel::lower(BaseVowel::ECircumflex), Tone::Dot),
    // 0x1EC8 - 0x1EC9 (Ỉ, ỉ)
    (CasedBaseVowel::upper(BaseVowel::I), Tone::Hook),
    (CasedBaseVowel::lower(BaseVowel::I), Tone::Hook),
    // 0x1ECA - 0x1ECB (Ị, ị)
    (CasedBaseVowel::upper(BaseVowel::I), Tone::Dot),
    (CasedBaseVowel::lower(BaseVowel::I), Tone::Dot),
    // 0x1ECC - 0x1ECD (Ọ, ọ)
    (CasedBaseVowel::upper(BaseVowel::O), Tone::Dot),
    (CasedBaseVowel::lower(BaseVowel::O), Tone::Dot),
    // 0x1ECE - 0x1ECF (Ỏ, ỏ)
    (CasedBaseVowel::upper(BaseVowel::O), Tone::Hook),
    (CasedBaseVowel::lower(BaseVowel::O), Tone::Hook),
    // 0x1ED0 - 0x1ED1 (Ố, ố)
    (CasedBaseVowel::upper(BaseVowel::OCircumflex), Tone::Acute),
    (CasedBaseVowel::lower(BaseVowel::OCircumflex), Tone::Acute),
    // 0x1ED2 - 0x1ED3 (Ồ, ồ)
    (CasedBaseVowel::upper(BaseVowel::OCircumflex), Tone::Grave),
    (CasedBaseVowel::lower(BaseVowel::OCircumflex), Tone::Grave),
    // 0x1ED4 - 0x1ED5 (Ổ, ổ)
    (CasedBaseVowel::upper(BaseVowel::OCircumflex), Tone::Hook),
    (CasedBaseVowel::lower(BaseVowel::OCircumflex), Tone::Hook),
    // 0x1ED6 - 0x1ED7 (Ỗ, ỗ)
    (CasedBaseVowel::upper(BaseVowel::OCircumflex), Tone::Tilde),
    (CasedBaseVowel::lower(BaseVowel::OCircumflex), Tone::Tilde),
    // 0x1ED8 - 0x1ED9 (Ộ, ộ)
    (CasedBaseVowel::upper(BaseVowel::OCircumflex), Tone::Dot),
    (CasedBaseVowel::lower(BaseVowel::OCircumflex), Tone::Dot),
    // 0x1EDA - 0x1EDB (Ớ, ớ)
    (CasedBaseVowel::upper(BaseVowel::OHorn), Tone::Acute),
    (CasedBaseVowel::lower(BaseVowel::OHorn), Tone::Acute),
    // 0x1EDC - 0x1EDD (Ờ, ờ)
    (CasedBaseVowel::upper(BaseVowel::OHorn), Tone::Grave),
    (CasedBaseVowel::lower(BaseVowel::OHorn), Tone::Grave),
    // 0x1EDE - 0x1EDF (Ở, ở)
    (CasedBaseVowel::upper(BaseVowel::OHorn), Tone::Hook),
    (CasedBaseVowel::lower(BaseVowel::OHorn), Tone::Hook),
    // 0x1EE0 - 0x1EE1 (Ỡ, ỡ)
    (CasedBaseVowel::upper(BaseVowel::OHorn), Tone::Tilde),
    (CasedBaseVowel::lower(BaseVowel::OHorn), Tone::Tilde),
    // 0x1EE2 - 0x1EE3 (Ợ, ợ)
    (CasedBaseVowel::upper(BaseVowel::OHorn), Tone::Dot),
    (CasedBaseVowel::lower(BaseVowel::OHorn), Tone::Dot),
    // 0x1EE4 - 0x1EE5 (Ụ, ụ)
    (CasedBaseVowel::upper(BaseVowel::U), Tone::Dot),
    (CasedBaseVowel::lower(BaseVowel::U), Tone::Dot),
    // 0x1EE6 - 0x1EE7 (Ủ, ủ)
    (CasedBaseVowel::upper(BaseVowel::U), Tone::Hook),
    (CasedBaseVowel::lower(BaseVowel::U), Tone::Hook),
    // 0x1EE8 - 0x1EE9 (Ứ, ứ)
    (CasedBaseVowel::upper(BaseVowel::UHorn), Tone::Acute),
    (CasedBaseVowel::lower(BaseVowel::UHorn), Tone::Acute),
    // 0x1EEA - 0x1EEB (Ừ, ừ)
    (CasedBaseVowel::upper(BaseVowel::UHorn), Tone::Grave),
    (CasedBaseVowel::lower(BaseVowel::UHorn), Tone::Grave),
    // 0x1EEC - 0x1EED (Ử, sử)
    (CasedBaseVowel::upper(BaseVowel::UHorn), Tone::Hook),
    (CasedBaseVowel::lower(BaseVowel::UHorn), Tone::Hook),
    // 0x1EEE - 0x1EEF (Ữ, ữ)
    (CasedBaseVowel::upper(BaseVowel::UHorn), Tone::Tilde),
    (CasedBaseVowel::lower(BaseVowel::UHorn), Tone::Tilde),
    // 0x1EF0 - 0x1EF1 (Ự, ự)
    (CasedBaseVowel::upper(BaseVowel::UHorn), Tone::Dot),
    (CasedBaseVowel::lower(BaseVowel::UHorn), Tone::Dot),
    // 0x1EF2 - 0x1EF3 (Ỳ, ỳ)
    (CasedBaseVowel::upper(BaseVowel::Y), Tone::Grave),
    (CasedBaseVowel::lower(BaseVowel::Y), Tone::Grave),
    // 0x1EF4 - 0x1EF5 (Ỵ, ỵ)
    (CasedBaseVowel::upper(BaseVowel::Y), Tone::Dot),
    (CasedBaseVowel::lower(BaseVowel::Y), Tone::Dot),
    // 0x1EF6 - 0x1EF7 (Ỷ, ỷ)
    (CasedBaseVowel::upper(BaseVowel::Y), Tone::Hook),
    (CasedBaseVowel::lower(BaseVowel::Y), Tone::Hook),
    // 0x1EF8 - 0x1EF9 (Ỹ, ỹ)
    (CasedBaseVowel::upper(BaseVowel::Y), Tone::Tilde),
    (CasedBaseVowel::lower(BaseVowel::Y), Tone::Tilde),
];

/// Decodes a precomposed Vietnamese vowel character into a `(BaseVowelCased,
/// Tone)` pair.
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
pub const fn decode_vowel(character: char) -> Option<(CasedBaseVowel, Tone)> {
    let code = character as u32;

    match code {
        // 1. ASCII Block (Fast path - Keystrokes)
        0x00..=0x7F => match character {
            'a' => Some((CasedBaseVowel::lower(BaseVowel::A), Tone::Flat)),
            'A' => Some((CasedBaseVowel::upper(BaseVowel::A), Tone::Flat)),
            'o' => Some((CasedBaseVowel::lower(BaseVowel::O), Tone::Flat)),
            'O' => Some((CasedBaseVowel::upper(BaseVowel::O), Tone::Flat)),
            'e' => Some((CasedBaseVowel::lower(BaseVowel::E), Tone::Flat)),
            'E' => Some((CasedBaseVowel::upper(BaseVowel::E), Tone::Flat)),
            'i' => Some((CasedBaseVowel::lower(BaseVowel::I), Tone::Flat)),
            'I' => Some((CasedBaseVowel::upper(BaseVowel::I), Tone::Flat)),
            'u' => Some((CasedBaseVowel::lower(BaseVowel::U), Tone::Flat)),
            'U' => Some((CasedBaseVowel::upper(BaseVowel::U), Tone::Flat)),
            'y' => Some((CasedBaseVowel::lower(BaseVowel::Y), Tone::Flat)),
            'Y' => Some((CasedBaseVowel::upper(BaseVowel::Y), Tone::Flat)),
            _ => None,
        },

        // 2. Latin-1 Supplement (U+00C0..U+00FF)
        0x80..=0xFF => match character {
            'ê' => Some((CasedBaseVowel::lower(BaseVowel::ECircumflex), Tone::Flat)),
            'Ê' => Some((CasedBaseVowel::upper(BaseVowel::ECircumflex), Tone::Flat)),
            'ô' => Some((CasedBaseVowel::lower(BaseVowel::OCircumflex), Tone::Flat)),
            'Ô' => Some((CasedBaseVowel::upper(BaseVowel::OCircumflex), Tone::Flat)),
            'â' => Some((CasedBaseVowel::lower(BaseVowel::ACircumflex), Tone::Flat)),
            'Â' => Some((CasedBaseVowel::upper(BaseVowel::ACircumflex), Tone::Flat)),
            'á' => Some((CasedBaseVowel::lower(BaseVowel::A), Tone::Acute)),
            'Á' => Some((CasedBaseVowel::upper(BaseVowel::A), Tone::Acute)),
            'à' => Some((CasedBaseVowel::lower(BaseVowel::A), Tone::Grave)),
            'À' => Some((CasedBaseVowel::upper(BaseVowel::A), Tone::Grave)),
            'ã' => Some((CasedBaseVowel::lower(BaseVowel::A), Tone::Tilde)),
            'Ã' => Some((CasedBaseVowel::upper(BaseVowel::A), Tone::Tilde)),
            'ó' => Some((CasedBaseVowel::lower(BaseVowel::O), Tone::Acute)),
            'Ó' => Some((CasedBaseVowel::upper(BaseVowel::O), Tone::Acute)),
            'ò' => Some((CasedBaseVowel::lower(BaseVowel::O), Tone::Grave)),
            'Ò' => Some((CasedBaseVowel::upper(BaseVowel::O), Tone::Grave)),
            'õ' => Some((CasedBaseVowel::lower(BaseVowel::O), Tone::Tilde)),
            'Õ' => Some((CasedBaseVowel::upper(BaseVowel::O), Tone::Tilde)),
            'é' => Some((CasedBaseVowel::lower(BaseVowel::E), Tone::Acute)),
            'É' => Some((CasedBaseVowel::upper(BaseVowel::E), Tone::Acute)),
            'è' => Some((CasedBaseVowel::lower(BaseVowel::E), Tone::Grave)),
            'È' => Some((CasedBaseVowel::upper(BaseVowel::E), Tone::Grave)),
            'í' => Some((CasedBaseVowel::lower(BaseVowel::I), Tone::Acute)),
            'Í' => Some((CasedBaseVowel::upper(BaseVowel::I), Tone::Acute)),
            'ì' => Some((CasedBaseVowel::lower(BaseVowel::I), Tone::Grave)),
            'Ì' => Some((CasedBaseVowel::upper(BaseVowel::I), Tone::Grave)),
            'ú' => Some((CasedBaseVowel::lower(BaseVowel::U), Tone::Acute)),
            'Ú' => Some((CasedBaseVowel::upper(BaseVowel::U), Tone::Acute)),
            'ù' => Some((CasedBaseVowel::lower(BaseVowel::U), Tone::Grave)),
            'Ù' => Some((CasedBaseVowel::upper(BaseVowel::U), Tone::Grave)),
            'ý' => Some((CasedBaseVowel::lower(BaseVowel::Y), Tone::Acute)),
            'Ý' => Some((CasedBaseVowel::upper(BaseVowel::Y), Tone::Acute)),
            _ => None,
        },

        // 3. Latin Extended
        0x0100..=0x01B0 => match character {
            'ơ' => Some((CasedBaseVowel::lower(BaseVowel::OHorn), Tone::Flat)),
            'Ơ' => Some((CasedBaseVowel::upper(BaseVowel::OHorn), Tone::Flat)),
            'ă' => Some((CasedBaseVowel::lower(BaseVowel::ABreve), Tone::Flat)),
            'Ă' => Some((CasedBaseVowel::upper(BaseVowel::ABreve), Tone::Flat)),
            'ư' => Some((CasedBaseVowel::lower(BaseVowel::UHorn), Tone::Flat)),
            'Ư' => Some((CasedBaseVowel::upper(BaseVowel::UHorn), Tone::Flat)),
            'ĩ' => Some((CasedBaseVowel::lower(BaseVowel::I), Tone::Tilde)),
            'Ĩ' => Some((CasedBaseVowel::upper(BaseVowel::I), Tone::Tilde)),
            'ũ' => Some((CasedBaseVowel::lower(BaseVowel::U), Tone::Tilde)),
            'Ũ' => Some((CasedBaseVowel::upper(BaseVowel::U), Tone::Tilde)),
            _ => None,
        },

        // 4. Vietnamese block (U+1EA0..U+1EF9) -> Direct Indexing Table!
        0x1EA0..=0x1EF9 => {
            let offset = (code - 0x1EA0) as usize;
            let (cased, tone) = DECODED_VIETNAMESE_BLOCK_LUT[offset];
            Some((cased, tone))
        }

        _ => None,
    }
}

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
