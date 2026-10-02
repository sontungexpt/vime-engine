use std::cmp::Ordering;

/// The root letter of a base vowel: `y`, `u`, `i`, `e`, `o`, `a`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum RootVowel {
    Y = 0,
    U = 1,
    I = 2,
    E = 3,
    O = 4,
    A = 5,
}

impl RootVowel {
    #[inline(always)]
    pub const fn id(self) -> u8 {
        self as u8
    }
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
    /// Whether a diacritic is present (i.e. not `None`).
    #[inline(always)]
    pub const fn is_some(self) -> bool {
        !matches!(self, Shape::None)
    }

    #[inline(always)]
    pub const fn id(self) -> u8 {
        self as u8
    }
}

/// One of the six Vietnamese tones: ngang, sắc, huyền, hỏi, ngã, nặng.
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
    #[inline(always)]
    pub const fn is_some(self) -> bool {
        !matches!(self, Self::Flat)
    }

    #[inline(always)]
    pub const fn id(self) -> u8 {
        self as u8
    }
}

/// A Vietnamese base vowel packed into a `u8`.
///
/// ```text
///  7       5 4       2 1       0
/// ┌─────────┬─────────┬─────────┐
/// │ unused  │  ROOT   │  SHAPE  │
/// │         │  3 bits │  2 bits │
/// └─────────┴─────────┴─────────┘
/// ```
///
/// Variants are declared in tone-placement priority order, which is also the
/// order [`Ord`] compares in — see the manual impl below.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
#[rustfmt::skip]
pub enum BaseVowel {
    // ID 0..=2: closed vowels (lowest).
    Y           = Self::encode(RootVowel::Y, Shape::None),
    U           = Self::encode(RootVowel::U, Shape::None),
    I           = Self::encode(RootVowel::I, Shape::None),

    // ID 3..=5: open plain vowels.
    E           = Self::encode(RootVowel::E, Shape::None),
    O           = Self::encode(RootVowel::O, Shape::None),
    A           = Self::encode(RootVowel::A, Shape::None),

    // ID 6..=9: vowels with a structural shape.
    UHorn       = Self::encode(RootVowel::U, Shape::Horn),
    ACircumflex = Self::encode(RootVowel::A, Shape::Circumflex),
    OCircumflex = Self::encode(RootVowel::O, Shape::Circumflex),
    ABreve      = Self::encode(RootVowel::A, Shape::Breve),

    // ID 10..=11: highest.
    ECircumflex = Self::encode(RootVowel::E, Shape::Circumflex),
    OHorn       = Self::encode(RootVowel::O, Shape::Horn),
}

/// Dense `0..=11` ID of a [`BaseVowel`]: its tone-placement priority, and its
/// bit index in a phonotactic mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum BaseVowelId {
    Y = 0,
    U = 1,
    I = 2,
    E = 3,
    O = 4,
    A = 5,
    UHorn = 6,
    ACircumflex = 7,
    OCircumflex = 8,
    ABreve = 9,
    ECircumflex = 10,
    OHorn = 11,
}

impl BaseVowelId {
    pub const COUNT: usize = 12;

    #[inline(always)]
    pub const fn id(self) -> u8 {
        self as u8
    }

    /// Converts a raw `u8` ID into `Self` with no bounds check.
    ///
    /// # Safety
    ///
    /// `id` must be less than [`Self::COUNT`]; otherwise the transmute produces
    /// an invalid variant, which is undefined behavior.
    #[inline(always)]
    pub const unsafe fn from_u8_unchecked(id: u8) -> Self {
        // SAFETY: `id < COUNT` is the caller's whole contract for this function.
        unsafe { std::mem::transmute(id) }
    }

    /// Converts a raw `u8` ID into `Self`, or `None` when `id >= [`Self::COUNT`].
    #[inline(always)]
    pub const fn from_u8(id: u8) -> Option<Self> {
        if id < Self::COUNT as u8 {
            // Safety: Bounds check ensures `id` is a valid discriminant.
            Some(unsafe { Self::from_u8_unchecked(id) })
        } else {
            None
        }
    }
}

impl BaseVowel {
    // ─────────────── Size and bit layout ───────────────

    /// Number of valid Vietnamese base vowels.
    pub const COUNT: usize = 12;

    // Field widths, offsets and masks, all in bits.
    const SHAPE_WIDTH: usize = 2;
    const ROOT_WIDTH: usize = 3;

    const SHAPE_OFFSET: usize = 0;
    const ROOT_OFFSET: usize = Self::SHAPE_OFFSET + Self::SHAPE_WIDTH;

