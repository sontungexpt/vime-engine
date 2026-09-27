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

/// A Vietnamese base vowel stored as a packed `u16`.
///
/// The packed value contains three fields:
///
/// - Bits 0–1 store the [`Shape`].
/// - Bits 2–4 store the [`RootVowel`].
/// - Bits 5–8 store the tone-placement ID (`0..=11`). A higher ID has
///   higher tone-placement priority.
/// - Bits 9–15 are reserved and are zero for these vowels.
///
/// ```text
/// // ┌─────────────── u16 ───────────────┐
/// // │ Unused  │  ID  │ ROOT │ SHAPE │
/// // │ 15    9 │ 8  5 │ 4  2 │ 1   0 │
/// // └───────────────────────────────────┘
/// ```
///
/// The enum variants are declared in tone-placement ID order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u16)]
#[rustfmt::skip]
pub enum BaseVowel {
    // ID 0..=2: closed vowels (lowest).
    Y           = (0  << 5) | ((RootVowel::Y as u16) << 2) | Shape::None as u16,
    U           = (1  << 5) | ((RootVowel::U as u16) << 2) | Shape::None as u16,
    I           = (2  << 5) | ((RootVowel::I as u16) << 2) | Shape::None as u16,

    // ID 3..=5: open plain vowels.
    E           = (3  << 5) | ((RootVowel::E as u16) << 2) | Shape::None as u16,
    O           = (4  << 5) | ((RootVowel::O as u16) << 2) | Shape::None as u16,
    A           = (5  << 5) | ((RootVowel::A as u16) << 2) | Shape::None as u16,

    // ID 6..=9: vowels with a structural shape.
    UHorn       = (6  << 5) | ((RootVowel::U as u16) << 2) | Shape::Horn as u16,
    ACircumflex = (7  << 5) | ((RootVowel::A as u16) << 2) | Shape::Circumflex as u16,
    OCircumflex = (8  << 5) | ((RootVowel::O as u16) << 2) | Shape::Circumflex as u16,
    ABreve      = (9  << 5) | ((RootVowel::A as u16) << 2) | Shape::Breve as u16,

    // ID 10..=11: highest.
    ECircumflex = (10 << 5) | ((RootVowel::E as u16) << 2) | Shape::Circumflex as u16,
    OHorn       = (11 << 5) | ((RootVowel::O as u16) << 2) | Shape::Horn as u16,
}

impl BaseVowel {
    // ─────────────── Size and bit layout ───────────────

    /// Number of valid base vowels.
    pub const COUNT: usize = 12;

    // Width of each packed field, in bits.
    const SHAPE_WIDTH: usize = 2;
    const ROOT_WIDTH: usize = 3;

    // Starting bit position of each field.
    const SHAPE_OFFSET: usize = 0;
    const ROOT_OFFSET: usize = Self::SHAPE_OFFSET + Self::SHAPE_WIDTH;
    const ID_OFFSET: usize = Self::ROOT_OFFSET + Self::ROOT_WIDTH;

    // Masks for extracting fields from the packed value.
    const SHAPE_MASK: u8 = (1u8 << Self::SHAPE_WIDTH) - 1;
    const ROOT_MASK: u8 = (1u8 << Self::ROOT_WIDTH) - 1;

    // ─────────────── Lookup tables ───────────────

    /// Base vowels ordered by ID.
    ///
    /// The array index is the vowel's ID.
    const VARIANTS_BY_ID: [Self; Self::COUNT] = [
        Self::Y,
        Self::U,
        Self::I,
        Self::E,
        Self::O,
        Self::A,
        Self::UHorn,
        Self::ACircumflex,
        Self::OCircumflex,
        Self::ABreve,
        Self::ECircumflex,
        Self::OHorn,
    ];

    /// Maps each root and shape pair to its tone-placement ID.
    ///
    /// Each root has four entries, one for each shape in enum order:
    /// None, Circumflex, Breve, Horn. The table index is `root * 4 + shape`.
    ///
    /// `None` means Vietnamese has no base vowel with that combination.
    const ID_BY_ROOT_SHAPE: [Option<u8>; 24] = [
        // A
        Some(Self::A.id()),
        Some(Self::ACircumflex.id()),
        Some(Self::ABreve.id()),
        None,
        // E
        Some(Self::E.id()),
        Some(Self::ECircumflex.id()),
        None,
        None,
        // I
        Some(Self::I.id()),
        None,
        None,
        None,
        // O
        Some(Self::O.id()),
        Some(Self::OCircumflex.id()),
        None,
        Some(Self::OHorn.id()),
        // U
        Some(Self::U.id()),
        None,
        None,
        Some(Self::UHorn.id()),
        // Y
        Some(Self::Y.id()),
        None,
        None,
        None,
    ];

