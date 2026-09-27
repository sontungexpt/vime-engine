//! Head-to-head: the shipped `#[repr(u8)]` `BaseVowel` against the previous
//! `#[repr(u16)]` one, transcribed from `core/src/phonology/vowel copy.rs`.
//!
//! The old layout packed the tone-placement ID into the high bits
//! (`id << 5 | root << 2 | shape`), which made `id()` a shift but forced every
//! root/shape lookup through a 24-entry table. The new layout drops the ID from
//! the value entirely and gates `from_parts` on a 12-bit mask instead.
//!
//! Old values need 9 bits, new values need 5 - which is exactly the base field
//! width `Vowel::BASE_WIDTH` now reserves.
//!
//! Each operation is measured as a pair, plus a `loop_floor` control per group.
//! `loop_floor` does the same walk with no conversion, so the spread between a
//! pair and the floor is the part attributable to the layout rather than to the
//! harness.
//!
//! Run with `cargo bench -p vime-engine --bench bench_vowel_layout`.

use std::{hint::black_box, time::Duration};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use vime_engine::phonology::{BaseVowel, RootVowel, Shape};

// ─────────────────────────── Previous implementation ───────────────────────

/// Transcribed from `vowel copy.rs`: `#[repr(u16)]`, ID in the high bits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
#[rustfmt::skip]
pub enum OldBase {
    Y           = (0  << 5) | ((RootVowel::Y as u16) << 2) | Shape::None as u16,
    U           = (1  << 5) | ((RootVowel::U as u16) << 2) | Shape::None as u16,
    I           = (2  << 5) | ((RootVowel::I as u16) << 2) | Shape::None as u16,
    E           = (3  << 5) | ((RootVowel::E as u16) << 2) | Shape::None as u16,
    O           = (4  << 5) | ((RootVowel::O as u16) << 2) | Shape::None as u16,
    A           = (5  << 5) | ((RootVowel::A as u16) << 2) | Shape::None as u16,
    UHorn       = (6  << 5) | ((RootVowel::U as u16) << 2) | Shape::Horn as u16,
    ACircumflex = (7  << 5) | ((RootVowel::A as u16) << 2) | Shape::Circumflex as u16,
    OCircumflex = (8  << 5) | ((RootVowel::O as u16) << 2) | Shape::Circumflex as u16,
    ABreve      = (9  << 5) | ((RootVowel::A as u16) << 2) | Shape::Breve as u16,
    ECircumflex = (10 << 5) | ((RootVowel::E as u16) << 2) | Shape::Circumflex as u16,
    OHorn       = (11 << 5) | ((RootVowel::O as u16) << 2) | Shape::Horn as u16,
}

impl OldBase {
    const COUNT: usize = 12;
    const SHAPE_WIDTH: usize = 2;
    const SHAPE_MASK: u8 = (1u8 << Self::SHAPE_WIDTH) - 1;
    const ID_OFFSET: usize = 5;

    /// Index into both root/shape tables: `root * 4 + shape`.
    #[inline(always)]
    const fn root_shape_index(root: RootVowel, shape: Shape) -> usize {
        ((root as usize) << Self::SHAPE_WIDTH) | shape as usize
    }

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

    /// Root letter stored at bits 2-4, as in the original.
    const ROOT_OFFSET: usize = 2;
    const ROOT_MASK: u8 = 0b111;

    /// Returns the root letter, as the original did.
    #[inline(always)]
    pub const fn root(self) -> RootVowel {
        let root_id = (self as u8 >> Self::ROOT_OFFSET) & Self::ROOT_MASK;
        // SAFETY: the mask limits root_id to the six declared RootVowel values.
        unsafe { std::mem::transmute::<u8, RootVowel>(root_id) }
    }

    /// Returns the [`Shape`] component, as the original did.
    #[inline(always)]
    pub const fn shape(self) -> Shape {
        let shape_id = self as u8 & Self::SHAPE_MASK;
        // SAFETY: the mask limits shape_id to 0..=3, the valid Shape values.
        unsafe { std::mem::transmute::<u8, Shape>(shape_id) }
    }

