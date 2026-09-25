//! Vietnamese vowel codec: the 12 base vowels, their diacritic shapes, tones,
//! case, and the precomposed-character encode/decode pair
//! ([`encode_vowel`] / [`decode_vowel`]).
//!
//! [`BaseVowel`] packs `(Priority ID | Shape | Root)` into a `u16` discriminant;
//! [`ExtendedBaseVowel`] wraps it with a case flag and reserved extension bits.
//! [`decode_vowel`] splits a precomposed character back into those parts.

/// Base ASCII vowel letter independent of shape, tone, and case.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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

impl Shape {
    /// Returns `true` if the shape is a real diacritic (Horn, Circumflex,
    /// Breve) rather than `None`.
    #[inline(always)]
    pub const fn is_some(self) -> bool {
        !matches!(self, Shape::None)
    }
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

    #[inline(always)]
    pub const fn is_some(self) -> bool {
        !matches!(self, Self::Flat)
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
    // Priority 0..=2: closed vowels (lowest).
    Y           = (0 << 5)  | ((Shape::None as u16) << 3)        | RootVowel::Y as u16,
    U           = (1 << 5)  | ((Shape::None as u16) << 3)        | RootVowel::U as u16,
    I           = (2 << 5)  | ((Shape::None as u16) << 3)        | RootVowel::I as u16,

    // Priority 3..=5: open plain vowels (e, o, a).
    E           = (3 << 5)  | ((Shape::None as u16) << 3)        | RootVowel::E as u16,
    O           = (4 << 5)  | ((Shape::None as u16) << 3)        | RootVowel::O as u16,
    A           = (5 << 5)  | ((Shape::None as u16) << 3)        | RootVowel::A as u16,

    // Priority 6..=9: vowels with a diacritic (ư, â, ô, ă).
    UHorn       = (6 << 5)  | ((Shape::Horn as u16) << 3)        | RootVowel::U as u16,
    ACircumflex = (7 << 5)  | ((Shape::Circumflex as u16) << 3)  | RootVowel::A as u16,
    OCircumflex = (8 << 5)  | ((Shape::Circumflex as u16) << 3)  | RootVowel::O as u16,
    ABreve      = (9 << 5)  | ((Shape::Breve as u16) << 3)       | RootVowel::A as u16,

    // Priority 10..=11: highest (ê, ơ).
    // The tone always lands on ơ / ê, e.g. ươ, ơi, iê, uê.
    ECircumflex = (10 << 5) | ((Shape::Circumflex as u16) << 3)  | RootVowel::E as u16,
    OHorn       = (11 << 5) | ((Shape::Horn as u16) << 3)        | RootVowel::O as u16,
}

impl BaseVowel {
    // ─────────────── Cardinality ───────────────
    pub const COUNT: usize = 12;

    // ─────────────── Bit-field layout ───────────────
    // Packed `u16`: [Reserved | Priority ID | Shape | Root], matching the
    // enum discriminants: the root takes 3 bits, the shape 2.

    // ── field widths ──
    /// Width of the [`RootVowel`] field — 3 bits cover `RootVowel::A..=Y` (0..=5).
    const ROOT_WIDTH: u32 = 3;
    /// Width of the [`Shape`] field — 2 bits cover `Shape::None..=Horn` (0..=3).
    const SHAPE_WIDTH: u32 = 2;

    const ID_WIDTH: u32 = 4;

    // ── field shifts ──
    /// Shift of the [`Shape`] field (sits directly above the root field).
    const SHAPE_OFFSET: u32 = Self::ROOT_WIDTH;
    /// Shift of the Priority ID field (sits directly above the shape field).
    const ID_OFFSET: u32 = Self::SHAPE_OFFSET + Self::SHAPE_WIDTH;
    /// First bit available after the BaseVowel bit-field.
    const NEXT_OFFSET: u32 = Self::ID_OFFSET + Self::ID_WIDTH;

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
        if id < Self::COUNT {
            Ok(Self::VARIANTS_BY_ID[id])
        } else {
            Err(())
        }
    }

    #[inline(always)]
    pub unsafe fn from_id_unchecked(id: usize) -> Self {
        debug_assert!(id < Self::COUNT);
        *Self::VARIANTS_BY_ID.get_unchecked(id)
    }