    /// Maps each root and shape pair to the base vowel itself.
    ///
    /// Indexing and layout match [`Self::ID_BY_ROOT_SHAPE`]: one entry per
    /// shape in enum order, at `root * 4 + shape`.
    ///
    /// `None` means Vietnamese has no base vowel with that combination. One
    /// lookup returns the answer, where going through the ID costs a second
    /// dependent load.
    const VARIANTS_BY_ROOT_SHAPE: [Option<Self>; 24] = [
        // A
        Some(Self::A),
        Some(Self::ACircumflex),
        Some(Self::ABreve),
        None,
        // E
        Some(Self::E),
        Some(Self::ECircumflex),
        None,
        None,
        // I
        Some(Self::I),
        None,
        None,
        None,
        // O
        Some(Self::O),
        Some(Self::OCircumflex),
        None,
        Some(Self::OHorn),
        // U
        Some(Self::U),
        None,
        None,
        Some(Self::UHorn),
        // Y
        Some(Self::Y),
        None,
        None,
        None,
    ];

    // ─────────────── Construction ───────────────

    /// Returns the unshaped vowel for a root letter.
    #[inline(always)]
    pub const fn from_root(root: RootVowel) -> Self {
        match root {
            RootVowel::A => Self::A,
            RootVowel::E => Self::E,
            RootVowel::I => Self::I,
            RootVowel::O => Self::O,
            RootVowel::U => Self::U,
            RootVowel::Y => Self::Y,
        }
    }

    /// Returns the base vowel with the given tone-placement ID.
    ///
    /// Returns `None` when `vowel_id` is outside `0..COUNT`.
    #[inline(always)]
    pub const fn from_id(vowel_id: usize) -> Option<Self> {
        if vowel_id < Self::COUNT {
            return Some(Self::VARIANTS_BY_ID[vowel_id]);
        }
        None
    }

    /// Returns the base vowel with the given ID without checking its range.
    ///
    /// # Safety
    ///
    /// `vowel_id` must be less than [`Self::COUNT`].
    #[inline(always)]
    pub unsafe fn from_id_unchecked(vowel_id: usize) -> Self {
        debug_assert!(vowel_id < Self::COUNT);
        *Self::VARIANTS_BY_ID.get_unchecked(vowel_id)
    }

    /// Returns the base vowel for a root letter and shape.
    ///
    /// Returns `None` when Vietnamese has no base vowel with that combination.
    #[inline(always)]
    pub const fn from_parts(root: RootVowel, shape: Shape) -> Option<Self> {
        Self::VARIANTS_BY_ROOT_SHAPE[Self::root_shape_index(root, shape)]
    }

    /// Shared `root * 4 + shape` index for both root/shape tables, so
    /// [`Self::from_parts`] and [`Self::id_from_parts`] cannot drift apart.
    #[inline(always)]
    const fn root_shape_index(root: RootVowel, shape: Shape) -> usize {
        ((root as usize) << Self::SHAPE_WIDTH) | shape as usize
    }

    // ─────────────── ID ───────────────

    /// Returns this vowel's tone-placement ID (`0..=11`).
    ///
    /// Higher ID means higher tone-placement priority.
    #[inline(always)]
    pub const fn id(self) -> u8 {
        (self as u16 >> Self::ID_OFFSET) as u8
    }

    /// Returns the tone-placement ID for a valid root and shape pair.
    ///
    /// Returns `None` when the pair does not form a Vietnamese base vowel.
    #[inline(always)]
    pub const fn id_from_parts(root: RootVowel, shape: Shape) -> Option<u8> {
        Self::ID_BY_ROOT_SHAPE[Self::root_shape_index(root, shape)]
    }

    // ─────────────── Components ───────────────