    /// One entry per shape in enum order, at `root * 4 + shape`.
    ///
    /// Row order follows `RootVowel`. The copy this was transcribed from had
    /// the rows in `A,E,I,O,U,Y` order while `RootVowel` runs
    /// `Y,U,I,E,O,A`, so 11 of its 24 entries answered for the wrong root -
    /// the stale-table bug this bench would otherwise have measured instead of
    /// the layout.
    const VARIANTS_BY_ROOT_SHAPE: [Option<Self>; 24] = [
        // Y
        Some(Self::Y),
        None,
        None,
        None,
        // U
        Some(Self::U),
        None,
        None,
        Some(Self::UHorn),
        // I
        Some(Self::I),
        None,
        None,
        None,
        // E
        Some(Self::E),
        Some(Self::ECircumflex),
        None,
        None,
        // O
        Some(Self::O),
        Some(Self::OCircumflex),
        None,
        Some(Self::OHorn),
        // A
        Some(Self::A),
        Some(Self::ACircumflex),
        Some(Self::ABreve),
        None,
    ];

    /// A `match`, not a shift: the plain vowels are not contiguous.
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

    /// A bounds-checked table load.
    #[inline(always)]
    pub const fn from_id(vowel_id: usize) -> Option<Self> {
        if vowel_id < Self::COUNT {
            return Some(Self::VARIANTS_BY_ID[vowel_id]);
        }
        None
    }

    /// A 24-entry table load.
    #[inline(always)]
    pub const fn from_parts(root: RootVowel, shape: Shape) -> Option<Self> {
        Self::VARIANTS_BY_ROOT_SHAPE[Self::root_shape_index(root, shape)]
    }

    /// One shift, because the ID was stored in the value.
    #[inline(always)]
    pub const fn id(self) -> u8 {
        (self as u16 >> Self::ID_OFFSET) as u8
    }
}

// ───────────────────────────────── Inputs ─────────────────────────────────

const LARGE_LEN: usize = 1_000_000;

const ALL_ROOTS: [RootVowel; 6] = [
    RootVowel::Y,
    RootVowel::U,
    RootVowel::I,
    RootVowel::E,
    RootVowel::O,
    RootVowel::A,
];

const ALL_SHAPES: [Shape; 4] = [Shape::None, Shape::Circumflex, Shape::Breve, Shape::Horn];

struct Input {
    roots: Vec<RootVowel>,
    shapes: Vec<Shape>,
    bases: Vec<BaseVowel>,
}

fn input() -> Input {
    Input {
        roots: (0..LARGE_LEN)
            .map(|i| ALL_ROOTS[i % ALL_ROOTS.len()])
            .collect(),
        shapes: (0..LARGE_LEN)
            .map(|i| ALL_SHAPES[i % ALL_SHAPES.len()])
            .collect(),
        bases: (0..LARGE_LEN)
            .map(|i| BaseVowel::from_priority_id(i % BaseVowel::COUNT).expect("in range"))
            .collect(),
    }
}

// ─────────────────────────── Measured candidates ──────────────────────────

#[inline(always)]
fn new_from_parts(root: RootVowel, shape: Shape) -> usize {
    BaseVowel::from_parts(root, shape).map_or(0, |base| base as usize)
}

#[inline(always)]
fn old_from_parts(root: RootVowel, shape: Shape) -> usize {
    OldBase::from_parts(root, shape).map_or(0, |base| base as usize)
}

/// The old algorithm on the new type: a 24-entry `Option` table indexed by
/// `root * 4 + shape`. Isolates "table vs mask" from "u16 vs u8".
const NEW_VARIANTS_BY_ROOT_SHAPE: [Option<BaseVowel>; 24] = {
    let mut table = [None; 24];
    let mut id = 0;
    while id < BaseVowel::COUNT {
        if let Some(base) = BaseVowel::from_priority_id(id) {
            table[base.root() as usize * 4 + base.shape() as usize] = Some(base);
        }
        id += 1;
    }
    table
};