    const SHAPE_MASK: u8 = (1u8 << Self::SHAPE_WIDTH) - 1;
    const ROOT_MASK: u8 = (1u8 << Self::ROOT_WIDTH) - 1;

    // ─────────────── Encoding ───────────────

    /// Packs a root and shape into the `u8` layout. Every discriminant is
    /// written in terms of this, so it is the one definition of the layout.
    #[inline(always)]
    const fn encode(root: RootVowel, shape: Shape) -> u8 {
        ((root as u8) << Self::ROOT_OFFSET) | shape as u8
    }

    /// Packs a root into the value of its unshaped vowel.
    #[inline(always)]
    const fn encode_from_root(root: RootVowel) -> u8 {
        (root as u8) << Self::ROOT_OFFSET
    }

    // ─────────────── Validity ───────────────

    /// Bit `i` is set when `i` is one of the 12 declared encodings, so
    /// [`Self::is_declared`] answers with one mask test instead of a 12-way match.
    #[rustfmt::skip]
    const DECLARED_MASK: u32 =
          (1 << Self::Y as u8)
        | (1 << Self::U as u8)
        | (1 << Self::I as u8)
        | (1 << Self::E as u8)
        | (1 << Self::O as u8)
        | (1 << Self::A as u8)
        | (1 << Self::UHorn as u8)
        | (1 << Self::ACircumflex as u8)
        | (1 << Self::OCircumflex as u8)
        | (1 << Self::ABreve as u8)
        | (1 << Self::ECircumflex as u8)
        | (1 << Self::OHorn as u8);

    /// Whether `value` is one of the 12 declared encodings. Requires `value < 32`
    /// — a range obligation, not a memory-safety one, so this stays safe: a
    /// violation trips the `debug_assert!` (in release the shift would mask).
    #[inline(always)]
    const fn is_declared(value: u8) -> bool {
        debug_assert!(
            value < 32,
            "DECLARED_MASK is a u32; value would overflow the shift"
        );
        (Self::DECLARED_MASK & (1u32 << value)) != 0
    }

    // ─────────────── Construction ───────────────

    /// Returns the unshaped base vowel for a root letter.
    #[inline(always)]
    pub const fn from_root(root: RootVowel) -> Self {
        // SAFETY: this is `encode(root, Shape::None)`, which is a declared
        // discriminant.
        unsafe { std::mem::transmute(Self::encode_from_root(root)) }
    }

    /// Returns the base vowel for a root letter and shape, or `None` when the
    /// combination is not one Vietnamese spells.
    #[inline(always)]
    pub const fn from_parts(root: RootVowel, shape: Shape) -> Option<Self> {
        // Root <= 5 and shape <= 3, so `value` <= 23: in range for the shift.
        let value = Self::encode(root, shape);
        if Self::is_declared(value) {
            // SAFETY: `is_declared` holds only for the 12 declared discriminants.
            Some(unsafe { std::mem::transmute(value) })
        } else {
            None
        }
    }

    // ─────────────── ID / Priority ───────────────

    /// The base vowel for a tone-placement ID — the inverse of [`Self::id`].
    #[inline(always)]
    pub const fn from_id(vowel_id: BaseVowelId) -> Self {
        match vowel_id {
            BaseVowelId::Y => Self::Y,
            BaseVowelId::U => Self::U,
            BaseVowelId::I => Self::I,
            BaseVowelId::E => Self::E,
            BaseVowelId::O => Self::O,
            BaseVowelId::A => Self::A,
            BaseVowelId::UHorn => Self::UHorn,
            BaseVowelId::ACircumflex => Self::ACircumflex,
            BaseVowelId::OCircumflex => Self::OCircumflex,
            BaseVowelId::ABreve => Self::ABreve,
            BaseVowelId::ECircumflex => Self::ECircumflex,
            BaseVowelId::OHorn => Self::OHorn,
        }
    }

    /// This vowel's tone-placement ID (`0..=11`): higher is higher tone-placement
    /// priority, and it is not the packed value (`UHorn` packs as 11, ID 6).
    #[inline(always)]
    pub const fn id(self) -> BaseVowelId {
        match self {
            Self::Y => BaseVowelId::Y,
            Self::U => BaseVowelId::U,
            Self::I => BaseVowelId::I,
            Self::E => BaseVowelId::E,
            Self::O => BaseVowelId::O,
            Self::A => BaseVowelId::A,
            Self::UHorn => BaseVowelId::UHorn,
            Self::ACircumflex => BaseVowelId::ACircumflex,
            Self::OCircumflex => BaseVowelId::OCircumflex,
            Self::ABreve => BaseVowelId::ABreve,
            Self::ECircumflex => BaseVowelId::ECircumflex,
            Self::OHorn => BaseVowelId::OHorn,
        }
    }

