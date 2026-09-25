//! Compare the current `(BaseVowel, ExtendedBaseVowel)` pair with a merged
//! alternative where case lives inside a single `#[repr(transparent)] u16`
//! struct (`[6 reserved | 1 Upper | 4 Priority ID | 2 Shape | 3 Root]`).
//!
//! The merged type lives only in this benchmark; production code is unchanged.
//!
//! Run with:
//!   cargo bench --bench bench_base_vowel_merged
//!
//! Layout note: the merged value is bit-identical to today's `ExtendedBaseVowel`
//! raw bits (base fields at 0..8, case bit at 9), so `decode_vowel`/`encode_vowel`
//! results can be reinterpreted verbatim. The measured deltas are the operations
//! the wrapper forced us through: `.get()` before every field extraction, and a
//! separate `(value, is_upper)` everywhere render used encode.

use std::hint::black_box;
use std::mem::{size_of, transmute};
use std::time::{Duration, Instant};

use vime_engine::phonology::{
    decode_vowel, BaseVowel as ProdBaseVowel, ExtendedBaseVowel as ProdCased, RootVowel, Shape, Tone,
};

// ─── Candidate: the merged BaseVowel as noted ───

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct BaseVowel(u16);

impl BaseVowel {
    // ─────────────── Bit offsets & masks ───────────────
    const ROOT_OFFSET: u32 = 0;
    const SHAPE_OFFSET: u32 = 3;
    const ID_OFFSET: u32 = 5;
    const UPPER_OFFSET: u32 = 9;

    const ROOT_MASK: u16 = 0b000_0111;
    const SHAPE_MASK: u16 = 0b0001_1000; // bits 3..=4
    const ID_MASK: u16 = 0b0001_1110_0000; // bits 5..=8
    const UPPER_MASK: u16 = 1 << Self::UPPER_OFFSET;

    // ─────────────── Constants ───────────────
    pub const Y: Self = Self::new_raw(0, Shape::None, RootVowel::Y);
    pub const U: Self = Self::new_raw(1, Shape::None, RootVowel::U);
    pub const I: Self = Self::new_raw(2, Shape::None, RootVowel::I);
    pub const E: Self = Self::new_raw(3, Shape::None, RootVowel::E);
    pub const O: Self = Self::new_raw(4, Shape::None, RootVowel::O);
    pub const A: Self = Self::new_raw(5, Shape::None, RootVowel::A);
    pub const UHorn: Self = Self::new_raw(6, Shape::Horn, RootVowel::U);
    pub const ACircumflex: Self = Self::new_raw(7, Shape::Circumflex, RootVowel::A);
    pub const OCircumflex: Self = Self::new_raw(8, Shape::Circumflex, RootVowel::O);
    pub const ABreve: Self = Self::new_raw(9, Shape::Breve, RootVowel::A);
    pub const ECircumflex: Self = Self::new_raw(10, Shape::Circumflex, RootVowel::E);
    pub const OHorn: Self = Self::new_raw(11, Shape::Horn, RootVowel::O);

    #[inline(always)]
    const fn new_raw(id: u16, shape: Shape, root: RootVowel) -> Self {
        Self((id << Self::ID_OFFSET) | ((shape as u16) << Self::SHAPE_OFFSET) | (root as u16))
    }

    // SAFETY: bit layout is identical (base 0..8, case 9), so a plain
    // reinterpretation of a production ExtendedBaseVowel is a valid merged value.
    #[inline(always)]
    const fn from_cased(cased: ProdCased) -> Self {
        // SAFETY: ProdCased is repr(transparent) over u16 with the same layout.
        unsafe { transmute::<ProdCased, BaseVowel>(cased) }
    }

    // ─────────────── Case operations ───────────────
    #[inline(always)]
    pub const fn is_upper(self) -> bool {
        (self.0 & Self::UPPER_MASK) != 0
    }

    #[inline(always)]
    pub const fn to_upper(self) -> Self {
        Self(self.0 | Self::UPPER_MASK)
    }