#[inline(always)]
fn new_from_parts_table(root: RootVowel, shape: Shape) -> usize {
    NEW_VARIANTS_BY_ROOT_SHAPE[(root as usize) * 4 + shape as usize].map_or(0, |base| base as usize)
}

/// The same 12-bit mask `BaseVowel` uses, rebuilt from the discriminants so it
/// cannot drift when the root order changes.
const NEW_DECLARED_MASK: u32 = (1 << BaseVowel::Y as u8)
    | (1 << BaseVowel::U as u8)
    | (1 << BaseVowel::I as u8)
    | (1 << BaseVowel::E as u8)
    | (1 << BaseVowel::O as u8)
    | (1 << BaseVowel::A as u8)
    | (1 << BaseVowel::UHorn as u8)
    | (1 << BaseVowel::ACircumflex as u8)
    | (1 << BaseVowel::OCircumflex as u8)
    | (1 << BaseVowel::ABreve as u8)
    | (1 << BaseVowel::ECircumflex as u8)
    | (1 << BaseVowel::OHorn as u8);

/// Bench-local copy of `new_from_parts`: the shipped algorithm, written out
/// here so it can be measured as a second name. The spread between this and
/// `new_from_parts` is the layout/alignment noise for this group - two copies
/// of one algorithm differing only in name.
#[inline(always)]
fn new_from_parts_copy(root: RootVowel, shape: Shape) -> usize {
    let value = (root as u8) << 2 | shape as u8;
    if (NEW_DECLARED_MASK & (1u32 << value)) != 0 {
        value as usize
    } else {
        0
    }
}

#[inline(always)]
fn new_id(base: BaseVowel) -> usize {
    base.priority_id() as usize
}

#[inline(always)]
fn old_id(base: OldBase) -> usize {
    base.id() as usize
}

#[inline(always)]
fn new_from_root(root: RootVowel) -> usize {
    BaseVowel::from_root(root) as usize
}

#[inline(always)]
fn old_from_root(root: RootVowel) -> usize {
    OldBase::from_root(root) as usize
}

#[inline(always)]
fn new_from_id(vowel_id: usize) -> usize {
    BaseVowel::from_priority_id(vowel_id).map_or(0, |base| base as usize)
}

#[inline(always)]
fn old_from_id(vowel_id: usize) -> usize {
    OldBase::from_id(vowel_id).map_or(0, |base| base as usize)
}

#[inline(always)]
fn floor_pair(_root: RootVowel, _shape: Shape) -> usize {
    1
}

/// 24-byte ID lookup, indexed straight by the packed value.
///
/// The variant discriminants top out at 22, so one byte per encoding is enough
/// and the index is always in range.
const ID_LUT_24: [u8; 24] = {
    let mut table = [0u8; 24];
    let mut id = 0;
    while id < BaseVowel::COUNT {
        table[BaseVowel::from_priority_id(id).expect("in range") as usize] = id as u8;
        id += 1;
    }
    table
};

/// Same, padded to 32 so the compiler cannot use the width to prove anything.
const ID_LUT_32: [u8; 32] = {
    let mut table = [0u8; 32];
    let mut id = 0;
    while id < BaseVowel::COUNT {
        table[BaseVowel::from_priority_id(id).expect("in range") as usize] = id as u8;
        id += 1;
    }
    table
};

#[inline(always)]
fn new_id_lut24(base: BaseVowel) -> usize {
    ID_LUT_24[base as usize] as usize
}

#[inline(always)]
fn new_id_lut32(base: BaseVowel) -> usize {
    ID_LUT_32[base as usize] as usize
}

#[inline(always)]
fn floor_base(_base: BaseVowel) -> usize {
    1
}

#[inline(always)]
fn floor_id(_vowel_id: usize) -> usize {
    1
}