    // ─────────────── Components ───────────────

    /// Returns the [`RootVowel`] component.
    #[inline(always)]
    pub const fn root(self) -> RootVowel {
        let root = (self as u8 >> Self::ROOT_OFFSET) & Self::ROOT_MASK;
        // SAFETY: ROOT_MASK limits bits to 0..=5, matching valid RootVowel variants.
        unsafe { std::mem::transmute::<u8, RootVowel>(root) }
    }

    /// Returns the [`Shape`] component.
    #[inline(always)]
    pub const fn shape(self) -> Shape {
        let shape = self as u8 & Self::SHAPE_MASK;
        // SAFETY: SHAPE_MASK limits bits to 0..=3, matching valid Shape variants.
        unsafe { std::mem::transmute::<u8, Shape>(shape) }
    }

    /// Returns `true` when this vowel's shape is exactly `shape`.
    #[inline(always)]
    pub const fn is_shape(self, shape: Shape) -> bool {
        (self as u8 & Self::SHAPE_MASK) == shape as u8
    }

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

    // ─────────────── Shape manipulation ───────────────

    /// Keeps the root letter and replaces the shape, or returns `None` when the
    /// result is not one Vietnamese spells.
    #[inline(always)]
    pub const fn replace_shape(self, shape: Shape) -> Option<Self> {
        // Bits 5-7 are zero in every `BaseVowel`, and clearing the shape bits
        // can only clear more, so `value` < 32.
        let value = (self as u8 & !Self::SHAPE_MASK) | shape as u8;
        if Self::is_declared(value) {
            // SAFETY: as in `from_parts`.
            Some(unsafe { std::mem::transmute(value) })
        } else {
            None
        }
    }

    /// Removes the shape and returns the unshaped vowel with the same root.
    #[inline(always)]
    pub const fn remove_shape(self) -> Self {
        let value = self as u8 & !Self::SHAPE_MASK;
        // SAFETY: every root has a declared plain vowel, so the result is valid.
        unsafe { std::mem::transmute(value) }
    }
}

/// Orders by tone-placement priority ([`BaseVowel::id`]), not by packed value.
///
/// The two disagree (`UHorn` packs as 11 but has ID 6), so a derived `Ord`
/// would pick the wrong `max`/`min`; it still agrees with `PartialEq`, because
/// `id` is injective over the 12 variants.
impl Ord for BaseVowel {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.id().cmp(&other.id())
    }
}

impl PartialOrd for BaseVowel {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A base vowel with its tone and case stored together in a `u16`.
///
/// ```text
///  15       9 8      4 3      1 0
/// ┌──────────┬────────┬────────┬─┐
/// │  unused  │  BASE  │  TTT   │C│
/// │  7 bits  │ 5 bits │ 3 bits │1│
/// └──────────┴────────┴────────┴─┘
/// ```
///
/// `BASE` is the packed [`BaseVowel`] (5 of its 8 bits), `TTT` the [`Tone`]
/// (`0` is Flat), `C` the case (`1` is uppercase), and the top 7 bits stay zero.
///
/// Deliberately not [`Ord`]: any order must follow tone-placement priority (see
/// [`BaseVowel`]) rather than the packed value. Key or sort on [`Self::base`]
/// where a base-level order is what is wanted.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Vowel(u16);

impl Vowel {
    // Field widths and bit positions in the packed `u16`.
    const CASE_WIDTH: usize = 1;
    const TONE_WIDTH: usize = 3;
    const BASE_WIDTH: usize = 5;

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

        // SAFETY: `BASE_WIDTH` is 5, so `bits` fits a `u8` with no truncation
        // and can only hold a value `BaseVowel` actually has.
        unsafe { std::mem::transmute(bits as u8) }
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

