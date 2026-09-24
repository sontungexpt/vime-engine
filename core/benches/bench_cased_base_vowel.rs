//! Compare the current `Cased<BaseVowel>` representation with a packed `u16`
//! candidate. The candidate lives only in this benchmark; production code is
//! unchanged.
//!
//! Run with:
//!   cargo bench --bench bench_cased_base_vowel

use std::hint::black_box;
use std::mem::size_of;
use std::time::{Duration, Instant};

use vime_engine::phonology::{BaseVowel, CasedBaseVowel, Tone};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
struct PackedCasedBaseVowel(u16);

impl PackedCasedBaseVowel {
    const UPPER_SHIFT: u32 = 9;
    const UPPER_MASK: u16 = 1 << Self::UPPER_SHIFT;
    const VALUE_MASK: u16 = Self::UPPER_MASK - 1;

    #[inline(always)]
    const fn new(value: BaseVowel, is_upper: bool) -> Self {
        Self((value as u16) | ((is_upper as u16) << Self::UPPER_SHIFT))
    }

    #[inline(always)]
    const fn value(self) -> BaseVowel {
        // SAFETY: BaseVowel's discriminants are all below VALUE_MASK.
        unsafe { std::mem::transmute::<u16, BaseVowel>(self.0 & Self::VALUE_MASK) }
    }

    #[inline(always)]
    const fn is_upper(self) -> bool {
        self.0 & Self::UPPER_MASK != 0
    }

    #[inline(always)]
    const fn set_upper(&mut self, is_upper: bool) {
        if is_upper {
            self.0 |= Self::UPPER_MASK;
        } else {
            self.0 &= !Self::UPPER_MASK;
        }
    }

    #[inline(always)]
    const fn set_value(&mut self, value: BaseVowel) {
        self.0 = (self.0 & Self::UPPER_MASK) | value as u16;
    }

    #[inline(always)]
    const fn to_char(self) -> char {
        vime_engine::phonology::encode_vowel(self.value(), Tone::Flat, self.is_upper())
    }