    /// Returns `true` when this vowel has a non-None shape.
    #[inline(always)]
    pub const fn is_shaped(self) -> bool {
        (self as u8 & Self::SHAPE_MASK) != 0
    }

    /// Returns `true` when this vowel has no shape.
    #[inline(always)]
    pub const fn is_plain(self) -> bool {
        !self.is_shaped()
    }

    /// Returns the [`Shape`] component.
    #[inline(always)]
    pub const fn shape(self) -> Shape {
        // SAFETY: the mask limits shape_id to 0..=3, the valid Shape values.
        let shape_id = self as u8 & Self::SHAPE_MASK;
        unsafe { std::mem::transmute::<u8, Shape>(shape_id) }
    }

    #[inline(always)]
    pub const fn has_shape(self, shape: Shape) -> bool {
        (self as u8 & Self::SHAPE_MASK) == shape as u8
    }

    /// Returns the root letter stored in this vowel.
    #[inline(always)]
    pub const fn root(self) -> RootVowel {
        let root_id = (self as u8 >> Self::ROOT_OFFSET) & Self::ROOT_MASK;

        // SAFETY: the mask limits root_id to 0..=5, the valid RootVowel values.
        unsafe { std::mem::transmute::<u8, RootVowel>(root_id) }
    }

    // ─────────────── Shape manipulation ───────────────

    /// Keeps the root letter and replaces the shape.
    ///
    /// Returns `None` when Vietnamese has no base vowel with that root and shape.
    #[inline(always)]
    pub const fn replace_shape(self, shape: Shape) -> Option<Self> {
        Self::from_parts(self.root(), shape)
    }

    /// Removes the shape and returns the unshaped vowel with the same root.
    #[inline(always)]
    pub const fn remove_shape(self) -> Self {
        Self::from_root(self.root())
    }
}

/// A base vowel with its tone and case stored together in a `u16`.
///
/// ```text
/// ┌─────────────── 16 bits ───────────────┐
/// │ RRR │    BASE VOWEL    │  TTT  │  C  │
/// │15 13│      12..4       │ 3..1  │  0  │
/// └───────────────────────────────────────┘
/// ```
///
/// - `RRR`: reserved bits, set to zero.
/// - `BASE VOWEL`: the packed [`BaseVowel`] value.
/// - `TTT`: the [`Tone`] (`0` is Flat).
/// - `C`: letter case (`0` is lowercase, `1` is uppercase).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Vowel(u16);

impl Vowel {
    // Field widths and bit positions in the packed `u16`.
    const CASE_WIDTH: usize = 1;
    const TONE_WIDTH: usize = 3;
    const BASE_WIDTH: usize = 9;

    const CASE_OFFSET: usize = 0;
    const TONE_OFFSET: usize = Self::CASE_OFFSET + Self::CASE_WIDTH;
    const BASE_OFFSET: usize = Self::TONE_OFFSET + Self::TONE_WIDTH;

    const CASE_MASK: u8 = (1u8 << Self::CASE_WIDTH) - 1;
    const TONE_MASK: u8 = ((1u8 << Self::TONE_WIDTH) - 1) << Self::TONE_OFFSET;
    const BASE_MASK: u16 = ((1u16 << Self::BASE_WIDTH) - 1) << Self::BASE_OFFSET;

    // ─────────────── Construction ───────────────

    /// Packs a base vowel, tone, and case into one value.
    #[inline(always)]
    pub const fn new(base: BaseVowel, tone: Tone, upper: bool) -> Self {
        Self(
            ((base as u16) << Self::BASE_OFFSET)
                | ((tone as u16) << Self::TONE_OFFSET)
                | ((upper as u16) << Self::CASE_OFFSET),
        )
    }

    /// Creates an uppercase vowel.
    #[inline(always)]
    pub const fn upper(base: BaseVowel, tone: Tone) -> Self {
        Self::new(base, tone, true)
    }

    /// Creates a lowercase vowel.
    #[inline(always)]
    pub const fn lower(base: BaseVowel, tone: Tone) -> Self {
        Self::new(base, tone, false)
    }

    // ─────────────── Accessors ───────────────

    /// Returns the base vowel.
    #[inline(always)]
    pub const fn base(self) -> BaseVowel {
        let bits = (self.0 & Self::BASE_MASK) >> Self::BASE_OFFSET;

        // SAFETY: constructors and setters only store valid BaseVowel values.
        unsafe { std::mem::transmute(bits) }
    }

