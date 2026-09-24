# Complexity Notes — `vime-engine` (core)

Time-complexity analysis of each function in `core/src`, as of HEAD.

> **Read this first.** The engine models a single Vietnamese syllable with
> **fixed-size buffers**:
>
> | component | capacity |
> |-----------|----------|
> | onset     | `Onset::MAX_CHARS = 3` (`ArrayVec<char, 3>`) |
> | vowels    | `3` (`ArrayVec<CasedBaseVowel, 3>`) |
> | coda      | `Coda::MAX_CHARS = 2` (`ArrayVec<char, 2>`) |
>
> So a building syllable holds **at most 8 chars** and every operation on it is
> physically bounded. Whenever a function is labeled `O(1)` below it means
> "constant, because the inputs are the fixed, tiny syllable buffers" — the
> *nominal* loop length (e.g. `min(3, vowels.len())`) is a constant. The only
> functions whose cost is genuinely data-dependent are those that touch the
> unbounded `Vec`s: `DeadSyllableBuilder` (dead buffer) and
> `Composition::raw` (raw keystroke buffer).

Legend: `V` = vowels.len() (≤ 3), `P` = onset.len() (≤ 3), `C` = coda.len()
(≤ 2), `n` = a named, unbounded input length.

---

## `composition/syllable/building.rs`

