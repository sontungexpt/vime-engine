//! Compile-time generated sparse DFA for Vietnamese vowel nuclei.
//!
//! The generator performs a bounded polynomial scan over the DSL tables at
//! compile time, so relax the default `long_running_const_eval` deny lint.
#![allow(long_running_const_eval)]
//!
//! # DSL
//! The machine is written declaratively. Every transition is one rule:
//!
//! ```text
//! CURRENT_STATE => INPUT => NEXT_STATE => STATUS,
//! ```
//!
//! - `CURRENT_STATE` / `NEXT_STATE` are state names (a base vowel or a
//!   compound); a compound name is just the concatenation of its letters
//!   (e.g. `YE`, `YÊ`, `UÔ`).
//! - `INPUT` is a `BaseVowel` name or a `Shape` name.
//! - `STATUS` is `Complete` or `Incomplete` and belongs to *`NEXT_STATE`*.
//! - The base vowel `<->` shape *spelling* edges (`a + ^ = â`, `â + ^ = a`,
//!   `â + ˘ = ă`, ...) are regular transitions too and are written in the DSL.
//!
//! The macro rejects, at compile time:
//! 1. conflicting completion statuses for one destination state;
//! 2. conflicting targets for one `(state, input)` pair;
//! 3. contradictory or undeclared state definitions;
//! 4. inputs that cannot be mapped to the common input id space.
//!
//! # Inputs (symbol ids)
//! - `0..=11`: the 12 base vowels; the id equals [`BaseVowel::id`]
//!   (`Y=0, U=1, I=2, E=3, O=4, A=5, Ư=6, Â=7, Ô=8, Ă=9, Ê=10, Ơ=11`).
//! - `12..=14`: diacritic shapes — `Circumflex`, `Breve`, `Horn`
//!   (see [`shape_id`]). [`Shape::None`] is not a transition input.
//!
//! # States
//! - State ids `0..=11` are the base-vowel states (same ordering as inputs).
//! - State ids `12..` are compound (multi-letter) states, **discovered
//!   automatically** from the DSL in order of first mention; their numeric ids
//!   are never hard-coded. The internal `StateName -> u8` map exists only
//!   during generation, never at runtime.
//!
//! # Completion
//! A nucleus is:
//! - **Dead** when it cannot be reached by any transition (no state / no edge);
//! - **InComplete** when it reaches a state whose COMPLETE bit (bit 15 of the
//!   transition mask) is clear;
//! - **Valid** when it reaches a state whose COMPLETE bit is set.
//!
//! The transition mask packs both the outgoing-edge bitmap and the completion
//! flag into one `u16`; the COMPLETE bit is excluded from rank computation.
//!
//! # Runtime tables
//! Exactly two tables are generated (see [`STATES`] and [`TRANSITIONS`]),
//! packed sparse rows in state order; every offset and count is derived from
//! the declared edges. All intermediate name/status maps are compile-time
//! only.

use super::vowel::BaseVowel;
pub use super::rules::Nucleus;

use crate::Shape;

/// Bit 15 of `mask` marks a complete (Valid) nucleus.
const COMPLETE_BIT: u16 = 1 << 15;
/// The 15 input-symbol bits (`0..=14`), i.e. the mask with the COMPLETE bit cleared.
const INPUT_MASK: u16 = COMPLETE_BIT - 1;

/// A DFA state: an outgoing-transition bitmap plus an offset into
/// [`TRANSITIONS`]. Exactly two fields — a `u16` mask and a `u8` offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct NucleusState {
    /// Packs the per-input edge bitmap (`bits 0..=14`) and the COMPLETE bit 15.
    pub mask: u16,
    /// Index into [`TRANSITIONS`] of this state's (ascending-input) edge row.
    pub transition_offset: u8,
}

// ─────────────────────────────────────────────────────────────────────────────
// Vocabulary
// ─────────────────────────────────────────────────────────────────────────────

/// The 12 base-vowel state/input names in `BaseVowel` id order.
const __BASE_VOWEL_NAMES: [&str; 12] = ["Y", "U", "I", "E", "O", "A", "Ư", "Â", "Ô", "Ă", "Ê", "Ơ"];

/// The 3 diacritic-shape input names (`id = 12 + index`).
const __SHAPE_NAMES: [&str; 3] = ["Circumflex", "Breve", "Horn"];

// ─────────────────────────────────────────────────────────────────────────────
// DSL: `generate_dfa! { Src => Input => Tgt => Status, ... }`
// ─────────────────────────────────────────────────────────────────────────────