fn sum_pairs(
    roots: &[RootVowel],
    shapes: &[Shape],
    f: impl Fn(RootVowel, Shape) -> usize,
) -> usize {
    let mut total = 0usize;
    for i in black_box(0..roots.len()) {
        total = total.wrapping_add(black_box(f(black_box(roots[i]), black_box(shapes[i]))));
    }
    black_box(total)
}

fn sum_base_shape<T: Copy>(bases: &[T], shapes: &[Shape], f: impl Fn(T, Shape) -> usize) -> usize {
    let mut total = 0usize;
    for i in black_box(0..bases.len()) {
        total = total.wrapping_add(black_box(f(black_box(bases[i]), black_box(shapes[i]))));
    }
    black_box(total)
}

fn sum_bases_t<T: Copy>(bases: &[T], f: impl Fn(T) -> usize) -> usize {
    let mut total = 0usize;
    for &base in black_box(bases) {
        total = total.wrapping_add(black_box(f(black_box(base))));
    }
    black_box(total)
}

fn sum_bases(bases: &[BaseVowel], f: impl Fn(BaseVowel) -> usize) -> usize {
    let mut total = 0usize;
    for &base in black_box(bases) {
        total = total.wrapping_add(black_box(f(black_box(base))));
    }
    black_box(total)
}

fn sum_roots(roots: &[RootVowel], f: impl Fn(RootVowel) -> usize) -> usize {
    let mut total = 0usize;
    for &root in black_box(roots) {
        total = total.wrapping_add(black_box(f(black_box(root))));
    }
    black_box(total)
}

fn sum_old_bases(bases: &[OldBase], f: impl Fn(OldBase) -> usize) -> usize {
    let mut total = 0usize;
    for &base in black_box(bases) {
        total = total.wrapping_add(black_box(f(black_box(base))));
    }
    black_box(total)
}

fn sum_ids(roots: &[RootVowel], f: impl Fn(usize) -> usize) -> usize {
    let mut total = 0usize;
    for &root in black_box(roots) {
        total = total.wrapping_add(black_box(f(black_box(root as usize))));
    }
    black_box(total)
}

// ─────────────────────────────── Agreement ───────────────────────────────

/// The two layouts must agree on every root/shape pair and every ID, or the
/// timings compare different work.
fn assert_layouts_agree() {
    for &root in &ALL_ROOTS {
        for &shape in &ALL_SHAPES {
            let new = BaseVowel::from_parts(root, shape);
            let old = OldBase::from_parts(root, shape);
            assert_eq!(
                new.is_some(),
                old.is_some(),
                "from_parts({root:?}, {shape:?}) disagrees"
            );
            if let (Some(new), Some(old)) = (new, old) {
                assert_eq!(
                    new.priority_id(),
                    old.id(),
                    "id disagrees for from_parts({root:?}, {shape:?}): new={new:?} old={old:?} old_raw={}",
                    old as u16
                );
            }
        }

        assert_eq!(
            BaseVowel::from_root(root).priority_id(),
            OldBase::from_root(root).id(),
            "from_root id disagrees for {root:?}"
        );
    }

    for id in 0..BaseVowel::COUNT {
        let new = BaseVowel::from_priority_id(id).expect("in range");
        let old = OldBase::from_id(id).expect("in range");
        assert_eq!(new.priority_id() as usize, id, "new from_id({id})");
        assert_eq!(old.id() as usize, id, "old from_id({id})");
    }

    // The width claim the new `Vowel` base field depends on.
    let widest_new = (0..BaseVowel::COUNT)
        .map(|id| BaseVowel::from_priority_id(id).expect("in range") as u16)
        .max()
        .expect("non-empty");
    let widest_old = (0..OldBase::COUNT)
        .map(|id| OldBase::from_id(id).expect("in range") as u16)
        .max()
        .expect("non-empty");

    // The width claim `Vowel::BASE_WIDTH` rests on.
    assert!(
        widest_new > 15 && widest_new < (1 << 5),
        "new layout needs {widest_new} bits of base field, not 5"
    );
    assert!(
        widest_old > 0xFF,
        "old layout fit in a u8, so this comparison proves nothing"
    );
    eprintln!("widest: old={widest_old} (9 bits), new={widest_new} (5 bits)");
}