    #[inline(always)]
    pub const fn to_lower(self) -> Self {
        Self(self.0 & !Self::UPPER_MASK)
    }

    #[inline(always)]
    pub const fn with_case(self, upper: bool) -> Self {
        if upper { self.to_upper() } else { self.to_lower() }
    }

    #[inline(always)]
    pub const fn bare(self) -> Self {
        self.to_lower()
    }

    // ─────────────── Getters ───────────────
    #[inline(always)]
    pub const fn id(self) -> usize {
        ((self.0 & Self::ID_MASK) >> Self::ID_OFFSET) as usize
    }

    #[inline(always)]
    pub const fn shape(self) -> Shape {
        let raw = (self.0 & Self::SHAPE_MASK) >> Self::SHAPE_OFFSET;
        // SAFETY: field is 2 bits (0..=3), matching Shape's u8 discriminants.
        unsafe { transmute::<u8, Shape>(raw as u8) }
    }

    #[inline(always)]
    pub const fn root(self) -> RootVowel {
        let raw = self.0 & Self::ROOT_MASK;
        // SAFETY: values built by new_raw only ever hold roots 0..=5, matching
        // RootVowel's u8 discriminants. (Bare corruption could hold 6..=7.)
        unsafe { transmute::<u8, RootVowel>(raw as u8) }
    }
}

// ─── Render: same 144-entry precomposed table, indexed off the merged value ───

const ENCODED_CHARS: [char; 144] = [
    'y', 'Y', 'ý', 'Ý', 'ỳ', 'Ỳ', 'ỷ', 'Ỷ', 'ỹ', 'Ỹ', 'ỵ', 'Ỵ', 'u', 'U', 'ú', 'Ú', 'ù', 'Ù', 'ủ',
    'Ủ', 'ũ', 'Ũ', 'ụ', 'Ụ', 'i', 'I', 'í', 'Í', 'ì', 'Ì', 'ỉ', 'Ỉ', 'ĩ', 'Ĩ', 'ị', 'Ị', 'e', 'E',
    'é', 'É', 'è', 'È', 'ẻ', 'Ẻ', 'ẽ', 'Ẽ', 'ẹ', 'Ẹ', 'o', 'O', 'ó', 'Ó', 'ò', 'Ò', 'ỏ', 'Ỏ', 'õ',
    'Õ', 'ọ', 'Ọ', 'a', 'A', 'á', 'Á', 'à', 'À', 'ả', 'Ả', 'ã', 'Ã', 'ạ', 'Ạ', 'ư', 'Ư', 'ứ', 'Ứ',
    'ừ', 'Ừ', 'ử', 'Ử', 'ữ', 'Ữ', 'ự', 'Ự', 'â', 'Â', 'ấ', 'Ấ', 'ầ', 'Ầ', 'ẩ', 'Ẩ', 'ẫ', 'Ẫ', 'ậ',
    'Ậ', 'ô', 'Ô', 'ố', 'Ố', 'ồ', 'Ồ', 'ổ', 'Ổ', 'ỗ', 'Ỗ', 'ộ', 'Ộ', 'ă', 'Ă', 'ắ', 'Ắ', 'ằ', 'Ằ',
    'ẳ', 'Ẳ', 'ẵ', 'Ẵ', 'ặ', 'Ặ', 'ê', 'Ê', 'ế', 'Ế', 'ề', 'Ề', 'ể', 'Ể', 'ễ', 'Ễ', 'ệ', 'Ệ', 'ơ',
    'Ơ', 'ớ', 'Ớ', 'ờ', 'Ờ', 'ở', 'Ở', 'ỡ', 'Ỡ', 'ợ', 'Ợ',
];

#[inline(always)]
fn merged_render(v: BaseVowel, tone: Tone) -> char {
    let idx = ((v.id() * 6 + tone as usize) << 1) | (v.is_upper() as usize);
    ENCODED_CHARS[idx]
}