    #[inline(always)]
    const fn to_char_tone(self, tone: Tone) -> char {
        vime_engine::phonology::encode_vowel(self.value(), tone, self.is_upper())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
struct IdCasedBaseVowel(u8);

impl IdCasedBaseVowel {
    const ID_MASK: u8 = 0x0F;
    const CASE_MASK: u8 = 0x10;

    #[inline(always)]
    const fn new(value: BaseVowel, is_upper: bool) -> Self {
        Self(value.id() as u8 | ((is_upper as u8) << 4))
    }

    #[inline(always)]
    fn value(self) -> BaseVowel {
        // SAFETY: new/set_value only store BaseVowel IDs (0..12), and the
        // private field prevents callers from constructing invalid IDs safely.
        unsafe { BaseVowel::from_id_unchecked((self.0 & Self::ID_MASK) as usize) }
    }

    #[inline(always)]
    const fn is_upper(self) -> bool {
        self.0 & Self::CASE_MASK != 0
    }

    #[inline(always)]
    const fn set_upper(&mut self, is_upper: bool) {
        if is_upper {
            self.0 |= Self::CASE_MASK;
        } else {
            self.0 &= !Self::CASE_MASK;
        }
    }

    #[inline(always)]
    const fn set_value(&mut self, value: BaseVowel) {
        self.0 = (self.0 & Self::CASE_MASK) | value.id() as u8;
    }

    #[inline(always)]
    fn to_char(self) -> char {
        vime_engine::phonology::encode_vowel(self.value(), Tone::Flat, self.is_upper())
    }

    #[inline(always)]
    fn to_char_tone(self, tone: Tone) -> char {
        vime_engine::phonology::encode_vowel(self.value(), tone, self.is_upper())
    }
}

const VOWELS: [BaseVowel; 12] = [
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

fn report(
    name: &str,
    current: Duration,
    packed: Duration,
    operations_per_pass: usize,
    iters: usize,
) {
    let current_ns = current.as_nanos() as f64 / (iters * operations_per_pass) as f64;
    let packed_ns = packed.as_nanos() as f64 / (iters * operations_per_pass) as f64;
    println!(
        "{name:<20} current {current_ns:>7.2} ns/op  packed {packed_ns:>7.2} ns/op  packed/current {:.3}x",
        packed_ns / current_ns
    );
}

#[inline(always)]
fn nucleus_bases_loop(vowels: &[CasedBaseVowel]) -> [BaseVowel; 3] {
    let mut bases = [BaseVowel::A; 3];
    for (dst, vowel) in bases.iter_mut().zip(vowels) {
        *dst = *vowel.get();
    }
    bases
}

#[inline(always)]
fn nucleus_bases_match(vowels: &[CasedBaseVowel]) -> [BaseVowel; 3] {
    use BaseVowel::A;
    match vowels {
        [] => [A; 3],
        [a] => [*a.get(), A, A],
        [a, b] => [*a.get(), *b.get(), A],
        [a, b, c] => [*a.get(), *b.get(), *c.get()],
        _ => unreachable!("nucleus capacity is 3"),
    }
}

fn main() {
    let current: [CasedBaseVowel; 24] = std::array::from_fn(|i| {
        CasedBaseVowel::new(VOWELS[i % VOWELS.len()], i / VOWELS.len() != 0)
    });
    let packed: [PackedCasedBaseVowel; 24] = std::array::from_fn(|i| {
        PackedCasedBaseVowel::new(VOWELS[i % VOWELS.len()], i / VOWELS.len() != 0)
    });
    let id_packed: [IdCasedBaseVowel; 24] = std::array::from_fn(|i| {
        IdCasedBaseVowel::new(VOWELS[i % VOWELS.len()], i / VOWELS.len() != 0)
    });
    let nuclei = [&current[..0], &current[..1], &current[..2], &current[..3]];

    // Verify all vowel/case pairs and all tone encodings before timing.
    for &vowel in &VOWELS {
        for is_upper in [false, true] {
            let a = CasedBaseVowel::new(vowel, is_upper);
            let b = PackedCasedBaseVowel::new(vowel, is_upper);
            let c = IdCasedBaseVowel::new(vowel, is_upper);
            assert_eq!(*a.get(), b.value());
            assert_eq!(*a.get(), c.value());
            assert_eq!(a.is_upper(), b.is_upper());
            assert_eq!(a.is_upper(), c.is_upper());
            assert_eq!(a.to_char(), b.to_char());
            assert_eq!(a.to_char(), c.to_char());
            for tone in TONES {
                assert_eq!(a.to_char_tone(tone), b.to_char_tone(tone));
                assert_eq!(a.to_char_tone(tone), c.to_char_tone(tone));
            }

            let mut a = a;
            let mut b = b;
            let mut c = c;
            a.set_upper(!is_upper);
            b.set_upper(!is_upper);
            c.set_upper(!is_upper);
            assert_eq!(a.is_upper(), b.is_upper());
            assert_eq!(a.is_upper(), c.is_upper());
            a.set_value(BaseVowel::OHorn);
            b.set_value(BaseVowel::OHorn);
            c.set_value(BaseVowel::OHorn);
            assert_eq!((*a.get(), a.is_upper()), (b.value(), b.is_upper()));
            assert_eq!((*a.get(), a.is_upper()), (c.value(), c.is_upper()));
        }
    }

    println!(
        "size: current {} bytes, u16-packed {} bytes, ID-packed {} bytes",
        size_of::<CasedBaseVowel>(),
        size_of::<PackedCasedBaseVowel>(),
        size_of::<IdCasedBaseVowel>()
    );
    let rounds = 40;
    let iters = 200_000;
    let count = current.len();

    println!("nucleus array construction (ns/nucleus):");
    for (len, nucleus) in nuclei.iter().enumerate() {
        let loop_time = time(
            || {
                black_box(nucleus_bases_loop(black_box(nucleus)));
            },
            rounds,
            iters,
        );
        let match_time = time(
            || {
                black_box(nucleus_bases_match(black_box(nucleus)));
            },
            rounds,
            iters,
        );
        let loop_ns = loop_time.as_nanos() as f64 / iters as f64;
        let match_ns = match_time.as_nanos() as f64 / iters as f64;
        println!(
            "  len {len}: loop {loop_ns:.2}, match {match_ns:.2} ns  (match/loop {:.3}x)",
            match_ns / loop_ns
        );
    }

    macro_rules! compare_id {
        ($name:literal, $current_op:expr, $packed_op:expr) => {{
            let a = time(
                || {
                    for value in &current {
                        black_box($current_op(black_box(*value)));
                    }
                },
                rounds,
                iters,
            );
            let b = time(
                || {
                    for value in &id_packed {
                        black_box($packed_op(black_box(*value)));
                    }
                },
                rounds,
                iters,
            );
            report($name, a, b, count, iters);
        }};
    }

    macro_rules! compare {
        ($name:literal, $current_op:expr, $packed_op:expr) => {{
            let a = time(
                || {
                    for value in &current {
                        black_box($current_op(black_box(*value)));
                    }
                },
                rounds,
                iters,
            );
            let b = time(
                || {
                    for value in &packed {
                        black_box($packed_op(black_box(*value)));
                    }
                },
                rounds,
                iters,
            );
            report($name, a, b, count, iters);
        }};
    }

    compare!(
        "get_value",
        |v: CasedBaseVowel| *v.get(),
        |v: PackedCasedBaseVowel| v.value()
    );
    compare!(
        "get_case",
        |v: CasedBaseVowel| v.is_upper(),
        |v: PackedCasedBaseVowel| v.is_upper()
    );
    compare_id!(
        "get_value (ID)",
        |v: CasedBaseVowel| *v.get(),
        |v: IdCasedBaseVowel| v.value()
    );
    compare_id!(
        "get_case (ID)",
        |v: CasedBaseVowel| v.is_upper(),
        |v: IdCasedBaseVowel| v.is_upper()
    );
    compare!(
        "to_char",
        |v: CasedBaseVowel| v.to_char(),
        |v: PackedCasedBaseVowel| v.to_char()
    );
    compare_id!(
        "to_char (ID)",
        |v: CasedBaseVowel| v.to_char(),
        |v: IdCasedBaseVowel| v.to_char()
    );

    let current_tone = time(
        || {
            for (i, value) in current.iter().enumerate() {
                black_box(value.to_char_tone(black_box(TONES[i % TONES.len()])));
            }
        },
        rounds,
        iters,
    );
    let packed_tone = time(
        || {
            for (i, value) in packed.iter().enumerate() {
                black_box(value.to_char_tone(black_box(TONES[i % TONES.len()])));
            }
        },
        rounds,
        iters,
    );
    report("to_char_tone", current_tone, packed_tone, count, iters);
    let id_tone = time(
        || {
            for (i, value) in id_packed.iter().enumerate() {
                black_box(value.to_char_tone(black_box(TONES[i % TONES.len()])));
            }
        },
        rounds,
        iters,
    );
    report("to_char_tone (ID)", current_tone, id_tone, count, iters);

    let current_case_setter = time(
        || {
            for value in &current {
                let mut value = black_box(*value);
                value.set_upper(black_box(!value.is_upper()));
                black_box(value);
            }
        },
        rounds,
        iters,
    );
    let packed_case_setter = time(
        || {
            for value in &packed {
                let mut value = black_box(*value);
                value.set_upper(black_box(!value.is_upper()));
                black_box(value);
            }
        },
        rounds,
        iters,
    );
    report(
        "set_upper",
        current_case_setter,
        packed_case_setter,
        count,
        iters,
    );
    let id_case_setter = time(
        || {
            for value in &id_packed {
                let mut value = black_box(*value);
                value.set_upper(black_box(!value.is_upper()));
                black_box(value);
            }
        },
        rounds,
        iters,
    );
    report(
        "set_upper (ID)",
        current_case_setter,
        id_case_setter,
        count,
        iters,
    );

    let current_value_setter = time(
        || {
            for (i, value) in current.iter().enumerate() {
                let mut value = black_box(*value);
                value.set_value(black_box(VOWELS[(i + 1) % VOWELS.len()]));
                black_box(value);
            }
        },
        rounds,
        iters,
    );
    let packed_value_setter = time(
        || {
            for (i, value) in packed.iter().enumerate() {
                let mut value = black_box(*value);
                value.set_value(black_box(VOWELS[(i + 1) % VOWELS.len()]));
                black_box(value);
            }
        },
        rounds,
        iters,
    );
    report(
        "set_value",
        current_value_setter,
        packed_value_setter,
        count,
        iters,
    );
    let id_value_setter = time(
        || {
            for (i, value) in id_packed.iter().enumerate() {
                let mut value = black_box(*value);
                value.set_value(black_box(VOWELS[(i + 1) % VOWELS.len()]));
                black_box(value);
            }
        },
        rounds,
        iters,
    );
    report(
        "set_value (ID)",
        current_value_setter,
        id_value_setter,
        count,
        iters,
    );
}