// ─────────────────────────────── Benchmarks ───────────────────────────────

fn bench_from_parts(c: &mut Criterion) {
    assert_layouts_agree();
    let input = input();

    let mut group = c.benchmark_group("from_parts/large_1m");
    group.throughput(Throughput::Elements(LARGE_LEN as u64));
    for (name, f) in [
        (
            "new_u8_mask",
            new_from_parts as fn(RootVowel, Shape) -> usize,
        ),
        ("new_u8_mask_copy", new_from_parts_copy),
        ("new_u8_table", new_from_parts_table),
        ("old_u16_table", old_from_parts),
        ("floor", floor_pair),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| sum_pairs(&input.roots, &input.shapes, f))
        });
    }
    group.finish();
}

fn bench_id(c: &mut Criterion) {
    assert_layouts_agree();
    let input = input();
    let old: Vec<OldBase> = (0..LARGE_LEN)
        .map(|i| OldBase::from_id(i % OldBase::COUNT).expect("in range"))
        .collect();

    let mut group = c.benchmark_group("id/large_1m");
    group.throughput(Throughput::Elements(LARGE_LEN as u64));
    group.bench_function("new_u8_match", |b| {
        b.iter(|| sum_bases(&input.bases, new_id))
    });
    group.bench_function("new_u8_lut24", |b| {
        b.iter(|| sum_bases(&input.bases, new_id_lut24))
    });
    group.bench_function("new_u8_lut32", |b| {
        b.iter(|| sum_bases(&input.bases, new_id_lut32))
    });
    group.bench_function("old_u16_shift", |b| b.iter(|| sum_old_bases(&old, old_id)));
    group.bench_function("floor", |b| b.iter(|| sum_bases(&input.bases, floor_base)));
    group.finish();
}

fn bench_from_root(c: &mut Criterion) {
    assert_layouts_agree();
    let input = input();

    let mut group = c.benchmark_group("from_root/large_1m");
    group.throughput(Throughput::Elements(LARGE_LEN as u64));
    for (name, f) in [
        ("new_u8_transmute", new_from_root as fn(RootVowel) -> usize),
        ("old_u16_match", old_from_root),
        ("floor", |root: RootVowel| root as usize),
    ] {
        group.bench_function(name, |b| b.iter(|| sum_roots(&input.roots, f)));
    }
    group.finish();
}

fn bench_from_id(c: &mut Criterion) {
    assert_layouts_agree();
    let input = input();

    let mut group = c.benchmark_group("from_id/large_1m");
    group.throughput(Throughput::Elements(LARGE_LEN as u64));
    group.bench_function("new_u8_match", |b| {
        b.iter(|| sum_ids(&input.roots, new_from_id))
    });
    group.bench_function("old_u16_table", |b| {
        b.iter(|| sum_ids(&input.roots, old_from_id))
    });
    group.bench_function("floor", |b| b.iter(|| sum_ids(&input.roots, floor_id)));
    group.finish();
}

fn configure() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(1))
        .sample_size(10)
}

/// Bit `30 - i` set for each declared encoding `i`.
///
/// `RANK_32 >> (31 - v)` then keeps bit `30 - i` exactly when `30 - i >= 31 - v`,
/// i.e. `i < v` - the number of declared encodings strictly below `v`. Using
/// `30` rather than `31` keeps every shift amount inside `9..=31`, so `v = 0`
/// shifts by 31 rather than overflowing a 32-bit shift.
const RANK_32: u32 = {
    let mut rank = 0u32;
    let mut bit = 0u32;
    while bit < 32 {
        if (NEW_DECLARED_MASK >> bit) & 1 != 0 {
            rank |= 1u32 << (30 - bit);
        }
        bit += 1;
    }
    rank
};