// ─── Decode: the production decode tree, reinterpreted per the layout ───

#[inline(always)]
fn merged_decode(ch: char) -> Option<(BaseVowel, Tone)> {
    decode_vowel(ch).map(|(cased, tone)| (BaseVowel::from_cased(cased), tone))
}

// ─── Harness helpers ───

const VOWELS: [ProdBaseVowel; 12] = [
    ProdBaseVowel::Y,
    ProdBaseVowel::U,
    ProdBaseVowel::I,
    ProdBaseVowel::E,
    ProdBaseVowel::O,
    ProdBaseVowel::A,
    ProdBaseVowel::UHorn,
    ProdBaseVowel::ACircumflex,
    ProdBaseVowel::OCircumflex,
    ProdBaseVowel::ABreve,
    ProdBaseVowel::ECircumflex,
    ProdBaseVowel::OHorn,
];

const TONES: [Tone; 6] = [
    Tone::Flat,
    Tone::Acute,
    Tone::Grave,
    Tone::Hook,
    Tone::Tilde,
    Tone::Dot,
];

fn time(mut f: impl FnMut(), rounds: usize, iters: usize) -> Duration {
    let mut best = Duration::MAX;
    for _ in 0..rounds {
        let start = Instant::now();
        for _ in 0..iters {
            f();
        }
        best = best.min(start.elapsed());
    }
    best
}

fn report(name: &str, current: Duration, merged: Duration, count: usize, iters: usize) {
    let current_ns = current.as_nanos() as f64 / (iters * count) as f64;
    let merged_ns = merged.as_nanos() as f64 / (iters * count) as f64;
    println!(
        "{name:<22} current {current_ns:>7.2} ns  merged {merged_ns:>7.2} ns  merged/current {:.3}x",
        merged_ns / current_ns
    );
}

macro_rules! compare {
    ($name:literal, $count:expr, $current:expr, $merged:expr, $current_op:expr, $merged_op:expr, $rounds:expr, $iters:expr) => {{
        let a = time(
            || {
                for value in $current {
                    black_box($current_op(black_box(*value)));
                }
            },
            $rounds,
            $iters,
        );
        let b = time(
            || {
                for value in $merged {
                    black_box($merged_op(black_box(*value)));
                }
            },
            $rounds,
            $iters,
        );
        report($name, a, b, $count, $iters);
    }};
}