    /// Returns the root letter of the base vowel.
    #[inline(always)]
    pub const fn root(self) -> RootVowel {
        self.base().root()
    }

    /// Returns the tone.
    #[inline(always)]
    pub const fn tone(self) -> Tone {
        let bits = (self.0 as u8 & Self::TONE_MASK) >> Self::TONE_OFFSET;

        // SAFETY: constructors and setters only store valid Tone values.
        unsafe { std::mem::transmute(bits) }
    }

    /// Returns `true` if this vowel is uppercase.
    #[inline(always)]
    pub const fn is_upper(self) -> bool {
        self.0 as u8 & Self::CASE_MASK != 0
    }

    /// Returns the packed bits.
    #[inline(always)]
    pub const fn bits(self) -> u16 {
        self.0
    }

    // ─────────────── Mutating updates ───────────────

    /// Replaces the base vowel and keeps the tone and case.
    #[inline(always)]
    pub const fn set_base(&mut self, base: BaseVowel) {
        self.0 = (self.0 & !Self::BASE_MASK) | ((base as u16) << Self::BASE_OFFSET);
    }

    /// Replaces the tone and keeps the base vowel and case.
    #[inline(always)]
    pub const fn set_tone(&mut self, tone: Tone) {
        self.0 = (self.0 & !(Self::TONE_MASK as u16)) | ((tone as u16) << Self::TONE_OFFSET);
    }

    /// Replaces the case and keeps the base vowel and tone.
    #[inline(always)]
    pub const fn set_upper(&mut self, upper: bool) {
        self.0 = (self.0 & !(Self::CASE_MASK as u16)) | upper as u16;
    }

    // ─────────────── Copying updates ───────────────

    /// Returns a copy with a different base vowel, keeping its tone and case.
    #[inline(always)]
    pub const fn with_base(mut self, base: BaseVowel) -> Self {
        self.set_base(base);
        self
    }

    /// Returns a copy with a different tone, keeping its base vowel and case.
    #[inline(always)]
    pub const fn with_tone(mut self, tone: Tone) -> Self {
        self.set_tone(tone);
        self
    }

    /// Returns a copy with a different case, keeping its base vowel and tone.
    #[inline(always)]
    pub const fn with_upper(mut self, upper: bool) -> Self {
        self.set_upper(upper);
        self
    }

    // ─────────────── Tone and shape removal ───────────────

    /// Clears the tone and keeps the base vowel and case.
    #[inline(always)]
    pub const fn remove_tone(&mut self) -> &mut Self {
        self.0 &= !(Self::TONE_MASK as u16);
        self
    }

    /// Returns a copy with the tone cleared.
    #[inline(always)]
    pub const fn without_tone(mut self) -> Self {
        *self.remove_tone()
    }

    /// Removes the base vowel's shape and keeps its tone and case.
    #[inline(always)]
    pub const fn remove_shape(&mut self) -> &mut Self {
        self.set_base(self.base().remove_shape());
        self
    }

    /// Returns a copy with the base vowel's shape removed.
    #[inline(always)]
    pub const fn without_shape(mut self) -> Self {
        *self.remove_shape()
    }

    /// Encodes this vowel as its precomposed Vietnamese character.
    #[inline(always)]
    pub const fn to_char(self) -> char {
        encode_vowel(self.base(), self.tone(), self.is_upper())
    }

