use vime_engine::phonology::{BaseVowel, RootVowel, Shape};

/// Benchmark-only candidate representation without priority-ID bits.
/// Benchmark-only candidate representation without priority-ID bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StructuralBaseVowel {
    Y = ((RootVowel::Y as u8) << 2) | Shape::None as u8,
    U = ((RootVowel::U as u8) << 2) | Shape::None as u8,
    I = ((RootVowel::I as u8) << 2) | Shape::None as u8,
    E = ((RootVowel::E as u8) << 2) | Shape::None as u8,
    O = ((RootVowel::O as u8) << 2) | Shape::None as u8,
    A = ((RootVowel::A as u8) << 2) | Shape::None as u8,

    UHorn = ((RootVowel::U as u8) << 2) | Shape::Horn as u8,
    ACircumflex = ((RootVowel::A as u8) << 2) | Shape::Circumflex as u8,
    OCircumflex = ((RootVowel::O as u8) << 2) | Shape::Circumflex as u8,
    ABreve = ((RootVowel::A as u8) << 2) | Shape::Breve as u8,
    ECircumflex = ((RootVowel::E as u8) << 2) | Shape::Circumflex as u8,
    OHorn = ((RootVowel::O as u8) << 2) | Shape::Horn as u8,
}

pub const ALL_OLD_VOWELS: [BaseVowel; 12] = [
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

pub const ALL_STRUCTURAL_VOWELS: [StructuralBaseVowel; 12] = [
    StructuralBaseVowel::Y,
    StructuralBaseVowel::U,
    StructuralBaseVowel::I,
    StructuralBaseVowel::E,
    StructuralBaseVowel::O,
    StructuralBaseVowel::A,
    StructuralBaseVowel::UHorn,
    StructuralBaseVowel::ACircumflex,
    StructuralBaseVowel::OCircumflex,
    StructuralBaseVowel::ABreve,
    StructuralBaseVowel::ECircumflex,
    StructuralBaseVowel::OHorn,
];

pub const INVALID: u8 = 0xFF;

pub const ROOT_SHAPE_TO_ID: [u8; 24] = [
    // A
    5, 7, 9, INVALID, // E
    3, 10, INVALID, INVALID, // I
    2, INVALID, INVALID, INVALID, // O
    4, 8, INVALID, 11, // U
    1, INVALID, INVALID, 6, // Y
    0, INVALID, INVALID, INVALID,
];

pub fn old_id(base: BaseVowel) -> usize {
    (base as u16 >> 5) as usize
}

#[inline(always)]
pub fn lut_id(base: StructuralBaseVowel) -> usize {
    ROOT_SHAPE_TO_ID[base as usize] as usize
}

#[inline(always)]
pub fn match_id(base: StructuralBaseVowel) -> usize {
    match base {
        StructuralBaseVowel::Y => 0,
        StructuralBaseVowel::U => 1,
        StructuralBaseVowel::I => 2,
        StructuralBaseVowel::E => 3,
        StructuralBaseVowel::O => 4,
        StructuralBaseVowel::A => 5,
        StructuralBaseVowel::UHorn => 6,
        StructuralBaseVowel::ACircumflex => 7,
        StructuralBaseVowel::OCircumflex => 8,
        StructuralBaseVowel::ABreve => 9,
        StructuralBaseVowel::ECircumflex => 10,
        StructuralBaseVowel::OHorn => 11,
    }
}

/// Packed 24 × 4-bit priority IDs into a single u128.
pub const PACKED_U128_ROOT_SHAPE_TO_ID: u128 = 5
    | (7 << 4)
    | (9 << 8)
    | (15 << 12)
    | (3 << 16)
    | (10 << 20)
    | (15 << 24)
    | (15 << 28)
    | (2 << 32)
    | (15 << 36)
    | (15 << 40)
    | (15 << 44)
    | (4 << 48)
    | (8 << 52)
    | (15 << 56)
    | (11 << 60)
    | (1 << 64)
    | (15 << 68)
    | (15 << 72)
    | (6 << 76)
    | (0 << 80)
    | (15 << 84)
    | (15 << 88)
    | (15 << 92);

#[inline(always)]
pub fn packed_u128_id(base: StructuralBaseVowel) -> usize {
    ((PACKED_U128_ROOT_SHAPE_TO_ID >> ((base as u32) << 2)) & 0xF) as usize
}

pub const PACKED_LO_ROOT_SHAPE_TO_ID: u64 = 5
    | (7 << 4)
    | (9 << 8)
    | (15 << 12)
    | (3 << 16)
    | (10 << 20)
    | (15 << 24)
    | (15 << 28)
    | (2 << 32)
    | (15 << 36)
    | (15 << 40)
    | (15 << 44)
    | (4 << 48)
    | (8 << 52)
    | (15 << 56)
    | (11 << 60);

pub const PACKED_HI_ROOT_SHAPE_TO_ID: u64 =
    1 | (15 << 4) | (15 << 8) | (6 << 12) | (0 << 16) | (15 << 20) | (15 << 24) | (15 << 28);

#[inline(always)]
pub fn packed_u64_id(base: StructuralBaseVowel) -> usize {
    let index = base as u32;

    if index < 16 {
        ((PACKED_LO_ROOT_SHAPE_TO_ID >> (index << 2)) & 0xF) as usize
    } else {
        ((PACKED_HI_ROOT_SHAPE_TO_ID >> ((index - 16) << 2)) & 0xF) as usize
    }
}