| function | complexity | notes |
|----------|-----------|-------|
| `is_q`, `is_i` | O(1) | integer masking |
| `len` | O(1) | sums three `.len()` |
| `onset`, `onset_kind`, `coda`, `coda_kind`, `vowels`, `tone` | O(1) | accessors |
| `reset` | O(1) | `.clear()` on fixed arrays |
| `tone_index` | O(1) | delegates to `TonePlacement::tone_index` (≤3 vowels) |
| `to_chars` | O(n), n ≤ 8 | `Vec::with_capacity` + extends over onset/vowels/coda; bounded by the fixed buffers → constant in practice |
| `transaction` | O(1) | clones the fixed-size struct, runs a closure, restores on `Err` |
| `try_update_coda` | O(1) | at most 2 chars; `Coda::from_chars` on ≤2 |
| `try_update_onset` | O(1) | at most 3 chars |
| `normalize_uo_horn` | O(1) | peeks `vowels[0..2]` |
| `normalize_i_placement` | O(1) | constant `ArrayVec` shifts |
| `validate_vowels` | O(1) | copies ≤3 vowels into a `[BaseVowel; 3]`, then `from_vowels` |
| `apply_tone` | O(1) | |
| `toggle_d_stroke` | O(1) | indexes `onset[0]` |
| `vowels_starts_with_uo` | O(1) | peeks 2 vowels |
| `apply_vowel_shape` | O(1) | one index write + `validate_vowels` |
| `apply_uo_shape` | O(1) | one `apply_vowel_shape` or two writes (unified Horn/Circumflex) |
| `try_transform_shape` | O(V·S), V ≤ 3 | loops `(0..V).rev()`; each iteration calls `keymap.decode_shape`, which linearly scans the shape rules (`S` = # shape rules, small fixed table for the built-in keymaps). Effectively constant |
| `try_toggle_d_stroke` | O(1) | `keymap.is_stroke_key` on a small fixed list |
| `try_transform` | O(T + S), small | tone-table scan + shape-table scan on tiny rule slices |
| `insert` | O(1) | bounded partition of the 8-char syllable; `ArrayVec::insert` shifts ≤7 elements |
| `insert_onset` | O(1) | via `try_update_onset` |
| `insert_vowel` | O(V), V ≤ 3 | `ArrayVec::insert` shifts ≤3 |
| `insert_coda` | O(1) | via `try_update_coda` |
| `push` | O(1) | same bounded helpers as `insert` |
| `push_onset` | O(1) | |
| `push_vowel` | O(V), V ≤ 3 | push + validate + rollback |
| `push_coda` | O(1) | |
| `remove` | O(1) | `transaction` (fixed clone) + bounded removals; `ArrayVec::remove` shifts ≤7 |
| `remove_onset` | O(1) | `Cell` capture + reinsert into ≤3 buffer |
| `remove_vowel` | O(1) | one `ArrayVec::remove` on ≤3 + tone recompute |
| `remove_coda` | O(1) | |

## `composition/syllable/dead.rs`  *(unbounded buffer — `Vec<CharStatus>`)*

| function | complexity | notes |
|----------|-----------|-------|
| `CharStatus::char` | O(1) | |
| `from_accepted` | **O(n)** | `Vec::with_capacity` + `extend` over the accepted chars (`n` = len of the last valid syllable) |
| `len`, `chars` | O(1) | |
| `to_chars` | **O(n)** | `map().collect()` over the buffer |
| `reset` | O(1) | `Vec::clear` |
| `push` | O(1) amortized | `Vec::push` |
| `insert` | **O(n)** | `Vec::insert(index)` shifts the tail |
| `remove` | **O(n)** | `Vec::remove(index)` shifts the tail |

## `composition/syllable/mod.rs` (`SyllableBuilder<KM>`)

| function | complexity | notes |
|----------|-----------|-------|
| `new` | O(1) | |
| `len` | O(1) | dispatch |
| `push` | O(1) | building path incurs the small, bounded `BuildingSyllableBuilder::push`; the dead path is a `Vec::push`. `Err` path calls `to_chars` (≤8) then `dead.push` — still bounded |
| `insert` | O(1) | same; dead path is `Vec::insert` (O(dead_len)) but the *building* decode/merge path is bounded |

## `composition/cursor.rs`

All methods (`new`, `position`, `reset`, `move_left`, `move_right`,
`move_to_start`, `move_to_end`, `set`, `is_at_start`, `is_at_end`,
`Default::default`) are **O(1)** — pure integer clamping.

## `composition/mod.rs` (`Composition<KM>`)

| function | complexity | notes |
|----------|-----------|-------|
| `new` | O(1) | |
| `reset` | O(1) | `.clear()` + cursor resets |
| `is_empty` | O(1) | `Vec::is_empty` |
| `syllable` | O(1) | accessor |
| `backspace`, `delete` | O(1) | `todo!()` stubs |
| `move_left`, `move_right` | O(1) | cursor clamps |
| `insert` | **O(M)** | `raw: Vec<char>.insert(pos, ch)` shifts the whole raw tail (`M` = `raw.len()`). Syllable path is bounded/O(1). Since `raw` accumulates every keystroke, the dominant term is the raw-buffer insert |
| `remove_at` | O(1) | placeholder (no-op body) |

## `engine.rs` (`Engine<R, KM>`)

| function | complexity | notes |
|----------|-----------|-------|
| `config` | O(1) | |
| `rendered` | O(n) | delegates to `Renderer::render` → proportional to output length (≤8 for a building syllable; dead buffer is unbounded) |
| `reset` | O(1) | `composition.reset` |
| `process_key` | O(1) | dispatch; delegates to the table below |
| `commit` / `commit_with_suffix` | O(n) | `rendered()` (O(n)) + `push_str(suffix)` + `reset` |
| `insert` | **O(M)** | delegates to `Composition::insert` (raw-vec shift) |
| `backspace`, `delete` | O(1) | `todo!()` underneath |
| `move_left`, `move_right` | O(1) | |
| `apply` | O(1) | `compose.is_empty()` + one op |
| `new`, `set_layout`, `default` | O(1) | |

## `keymap/`

### `api.rs` (trait)
Method signatures only — cost lives in the impls.

### `default/mod.rs` (`DefaultKeymap`)
All lookups are **linear scans over small, fixed rule slices** (built-in
layouts: telex/vni/viqr have ≤6 tone rules, ≤7 shape rules, 1 stroke key):

| function | complexity |
|----------|-----------|
| `new`, `telex`, `vni`, `viqr`, `config` | O(1) |
| `is_tone_key` | **O(T)**, T = #tone rules |
| `is_shape_key` | **O(S)**, S = #shape rules |
| `is_stroke_key` | **O(L)**, L = #stroke keys |
| `decode_tone` | **O(T)** |
| `decode_shape` | **O(S)** |

With the fixed built-in configs these are all effectively O(1).

### `default/config.rs`
`Rules::new` runs **nested validation loops**: `O(T² + T·S + S² + L² + L·T +
L·S)` worst case. This is a `const fn` — it executes **at compile time**
(const-layout validation), not per-keystroke.

## `phonology/case.rs`

`Cased::new`, `upper`, `lower` — **O(1)**.

## `phonology/coda.rs`

| function | complexity | notes |
|----------|-----------|-------|
| `from_id` | O(1) | range check + transmute |
| `is_possible_first_char`, `is_possible_char` | O(1) | bit-masked `matches!` |
| `from_bytes` | O(1) | `match` on ≤2-byte slices |
| `from_chars` | O(1) | converts 1–2 chars, delegates to `from_bytes` |
| `from_str` | O(1) | `from_bytes` on the byte slice |

## `phonology/onset.rs`

| function | complexity | notes |
|----------|-----------|-------|
| `from_id` | O(1) | |
| `from_bytes` | O(1) | `match` on ≤3-byte slices |
| `from_chars` | O(1) | delegates to `from_bytes` |
| `is_possible_first_char`, `is_possible_char` | O(1) | bit-masked `matches!` |
| `from_str` | O(1) | |

## `phonology/vowel.rs`

| function | complexity | notes |
|----------|-----------|-------|
| `Shape::is_some` | O(1) | |
| `Tone::from_id`, `Tone::is_some` | O(1) | range check + transmute |
| `BaseVowel::id`, `from_id`, `from_parts`, `from_root`, `shape`, `has_shape`, `is_shaped`, `is_plain`, `root`, `replace_shape`, `remove_shape` | O(1) | bit masking / small `match` / LUT |
| `CasedBaseVowel::to_char`, `to_char_tone` | O(1) | → `encode_vowel` |
| `encode_vowel` | **O(1)** | single LUT index `[(id·6 + tone)·2 + upper]` on a 144-char table |
| `decode_vowel` | **O(1)** | code-range `match` → the Vietnamese block is a direct LUT index (O(1) array access) |
| `is_vowel` | **O(1)** | u64 bit-mask + shift/range checks |

## `phonology/rules/nucleus.rs`

`NucleusState::from_vowels` — **O(1)**. Pattern-matches slices of length ≤3;
the `_ => Dead` arm catches everything longer. Match cost is constant.

## `phonology/rules/tone_placement.rs`

| function | complexity | notes |
|----------|-----------|-------|
| `TonePlacement::tone_index` | O(1) | dispatch |
| `tone_index_modern` | O(1) | reads ≤2 vowels via `at()` |
| `tone_index_old` | O(1) | same |
| `tone_index_3` | O(1) | 3 constant checks |
| `fallback_tone_index` | **O(V)** | linear scan; only reachable for >3 vowels, which the composition model never builds → dead code in practice |

## `phonology/vowel_dfa.rs`

### Runtime API

| function | complexity | notes |
|----------|-----------|-------|
| `state_count`, `transition_count` | O(1) | `const fn` over generated arrays |
| `shape_id` | O(1) | |
| `transition` | **O(1)** | mask probe + `count_ones` rank + one array index |
| `is_complete`, `is_state_complete` | O(1) | bit test |
| `check_nucleus_validity` | **O(L)**, L ≤ 3 | one `transition` per vowel; bounded by the 3-vowel model |

### Compile-time generator (`const` only — cost is paid by the compiler, not the runtime)

| function | complexity |
|----------|-----------|
| `__bytes_eq` | O(len) |
| `__name_index` | O(table_len · name_len) |
| `__name_in` | O(limit) |
| `__is_compound` | O(12) |
| `__compound_count` | O(E·C) — edges × distinct-compound scan (C ≤ 128) |
| `__state_id`, `__status_id`, `__input_id` | O(table_len) |
| `__STATUS`, `__MASKS`, `__EDGE_TABLE` | O(E) linear over edges |
| `__state_mask`, `__state_edge_count` | O(1) |
| `__transition_offset` | **O(S)** per state (runs a prefix sum) |
| `__build_states` | **O(S²)** — calls `__transition_offset` for each state |
| `__build_transitions` | **O(S · 15)** — scans the 15-input alphabet per state |

(Here `E` = number of DSL rules, `S` = generated state count; both small and
compile-time only.)

## `phonology/vowel_sequence.rs`

`VowelSequence` impls for `[BaseVowel]` and `ArrayVec<CasedBaseVowel, N>` —
`len`, `at`: **O(1)**.

## `renderer/default.rs`

| function | complexity | notes |
|----------|-----------|-------|
| `new`, `default` | O(1) | |
| `render` | O(n) | building → `render_building` (≤8); dead → `map().collect()` over the unbounded dead buffer |
| `render_building` | O(n), n ≤ 8 | `String` extends over onset/vowels/coda + one `tone_index`; bounded by the fixed buffers |

## `renderer/api.rs`

`Renderer::render` — trait signature; see `default.rs`.

## Trivial modules

- `config.rs` (`Config`, `Default`) — O(1).
- `event.rs` (`KeyEvent::key`) — O(1).
- `result.rs` (`Result`) — data only.
- `lib.rs` — re-exports only.
- `renderer/mod.rs` — module wiring.

---

## Hot path summary (per keystroke)

1. `Engine::insert` → `Composition::insert`
   → `raw: Vec::insert`  — **O(M)**, `M` = raw-buffer length
   → `SyllableBuilder::push/insert` — **O(1)** (bounded 8-char syllable)
   → `BuildingSyllableBuilder::push/insert` + `validate_vowels`
   (DFA `check_nucleus_validity`) — **O(1)**.
2. `Engine::rendered` → `Renderer::render` — **O(n)**, n ≤ 8 while Building,
   **O(dead buffer)** in the Dead phase.

The only asymptotically unbounded operations in the whole crate are the
`Vec` shifts/inserts of the **raw keystroke buffer** (`Composition::raw`),
the **dead syllable buffer** (`DeadSyllableBuilder::chars`), and rendering
those buffers. Everything else is constant bounded by the 8-character
Vietnamese syllable model.