    /// Decodes a precomposed Vietnamese vowel, or returns `None` if `ch` is not one.
    #[inline(always)]
    pub const fn from_char(ch: char) -> Option<Self> {
        decode_vowel(ch)
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
    let idx = ((base.id() as usize * 6 + tone as usize) << 1) | (uppercase as usize);
    ENCODED_VOWELS[idx]
}

/// Decodes a precomposed Vietnamese vowel into a packed [`Vowel`], or returns
/// `None` if `character` is not a vowel.
///
/// Fastest measured variant: four non-overlapping code-point regions keep the
/// branch tree small — ASCII (match), Latin-1, Latin Extended (match), and the
/// Vietnamese block (U+1EA0..=U+1EF9) via O(1) direct LUT indexing.
#[inline(always)]
pub const fn decode_vowel(character: char) -> Option<Vowel> {
    use BaseVowel::*;
    use Tone::*;

    let code = character as u32;

    match code {
        // 1. ASCII Block (Fast path - Keystrokes)
        0x00..=0x7F => match character {
            'a' => Some(Vowel::lower(A, Flat)),
            'A' => Some(Vowel::upper(A, Flat)),
            'o' => Some(Vowel::lower(O, Flat)),
            'O' => Some(Vowel::upper(O, Flat)),
            'e' => Some(Vowel::lower(E, Flat)),
            'E' => Some(Vowel::upper(E, Flat)),
            'i' => Some(Vowel::lower(I, Flat)),
            'I' => Some(Vowel::upper(I, Flat)),
            'u' => Some(Vowel::lower(U, Flat)),
            'U' => Some(Vowel::upper(U, Flat)),
            'y' => Some(Vowel::lower(Y, Flat)),
            'Y' => Some(Vowel::upper(Y, Flat)),
            _ => None,
        },

        // 2. Latin-1 Supplement (U+00C0..U+00FF)
        0x80..=0xFF => match character {
            'ê' => Some(Vowel::lower(ECircumflex, Flat)),
            'Ê' => Some(Vowel::upper(ECircumflex, Flat)),
            'ô' => Some(Vowel::lower(OCircumflex, Flat)),
            'Ô' => Some(Vowel::upper(OCircumflex, Flat)),
            'â' => Some(Vowel::lower(ACircumflex, Flat)),
            'Â' => Some(Vowel::upper(ACircumflex, Flat)),
            'á' => Some(Vowel::lower(A, Acute)),
            'Á' => Some(Vowel::upper(A, Acute)),
            'à' => Some(Vowel::lower(A, Grave)),
            'À' => Some(Vowel::upper(A, Grave)),
            'ã' => Some(Vowel::lower(A, Tilde)),
            'Ã' => Some(Vowel::upper(A, Tilde)),
            'ó' => Some(Vowel::lower(O, Acute)),
            'Ó' => Some(Vowel::upper(O, Acute)),
            'ò' => Some(Vowel::lower(O, Grave)),
            'Ò' => Some(Vowel::upper(O, Grave)),
            'õ' => Some(Vowel::lower(O, Tilde)),
            'Õ' => Some(Vowel::upper(O, Tilde)),
            'é' => Some(Vowel::lower(E, Acute)),
            'É' => Some(Vowel::upper(E, Acute)),
            'è' => Some(Vowel::lower(E, Grave)),
            'È' => Some(Vowel::upper(E, Grave)),
            'í' => Some(Vowel::lower(I, Acute)),
            'Í' => Some(Vowel::upper(I, Acute)),
            'ì' => Some(Vowel::lower(I, Grave)),
            'Ì' => Some(Vowel::upper(I, Grave)),
            'ú' => Some(Vowel::lower(U, Acute)),
            'Ú' => Some(Vowel::upper(U, Acute)),
            'ù' => Some(Vowel::lower(U, Grave)),
            'Ù' => Some(Vowel::upper(U, Grave)),
            'ý' => Some(Vowel::lower(Y, Acute)),
            'Ý' => Some(Vowel::upper(Y, Acute)),
            _ => None,
        },

        // 3. Latin Extended
        0x0100..=0x01B0 => match character {
            'ơ' => Some(Vowel::lower(OHorn, Flat)),
            'Ơ' => Some(Vowel::upper(OHorn, Flat)),
            'ă' => Some(Vowel::lower(ABreve, Flat)),
            'Ă' => Some(Vowel::upper(ABreve, Flat)),
            'ư' => Some(Vowel::lower(UHorn, Flat)),
            'Ư' => Some(Vowel::upper(UHorn, Flat)),
            'ĩ' => Some(Vowel::lower(I, Tilde)),
            'Ĩ' => Some(Vowel::upper(I, Tilde)),
            'ũ' => Some(Vowel::lower(U, Tilde)),
            'Ũ' => Some(Vowel::upper(U, Tilde)),
            _ => None,
        },

        // 4. Vietnamese block (U+1EA0..U+1EF9) -> Direct Indexing Table!
        0x1EA0..=0x1EF9 => {
            /// Direct lookup table (LUT) for the precomposed Vietnamese block (U+1EA0..=U+1EF9).
            /// Index = `(code - 0x1EA0) as usize`, giving O(1) `Vowel` lookup.
            const DECODED_VIETNAMESE_BLOCK_LUT: [Vowel; 90] = [
                // 0x1EA0 - 0x1EA1 (Ạ, ạ)
                Vowel::upper(A, Dot),
                Vowel::lower(A, Dot),
                // 0x1EA2 - 0x1EA3 (Ả, ả)
                Vowel::upper(A, Hook),
                Vowel::lower(A, Hook),
                // 0x1EA4 - 0x1EA5 (Ấ, ấ)
                Vowel::upper(ACircumflex, Acute),
                Vowel::lower(ACircumflex, Acute),
                // 0x1EA6 - 0x1EA7 (Ầ, ầ)
                Vowel::upper(ACircumflex, Grave),
                Vowel::lower(ACircumflex, Grave),
                // 0x1EA8 - 0x1EA9 (Ẩ, ẩ)
                Vowel::upper(ACircumflex, Hook),
                Vowel::lower(ACircumflex, Hook),
                // 0x1EAA - 0x1EAB (Ẫ, ẫ)
                Vowel::upper(ACircumflex, Tilde),
                Vowel::lower(ACircumflex, Tilde),
                // 0x1EAC - 0x1EAD (Ậ, ậ)
                Vowel::upper(ACircumflex, Dot),
                Vowel::lower(ACircumflex, Dot),
                // 0x1EAE - 0x1EAF (Ắ, ắ)
                Vowel::upper(ABreve, Acute),
                Vowel::lower(ABreve, Acute),
                // 0x1EB0 - 0x1EB1 (Ằ, ằ)
                Vowel::upper(ABreve, Grave),
                Vowel::lower(ABreve, Grave),
                // 0x1EB2 - 0x1EB3 (Ẳ, ẳ)
                Vowel::upper(ABreve, Hook),
                Vowel::lower(ABreve, Hook),
                // 0x1EB4 - 0x1EB5 (Ẵ, ẵ)
                Vowel::upper(ABreve, Tilde),
                Vowel::lower(ABreve, Tilde),
                // 0x1EB6 - 0x1EB7 (Ặ, ặ)
                Vowel::upper(ABreve, Dot),
                Vowel::lower(ABreve, Dot),
                // 0x1EB8 - 0x1EB9 (Ẹ, ẹ)
                Vowel::upper(E, Dot),
                Vowel::lower(E, Dot),
                // 0x1EBA - 0x1EBB (Ẻ, ẻ)
                Vowel::upper(E, Hook),
                Vowel::lower(E, Hook),
                // 0x1EBC - 0x1EBD (Ẽ, ẽ)
                Vowel::upper(E, Tilde),
                Vowel::lower(E, Tilde),
                // 0x1EBE - 0x1EBF (Ế, ế)
                Vowel::upper(ECircumflex, Acute),
                Vowel::lower(ECircumflex, Acute),
                // 0x1EC0 - 0x1EC1 (Ề, ề)
                Vowel::upper(ECircumflex, Grave),
                Vowel::lower(ECircumflex, Grave),
                // 0x1EC2 - 0x1EC3 (Ể, ể)
                Vowel::upper(ECircumflex, Hook),
                Vowel::lower(ECircumflex, Hook),
                // 0x1EC4 - 0x1EC5 (Ễ, ễ)
                Vowel::upper(ECircumflex, Tilde),
                Vowel::lower(ECircumflex, Tilde),
                // 0x1EC6 - 0x1EC7 (Ệ, ệ)
                Vowel::upper(ECircumflex, Dot),
                Vowel::lower(ECircumflex, Dot),
                // 0x1EC8 - 0x1EC9 (Ỉ, ỉ)
                Vowel::upper(I, Hook),
                Vowel::lower(I, Hook),
                // 0x1ECA - 0x1ECB (Ị, ị)
                Vowel::upper(I, Dot),
                Vowel::lower(I, Dot),
                // 0x1ECC - 0x1ECD (Ọ, ọ)
                Vowel::upper(O, Dot),
                Vowel::lower(O, Dot),
                // 0x1ECE - 0x1ECF (Ỏ, ỏ)
                Vowel::upper(O, Hook),
                Vowel::lower(O, Hook),
                // 0x1ED0 - 0x1ED1 (Ố, ố)
                Vowel::upper(OCircumflex, Acute),
                Vowel::lower(OCircumflex, Acute),
                // 0x1ED2 - 0x1ED3 (Ồ, ồ)
                Vowel::upper(OCircumflex, Grave),
                Vowel::lower(OCircumflex, Grave),
                // 0x1ED4 - 0x1ED5 (Ổ, ổ)
                Vowel::upper(OCircumflex, Hook),
                Vowel::lower(OCircumflex, Hook),
                // 0x1ED6 - 0x1ED7 (Ỗ, ỗ)
                Vowel::upper(OCircumflex, Tilde),
                Vowel::lower(OCircumflex, Tilde),
                // 0x1ED8 - 0x1ED9 (Ộ, ộ)
                Vowel::upper(OCircumflex, Dot),
                Vowel::lower(OCircumflex, Dot),
                // 0x1EDA - 0x1EDB (Ớ, ớ)
                Vowel::upper(OHorn, Acute),
                Vowel::lower(OHorn, Acute),
                // 0x1EDC - 0x1EDD (Ờ, ờ)
                Vowel::upper(OHorn, Grave),
                Vowel::lower(OHorn, Grave),
                // 0x1EDE - 0x1EDF (Ở, ở)
                Vowel::upper(OHorn, Hook),
                Vowel::lower(OHorn, Hook),
                // 0x1EE0 - 0x1EE1 (Ỡ, ỡ)
                Vowel::upper(OHorn, Tilde),
                Vowel::lower(OHorn, Tilde),
                // 0x1EE2 - 0x1EE3 (Ợ, ợ)
                Vowel::upper(OHorn, Dot),
                Vowel::lower(OHorn, Dot),
                // 0x1EE4 - 0x1EE5 (Ụ, ụ)
                Vowel::upper(U, Dot),
                Vowel::lower(U, Dot),
                // 0x1EE6 - 0x1EE7 (Ủ, ủ)
                Vowel::upper(U, Hook),
                Vowel::lower(U, Hook),
                // 0x1EE8 - 0x1EE9 (Ứ, ứ)
                Vowel::upper(UHorn, Acute),
                Vowel::lower(UHorn, Acute),
                // 0x1EEA - 0x1EEB (Ừ, ừ)
                Vowel::upper(UHorn, Grave),
                Vowel::lower(UHorn, Grave),
                // 0x1EEC - 0x1EED (Ử, ử)
                Vowel::upper(UHorn, Hook),
                Vowel::lower(UHorn, Hook),
                // 0x1EEE - 0x1EEF (Ữ, ữ)
                Vowel::upper(UHorn, Tilde),
                Vowel::lower(UHorn, Tilde),
                // 0x1EF0 - 0x1EF1 (Ự, ự)
                Vowel::upper(UHorn, Dot),
                Vowel::lower(UHorn, Dot),
                // 0x1EF2 - 0x1EF3 (Ỳ, ỳ)
                Vowel::upper(Y, Grave),
                Vowel::lower(Y, Grave),
                // 0x1EF4 - 0x1EF5 (Ỵ, ỵ)
                Vowel::upper(Y, Dot),
                Vowel::lower(Y, Dot),
                // 0x1EF6 - 0x1EF7 (Ỷ, ỷ)
                Vowel::upper(Y, Hook),
                Vowel::lower(Y, Hook),
                // 0x1EF8 - 0x1EF9 (Ỹ, ỹ)
                Vowel::upper(Y, Tilde),
                Vowel::lower(Y, Tilde),
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

/// Fails to compile if the two root/shape tables ever disagree.
const _: () = {
    let mut index = 0;
    while index < BaseVowel::ID_BY_ROOT_SHAPE.len() {
        let expected_id = match BaseVowel::ID_BY_ROOT_SHAPE[index] {
            Some(vowel_id) => {
                assert!(BaseVowel::VARIANTS_BY_ID[vowel_id as usize].id() == vowel_id);
                vowel_id
            }
            None => 0xFF,
        };
        let actual_id = match BaseVowel::VARIANTS_BY_ROOT_SHAPE[index] {
            Some(vowel) => vowel.id(),
            None => 0xFF,
        };
        assert!(actual_id == expected_id, "root/shape tables disagree");
        index += 1;
    }
};