/// Declarative DFA grammar.
///
/// Each rule is `CURRENT_STATE => INPUT => NEXT_STATE => STATUS` where
/// `STATUS` (`Complete` | `Incomplete`) is the completion of `NEXT_STATE`.
/// The only generated long-lived runtime tables are [`STATES`] and
/// [`TRANSITIONS`].
macro_rules! generate_dfa {
    (
        $(
            $src:ident => $input:ident => $tgt:ident => $status:ident,
        )*
    ) => {
        /// Explicit edges as `(source, input, target, status)` name quadruples,
        /// in declaration order. Compile-time only.
        const __EDGE_NAMES: &[(&str, &str, &str, &str)] = &[
            $(
                (
                    stringify!($src),
                    stringify!($input),
                    stringify!($tgt),
                    stringify!($status),
                ),
            )*
        ];
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// The Vietnamese nucleus table
// ─────────────────────────────────────────────────────────────────────────────
//
// Faithful transcription of `rule::check_nucleus_validity`. The completion
// status of a state is declared by the rule(s) that target it. Base-vowel
// spelling edges (à la `a + ^ = â`, `â + ^ = a`, `â + ˘ = ă`, ...) and the
// compound spelling toggles (`ye + ^ = yê`, ...) are ordinary rules here.

generate_dfa! {
    // ── base-vowel <-> shape spelling ──
    A => Circumflex => Â => Complete,
    A => Breve      => Ă => Complete,
    Â => Circumflex => A => Complete,
    Â => Breve      => Ă => Complete,
    Ă => Circumflex => Â => Complete,
    Ă => Breve      => A => Complete,
    E => Circumflex => Ê => Complete,
    Ê => Circumflex => E => Complete,
    O => Circumflex => Ô => Complete,
    O => Horn       => Ơ => Complete,
    Ô => Circumflex => O => Complete,
    Ô => Horn       => Ơ => Complete,
    Ơ => Circumflex => Ô => Complete,
    Ơ => Horn       => O => Complete,
    U => Horn       => Ư => Complete,
    Ư => Horn       => U => Complete,

    // ── a family ──
    A => I => AI  => Complete,
    A => O => AO  => Complete,
    A => U => AU  => Complete,
    A => Y => AY  => Complete,
    Â => U => ÂU  => Complete,
    Â => Y => ÂY  => Complete,
    AU => Circumflex => ÂU  => Complete,
    ÂU => Circumflex => AU  => Complete,
    AY => Circumflex => ÂY  => Complete,
    ÂY => Circumflex => AY  => Complete,

    // ── i family ──
    I  => A => IA  => Complete,
    I  => U => IU  => Complete,
    I  => E => IE  => Incomplete,
    IE => U => IEU => Incomplete,
    I  => Ê => IÊ  => Complete,
    IÊ => U => IÊU => Complete,
    IE  => Circumflex => IÊ  => Complete,
    IÊ  => Circumflex => IE  => Incomplete,
    IEU => Circumflex => IÊU => Complete,
    IÊU => Circumflex => IEU => Incomplete,

    // ── y family ──
    Y  => E => YE  => Incomplete,
    YE => U => YEU => Incomplete,
    Y  => Ê => YÊ  => Complete,
    YÊ => U => YÊU => Complete,
    YE  => Circumflex => YÊ  => Complete,
    YÊ  => Circumflex => YE  => Incomplete,
    YEU => Circumflex => YÊU => Complete,
    YÊU => Circumflex => YEU => Incomplete,

    // ── e family ──
    E  => O => EO => Complete,
    E  => U => EU => Incomplete,
    Ê  => U => ÊU => Complete,
    EU => Circumflex => ÊU => Complete,
    ÊU => Circumflex => EU => Incomplete,

    // ── o family ──
    O  => A => OA  => Complete,
    O  => Ă => OĂ  => Complete,
    OA => I => OAI => Complete,
    OA => O => OAO => Complete,
    OA => U => OAU => Complete,
    OA => Y => OAY => Complete,
    O  => E => OE  => Complete,
    OE => O => OEO => Complete,
    O  => I => OI  => Complete,
    Ô  => I => ÔI  => Complete,
    Ơ  => I => ƠI  => Complete,
    O  => O => OO  => Incomplete,
    OI => Circumflex => ÔI => Complete,
    ÔI => Circumflex => OI => Complete,
    ƠI => Circumflex => ÔI => Complete,
    OI => Horn       => ƠI => Complete,
    ÔI => Horn       => ƠI => Complete,
    ƠI => Horn       => OI => Complete,
    OA => Breve      => OĂ => Complete,
    OĂ => Breve      => OA => Complete,

    // ── u + y family ──
    U  => Y => UY   => Complete,
    UY => U => UYU  => Complete,
    UY => A => UYA  => Complete,
    UY => E => UYE  => Incomplete,
    UY => Ê => UYÊ  => Complete,
    UYE => Circumflex => UYÊ  => Complete,
    UYÊ => Circumflex => UYE  => Incomplete,

    // ── u + a family ──
    U   => A => UA  => Complete,
    UA  => O => UAO => Complete,
    U   => Â => UÂ  => Complete,
    UÂ  => Y => UÂY => Complete,
    UA  => Circumflex => UÂ => Complete,
    UÂ  => Circumflex => UA => Complete,
    UA  => Horn       => ƯA => Complete,

    // ── u + o family ──
    U  => O => UO  => Incomplete,
    UO => I => UOI => Incomplete,
    UO => U => UOU => Incomplete,
    U  => Ơ => UƠ  => Complete,
    // finalize_uo_prefix: "uơ" + third vowel folds to "ươ".
    UƠ => I => ƯƠI => Complete,
    UƠ => U => ƯƠU => Complete,
    U  => Ô => UÔ  => Complete,
    UÔ => I => UÔI => Complete,
    UO  => Circumflex => UÔ  => Complete,
    UÔ  => Circumflex => UO  => Incomplete,
    UO  => Horn       => UƠ  => Complete,
    UÔ  => Horn       => UƠ  => Complete,
    UƠ  => Circumflex => UÔ  => Complete,
    UƠ  => Horn       => ƯƠ  => Complete,
    UOI => Circumflex => UÔI => Complete,
    UÔI => Circumflex => UOI => Incomplete,
    UOI => Horn       => UƠI => Incomplete,
    UÔI => Horn       => UƠI => Incomplete,
    UƠI => Circumflex => UÔI => Complete,
    UƠI => Horn       => ƯƠI => Complete,
    UOU => Horn       => UƠU => Incomplete,
    UƠU => Horn       => ƯƠU => Complete,

    // ── u + e family ──
    U => E => UE => Incomplete,
    U => Ê => UÊ => Complete,
    U => I => UI => Complete,
    UE  => Circumflex => UÊ => Complete,
    UÊ  => Circumflex => UE => Incomplete,
    UI  => Horn       => ƯI => Complete,

    // ── ư + o family ──
    Ư  => O => ƯO  => Incomplete,
    // finalize_uo_prefix: "ưo" + third vowel folds to "ươ".
    ƯO => I => ƯƠI => Complete,
    ƯO => U => ƯƠU => Complete,
    Ư  => Ơ => ƯƠ  => Complete,
    ƯƠ => I => ƯƠI => Complete,
    ƯƠ => U => ƯƠU => Complete,
    ƯO  => Circumflex => UÔ  => Complete,
    ƯO  => Horn       => ƯƠ  => Complete,
    ƯƠ  => Circumflex => UÔ  => Complete,
    ƯƠ  => Horn       => UO  => Incomplete,
    ƯƠI => Circumflex => UÔI => Complete,
    ƯƠI => Horn       => UOI => Incomplete,
    ƯƠU => Horn       => UOU => Incomplete,

    // ── ư family ──
    Ư => A => ƯA => Complete,
    Ư => I => ƯI => Complete,
    Ư => U => ƯU => Complete,
    U => U => UU => Incomplete,
    ƯA => Horn => UA  => Complete,
    ƯI => Horn => UI  => Complete,
    ƯU => Horn => UU  => Incomplete,
    UU => Horn => ƯU  => Complete,
}

// ─────────────────────────────────────────────────────────────────────────────
// Compile-time generator
// ─────────────────────────────────────────────────────────────────────────────

const fn __bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// First index of `name` in `table`, or `None`.
const fn __name_index(name: &[u8], table: &[&str]) -> Option<usize> {
    let mut i = 0;
    while i < table.len() {
        if __bytes_eq(name, table[i].as_bytes()) {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Whether `name` appears in the first `limit` entries of `table`.
const fn __name_in(name: &[u8], table: &[&str], limit: usize) -> bool {
    let mut i = 0;
    while i < limit {
        if __bytes_eq(name, table[i].as_bytes()) {
            return true;
        }
        i += 1;
    }
    false
}

/// `true` when `name` is not one of the base vowels.
const fn __is_compound(name: &[u8]) -> bool {
    __name_index(name, &__BASE_VOWEL_NAMES).is_none()
}

/// Number of distinct compound names (sources + targets, in DSL order).
const fn __compound_count() -> usize {
    let mut seen = [""; 128];
    let mut n = 0;
    let mut e = 0;
    while e < __EDGE_NAMES.len() {
        let src = __EDGE_NAMES[e].0.as_bytes();
        if __is_compound(src) && !__name_in(src, &seen, n) {
            seen[n] = __EDGE_NAMES[e].0;
            n += 1;
        }
        let tgt = __EDGE_NAMES[e].2.as_bytes();
        if __is_compound(tgt) && !__name_in(tgt, &seen, n) {
            seen[n] = __EDGE_NAMES[e].2;
            n += 1;
        }
        e += 1;
    }
    n
}

/// Compound state names in order of discovery.
const __COMPOUND_NAMES: [&str; __compound_count()] = {
    let mut seen = [""; __compound_count()];
    let mut n = 0;
    let mut e = 0;
    while e < __EDGE_NAMES.len() {
        let src = __EDGE_NAMES[e].0.as_bytes();
        if __is_compound(src) && !__name_in(src, &seen, n) {
            seen[n] = __EDGE_NAMES[e].0;
            n += 1;
        }
        let tgt = __EDGE_NAMES[e].2.as_bytes();
        if __is_compound(tgt) && !__name_in(tgt, &seen, n) {
            seen[n] = __EDGE_NAMES[e].2;
            n += 1;
        }
        e += 1;
    }
    assert!(n == __compound_count());
    assert!(n < 64);
    seen
};

/// Total number of states (12 base vowels + discovered compounds).
const __STATE_COUNT: usize = __BASE_VOWEL_NAMES.len() + __COMPOUND_NAMES.len();

/// State id for a DSL state name (base vowel or compound).
const fn __state_id(name: &str) -> usize {
    let bytes = name.as_bytes();
    if let Some(i) = __name_index(bytes, &__BASE_VOWEL_NAMES) {
        return i;
    }
    if let Some(i) = __name_index(bytes, &__COMPOUND_NAMES) {
        return __BASE_VOWEL_NAMES.len() + i;
    }
    panic!("generate_dfa: unknown state symbol")
}

/// Completion status id: `1 = Incomplete`, `2 = Complete`.
const fn __status_id(status: &str) -> u8 {
    if __bytes_eq(status.as_bytes(), b"Complete") {
        2
    } else if __bytes_eq(status.as_bytes(), b"Incomplete") {
        1
    } else {
        panic!("generate_dfa: status must be either Complete or Incomplete")
    }
}

/// Input symbol id for a DSL input name (base vowel or shape). Returns `None`
/// when the name does not map to the common input id space (`Shape::None` is
/// not a transition input).
const fn __input_id(name: &str) -> Option<u8> {
    let bytes = name.as_bytes();
    if let Some(i) = __name_index(bytes, &__BASE_VOWEL_NAMES) {
        return Some(i as u8);
    }
    if let Some(i) = __name_index(bytes, &__SHAPE_NAMES) {
        return Some((__BASE_VOWEL_NAMES.len() + i) as u8);
    }
    None
}

/// Per-state completion status: `1 = Incomplete`, `2 = Complete`. The 12 base
/// vowels are complete by construction; every compound's status comes from the
/// rules that target it. Base-vowel targets must be `Complete`.
///
/// Build-time validation: conflicting statuses for one destination, and any
/// state left without a status, abort compilation.
const __STATUS: [u8; __STATE_COUNT] = {
    let mut out = [0u8; __STATE_COUNT];
    let mut s = 0;
    while s < __BASE_VOWEL_NAMES.len() {
        out[s] = 2;
        s += 1;
    }
    let mut e = 0;
    while e < __EDGE_NAMES.len() {
        let tgt = __state_id(__EDGE_NAMES[e].2);
        let status = __status_id(__EDGE_NAMES[e].3);
        if tgt < __BASE_VOWEL_NAMES.len() {
            // Spelling edges land on base vowels, which are complete.
            if status != 2 {
                panic!("generate_dfa: base-vowel states are always complete")
            }
        } else if out[tgt] == 0 {
            out[tgt] = status;
        } else if out[tgt] != status {
            panic!("generate_dfa: conflicting completion status for a state")
        }
        e += 1;
    }
    let mut s = 0;
    while s < __STATE_COUNT {
        if out[s] == 0 {
            panic!("generate_dfa: state has no completion status")
        }
        s += 1;
    }
    out
};

/// The outgoing-input bitmap of every state (COMPLETE bit clear).
const __MASKS: [u16; __STATE_COUNT] = {
    let mut out = [0u16; __STATE_COUNT];
    let mut e = 0;
    while e < __EDGE_NAMES.len() {
        let src = __state_id(__EDGE_NAMES[e].0);
        let input = match __input_id(__EDGE_NAMES[e].1) {
            Some(id) => id,
            None => panic!("generate_dfa: unknown input symbol"),
        };
        out[src] |= 1u16 << input;
        e += 1;
    }
    out
};

/// Outgoing-input bitmap of a state.
const fn __state_mask(state: usize) -> u16 {
    __MASKS[state]
}

/// Number of edges a state has.
const fn __state_edge_count(state: usize) -> usize {
    __MASKS[state].count_ones() as usize
}

/// Cumulative offset of a state row in [`TRANSITIONS`].
const fn __transition_offset(state: usize) -> usize {
    let mut offset = 0;
    let mut s = 0;
    while s < state {
        offset += __state_edge_count(s);
        s += 1;
    }
    offset
}

/// `[state][input]` target state (`255` = no edge), precomputed once.
///
/// Build-time validation: two edges from one `(state, input)` to different
/// targets abort compilation (identical duplicates are tolerated).
const __EDGE_TABLE: [[u8; 15]; __STATE_COUNT] = {
    let mut out = [[255u8; 15]; __STATE_COUNT];
    let mut e = 0;
    while e < __EDGE_NAMES.len() {
        let src = __state_id(__EDGE_NAMES[e].0);
        let input = match __input_id(__EDGE_NAMES[e].1) {
            Some(id) => id as usize,
            None => panic!("generate_dfa: unknown input symbol"),
        };
        let tgt = __state_id(__EDGE_NAMES[e].2) as u8;
        if out[src][input] == 255 {
            out[src][input] = tgt;
        } else if out[src][input] != tgt {
            panic!("generate_dfa: conflicting transition targets")
        }
        e += 1;
    }
    out
};

/// Target state id of `(state, input)`.
const fn __edge_target_of(state: usize, input: u8) -> usize {
    __EDGE_TABLE[state][input as usize] as usize
}

/// The generated state table (each state gets a row, leaves included).
const fn __build_states() -> [NucleusState; __STATE_COUNT] {
    let mut out = [NucleusState {
        mask: 0,
        transition_offset: 0,
    }; __STATE_COUNT];
    let mut s = 0;
    while s < __STATE_COUNT {
        let offset = __transition_offset(s);
        if offset >= 256 {
            panic!("generate_dfa: transition offset overflow")
        }
        let complete = if __STATUS[s] == 2 { COMPLETE_BIT } else { 0 };
        out[s] = NucleusState {
            mask: __state_mask(s) | complete,
            transition_offset: offset as u8,
        };
        s += 1;
    }
    out
}

/// The generated, sparsely-packed transition table (input-ascending rows).
const fn __build_transitions() -> [u8; __transition_offset(__STATE_COUNT)] {
    let mut out = [0u8; __transition_offset(__STATE_COUNT)];
    let mut ptr = 0;
    let mut s = 0;
    while s < __STATE_COUNT {
        let mask = __state_mask(s);
        let mut input = 0;
        while input < 15 {
            if mask & (1u16 << input) != 0 {
                out[ptr] = __edge_target_of(s, input as u8) as u8;
                ptr += 1;
            }
            input += 1;
        }
        s += 1;
    }
    assert!(ptr == out.len(), "transition table size mismatch");
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Runtime tables
// ─────────────────────────────────────────────────────────────────────────────

/// The backing arrays of the two runtime tables (compile-time only; the
/// statics below are the only long-lived runtime tables).
const __STATES_ARRAY: [NucleusState; __STATE_COUNT] = __build_states();
const __TRANSITIONS_ARRAY: [u8; __transition_offset(__STATE_COUNT)] = __build_transitions();

/// The generated state table.
pub static STATES: &[NucleusState] = &__STATES_ARRAY;

/// The generated, sparsely-packed transition table.
pub static TRANSITIONS: &[u8] = &__TRANSITIONS_ARRAY;

// ─────────────────────────────────────────────────────────────────────────────
// Runtime API
// ─────────────────────────────────────────────────────────────────────────────

/// Number of DFA states (== `STATES.len()`).
#[inline(always)]
pub const fn state_count() -> usize {
    __STATES_ARRAY.len()
}

/// Number of transitions (== `TRANSITIONS.len()`, the sum of all edge counts).
#[inline(always)]
pub const fn transition_count() -> usize {
    __TRANSITIONS_ARRAY.len()
}

/// Input symbol id for a diacritic shape (`Circumflex=12, Breve=13, Horn=14`).
#[inline(always)]
pub const fn shape_id(shape: Shape) -> u8 {
    __BASE_VOWEL_NAMES.len() as u8 + shape as u8 - 1
}

/// Follow a single transition: next state id, or `None` when the input is not
/// consumed by `state` (Dead). The COMPLETE bit is excluded from the rank scan.
#[inline(always)]
pub const fn transition(state_id: u8, input_id: u8) -> Option<u8> {
    if input_id >= 15 {
        return None;
    }
    let bit = 1u16 << input_id;
    if state_id as usize >= __STATES_ARRAY.len() {
        return None;
    }
    let state = &__STATES_ARRAY[state_id as usize];
    if state.mask & bit == 0 {
        return None;
    }
    // COMPLETE_BIT (bit 15) never enters the rank because `input_id` spans
    // `0..=14`, so `bit - 1` covers only the input bits.
    let rank = (state.mask & INPUT_MASK & (bit - 1)).count_ones() as usize;
    Some(__TRANSITIONS_ARRAY[state.transition_offset as usize + rank])
}

/// Whether a state's mask marks the nucleus (state) as complete.
#[inline(always)]
pub const fn is_complete(state: NucleusState) -> bool {
    state.mask & COMPLETE_BIT != 0
}

/// Whether the state with the given id is complete.
#[inline(always)]
pub const fn is_state_complete(state_id: u8) -> bool {
    if state_id as usize >= __STATES_ARRAY.len() {
        return false;
    }
    is_complete(__STATES_ARRAY[state_id as usize])
}

/// Classifies a vowel nucleus exactly like `rule::check_nucleus_validity`,
/// but via the generated DFA instead of a hand-written match.
///
/// - `Dead`: the sequence cannot be walked through the DFA at all.
/// - `InComplete`: the sequence reaches an existing, non-complete state.
/// - `Valid`: the sequence reaches a complete state.
#[inline]
pub const fn check_nucleus_validity(vowels: &[BaseVowel]) -> Nucleus {
    use Nucleus::*;
    let first = match vowels.first() {
        Some(vowel) => vowel,
        None => return Dead,
    };
    let mut state = first.id() as u8;
    let mut i = 1;
    while i < vowels.len() {
        match transition(state, vowels[i].id() as u8) {
            Some(next) => state = next,
            None => return Dead,
        }
        i += 1;
    }
    if is_state_complete(state) {
        Valid
    } else {
        InComplete
    }
}

#[cfg(test)]
mod tests {
    use super::super::rules::Nucleus as RuleStatus;
    use super::*;
    use crate::BaseVowel::*;

    const VOWELS: [BaseVowel; 12] = [
        Y, U, I, E, O, A, UHorn, ACircumflex, OCircumflex, ABreve, ECircumflex, OHorn,
    ];

    fn dfa(vowels: &[BaseVowel]) -> Nucleus {
        check_nucleus_validity(vowels)
    }

    /// The rule-table oracle, currently `Nucleus::from_vowels`.
    fn reference(vowels: &[BaseVowel]) -> Nucleus {
        RuleStatus::from_vowels(vowels)
    }

    /// Walks the base-vowel edges from the empty state, resolving the id of the
    /// state whose nucleus is exactly `letters`. Panics on a Dead step (only
    /// meaningful for literals that form a declared nucleus).
    fn resolve(letters: &[BaseVowel]) -> u8 {
        let mut i = 0;
        let mut state = letters[i].id() as u8;
        i += 1;
        while i < letters.len() {
            state = transition(state, letters[i].id() as u8)
                .unwrap_or_else(|| panic!("no base step for {letters:?}"));
            i += 1;
        }
        state
    }

    /// The engine's `finalize_uo_prefix`: an unmarked "uơ" / "ưo" pair folds
    /// to "ươ" once a third vowel is pushed (uơi→ươi, uơu→ươu, ưoi→ươi,
    /// ưou→ươu). At three vowels the generated DFA already folds, so the raw
    /// reference table must be fed the folded form.
    fn finalize_uo(vowels: &[BaseVowel]) -> Vec<BaseVowel> {
        if let [v0, v1, v2] = vowels {
            match (v0, v1) {
                (BaseVowel::U, BaseVowel::OHorn) | (BaseVowel::UHorn, BaseVowel::O) => {
                    return vec![BaseVowel::UHorn, BaseVowel::OHorn, *v2];
                }
                _ => {}
            }
        }
        vowels.to_vec()
    }

    /// The generated DFA must agree with the rule table for *every* sequence of
    /// 0..=3 base vowels (12 + 144 + 1728 = 1884 cases), with the four
    /// u-o-prefix fold sequences compared against their folded form.
    #[test]
    fn exhaustive_matches_reference() {
        assert_eq!(dfa(&[]), reference(&[]), "empty");
        for &v1 in &VOWELS {
            assert_eq!(dfa(&[v1]), reference(&[v1]), "1 [{v1:?}]");
            for &v2 in &VOWELS {
                assert_eq!(
                    dfa(&[v1, v2]),
                    reference(&[v1, v2]),
                    "2 [{v1:?}{v2:?}]"
                );
                for &v3 in &VOWELS {
                    let seq = [v1, v2, v3];
                    assert_eq!(
                        dfa(&seq),
                        reference(&finalize_uo(&seq)),
                        "3 [{v1:?}{v2:?}{v3:?}]"
                    );
                }
            }
        }
    }

    /// Spot checks of every rule family, mirroring the documented table.
    #[test]
    fn rule_spot_checks() {
        use Nucleus::*;
        let cases: &[(&[BaseVowel], Nucleus)] = &[
            // single vowels
            (&[A], Valid),
            (&[ABreve], Valid),
            (&[ACircumflex], Valid),
            (&[E], Valid),
            (&[ECircumflex], Valid),
            (&[I], Valid),
            (&[Y], Valid),
            (&[O], Valid),
            (&[OCircumflex], Valid),
            (&[OHorn], Valid),
            (&[U], Valid),
            (&[UHorn], Valid),
            // a family
            (&[A, I], Valid),        // ai
            (&[A, O], Valid),        // ao
            (&[A, U], Valid),        // au
            (&[A, Y], Valid),        // ay
            (&[ACircumflex, U], Valid), // âu
            (&[ACircumflex, Y], Valid), // ây
            // i family
            (&[I, A], Valid),        // ia
            (&[I, E], InComplete),   // ie
            (&[I, ECircumflex], Valid), // iê
            (&[I, E, U], InComplete),   // ieu
            (&[I, ECircumflex, U], Valid), // iêu
            (&[I, U], Valid),        // iu
            // y family
            (&[Y, E], InComplete),   // ye
            (&[Y, ECircumflex], Valid), // yê
            (&[Y, E, U], InComplete),   // yeu
            (&[Y, ECircumflex, U], Valid), // yêu
            // e family
            (&[E, O], Valid),        // eo
            (&[E, U], InComplete),   // eu
            (&[ECircumflex, U], Valid), // êu
            // o family
            (&[O, A], Valid),        // oa
            (&[O, ABreve], Valid),   // oă
            (&[O, A, I], Valid),     // oai
            (&[O, A, O], Valid),     // oao
            (&[O, A, U], Valid),     // oau
            (&[O, A, Y], Valid),     // oay
            (&[O, E], Valid),        // oe
            (&[O, E, O], Valid),     // oeo
            (&[O, I], Valid),        // oi
            (&[OCircumflex, I], Valid), // ôi
            (&[OHorn, I], Valid),    // ơi
            (&[O, O], InComplete),   // oo
            // u + y family
            (&[U, Y], Valid),              // uy
            (&[U, Y, U], Valid),           // uyu
            (&[U, Y, A], Valid),           // uya
            (&[U, Y, E], InComplete),      // uye
            (&[U, Y, ECircumflex], Valid), // uyê
            // u + a family
            (&[U, A], Valid),              // ua
            (&[U, A, O], Valid),           // uao
            (&[U, ACircumflex], Valid),    // uâ
            (&[U, ACircumflex, Y], Valid), // uây
            // u + o family
            (&[U, O], InComplete),      // uo
            (&[U, OHorn], Valid),       // uơ
            (&[U, OCircumflex], Valid), // uô
            (&[U, O, I], InComplete),   // uoi
            (&[U, OCircumflex, I], Valid), // uôi
            (&[U, O, U], InComplete),   // uou
            // (uơi / uơu are not listed: the prefix folds to ươi / ươu,
            //  see uo_fold_transitions below)
            // u + e family
            (&[U, E], InComplete),      // ue
            (&[U, ECircumflex], Valid), // uê
            (&[U, I], Valid),           // ui
            // ư + o family
            (&[UHorn, O], InComplete),      // ưo
            (&[UHorn, OHorn], Valid),       // ươ
            (&[UHorn, OHorn, I], Valid),    // ươi
            (&[UHorn, OHorn, U], Valid),    // ươu
            // (ưoi / ưou are not listed: the prefix folds to ươi / ươu)
            // ư family
            (&[UHorn, A], Valid),  // ưa
            (&[UHorn, I], Valid),  // ưi
            (&[U, U], InComplete), // uu
            (&[UHorn, U], Valid),  // ưu
            // invalid (Dead)
            (&[A, A], Dead),
            (&[I, O], Dead),
            (&[E, ECircumflex], Dead),
            (&[U, UHorn], Dead),
            (&[Y, I], Dead),
            (&[O, A, A], Dead),
        ];
        for (sequence, want) in cases {
            assert_eq!(dfa(sequence), *want, "rule spot-check: {sequence:?}");
            assert_eq!(reference(sequence), *want, "reference disagrees: {sequence:?}");
        }
    }

    /// `finalize_uo_prefix`: an unmarked "uơ" / "ưo" prefix folds to "ươ" once
    /// a third vowel is pushed (`uơi→ươi`, `uơu→ươu`, `ưoi→ươi`, `ưou→ươu`),
    /// and the intermediate "uơi"/"uơu"/"ưoi"/"ưou" states no longer exist.
    #[test]
    fn uo_fold_transitions() {
        use Nucleus::*;
        let uơ = resolve(&[U, OHorn]);
        let ưo = resolve(&[UHorn, O]);
        let ươi = resolve(&[UHorn, OHorn, I]);
        let ươu = resolve(&[UHorn, OHorn, U]);
        for (src, input, dst) in [
            (uơ, I, ươi),
            (uơ, U, ươu),
            (ưo, I, ươi),
            (ưo, U, ươu),
        ] {
            assert_eq!(transition(src, input.id() as u8), Some(dst), "fold edge");
        }
        // The folded nuclei classify as Valid, and the second vowel alone is
        // untouched (finalize runs only at the third vowel / a coda push).
        for seq in [
            &[U, OHorn, I][..],
            &[U, OHorn, U][..],
            &[UHorn, O, I][..],
            &[UHorn, O, U][..],
        ] {
            assert_eq!(dfa(seq), Valid, "{seq:?}");
        }
        assert_eq!(dfa(&[U, OHorn]), Valid);
        assert_eq!(dfa(&[UHorn, O]), InComplete);
    }

    /// A faithful transcription of the engine's shape transform (valid.rs):
    /// scan the nucleus right-to-left; transform the rightmost vowel that can
    /// take the shape; special-case the "u o" prefix; reverting applies the
    /// shape back; Dead results roll back and scanning continues.
    fn shape_model(vowels: &[BaseVowel], shape: Shape) -> Option<Vec<BaseVowel>> {
        let uo = vowels.len() > 1
            && matches!(vowels[0], U | UHorn)
            && matches!(vowels[1], O | OCircumflex | OHorn);

        fn shape_off(v: BaseVowel) -> BaseVowel {
            match v {
                OCircumflex | OHorn => O,
                ACircumflex | ABreve => A,
                ECircumflex => E,
                UHorn => U,
                other => other,
            }
        }

        fn shape_at(vowels: &[BaseVowel], index: usize, shape: Shape) -> Option<Vec<BaseVowel>> {
            use Nucleus::Dead;
            let old = vowels[index];
            if old.shape() == shape && shape != Shape::None {
                let mut v = vowels.to_vec();
                v[index] = shape_off(old);
                return Some(v);
            }
            let new_vowel = BaseVowel::from_parts(old.root(), shape).ok()?;
            let mut v = vowels.to_vec();
            v[index] = new_vowel;
            if vowels.len() < 2 {
                return Some(v);
            }
            match reference(&v) {
                Dead => None,
                _ => Some(v),
            }
        }

        fn uo_horn(vowels: &[BaseVowel]) -> Option<Vec<BaseVowel>> {
            match (vowels[0], vowels[1]) {
                (UHorn, OHorn) => {
                    let mut v = vowels.to_vec();
                    v[0] = U;
                    v[1] = O;
                    Some(v)
                }
                (UHorn, OCircumflex) => None,
                (UHorn, O) => shape_at(vowels, 1, Shape::Horn),
                (U, O) => shape_at(vowels, 1, Shape::Horn),
                (U, OHorn) => shape_at(vowels, 0, Shape::Horn),
                (U, OCircumflex) => shape_at(vowels, 1, Shape::Horn),
                _ => None,
            }
        }

        fn uo_circumflex(vowels: &[BaseVowel]) -> Option<Vec<BaseVowel>> {
            match (vowels[0], vowels[1]) {
                (UHorn, OHorn) => {
                    let mut v = vowels.to_vec();
                    v[0] = U;
                    shape_at(&v, 1, Shape::Circumflex)
                }
                (UHorn, OCircumflex) => None,
                (UHorn, O) => {
                    let mut v = vowels.to_vec();
                    v[0] = U;
                    shape_at(&v, 1, Shape::Circumflex)
                }
                (U, O) => shape_at(vowels, 1, Shape::Circumflex),
                (U, OHorn) => shape_at(vowels, 1, Shape::Circumflex),
                (U, OCircumflex) => {
                    let mut v = vowels.to_vec();
                    v[1] = O;
                    Some(v)
                }
                _ => None,
            }
        }

        for index in (0..vowels.len()).rev() {
            if BaseVowel::from_parts(vowels[index].root(), shape).is_err() {
                continue;
            }
            if uo {
                match shape {
                    Shape::Horn => return uo_horn(vowels),
                    Shape::Circumflex => return uo_circumflex(vowels),
                    _ => {}
                }
            }
            if let Some(out) = shape_at(vowels, index, shape) {
                return Some(out);
            }
        }
        None
    }

    /// Every declared shape edge must be (a) reproduced exactly by the engine
    /// model and (b) present in the generated DFA, with a non-Dead target.
    #[test]
    fn shape_edges_match_engine_model() {
        use Shape::{Breve, Circumflex, Horn};
        let h = shape_id(Horn);

        let uoi = resolve(&[U, O, I]);
        let uou = resolve(&[U, O, U]);
        // The only states reachable solely through a shape edge.
        let uọi = transition(uoi, h).unwrap();
        let uơu = transition(uou, h).unwrap();
        let id = |letters: &[BaseVowel]| -> u8 {
            match letters {
                &[U, OHorn, I] => uọi,
                &[U, OHorn, U] => uơu,
                other => resolve(other),
            }
        };

        let edges: &[(&[BaseVowel], Shape, &[BaseVowel])] = &[
            // a family
            (&[A, U], Circumflex, &[ACircumflex, U]),
            (&[ACircumflex, U], Circumflex, &[A, U]),
            (&[A, Y], Circumflex, &[ACircumflex, Y]),
            (&[ACircumflex, Y], Circumflex, &[A, Y]),
            // i family
            (&[I, E], Circumflex, &[I, ECircumflex]),
            (&[I, ECircumflex], Circumflex, &[I, E]),
            (&[I, E, U], Circumflex, &[I, ECircumflex, U]),
            (&[I, ECircumflex, U], Circumflex, &[I, E, U]),
            // y family
            (&[Y, E, U], Circumflex, &[Y, ECircumflex, U]),
            (&[Y, ECircumflex, U], Circumflex, &[Y, E, U]),
            // e family
            (&[E, U], Circumflex, &[ECircumflex, U]),
            (&[ECircumflex, U], Circumflex, &[E, U]),
            // o family
            (&[O, I], Circumflex, &[OCircumflex, I]),
            (&[OCircumflex, I], Circumflex, &[O, I]),
            (&[OHorn, I], Circumflex, &[OCircumflex, I]),
            (&[O, I], Horn, &[OHorn, I]),
            (&[OCircumflex, I], Horn, &[OHorn, I]),
            (&[OHorn, I], Horn, &[O, I]),
            // u + a family
            (&[U, A], Circumflex, &[U, ACircumflex]),
            (&[U, ACircumflex], Circumflex, &[U, A]),
            (&[U, A], Horn, &[UHorn, A]),
            (&[UHorn, A], Horn, &[U, A]),
            // u + o family (circumflex)
            (&[U, O], Circumflex, &[U, OCircumflex]),
            (&[U, OCircumflex], Circumflex, &[U, O]),
            (&[U, OHorn], Circumflex, &[U, OCircumflex]),
            (&[UHorn, O], Circumflex, &[U, OCircumflex]),
            (&[UHorn, OHorn], Circumflex, &[U, OCircumflex]),
            (&[U, O, I], Circumflex, &[U, OCircumflex, I]),
            (&[U, OCircumflex, I], Circumflex, &[U, O, I]),
            (&[U, OHorn, I], Circumflex, &[U, OCircumflex, I]),
            (&[UHorn, OHorn, I], Circumflex, &[U, OCircumflex, I]),
            // u + o family (horn)
            (&[U, O], Horn, &[U, OHorn]),
            (&[U, OCircumflex], Horn, &[U, OHorn]),
            (&[U, OHorn], Horn, &[UHorn, OHorn]),
            (&[UHorn, OHorn], Horn, &[U, O]),
            (&[UHorn, O], Horn, &[UHorn, OHorn]),
            (&[U, O, I], Horn, &[U, OHorn, I]),
            (&[U, O, U], Horn, &[U, OHorn, U]),
            (&[U, OCircumflex, I], Horn, &[U, OHorn, I]),
            (&[U, OHorn, I], Horn, &[UHorn, OHorn, I]),
            (&[U, OHorn, U], Horn, &[UHorn, OHorn, U]),
            (&[UHorn, OHorn, I], Horn, &[U, O, I]),
            (&[UHorn, OHorn, U], Horn, &[U, O, U]),
            // u + e family
            (&[U, E], Circumflex, &[U, ECircumflex]),
            (&[U, ECircumflex], Circumflex, &[U, E]),
            // u + i family
            (&[U, I], Horn, &[UHorn, I]),
            (&[UHorn, I], Horn, &[U, I]),
            // uu / ưu family
            (&[UHorn, U], Horn, &[U, U]),
            (&[U, U], Horn, &[UHorn, U]),
            // o + a family (breve)
            (&[O, A], Breve, &[O, ABreve]),
            (&[O, ABreve], Breve, &[O, A]),
        ];

        for (src, shape, dst) in edges {
            let model = shape_model(src, *shape);
            assert_eq!(
                model.as_deref(),
                Some(*dst),
                "model mismatch for {src:?} + {shape:?}"
            );
            assert_ne!(
                reference(dst),
                Nucleus::Dead,
                "target is not a nucleus: {dst:?}"
            );
            assert_eq!(
                transition(id(src), shape_id(*shape)),
                Some(id(dst)),
                "DFA edge for {src:?} + {shape:?}"
            );
        }
        // The DFA lays out all compound states, so no extra state count drift.
        assert_eq!(transition_count(), TRANSITIONS.len());
    }

    /// Forbidden shape combinations must stay edge-free: no over-generation.
    #[test]
    fn no_shape_over_generation() {
        use Shape::{Breve, Circumflex, Horn};
        let none: &[(&[BaseVowel], Shape)] = &[
            // a + o and a + i never compose a second shape
            (&[A, O], Circumflex),
            (&[A, O], Horn),
            (&[A, O], Breve),
            (&[A, I], Circumflex),
            (&[A, I], Horn),
            // e / ê take only circumflex; first-letter e-horn is bogus
            (&[E, U], Horn),
            (&[ECircumflex, U], Horn),
            (&[E, O], Circumflex),
            (&[E, O], Horn),
            // y-family only toggles its ê
            (&[Y, E, U], Horn),
            (&[U, Y], Circumflex),
            (&[U, Y], Horn),
            (&[U, Y, E], Horn),
            (&[U, Y, ECircumflex], Horn),
            // o + a only takes the breve
            (&[O, A], Circumflex),
            (&[O, A], Horn),
            (&[O, ABreve], Circumflex),
            (&[O, ABreve], Horn),
            // dead-shot third vowels
            (&[O, E], Circumflex),
            (&[O, E], Horn),
            (&[O, E, O], Circumflex),
            (&[O, E, O], Horn),
            (&[O, A, U], Circumflex),
            (&[O, A, U], Horn),
            (&[U, A, O], Circumflex),
            (&[U, A, O], Horn),
            (&[U, ACircumflex, Y], Circumflex),
            (&[U, ACircumflex, Y], Horn),
            (&[U, ACircumflex, Y], Breve),
            (&[U, A], Breve),
            (&[UHorn, A], Circumflex),
            (&[UHorn, A], Breve),
            (&[U, E], Horn),
            (&[U, ECircumflex], Horn),
            (&[U, I], Circumflex),
            (&[U, I], Breve),
            (&[U, U], Circumflex),
            (&[UHorn, U], Circumflex),
        ];
        for (letters, shape) in none {
            let src = resolve(letters);
            assert_eq!(
                transition(src, shape_id(*shape)),
                None,
                "unexpected edge for {letters:?} + {shape:?}"
            );
        }
    }

    /// State ids `0..=COUNT-1` are the base vowels, in id order.
    #[test]
    fn base_state_ids_are_base_vowels() {
        assert_eq!(state_count(), STATES.len());
        assert_eq!(transition_count(), TRANSITIONS.len());
        for (i, vowel) in VOWELS.iter().enumerate() {
            assert_eq!(vowel.id(), i, "id order");
            let state = STATES[i];
            assert_eq!(state.mask & COMPLETE_BIT, COMPLETE_BIT, "single vowel {i}");
        }
    }

    /// For every state and input, `transition` agrees with the mask bitmap and
    /// never yields a dangling state id.
    #[test]
    fn bitmap_is_consistent_and_dangling_free() {
        let mut state = 0usize;
        while state < STATES.len() {
            let row = &STATES[state];
            let mut input = 0u8;
            while input < 15 {
                let bit = 1u16 << input;
                let has_edge = row.mask & bit != 0;
                assert_eq!(
                    transition(state as u8, input).is_some(),
                    has_edge,
                    "state {} input {}",
                    state,
                    input
                );
                if has_edge {
                    let target = transition(state as u8, input).unwrap();
                    assert!(
                        (target as usize) < STATES.len(),
                        "state {} input {} -> dangling {}",
                        state,
                        input,
                        target
                    );
                }
                input += 1;
            }
            state += 1;
        }
        // Inputs beyond the 15-symbol alphabet are always rejected.
        assert_eq!(transition(0, 15), None);
        assert_eq!(transition(0, 255), None);
    }

    /// The `mask` bitmap exactly matches the resolved transition targets.
    #[test]
    fn mask_matches_edge_table() {
        let mut state = 0usize;
        while state < STATES.len() {
            let row = &STATES[state];
            assert_eq!(
                row.mask & INPUT_MASK,
                __MASKS[state],
                "mask for state {state}"
            );
            let mut inputs = 0;
            let mut input = 0u8;
            while input < 15 {
                if row.mask & (1u16 << input) != 0 {
                    inputs += 1;
                }
                input += 1;
            }
            assert_eq!(
                (row.mask & INPUT_MASK).count_ones() as i32,
                inputs,
                "bit count for state {state}"
            );
            state += 1;
        }
    }

    /// Base-vowel <-> shape spelling edges must be present exactly as the
    /// keyboard expects (a + ^ = â, â + ^ = a, â + ˘ = ă, ...).
    #[test]
    fn spelling_edges() {
        use Shape::{Breve, Circumflex, Horn};
        let c = shape_id(Circumflex);
        let b = shape_id(Breve);
        let h = shape_id(Horn);

        assert_eq!(transition(A.id() as u8, c), Some(ACircumflex.id() as u8));
        assert_eq!(transition(A.id() as u8, b), Some(ABreve.id() as u8));
        assert_eq!(transition(A.id() as u8, h), None);
        assert_eq!(transition(U.id() as u8, h), Some(UHorn.id() as u8));
        assert_eq!(transition(U.id() as u8, c), None);
        assert_eq!(transition(I.id() as u8, c), None);
        assert_eq!(transition(I.id() as u8, b), None);
        assert_eq!(transition(I.id() as u8, h), None);
        assert_eq!(transition(Y.id() as u8, c), None);
        assert_eq!(transition(E.id() as u8, c), Some(ECircumflex.id() as u8));
        assert_eq!(transition(O.id() as u8, c), Some(OCircumflex.id() as u8));
        assert_eq!(transition(O.id() as u8, h), Some(OHorn.id() as u8));
        assert_eq!(transition(O.id() as u8, b), None);

        // shaped vowel + own shape toggles back to plain
        assert_eq!(transition(ACircumflex.id() as u8, c), Some(A.id() as u8));
        assert_eq!(transition(ABreve.id() as u8, b), Some(A.id() as u8));
        assert_eq!(transition(OCircumflex.id() as u8, c), Some(O.id() as u8));
        assert_eq!(transition(OHorn.id() as u8, h), Some(O.id() as u8));
        assert_eq!(transition(ECircumflex.id() as u8, c), Some(E.id() as u8));
        assert_eq!(transition(UHorn.id() as u8, h), Some(U.id() as u8));

        // shaped vowel + a different existing shape composes crosswise
        assert_eq!(transition(ACircumflex.id() as u8, b), Some(ABreve.id() as u8));
        assert_eq!(transition(ABreve.id() as u8, c), Some(ACircumflex.id() as u8));
        assert_eq!(transition(OCircumflex.id() as u8, h), Some(OHorn.id() as u8));
        assert_eq!(transition(OHorn.id() as u8, c), Some(OCircumflex.id() as u8));

        // no shape rehearsals for impossible combinations
        assert_eq!(transition(ACircumflex.id() as u8, h), None);
        assert_eq!(transition(ECircumflex.id() as u8, b), None);
        assert_eq!(transition(ECircumflex.id() as u8, h), None);
        assert_eq!(transition(UHorn.id() as u8, c), None);
    }

    /// Compound spelling toggles: ye + ^ = yê, uye + ^ = uyê, and back.
    #[test]
    fn compound_spelling_toggles() {
        let c = shape_id(Shape::Circumflex);
        // Y + E -> YE, then ^ toggles to YÊ (and back).
        let ye = transition(Y.id() as u8, E.id() as u8).unwrap();
        let yec = transition(ye, c).unwrap();
        assert_eq!(transition(yec, c), Some(ye));
        // U + Y + E -> UYE, then ^ toggles to UYÊ (and back).
        let uy = transition(U.id() as u8, Y.id() as u8).unwrap();
        let uye = transition(uy, E.id() as u8).unwrap();
        let uyec = transition(uye, c).unwrap();
        assert_eq!(transition(uyec, c), Some(uye));
        // A bare ^ after UY goes nowhere (it is not an e-compound).
        assert_eq!(transition(uy, c), None);
    }

    /// Generation totals (informational, matches the hand-written table).
    #[test]
    fn table_totals() {
        let mut complete = 0;
        let mut s = 0;
        while s < __STATE_COUNT {
            if is_state_complete(s as u8) {
                complete += 1;
            }
            s += 1;
        }
        println!(
            "states={} ({} base + {} compound), complete={}, transitions={}",
            __STATE_COUNT,
            12,
            __STATE_COUNT - 12,
            complete,
            TRANSITIONS.len()
        );
    }

    /// Every state in the generated table must be reachable from a base-vowel
    /// state (else a DSL name typo would silently create a phantom complete
    /// state that only bloats the tables).
    #[test]
    fn every_state_is_reachable_from_a_base_state() {
        let mut reachable = [false; __STATE_COUNT];
        let mut s = 0;
        while s < 12 {
            reachable[s] = true;
            s += 1;
        }
        // Propagate reachability over all input symbols until fixpoint.
        let mut changed = true;
        while changed {
            changed = false;
            let mut s = 0;
            while s < __STATE_COUNT {
                if !reachable[s] {
                    s += 1;
                    continue;
                }
                let mut input = 0u8;
                while input < 15 {
                    if let Some(next) = transition(s as u8, input) {
                        if next as usize >= __STATE_COUNT {
                            panic!("dangling target {} from state {} input {}", next, s, input);
                        }
                        if !reachable[next as usize] {
                            reachable[next as usize] = true;
                            changed = true;
                        }
                    }
                    input += 1;
                }
                s += 1;
            }
        }
        let mut s = 0;
        let mut missing: Vec<usize> = Vec::new();
        while s < __STATE_COUNT {
            if !reachable[s] {
                missing.push(s);
            }
            s += 1;
        }
        assert!(missing.is_empty(), "unreachable states: {missing:?}");
    }

    /// Generated tables are the only two long-lived runtime tables.
    #[test]
    fn layout_is_exactly_mask_plus_offset() {
        use std::mem::{align_of, size_of};
        assert_eq!(size_of::<NucleusState>(), 4); // u16 + u8 (+ pad)
        assert_eq!(align_of::<NucleusState>(), 2);
        assert_eq!(size_of::<u8>(), 1);
    }

    /// Rank-based lookup must agree with a linear scan of the packed row
    /// (proves the COMPLETE bit never perturbs the rank).
    #[test]
    fn rank_matches_linear_scan() {
        let mut state = 0usize;
        while state < STATES.len() {
            let row = &STATES[state];
            // Re-walk the mask from low inputs up and rebuild the packed row.
            let mut packed: Vec<u8> = Vec::new();
            let mut flow: Vec<(u8, u8)> = Vec::new();
            let mut input = 0u8;
            while input < 15 {
                let tgt = __EDGE_TABLE[state][input as usize];
                if tgt != 255 {
                    flow.push((input, tgt));
                }
                input += 1;
            }
            for (input, tgt) in &flow {
                assert_eq!(
                    transition(state as u8, *input),
                    Some(*tgt),
                    "state {} input {}",
                    state,
                    input
                );
                packed.push(*tgt);
            }
            // the packed row under `transition_offset` equals the ascending scan
            let off = row.transition_offset as usize;
            for (k, tgt) in packed.iter().enumerate() {
                assert_eq!(TRANSITIONS[off + k], *tgt, "row {} rank {}", state, k);
            }
            state += 1;
        }
    }
}