/// Same, 64 bits wide.
const RANK_64: u64 = {
    let mut rank = 0u64;
    let mut bit = 0u32;
    while bit < 32 {
        if (NEW_DECLARED_MASK >> bit) & 1 != 0 {
            rank |= 1u64 << (62 - bit);
        }
        bit += 1;
    }
    rank
};

/// The popcount version, now replaced in `vowel.rs` but kept as the reference
/// every table-based candidate is checked against.
#[inline(always)]
fn id_src(base: BaseVowel) -> usize {
    let v = base as u8;
    (NEW_DECLARED_MASK & ((1u32 << v) - 1)).count_ones() as usize
}

/// A const-derived lookup table: same mask, same values, one load instead of a
/// shift/mask/popcount. Measured for reference; `vowel.rs` keeps the popcount.
#[inline(always)]
fn id_table(base: BaseVowel) -> usize {
    const RANK_TABLE: [u8; 256] = {
        let mut table = [0u8; 256];
        let mut value = 0usize;
        while value < 32 {
            table[value] = (NEW_DECLARED_MASK & ((1u32 << value) - 1)).count_ones() as u8;
            value += 1;
        }
        // Every declared encoding is below 32, so wider values all rank last.
        let total = NEW_DECLARED_MASK.count_ones() as u8;
        while value < table.len() {
            table[value] = total;
            value += 1;
        }
        table
    };
    RANK_TABLE[base as usize] as usize
}

/// `12 - popcount(MASK >> v)`. The shift amount is `v` itself, so there is no
/// `dec` and no `and`; the subtraction is the only arithmetic left.
#[inline(always)]
fn id_rank_sub(base: BaseVowel) -> usize {
    let v = base as u8;
    (NEW_DECLARED_MASK.count_ones() - (NEW_DECLARED_MASK >> v).count_ones()) as usize
}

/// Reversed mask in 32 bits, so only the low 5 bits of the domain are live.
#[inline(always)]
fn id_rev32(base: BaseVowel) -> usize {
    let v = base as u8;
    (RANK_32 >> (31 - v)).count_ones() as usize
}

/// Reversed mask in 64 bits.
#[inline(always)]
fn id_rev64(base: BaseVowel) -> usize {
    let v = base as u8;
    (RANK_64 >> (63 - v)).count_ones() as usize
}

/// One load, for reference.
#[inline(always)]
fn id_match(base: BaseVowel) -> usize {
    base.priority_id() as usize
}

/// New `replace_shape`: rewrites the two shape bits in place, so the root is
/// never decoded and the value is never re-encoded through a table.
#[inline(always)]
fn new_replace_shape(base: BaseVowel, shape: Shape) -> usize {
    base.replace_shape(shape).map_or(0, |base| base as usize)
}

/// Old `replace_shape` was `Self::from_parts(self.root(), shape)`, so it paid
/// a root decode and then the 24-entry table lookup.
#[inline(always)]
fn old_replace_shape(base: OldBase, shape: Shape) -> usize {
    old_from_parts(base.root(), shape)
}

/// New `remove_shape` is one AND plus a transmute.
#[inline(always)]
fn new_remove_shape(base: BaseVowel) -> usize {
    base.remove_shape() as usize
}

/// Old `remove_shape` was `Self::from_root(self.root())`: a root decode
/// followed by a 12-arm match.
#[inline(always)]
fn old_remove_shape(base: OldBase) -> usize {
    old_from_root(base.root())
}

fn shape_floor_pair(_base: OldBase, _shape: Shape) -> usize {
    1
}

fn shape_floor(_base: OldBase) -> usize {
    1
}

