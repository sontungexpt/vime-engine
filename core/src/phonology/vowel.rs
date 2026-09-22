use super::case::Cased;

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

impl Shape {
    /// Trả về true nếu shape là một dấu thực sự (Horn, Circumflex, Breve), không phải None.
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
        self as u8 != Self::Flat as u8
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
    pub const COUNT: u8 = 12;
    pub const MAX_ID: u8 = Self::COUNT - 1;

    // ─────────────── Bit-field layout ───────────────
    // Packed `u16`: [Reserved | Priority ID | Shape | Root], matching the
    // enum discriminants: the root takes 3 bits, the shape 2.

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

pub type CasedBaseVowel = Cased<BaseVowel>;

impl CasedBaseVowel {
    #[inline(always)]
    pub const fn to_char(self) -> char {
        encode_vowel(self.value, Tone::Flat, self.is_upper)
    }

    #[inline(always)]
    pub const fn to_char_tone(self, tone: Tone) -> char {
        encode_vowel(self.value, tone, self.is_upper)
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
    // Optimized without multiplication (0-byte memory overhead): base * 12 = (base << 3) + (base << 2).

    // Simple: let idx = ((base.id() * 6 + tone as usize) << 1) | (uppercase as usize);
    let base_id = base.id();
    let idx = (base_id << 3) + (base_id << 2) + ((tone as usize) << 1) | (uppercase as usize);
    ENCODED_VOWELS[idx]
}

/// Decodes a precomposed Vietnamese vowel into a `(CasedBaseVowel, Tone)` pair,
/// or `None` if `character` is not a vowel.
///
/// Fastest measured variant: four non-overlapping code-point regions keep the
/// branch tree small — ASCII (match), Latin-1, Latin Extended (match), and the
/// Vietnamese block (U+1EA0..=U+1EF9) via O(1) direct LUT indexing.
#[inline(always)]
pub const fn decode_vowel(character: char) -> Option<(CasedBaseVowel, Tone)> {
    use BaseVowel::*;
    use Tone::*;

    let code = character as u32;

    match code {
        // 1. ASCII Block (Fast path - Keystrokes)
        0x00..=0x7F => match character {
            'a' => Some((CasedBaseVowel::lower(A), Flat)),
            'A' => Some((CasedBaseVowel::upper(A), Flat)),
            'o' => Some((CasedBaseVowel::lower(O), Flat)),
            'O' => Some((CasedBaseVowel::upper(O), Flat)),
            'e' => Some((CasedBaseVowel::lower(E), Flat)),
            'E' => Some((CasedBaseVowel::upper(E), Flat)),
            'i' => Some((CasedBaseVowel::lower(I), Flat)),
            'I' => Some((CasedBaseVowel::upper(I), Flat)),
            'u' => Some((CasedBaseVowel::lower(U), Flat)),
            'U' => Some((CasedBaseVowel::upper(U), Flat)),
            'y' => Some((CasedBaseVowel::lower(Y), Flat)),
            'Y' => Some((CasedBaseVowel::upper(Y), Flat)),
            _ => None,
        },

        // 2. Latin-1 Supplement (U+00C0..U+00FF)
        0x80..=0xFF => match character {
            'ê' => Some((CasedBaseVowel::lower(ECircumflex), Flat)),
            'Ê' => Some((CasedBaseVowel::upper(ECircumflex), Flat)),
            'ô' => Some((CasedBaseVowel::lower(OCircumflex), Flat)),
            'Ô' => Some((CasedBaseVowel::upper(OCircumflex), Flat)),
            'â' => Some((CasedBaseVowel::lower(ACircumflex), Flat)),
            'Â' => Some((CasedBaseVowel::upper(ACircumflex), Flat)),
            'á' => Some((CasedBaseVowel::lower(A), Acute)),
            'Á' => Some((CasedBaseVowel::upper(A), Acute)),
            'à' => Some((CasedBaseVowel::lower(A), Grave)),
            'À' => Some((CasedBaseVowel::upper(A), Grave)),
            'ã' => Some((CasedBaseVowel::lower(A), Tilde)),
            'Ã' => Some((CasedBaseVowel::upper(A), Tilde)),
            'ó' => Some((CasedBaseVowel::lower(O), Acute)),
            'Ó' => Some((CasedBaseVowel::upper(O), Acute)),
            'ò' => Some((CasedBaseVowel::lower(O), Grave)),
            'Ò' => Some((CasedBaseVowel::upper(O), Grave)),
            'õ' => Some((CasedBaseVowel::lower(O), Tilde)),
            'Õ' => Some((CasedBaseVowel::upper(O), Tilde)),
            'é' => Some((CasedBaseVowel::lower(E), Acute)),
            'É' => Some((CasedBaseVowel::upper(E), Acute)),
            'è' => Some((CasedBaseVowel::lower(E), Grave)),
            'È' => Some((CasedBaseVowel::upper(E), Grave)),
            'í' => Some((CasedBaseVowel::lower(I), Acute)),
            'Í' => Some((CasedBaseVowel::upper(I), Acute)),
            'ì' => Some((CasedBaseVowel::lower(I), Grave)),
            'Ì' => Some((CasedBaseVowel::upper(I), Grave)),
            'ú' => Some((CasedBaseVowel::lower(U), Acute)),
            'Ú' => Some((CasedBaseVowel::upper(U), Acute)),
            'ù' => Some((CasedBaseVowel::lower(U), Grave)),
            'Ù' => Some((CasedBaseVowel::upper(U), Grave)),
            'ý' => Some((CasedBaseVowel::lower(Y), Acute)),
            'Ý' => Some((CasedBaseVowel::upper(Y), Acute)),
            _ => None,
        },

        // 3. Latin Extended
        0x0100..=0x01B0 => match character {
            'ơ' => Some((CasedBaseVowel::lower(OHorn), Flat)),
            'Ơ' => Some((CasedBaseVowel::upper(OHorn), Flat)),
            'ă' => Some((CasedBaseVowel::lower(ABreve), Flat)),
            'Ă' => Some((CasedBaseVowel::upper(ABreve), Flat)),
            'ư' => Some((CasedBaseVowel::lower(UHorn), Flat)),
            'Ư' => Some((CasedBaseVowel::upper(UHorn), Flat)),
            'ĩ' => Some((CasedBaseVowel::lower(I), Tilde)),
            'Ĩ' => Some((CasedBaseVowel::upper(I), Tilde)),
            'ũ' => Some((CasedBaseVowel::lower(U), Tilde)),
            'Ũ' => Some((CasedBaseVowel::upper(U), Tilde)),
            _ => None,
        },

        // 4. Vietnamese block (U+1EA0..U+1EF9) -> Direct Indexing Table!
        0x1EA0..=0x1EF9 => {
            /// Direct lookup table (LUT) for the precomposed Vietnamese block (U+1EA0..=U+1EF9).
            /// Index = `(code - 0x1EA0) as usize`, giving O(1) `(CasedBaseVowel, Tone)` lookup.
            const DECODED_VIETNAMESE_BLOCK_LUT: [(CasedBaseVowel, Tone); 90] = [
                // 0x1EA0 - 0x1EA1 (Ạ, ạ)
                (CasedBaseVowel::upper(A), Dot),
                (CasedBaseVowel::lower(A), Dot),
                // 0x1EA2 - 0x1EA3 (Ả, ả)
                (CasedBaseVowel::upper(A), Hook),
                (CasedBaseVowel::lower(A), Hook),
                // 0x1EA4 - 0x1EA5 (Ấ, ấ)
                (CasedBaseVowel::upper(ACircumflex), Acute),
                (CasedBaseVowel::lower(ACircumflex), Acute),
                // 0x1EA6 - 0x1EA7 (Ầ, ầ)
                (CasedBaseVowel::upper(ACircumflex), Grave),
                (CasedBaseVowel::lower(ACircumflex), Grave),
                // 0x1EA8 - 0x1EA9 (Ẩ, ẩ)
                (CasedBaseVowel::upper(ACircumflex), Hook),
                (CasedBaseVowel::lower(ACircumflex), Hook),
                // 0x1EAA - 0x1EAB (Ẫ, ẫ)
                (CasedBaseVowel::upper(ACircumflex), Tilde),
                (CasedBaseVowel::lower(ACircumflex), Tilde),
                // 0x1EAC - 0x1EAD (Ậ, ậ)
                (CasedBaseVowel::upper(ACircumflex), Dot),
                (CasedBaseVowel::lower(ACircumflex), Dot),
                // 0x1EAE - 0x1EAF (Ắ, ắ)
                (CasedBaseVowel::upper(ABreve), Acute),
                (CasedBaseVowel::lower(ABreve), Acute),
                // 0x1EB0 - 0x1EB1 (Ằ, ằ)
                (CasedBaseVowel::upper(ABreve), Grave),
                (CasedBaseVowel::lower(ABreve), Grave),
                // 0x1EB2 - 0x1EB3 (Ẳ, ẳ)
                (CasedBaseVowel::upper(ABreve), Hook),
                (CasedBaseVowel::lower(ABreve), Hook),
                // 0x1EB4 - 0x1EB5 (Ẵ, ẵ)
                (CasedBaseVowel::upper(ABreve), Tilde),
                (CasedBaseVowel::lower(ABreve), Tilde),
                // 0x1EB6 - 0x1EB7 (Ặ, ặ)
                (CasedBaseVowel::upper(ABreve), Dot),
                (CasedBaseVowel::lower(ABreve), Dot),
                // 0x1EB8 - 0x1EB9 (Ẹ, ẹ)
                (CasedBaseVowel::upper(E), Dot),
                (CasedBaseVowel::lower(E), Dot),
                // 0x1EBA - 0x1EBB (Ẻ, ẻ)
                (CasedBaseVowel::upper(E), Hook),
                (CasedBaseVowel::lower(E), Hook),
                // 0x1EBC - 0x1EBD (Ẽ, ẽ)
                (CasedBaseVowel::upper(E), Tilde),
                (CasedBaseVowel::lower(E), Tilde),
                // 0x1EBE - 0x1EBF (Ế, ế)
                (CasedBaseVowel::upper(ECircumflex), Acute),
                (CasedBaseVowel::lower(ECircumflex), Acute),
                // 0x1EC0 - 0x1EC1 (Ề, ề)
                (CasedBaseVowel::upper(ECircumflex), Grave),
                (CasedBaseVowel::lower(ECircumflex), Grave),
                // 0x1EC2 - 0x1EC3 (Ể, ể)
                (CasedBaseVowel::upper(ECircumflex), Hook),
                (CasedBaseVowel::lower(ECircumflex), Hook),
                // 0x1EC4 - 0x1EC5 (Ễ, ễ)
                (CasedBaseVowel::upper(ECircumflex), Tilde),
                (CasedBaseVowel::lower(ECircumflex), Tilde),
                // 0x1EC6 - 0x1EC7 (Ệ, ệ)
                (CasedBaseVowel::upper(ECircumflex), Dot),
                (CasedBaseVowel::lower(ECircumflex), Dot),
                // 0x1EC8 - 0x1EC9 (Ỉ, ỉ)
                (CasedBaseVowel::upper(I), Hook),
                (CasedBaseVowel::lower(I), Hook),
                // 0x1ECA - 0x1ECB (Ị, ị)
                (CasedBaseVowel::upper(I), Dot),
                (CasedBaseVowel::lower(I), Dot),
                // 0x1ECC - 0x1ECD (Ọ, ọ)
                (CasedBaseVowel::upper(O), Dot),
                (CasedBaseVowel::lower(O), Dot),
                // 0x1ECE - 0x1ECF (Ỏ, ỏ)
                (CasedBaseVowel::upper(O), Hook),
                (CasedBaseVowel::lower(O), Hook),
                // 0x1ED0 - 0x1ED1 (Ố, ố)
                (CasedBaseVowel::upper(OCircumflex), Acute),
                (CasedBaseVowel::lower(OCircumflex), Acute),
                // 0x1ED2 - 0x1ED3 (Ồ, ồ)
                (CasedBaseVowel::upper(OCircumflex), Grave),
                (CasedBaseVowel::lower(OCircumflex), Grave),
                // 0x1ED4 - 0x1ED5 (Ổ, ổ)
                (CasedBaseVowel::upper(OCircumflex), Hook),
                (CasedBaseVowel::lower(OCircumflex), Hook),
                // 0x1ED6 - 0x1ED7 (Ỗ, ỗ)
                (CasedBaseVowel::upper(OCircumflex), Tilde),
                (CasedBaseVowel::lower(OCircumflex), Tilde),
                // 0x1ED8 - 0x1ED9 (Ộ, ộ)
                (CasedBaseVowel::upper(OCircumflex), Dot),
                (CasedBaseVowel::lower(OCircumflex), Dot),
                // 0x1EDA - 0x1EDB (Ớ, ớ)
                (CasedBaseVowel::upper(OHorn), Acute),
                (CasedBaseVowel::lower(OHorn), Acute),
                // 0x1EDC - 0x1EDD (Ờ, ờ)
                (CasedBaseVowel::upper(OHorn), Grave),
                (CasedBaseVowel::lower(OHorn), Grave),
                // 0x1EDE - 0x1EDF (Ở, ở)
                (CasedBaseVowel::upper(OHorn), Hook),
                (CasedBaseVowel::lower(OHorn), Hook),
                // 0x1EE0 - 0x1EE1 (Ỡ, ỡ)
                (CasedBaseVowel::upper(OHorn), Tilde),
                (CasedBaseVowel::lower(OHorn), Tilde),
                // 0x1EE2 - 0x1EE3 (Ợ, ợ)
                (CasedBaseVowel::upper(OHorn), Dot),
                (CasedBaseVowel::lower(OHorn), Dot),
                // 0x1EE4 - 0x1EE5 (Ụ, ụ)
                (CasedBaseVowel::upper(U), Dot),
                (CasedBaseVowel::lower(U), Dot),
                // 0x1EE6 - 0x1EE7 (Ủ, ủ)
                (CasedBaseVowel::upper(U), Hook),
                (CasedBaseVowel::lower(U), Hook),
                // 0x1EE8 - 0x1EE9 (Ứ, ứ)
                (CasedBaseVowel::upper(UHorn), Acute),
                (CasedBaseVowel::lower(UHorn), Acute),
                // 0x1EEA - 0x1EEB (Ừ, ừ)
                (CasedBaseVowel::upper(UHorn), Grave),
                (CasedBaseVowel::lower(UHorn), Grave),
                // 0x1EEC - 0x1EED (Ử, ử)
                (CasedBaseVowel::upper(UHorn), Hook),
                (CasedBaseVowel::lower(UHorn), Hook),
                // 0x1EEE - 0x1EEF (Ữ, ữ)
                (CasedBaseVowel::upper(UHorn), Tilde),
                (CasedBaseVowel::lower(UHorn), Tilde),
                // 0x1EF0 - 0x1EF1 (Ự, ự)
                (CasedBaseVowel::upper(UHorn), Dot),
                (CasedBaseVowel::lower(UHorn), Dot),
                // 0x1EF2 - 0x1EF3 (Ỳ, ỳ)
                (CasedBaseVowel::upper(Y), Grave),
                (CasedBaseVowel::lower(Y), Grave),
                // 0x1EF4 - 0x1EF5 (Ỵ, ỵ)
                (CasedBaseVowel::upper(Y), Dot),
                (CasedBaseVowel::lower(Y), Dot),
                // 0x1EF6 - 0x1EF7 (Ỷ, ỷ)
                (CasedBaseVowel::upper(Y), Hook),
                (CasedBaseVowel::lower(Y), Hook),
                // 0x1EF8 - 0x1EF9 (Ỹ, ỹ)
                (CasedBaseVowel::upper(Y), Tilde),
                (CasedBaseVowel::lower(Y), Tilde),
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