fn main() {
    // ─── Verify the merged type is bit-equivalent to production ───

    let mut chars_match = true;
    for &prod in &VOWELS {
        for upper in [false, true] {
            let prod_cased = ProdCased::with_case(prod, upper);
            let merged = BaseVowel::from_cased(prod_cased);
            assert_eq!(
                merged.bare(),
                BaseVowel::from_cased(ProdCased::lower(prod)),
                "{prod:?} bare mismatch"
            );
            assert_eq!(
                merged.is_upper(),
                upper,
                "{prod:?} case mismatch"
            );
            let prod_id = prod_cased.get().id();
            let prod_shape = prod_cased.get().shape();
            let prod_root = prod_cased.get().root();
            assert_eq!(merged.id(), prod_id, "{prod:?} id mismatch");
            assert_eq!(merged.shape(), prod_shape, "{prod:?} shape mismatch");
            assert_eq!(merged.root(), prod_root, "{prod:?} root mismatch");
            assert!(
                merged.to_upper().is_upper() && !merged.to_lower().is_upper(),
                "{prod:?} case flip broken"
            );
            let p = prod_cased.to_char();
            let m = merged_render(merged, Tone::Flat);
            chars_match &= p == m;
            assert_eq!(p, m, "{prod:?} flat render mismatch");
        }
    }
    let norm = |r: Option<(ProdCased, Tone)>| {
        r.map(|(c, t)| (c.get().id(), c.get().shape(), c.get().root(), c.is_upper(), t))
    };
    for code in 0x0000..=0x3000 {
        let Some(ch) = char::from_u32(code) else { continue };
        assert_eq!(
            norm(decode_vowel(ch)),
            norm(merged_decode(ch).map(|(v, t)| (ProdCased::with_case(
                ProdBaseVowel::from_id(v.bare().id()).unwrap(),
                v.is_upper()
            ), t))),
            "decode mismatch at U+{code:04X}"
        );
    }

    println!(
        "size: current ExtendedBaseVowel {} bytes, merged BaseVowel {} bytes (no wrapper)",
        size_of::<ProdCased>(),
        size_of::<BaseVowel>()
    );
    // Re-derive production chars for the merged table to prove byte equality.
    let mut table_match = true;
    for &ch in &ENCODED_CHARS {
        if let Some((cased, tone)) = decode_vowel(ch) {
            table_match &= ch == cased.to_char_tone(tone);
        }
    }
    println!("merged render table == production encode: {table_match}");
    println!("all 144 precomposed chars identical: {chars_match}");

    let rounds = 40;
    let iters = 200_000;
    let current: Vec<ProdCased> = (0..24)
        .map(|i| ProdCased::with_case(VOWELS[i % VOWELS.len()], i / VOWELS.len() != 0))
        .collect();
    let merged: Vec<BaseVowel> = current.iter().map(|&c| BaseVowel::from_cased(c)).collect();
    let count = current.len();

    println!("\nfield extraction (ns/op):");
    compare!("id", count, &current, &merged, |c: ProdCased| c.get().id(), |v: BaseVowel| v.id(), rounds, iters);
    compare!("shape", count, &current, &merged, |c: ProdCased| c.get().shape(), |v: BaseVowel| v.shape(), rounds, iters);
    compare!("root", count, &current, &merged, |c: ProdCased| c.get().root(), |v: BaseVowel| v.root(), rounds, iters);
    compare!("is_upper", count, &current, &merged, |c: ProdCased| c.is_upper(), |v: BaseVowel| v.is_upper(), rounds, iters);

    println!("\ncase transforms (ns/op):");
    compare!(
        "to_lower",
        count,
        &current,
        &merged,
        |c: ProdCased| ProdCased::lower(c.get()),
        |v: BaseVowel| v.to_lower(),
        rounds,
        iters
    );
    compare!(
        "case_flip",
        count,
        &current,
        &merged,
        |c: ProdCased| ProdCased::with_case(c.get(), !c.is_upper()),
        |v: BaseVowel| v.with_case(!v.is_upper()),
        rounds,
        iters
    );

    println!("\nrender (ns/op):");
    compare!(
        "to_char (flat)",
        count,
        &current,
        &merged,
        |c: ProdCased| c.to_char(),
        |v: BaseVowel| merged_render(v, Tone::Flat),
        rounds,
        iters
    );
    let c_tone = time(
        || {
            for (i, value) in current.iter().enumerate() {
                black_box(value.to_char_tone(black_box(TONES[i % TONES.len()])));
            }
        },
        rounds,
        iters,
    );
    let m_tone = time(
        || {
            for (i, value) in merged.iter().enumerate() {
                black_box(merged_render(black_box(*value), black_box(TONES[i % TONES.len()])));
            }
        },
        rounds,
        iters,
    );
    report("to_char_tone", c_tone, m_tone, count, iters);

    println!("\ndecode->render keystroke pipeline (ns/vowel):");
    let keys: Vec<char> = "aoeiyuAEIOUY".chars().collect();
    let c_pipe = time(
        || {
            for &ch in &keys {
                let (cased, tone) = decode_vowel(black_box(ch)).expect("vowel key");
                black_box(cased.to_char_tone(tone));
            }
        },
        rounds,
        iters,
    );
    let m_pipe = time(
        || {
            for &ch in &keys {
                let (vowel, tone) = merged_decode(black_box(ch)).expect("vowel key");
                black_box(merged_render(vowel, tone));
            }
        },
        rounds,
        iters,
    );
    report("ascii key->render", c_pipe, m_pipe, keys.len(), iters);
}