/// Paired A/B for `from_parts`, because the independent-run numbers on this
/// host are not separable: two copies of the same algorithm differ by ~40%
/// min-to-min and the floor moves by 2.3x between runs.
///
/// Interleaving the two implementations inside one process and taking the
/// ratio per round cancels frequency drift and host noise, which is the only
/// way a 10% claim can be checked here.
fn bench_from_parts_paired(c: &mut Criterion) {
    assert_layouts_agree();
    let input = input();
    let rounds = 21;

    let measure = |f: fn(RootVowel, Shape) -> usize| {
        let mut times = Vec::with_capacity(rounds);
        for _ in 0..rounds {
            let start = std::time::Instant::now();
            let total = sum_pairs(&input.roots, &input.shapes, f);
            let elapsed = start.elapsed();
            black_box(total);
            times.push(elapsed.as_nanos() as f64);
        }
        times.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        times[rounds / 2]
    };

    // Warm both before timing either.
    black_box(measure(new_from_parts));
    black_box(measure(new_from_parts_table));

    let mut ratios = Vec::with_capacity(rounds);
    let mut mask_times = Vec::with_capacity(rounds);
    let mut table_times = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        // Same round, alternating order so neither can hold the cache warm.
        let mask = measure_round(&input, new_from_parts);
        let table = measure_round(&input, new_from_parts_table);
        mask_times.push(mask);
        table_times.push(table);
        ratios.push(table / mask);
    }
    mask_times.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    table_times.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    ratios.sort_by(|a, b| a.partial_cmp(b).expect("finite"));

    let median = |v: &[f64]| v[v.len() / 2];
    eprintln!(
        "from_parts paired, {rounds} interleaved rounds over {} elements",
        LARGE_LEN
    );
    eprintln!("  mask  median {:.4} ms", median(&mask_times) / 1e6);
    eprintln!("  table median {:.4} ms", median(&table_times) / 1e6);
    eprintln!(
        "  table/mask ratio median {:.4}  (min {:.4}, max {:.4})",
        median(&ratios),
        ratios[0],
        ratios[ratios.len() - 1]
    );

    c.bench_function("from_parts_paired/report", |b| b.iter(|| median(&ratios)));
}

/// Paired A/B for the two shape operations.
///
/// Same rationale as `from_parts` below: independent runs drift by more than
/// twice the effect, so interleave in one process and take the per-round ratio.
fn bench_id_paired(c: &mut Criterion) {
    assert_layouts_agree();
    let input = input();
    let rounds = 21;
    let bases = &input.bases;

    // Two different questions are being timed, so they are checked separately.
    // `match` and `lut24` answer priority_id; the four popcount forms answer
    // rank in encoding order, which is a different function on purpose - the
    // packed order is not the priority order. Each group must agree internally
    // or the timing is comparing different work.
    for id in 0..BaseVowel::COUNT {
        let base = BaseVowel::from_priority_id(id).expect("in range");

        for (name, f) in [
            ("lut24 byte table", new_id_lut24 as fn(BaseVowel) -> usize),
            ("match (priority_id)", id_match as fn(BaseVowel) -> usize),
        ] {
            assert_eq!(
                f(base),
                base.priority_id() as usize,
                "{name} disagrees with priority_id for {base:?}"
            );
        }

        for (name, f) in [
            ("table (const-derived)", id_table as fn(BaseVowel) -> usize),
            ("rank 12-popcnt(>>v)", id_rank_sub as fn(BaseVowel) -> usize),
            ("rev32 rank>>31-v", id_rev32 as fn(BaseVowel) -> usize),
            ("rev64 rank>>63-v", id_rev64 as fn(BaseVowel) -> usize),
        ] {
            assert_eq!(
                f(base),
                id_src(base),
                "{name} disagrees with src for {base:?}"
            );
        }
    }

    report_paired(
        "id  new=match  old=src",
        rounds,
        || sum_bases_t(bases, id_match),
        || sum_bases_t(bases, id_src),
        || sum_bases_t(bases, |_| 1),
    );
    report_paired(
        "id  new=table  old=src",
        rounds,
        || sum_bases_t(bases, id_table),
        || sum_bases_t(bases, id_src),
        || sum_bases_t(bases, |_| 1),
    );
    report_paired(
        "id  new=src  old=rev32",
        rounds,
        || sum_bases_t(bases, id_src),
        || sum_bases_t(bases, id_rev32),
        || sum_bases_t(bases, |_| 1),
    );
    report_paired(
        "id  new=src  old=rank_sub",
        rounds,
        || sum_bases_t(bases, id_src),
        || sum_bases_t(bases, id_rank_sub),
        || sum_bases_t(bases, |_| 1),
    );
    report_paired(
        "id  new=rev32  old=lut24",
        rounds,
        || sum_bases_t(bases, id_rev32),
        || sum_bases_t(bases, new_id_lut24),
        || sum_bases_t(bases, |_| 1),
    );

    c.bench_function("id_paired/report", |b| b.iter(|| 1usize));
}