    /// Returns `true` if this vowel is toned.
    #[inline(always)]
    pub const fn is_toned(self) -> bool {
        self.0 as u8 & Self::TONE_MASK != 0
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

/// Encodes a base vowel, tone, and case as a precomposed Vietnamese character.
///
/// This table is indexed by tone-placement priority, then tone and case.
#[inline(always)]
pub const fn encode_vowel(base: BaseVowel, tone: Tone, uppercase: bool) -> char {
    #[rustfmt::skip]
    const ENCODED: [char; 144] = [
        // Priority 0: Y
        'y', 'Y', 'ý', 'Ý', 'ỳ', 'Ỳ', 'ỷ', 'Ỷ', 'ỹ', 'Ỹ', 'ỵ', 'Ỵ',
        // Priority 1: U
        'u', 'U', 'ú', 'Ú', 'ù', 'Ù', 'ủ', 'Ủ', 'ũ', 'Ũ', 'ụ', 'Ụ',
        // Priority 2: I
        'i', 'I', 'í', 'Í', 'ì', 'Ì', 'ỉ', 'Ỉ', 'ĩ', 'Ĩ', 'ị', 'Ị',
        // Priority 3: E
        'e', 'E', 'é', 'É', 'è', 'È', 'ẻ', 'Ẻ', 'ẽ', 'Ẽ', 'ẹ', 'Ẹ',
        // Priority 4: O
        'o', 'O', 'ó', 'Ó', 'ò', 'Ò', 'ỏ', 'Ỏ', 'õ', 'Õ', 'ọ', 'Ọ',
        // Priority 5: A
        'a', 'A', 'á', 'Á', 'à', 'À', 'ả', 'Ả', 'ã', 'Ã', 'ạ', 'Ạ',
        // Priority 6: UHorn
        'ư', 'Ư', 'ứ', 'Ứ', 'ừ', 'Ừ', 'ử', 'Ử', 'ữ', 'Ữ', 'ự', 'Ự',
        // Priority 7: ACircumflex
        'â', 'Â', 'ấ', 'Ấ', 'ầ', 'Ầ', 'ẩ', 'Ẩ', 'ẫ', 'Ẫ', 'ậ', 'Ậ',
        // Priority 8: OCircumflex
        'ô', 'Ô', 'ố', 'Ố', 'ồ', 'Ồ', 'ổ', 'Ổ', 'ỗ', 'Ỗ', 'ộ', 'Ộ',
        // Priority 9: ABreve
        'ă', 'Ă', 'ắ', 'Ắ', 'ằ', 'Ằ', 'ẳ', 'Ẳ', 'ẵ', 'Ẵ', 'ặ', 'Ặ',
        // Priority 10: ECircumflex
        'ê', 'Ê', 'ế', 'Ế', 'ề', 'Ề', 'ể', 'Ể', 'ễ', 'Ễ', 'ệ', 'Ệ',
        // Priority 11: OHorn
        'ơ', 'Ơ', 'ớ', 'Ớ', 'ờ', 'Ờ', 'ở', 'Ở', 'ỡ', 'Ỡ', 'ợ', 'Ợ',
    ];

    // Keep these so the base_id do not need to rely on `BaseVowelId`.
    // So when the BaseVowelId changes, the ENCODED do not need to be updated.
    use BaseVowel::*;
    let base_id = match base {
        Y => 0,
        U => 1,
        I => 2,
        E => 3,
        O => 4,
        A => 5,
        UHorn => 6,
        ACircumflex => 7,
        OCircumflex => 8,
        ABreve => 9,
        ECircumflex => 10,
        OHorn => 11,
    };

    let idx = ((base_id * 6 + tone as usize) << 1) | uppercase as usize;

    // Safety: base_id (0..=11) * 12 + tone (0..=5) * 2 + uppercase (0..=1) <= 143
    unsafe { *ENCODED.as_ptr().add(idx) }
}

/// Decodes a precomposed Vietnamese vowel into a packed [`Vowel`], or returns
/// `None` if `character` is not a vowel.
///
/// Splits on four code-point regions to keep the branch tree small: ASCII,
/// Latin-1, Latin Extended, then the Vietnamese block (U+1EA0..=U+1EF9) by O(1)
/// table indexing.
#[inline(always)]
pub const fn decode_vowel(character: char) -> Option<Vowel> {
    use BaseVowel::*;
    use Tone::*;

    let code = character as u32;

    match code {
        // ASCII (A..Z, a..z), the common keystrokes.
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

        // Latin-1 Supplement (U+00C0..=U+00FF).
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

        // Latin Extended (U+0100..=U+01B0).
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

        // Vietnamese block (U+1EA0..U+1EF9), indexed straight into a table.
        0x1EA0..=0x1EF9 => {
            /// Maps `(code - 0x1EA0)` to the decoded vowel for the Vietnamese block.
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
            // SAFETY: the match above admits only `0x1EA0..=0x1EF9`, exactly 90
            // code points, matching the table's 90 entries, so `offset` is in bounds.
            Some(unsafe { *DECODED_VIETNAMESE_BLOCK_LUT.as_ptr().add(offset) })
        }

        _ => None,
    }
}

/// Whether `ch` is a Vietnamese vowel, without decoding its parts.
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