    /// The [`BaseVowel`] for a root letter and shape, or `Err` for a
    /// combination Vietnamese has no letter for.
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

    #[inline(always)]
    pub const fn has_shape(self, shape: Shape) -> bool {
        self.shape() as u8 == shape as u8
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ExtendedBaseVowel(u16);

impl ExtendedBaseVowel {
    // Bits 0..=8 are the BaseVowel value; bit 9 is the case flag (Upper);
    // bits 10..=15 are reserved for future extension flags.
    const BASE_MASK: u16 = (1 << BaseVowel::NEXT_OFFSET) - 1;
    const CASE_OFFSET: u32 = BaseVowel::NEXT_OFFSET;
    const CASE_MASK: u16 = 1 << Self::CASE_OFFSET;

    // ─────────────── Construction ───────────────
    /// Builds an [`ExtendedBaseVowel`] carrying only the base value; the case
    /// bit is left clear (the bare discriminant already fits below it).
    #[inline(always)]
    pub const fn new(value: BaseVowel) -> Self {
        Self(value as u16)
    }

    /// Builds a lowercase [`ExtendedBaseVowel`]: the value as-is, no masking.
    #[inline(always)]
    pub const fn lower(value: BaseVowel) -> Self {
        Self(value as u16)
    }

    /// Builds an uppercase [`ExtendedBaseVowel`] by setting the case bit.
    #[inline(always)]
    pub const fn upper(value: BaseVowel) -> Self {
        Self(value as u16 | Self::CASE_MASK)
    }

    /// Builds from a base value and an explicit case flag.
    #[inline(always)]
    pub const fn with_case(value: BaseVowel, upper: bool) -> Self {
        Self(value as u16 | ((upper as u16) << Self::CASE_OFFSET))
    }

    // ─────────────── Accessors ───────────────
    #[inline(always)]
    pub const fn get(self) -> BaseVowel {
        // SAFETY: bits below CASE_OFFSET contain a valid BaseVowel.
        unsafe { core::mem::transmute(self.0 & Self::BASE_MASK) }
    }

    #[inline(always)]
    pub const fn set(&mut self, value: BaseVowel) {
        // Preserve every extension bit above the base region (case + reserved).
        self.0 = (self.0 & !Self::BASE_MASK) | value as u16;
    }

    // ─────────────── Case flag ───────────────
    #[inline(always)]
    pub const fn is_upper(self) -> bool {
        self.0 & Self::CASE_MASK != 0
    }

    #[inline(always)]
    pub const fn set_case(&mut self, upper: bool) {
        self.0 = (self.0 & !Self::CASE_MASK) | ((upper as u16) * Self::CASE_MASK);
    }

    // ─────────────── Render ───────────────
    #[inline(always)]
    pub const fn to_char(self) -> char {
        encode_vowel(self.get(), Tone::Flat, self.is_upper())
    }

    #[inline(always)]
    pub const fn to_char_tone(self, tone: Tone) -> char {
        encode_vowel(self.get(), tone, self.is_upper())
    }
}

/// Encodes a `(base, tone, uppercase)` triple as a precomposed character.
///
/// Index = `(base.id() * 6 + tone) * 2 + uppercase`; the bounds guarantee
/// `0..=143`, so the lookup can't go out of range.
#[inline(always)]
pub const fn encode_vowel(base: BaseVowel, tone: Tone, uppercase: bool) -> char {
    // All 144 precomposed Vietnamese vowel characters, one 12-entry block per
    // base vowel in priority-ID order; within a block the 6 tones run in
    // Lower/Upper order: `(base.id() * 6 + tone) * 2 + uppercase`.
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

    // Compute index in a packed 144-element lookup table: (base * 6 + tone) * 2 + uppercase.
    let base_id = base.id();
    let idx = ((base_id * 6 + tone as usize) << 1) | (uppercase as usize);
    ENCODED_VOWELS[idx]
}

/// Decodes a precomposed Vietnamese vowel into a `(ExtendedBaseVowel, Tone)` pair,
/// or `None` if `character` is not a vowel.
///
/// Fastest measured variant: four non-overlapping code-point regions keep the
/// branch tree small — ASCII (match), Latin-1, Latin Extended (match), and the
/// Vietnamese block (U+1EA0..=U+1EF9) via O(1) direct LUT indexing.
#[inline(always)]
pub const fn decode_vowel(character: char) -> Option<(ExtendedBaseVowel, Tone)> {
    use BaseVowel::*;
    use Tone::*;

    let code = character as u32;

    match code {
        // 1. ASCII Block (Fast path - Keystrokes)
        0x00..=0x7F => match character {
            'a' => Some((ExtendedBaseVowel::lower(A), Flat)),
            'A' => Some((ExtendedBaseVowel::upper(A), Flat)),
            'o' => Some((ExtendedBaseVowel::lower(O), Flat)),
            'O' => Some((ExtendedBaseVowel::upper(O), Flat)),
            'e' => Some((ExtendedBaseVowel::lower(E), Flat)),
            'E' => Some((ExtendedBaseVowel::upper(E), Flat)),
            'i' => Some((ExtendedBaseVowel::lower(I), Flat)),
            'I' => Some((ExtendedBaseVowel::upper(I), Flat)),
            'u' => Some((ExtendedBaseVowel::lower(U), Flat)),
            'U' => Some((ExtendedBaseVowel::upper(U), Flat)),
            'y' => Some((ExtendedBaseVowel::lower(Y), Flat)),
            'Y' => Some((ExtendedBaseVowel::upper(Y), Flat)),
            _ => None,
        },

        // 2. Latin-1 Supplement (U+00C0..U+00FF)
        0x80..=0xFF => match character {
            'ê' => Some((ExtendedBaseVowel::lower(ECircumflex), Flat)),
            'Ê' => Some((ExtendedBaseVowel::upper(ECircumflex), Flat)),
            'ô' => Some((ExtendedBaseVowel::lower(OCircumflex), Flat)),
            'Ô' => Some((ExtendedBaseVowel::upper(OCircumflex), Flat)),
            'â' => Some((ExtendedBaseVowel::lower(ACircumflex), Flat)),
            'Â' => Some((ExtendedBaseVowel::upper(ACircumflex), Flat)),
            'á' => Some((ExtendedBaseVowel::lower(A), Acute)),
            'Á' => Some((ExtendedBaseVowel::upper(A), Acute)),
            'à' => Some((ExtendedBaseVowel::lower(A), Grave)),
            'À' => Some((ExtendedBaseVowel::upper(A), Grave)),
            'ã' => Some((ExtendedBaseVowel::lower(A), Tilde)),
            'Ã' => Some((ExtendedBaseVowel::upper(A), Tilde)),
            'ó' => Some((ExtendedBaseVowel::lower(O), Acute)),
            'Ó' => Some((ExtendedBaseVowel::upper(O), Acute)),
            'ò' => Some((ExtendedBaseVowel::lower(O), Grave)),
            'Ò' => Some((ExtendedBaseVowel::upper(O), Grave)),
            'õ' => Some((ExtendedBaseVowel::lower(O), Tilde)),
            'Õ' => Some((ExtendedBaseVowel::upper(O), Tilde)),
            'é' => Some((ExtendedBaseVowel::lower(E), Acute)),
            'É' => Some((ExtendedBaseVowel::upper(E), Acute)),
            'è' => Some((ExtendedBaseVowel::lower(E), Grave)),
            'È' => Some((ExtendedBaseVowel::upper(E), Grave)),
            'í' => Some((ExtendedBaseVowel::lower(I), Acute)),
            'Í' => Some((ExtendedBaseVowel::upper(I), Acute)),
            'ì' => Some((ExtendedBaseVowel::lower(I), Grave)),
            'Ì' => Some((ExtendedBaseVowel::upper(I), Grave)),
            'ú' => Some((ExtendedBaseVowel::lower(U), Acute)),
            'Ú' => Some((ExtendedBaseVowel::upper(U), Acute)),
            'ù' => Some((ExtendedBaseVowel::lower(U), Grave)),
            'Ù' => Some((ExtendedBaseVowel::upper(U), Grave)),
            'ý' => Some((ExtendedBaseVowel::lower(Y), Acute)),
            'Ý' => Some((ExtendedBaseVowel::upper(Y), Acute)),
            _ => None,
        },

        // 3. Latin Extended
        0x0100..=0x01B0 => match character {
            'ơ' => Some((ExtendedBaseVowel::lower(OHorn), Flat)),
            'Ơ' => Some((ExtendedBaseVowel::upper(OHorn), Flat)),
            'ă' => Some((ExtendedBaseVowel::lower(ABreve), Flat)),
            'Ă' => Some((ExtendedBaseVowel::upper(ABreve), Flat)),
            'ư' => Some((ExtendedBaseVowel::lower(UHorn), Flat)),
            'Ư' => Some((ExtendedBaseVowel::upper(UHorn), Flat)),
            'ĩ' => Some((ExtendedBaseVowel::lower(I), Tilde)),
            'Ĩ' => Some((ExtendedBaseVowel::upper(I), Tilde)),
            'ũ' => Some((ExtendedBaseVowel::lower(U), Tilde)),
            'Ũ' => Some((ExtendedBaseVowel::upper(U), Tilde)),
            _ => None,
        },

        // 4. Vietnamese block (U+1EA0..U+1EF9) -> Direct Indexing Table!
        0x1EA0..=0x1EF9 => {
            /// Direct lookup table (LUT) for the precomposed Vietnamese block (U+1EA0..=U+1EF9).
            /// Index = `(code - 0x1EA0) as usize`, giving O(1) `(ExtendedBaseVowel, Tone)` lookup.
            const DECODED_VIETNAMESE_BLOCK_LUT: [(ExtendedBaseVowel, Tone); 90] = [
                // 0x1EA0 - 0x1EA1 (Ạ, ạ)
                (ExtendedBaseVowel::upper(A), Dot),
                (ExtendedBaseVowel::lower(A), Dot),
                // 0x1EA2 - 0x1EA3 (Ả, ả)
                (ExtendedBaseVowel::upper(A), Hook),
                (ExtendedBaseVowel::lower(A), Hook),
                // 0x1EA4 - 0x1EA5 (Ấ, ấ)
                (ExtendedBaseVowel::upper(ACircumflex), Acute),
                (ExtendedBaseVowel::lower(ACircumflex), Acute),
                // 0x1EA6 - 0x1EA7 (Ầ, ầ)
                (ExtendedBaseVowel::upper(ACircumflex), Grave),
                (ExtendedBaseVowel::lower(ACircumflex), Grave),
                // 0x1EA8 - 0x1EA9 (Ẩ, ẩ)
                (ExtendedBaseVowel::upper(ACircumflex), Hook),
                (ExtendedBaseVowel::lower(ACircumflex), Hook),
                // 0x1EAA - 0x1EAB (Ẫ, ẫ)
                (ExtendedBaseVowel::upper(ACircumflex), Tilde),
                (ExtendedBaseVowel::lower(ACircumflex), Tilde),
                // 0x1EAC - 0x1EAD (Ậ, ậ)
                (ExtendedBaseVowel::upper(ACircumflex), Dot),
                (ExtendedBaseVowel::lower(ACircumflex), Dot),
                // 0x1EAE - 0x1EAF (Ắ, ắ)
                (ExtendedBaseVowel::upper(ABreve), Acute),
                (ExtendedBaseVowel::lower(ABreve), Acute),
                // 0x1EB0 - 0x1EB1 (Ằ, ằ)
                (ExtendedBaseVowel::upper(ABreve), Grave),
                (ExtendedBaseVowel::lower(ABreve), Grave),
                // 0x1EB2 - 0x1EB3 (Ẳ, ẳ)
                (ExtendedBaseVowel::upper(ABreve), Hook),
                (ExtendedBaseVowel::lower(ABreve), Hook),
                // 0x1EB4 - 0x1EB5 (Ẵ, ẵ)
                (ExtendedBaseVowel::upper(ABreve), Tilde),
                (ExtendedBaseVowel::lower(ABreve), Tilde),
                // 0x1EB6 - 0x1EB7 (Ặ, ặ)
                (ExtendedBaseVowel::upper(ABreve), Dot),
                (ExtendedBaseVowel::lower(ABreve), Dot),
                // 0x1EB8 - 0x1EB9 (Ẹ, ẹ)
                (ExtendedBaseVowel::upper(E), Dot),
                (ExtendedBaseVowel::lower(E), Dot),
                // 0x1EBA - 0x1EBB (Ẻ, ẻ)
                (ExtendedBaseVowel::upper(E), Hook),
                (ExtendedBaseVowel::lower(E), Hook),
                // 0x1EBC - 0x1EBD (Ẽ, ẽ)
                (ExtendedBaseVowel::upper(E), Tilde),
                (ExtendedBaseVowel::lower(E), Tilde),
                // 0x1EBE - 0x1EBF (Ế, ế)
                (ExtendedBaseVowel::upper(ECircumflex), Acute),
                (ExtendedBaseVowel::lower(ECircumflex), Acute),
                // 0x1EC0 - 0x1EC1 (Ề, ề)
                (ExtendedBaseVowel::upper(ECircumflex), Grave),
                (ExtendedBaseVowel::lower(ECircumflex), Grave),
                // 0x1EC2 - 0x1EC3 (Ể, ể)
                (ExtendedBaseVowel::upper(ECircumflex), Hook),
                (ExtendedBaseVowel::lower(ECircumflex), Hook),
                // 0x1EC4 - 0x1EC5 (Ễ, ễ)
                (ExtendedBaseVowel::upper(ECircumflex), Tilde),
                (ExtendedBaseVowel::lower(ECircumflex), Tilde),
                // 0x1EC6 - 0x1EC7 (Ệ, ệ)
                (ExtendedBaseVowel::upper(ECircumflex), Dot),
                (ExtendedBaseVowel::lower(ECircumflex), Dot),
                // 0x1EC8 - 0x1EC9 (Ỉ, ỉ)
                (ExtendedBaseVowel::upper(I), Hook),
                (ExtendedBaseVowel::lower(I), Hook),
                // 0x1ECA - 0x1ECB (Ị, ị)
                (ExtendedBaseVowel::upper(I), Dot),
                (ExtendedBaseVowel::lower(I), Dot),
                // 0x1ECC - 0x1ECD (Ọ, ọ)
                (ExtendedBaseVowel::upper(O), Dot),
                (ExtendedBaseVowel::lower(O), Dot),
                // 0x1ECE - 0x1ECF (Ỏ, ỏ)
                (ExtendedBaseVowel::upper(O), Hook),
                (ExtendedBaseVowel::lower(O), Hook),
                // 0x1ED0 - 0x1ED1 (Ố, ố)
                (ExtendedBaseVowel::upper(OCircumflex), Acute),
                (ExtendedBaseVowel::lower(OCircumflex), Acute),
                // 0x1ED2 - 0x1ED3 (Ồ, ồ)
                (ExtendedBaseVowel::upper(OCircumflex), Grave),
                (ExtendedBaseVowel::lower(OCircumflex), Grave),
                // 0x1ED4 - 0x1ED5 (Ổ, ổ)
                (ExtendedBaseVowel::upper(OCircumflex), Hook),
                (ExtendedBaseVowel::lower(OCircumflex), Hook),
                // 0x1ED6 - 0x1ED7 (Ỗ, ỗ)
                (ExtendedBaseVowel::upper(OCircumflex), Tilde),
                (ExtendedBaseVowel::lower(OCircumflex), Tilde),
                // 0x1ED8 - 0x1ED9 (Ộ, ộ)
                (ExtendedBaseVowel::upper(OCircumflex), Dot),
                (ExtendedBaseVowel::lower(OCircumflex), Dot),
                // 0x1EDA - 0x1EDB (Ớ, ớ)
                (ExtendedBaseVowel::upper(OHorn), Acute),
                (ExtendedBaseVowel::lower(OHorn), Acute),
                // 0x1EDC - 0x1EDD (Ờ, ờ)
                (ExtendedBaseVowel::upper(OHorn), Grave),
                (ExtendedBaseVowel::lower(OHorn), Grave),
                // 0x1EDE - 0x1EDF (Ở, ở)
                (ExtendedBaseVowel::upper(OHorn), Hook),
                (ExtendedBaseVowel::lower(OHorn), Hook),
                // 0x1EE0 - 0x1EE1 (Ỡ, ỡ)
                (ExtendedBaseVowel::upper(OHorn), Tilde),
                (ExtendedBaseVowel::lower(OHorn), Tilde),
                // 0x1EE2 - 0x1EE3 (Ợ, ợ)
                (ExtendedBaseVowel::upper(OHorn), Dot),
                (ExtendedBaseVowel::lower(OHorn), Dot),
                // 0x1EE4 - 0x1EE5 (Ụ, ụ)
                (ExtendedBaseVowel::upper(U), Dot),
                (ExtendedBaseVowel::lower(U), Dot),
                // 0x1EE6 - 0x1EE7 (Ủ, ủ)
                (ExtendedBaseVowel::upper(U), Hook),
                (ExtendedBaseVowel::lower(U), Hook),
                // 0x1EE8 - 0x1EE9 (Ứ, ứ)
                (ExtendedBaseVowel::upper(UHorn), Acute),
                (ExtendedBaseVowel::lower(UHorn), Acute),
                // 0x1EEA - 0x1EEB (Ừ, ừ)
                (ExtendedBaseVowel::upper(UHorn), Grave),
                (ExtendedBaseVowel::lower(UHorn), Grave),
                // 0x1EEC - 0x1EED (Ử, ử)
                (ExtendedBaseVowel::upper(UHorn), Hook),
                (ExtendedBaseVowel::lower(UHorn), Hook),
                // 0x1EEE - 0x1EEF (Ữ, ữ)
                (ExtendedBaseVowel::upper(UHorn), Tilde),
                (ExtendedBaseVowel::lower(UHorn), Tilde),
                // 0x1EF0 - 0x1EF1 (Ự, ự)
                (ExtendedBaseVowel::upper(UHorn), Dot),
                (ExtendedBaseVowel::lower(UHorn), Dot),
                // 0x1EF2 - 0x1EF3 (Ỳ, ỳ)
                (ExtendedBaseVowel::upper(Y), Grave),
                (ExtendedBaseVowel::lower(Y), Grave),
                // 0x1EF4 - 0x1EF5 (Ỵ, ỵ)
                (ExtendedBaseVowel::upper(Y), Dot),
                (ExtendedBaseVowel::lower(Y), Dot),
                // 0x1EF6 - 0x1EF7 (Ỷ, ỷ)
                (ExtendedBaseVowel::upper(Y), Hook),
                (ExtendedBaseVowel::lower(Y), Hook),
                // 0x1EF8 - 0x1EF9 (Ỹ, ỹ)
                (ExtendedBaseVowel::upper(Y), Tilde),
                (ExtendedBaseVowel::lower(Y), Tilde),
            ];
            let offset = (code - 0x1EA0) as usize;
            Some(DECODED_VIETNAMESE_BLOCK_LUT[offset])
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Canonical list of every [`BaseVowel`], kept in lockstep with the enum.
    const ALL_BASES: [BaseVowel; BaseVowel::COUNT] = BaseVowel::VARIANTS_BY_ID;

    /// Every [`BaseVowel`] discriminant must fit entirely below the case bit,
    /// otherwise `ExtendedBaseVowel::get()`'s mask would silently drop it and the
    /// round-trip would be lossy.
    #[test]
    fn base_vowel_discriminants_fit_below_the_case_bit() {
        for &base in &ALL_BASES {
            let raw = base as u16;
            assert!(
                raw < ExtendedBaseVowel::CASE_MASK,
                "{base:?} (0x{raw:x}) overlaps the case bit at bit {}",
                ExtendedBaseVowel::CASE_OFFSET,
            );
        }
    }

    /// All 24 `(base, case)` combinations must pack into distinct `u16`s, with
    /// the lower form equal to the bare discriminant and the upper form exactly
    /// one case-bit away. Reserved bits above the case bit stay clear.
    #[test]
    fn all_base_case_combinations_pack_distinctly() {
        const PACK_SPACE: usize = 1 << (BaseVowel::NEXT_OFFSET as usize + 1);
        let mut seen = [false; PACK_SPACE];

        for &base in &ALL_BASES {
            let lower = ExtendedBaseVowel::lower(base);
            let upper = ExtendedBaseVowel::upper(base);

            assert_eq!(
                lower.0, base as u16,
                "lower form must be the bare discriminant"
            );
            assert_eq!(
                upper.0,
                base as u16 | ExtendedBaseVowel::CASE_MASK,
                "upper form must set exactly the case bit",
            );

            for cased in [lower, upper] {
                assert_eq!(
                    cased.0 >> (ExtendedBaseVowel::CASE_OFFSET + 1),
                    0,
                    "reserved bits above the case bit are set in {cased:?}",
                );
                assert!(
                    !seen[cased.0 as usize],
                    "duplicate packed value for {cased:?}",
                );
                seen[cased.0 as usize] = true;
            }
        }
    }

    /// Setters must never push the packed value outside the `(base, case)` space.
    #[test]
    fn setters_preserve_packed_bounds() {
        for &base in &ALL_BASES {
            let mut cased = ExtendedBaseVowel::lower(base);

            cased.set_case(true);
            assert_eq!(cased, ExtendedBaseVowel::upper(base));

            cased.set_case(false);
            assert_eq!(cased, ExtendedBaseVowel::lower(base));

            for &replacement in &ALL_BASES {
                cased.set(replacement);
                assert_eq!(cased.get(), replacement);
                assert!(
                    cased.0 < ExtendedBaseVowel::CASE_MASK
                        || cased.0 - ExtendedBaseVowel::CASE_MASK < ExtendedBaseVowel::CASE_MASK,
                    "{replacement:?} pushed the pack out of bounds: 0x{:x}",
                    cased.0,
                );
            }
        }
    }

    // ─────────────── Construction aliases ───────────────
    /// `new`, `lower` and `upper` must be exact aliases of `with_case`.
    #[test]
    fn extended_constructors_are_with_case_aliases() {
        for &base in &ALL_BASES {
            let lower = ExtendedBaseVowel::with_case(base, false);
            let upper = ExtendedBaseVowel::with_case(base, true);

            assert_eq!(ExtendedBaseVowel::new(base), lower);
            assert_eq!(ExtendedBaseVowel::lower(base), lower);
            assert_eq!(ExtendedBaseVowel::upper(base), upper);
        }
    }

    // ─────────────── Reserved extension bits ───────────────
    /// `set` must swap the base while preserving the case flag and any reserved
    /// extension bits above the base region (integration tests cannot exercise
    /// this, since the packed field is private).
    #[test]
    fn set_preserves_reserved_extension_bits() {
        for &base in &ALL_BASES {
            for &replacement in &ALL_BASES {
                for upper in [false, true] {
                    for &reserved in &[1u16 << 10, 1u16 << 12, 1u16 << 15] {
                        let mut cased = ExtendedBaseVowel::with_case(base, upper);
                        cased.0 |= reserved;

                        cased.set(replacement);

                        assert_eq!(cased.get(), replacement, "set must replace the base");
                        assert_eq!(cased.is_upper(), upper, "set must not touch the case bit");
                        assert_ne!(
                            cased.0 & reserved,
                            0,
                            "set must preserve reserved extension bits"
                        );
                    }
                }
            }
        }
    }

    // ─────────────── BaseVowel field round-trips ───────────────
    /// `root()` / `shape()` must be the exact inverse of `from_parts` for every
    /// valid variant, and `id()` the inverse of `from_id`.
    #[test]
    fn base_vowel_fields_round_trip_through_from_parts() {
        for &base in &ALL_BASES {
            assert_eq!(base.id(), (base as u16 >> BaseVowel::ID_OFFSET as usize) as usize);
            assert_eq!(BaseVowel::from_id(base.id()), Ok(base));

            assert_eq!(
                BaseVowel::from_parts(base.root(), base.shape()),
                Ok(base),
                "root/shape extraction must rebuild {base:?}"
            );
        }
    }

    /// `remove_shape` must drop the shape and keep the plain root.
    #[test]
    fn remove_shape_matches_from_parts_with_none() {
        for &base in &ALL_BASES {
            assert_eq!(
                base.remove_shape(),
                BaseVowel::from_parts(base.root(), Shape::None)
                    .expect("a plain root always exists"),
                "remove_shape mismatch for {base:?}"
            );
        }
    }
}