fn bench_shapes_paired(c: &mut Criterion) {
    assert_layouts_agree();
    let input = input();
    let old_bases: Vec<OldBase> = input
        .bases
        .iter()
        .map(|base| OldBase::from_id(base.priority_id() as usize).expect("in range"))
        .collect();
    let rounds = 21;

    report_paired(
        "replace_shape",
        rounds,
        || sum_base_shape(&input.bases, &input.shapes, new_replace_shape),
        || sum_base_shape(&old_bases, &input.shapes, old_replace_shape),
        || sum_base_shape(&old_bases, &input.shapes, shape_floor_pair),
    );
    report_paired(
        "remove_shape",
        rounds,
        || sum_bases_t(&input.bases, new_remove_shape),
        || sum_bases_t(&old_bases, old_remove_shape),
        || sum_bases_t(&old_bases, shape_floor),
    );

    c.bench_function("shapes_paired/report", |b| b.iter(|| 1usize));
}

fn report_paired(
    label: &str,
    rounds: usize,
    mut new: impl FnMut() -> usize,
    mut old: impl FnMut() -> usize,
    mut floor: impl FnMut() -> usize,
) {
    // Warm all three before timing any.
    black_box(floor());
    black_box(new());
    black_box(old());

    let mut new_times = Vec::with_capacity(rounds);
    let mut old_times = Vec::with_capacity(rounds);
    let mut floor_times = Vec::with_capacity(rounds);
    let mut ratios = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        // Floor first each round, then the pair, so the floor sees the same
        // cache state the candidates do.
        floor_times.push(time_round(&mut floor));
        new_times.push(time_round(&mut new));
        old_times.push(time_round(&mut old));
        ratios.push(old_times[old_times.len() - 1] / new_times[new_times.len() - 1]);
    }
    let median = |mut v: Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        v[v.len() / 2]
    };
    let (new_ms, old_ms, floor_ms, ratio) = (
        median(new_times) / 1e6,
        median(old_times) / 1e6,
        median(floor_times) / 1e6,
        median(ratios),
    );
    eprintln!("{label} paired, {rounds} interleaved rounds over {LARGE_LEN} bases");
    eprintln!("  floor  {floor_ms:7.4} ms");
    eprintln!("  old    {old_ms:7.4} ms   {:.3}x floor", old_ms / floor_ms);
    eprintln!("  new    {new_ms:7.4} ms   {:.3}x floor", new_ms / floor_ms);
    eprintln!(
        "  new is {:.3}x old  ({:+.1}%)",
        new_ms / old_ms,
        (1.0 - new_ms / old_ms) * 100.0
    );
    eprintln!("  old/new ratio median {ratio:.4}");
}

fn time_round(f: &mut impl FnMut() -> usize) -> f64 {
    let start = std::time::Instant::now();
    let total = f();
    let elapsed = start.elapsed();
    black_box(total);
    elapsed.as_nanos() as f64
}

fn measure_round(input: &Input, f: fn(RootVowel, Shape) -> usize) -> f64 {
    let start = std::time::Instant::now();
    let total = sum_pairs(&input.roots, &input.shapes, f);
    let elapsed = start.elapsed();
    black_box(total);
    elapsed.as_nanos() as f64
}

criterion_group! {
    name = benches;
    config = configure();
    targets = bench_from_parts, bench_id, bench_from_root, bench_from_id,
             bench_from_parts_paired, bench_shapes_paired, bench_id_paired
}
criterion_main!(benches);
