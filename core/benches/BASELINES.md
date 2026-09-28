# Bench baselines

Recorded numbers for the benches in this directory, plus everything needed to
re-run them on comparable hardware and judge whether a refactor is a win.

Read this before trusting a delta. On the machine below, **two copies of
byte-identical code in the same binary differ by up to 27%**, so anything under
roughly 20% is code layout, not the algorithm. Compare candidates **within one
run**, never across runs.

## Environment

| Item | Value |
| --- | --- |
| Recorded | 2026-09-27 |
| CPU | Intel Core i7-1195G7 @ 2.90 GHz (8 threads, max freq 4.80 GHz) |
| Scaling governor | `powersave` — turbo/boost stays on, frequency is not pinned |
| OS | Arch Linux, kernel `7.1.8-arch1-3` |
| rustc | 1.93.1 (`01f6ddf75` 2026-02-11), LLVM 21.1.8 |
| cargo | 1.93.1 |
| Criterion | 0.8.2 |
| Target | `x86_64-unknown-linux-gnu` |
| `RUSTFLAGS` | empty — no `target-cpu=native`, no LTO, no `codegen-units` override |
| Profile | cargo `bench` defaults (`opt-level=3`, `lto=false`, `codegen-units=16`, `debug-assertions=false`) |

Because the governor is `powersave` and nothing pins a frequency, absolute
numbers drift 10-40% between runs. Pin the governor before a comparison run:

```sh
sudo cpupower frequency-set -g performance   # restore with -g powersave after
```

## Method

Shared by both benches in this file:

- `criterion` with `sample_size(20)`, `warm_up_time(1s)`, `measurement_time(2s)`.
- `std::hint::black_box` on every input, every call result, and the slice passed
  to the loop, so nothing folds away and the input array is not hoisted.
- `Throughput::Elements` set to the element count, so `thrpt` in the Criterion
  output is per-element throughput.
- Each bench asserts its candidates agree with the shipped function before
  measuring, so a "win" can never be a wrong answer.
- Every bench keeps a `loop_floor` candidate that runs the same loop without
  calling the function under test. That number is the harness cost; a candidate
  that does not beat `loop_floor` has no measurable headroom left.

Group shapes, in increasing order of how much they can be trusted:

1. `all_12` / `valid_12` / `all_24_pairs` — 12 or 24 values in L1. Useful for
   spotting a real regression, useless for a few percent.
2. `single_repeated` / `repeated` — one value hammered 256 times. Closest to
   the real keystroke path, where one base vowel is reshaped once per keypress.
3. `large_1m` — 1,000,000 values cycling a 12- or 24-entry source array. The
   input stream misses L1 while both tables always hit it, so per-call cost
   shows up. This is the group to quote.

## `BaseVowel::id()` — bench_base_vowel_id_production

Measures the shipped `BaseVowel::id()` (`src/phonology/vowel.rs`,
`(self as u16 >> ID_OFFSET) as u8`) against candidates a refactor might use.

```sh
cargo bench -p vime-engine --bench bench_base_vowel_id_production
```

| Group | Candidate | Time (1st run) |
| --- | --- | --- |
| `all_12` | `production` | 15.16 ns |
| `all_12` | `shift_u16` (bench-local copy of the shipped body) | 15.30 ns |
| `all_12` | `lut_id` (372-entry table keyed by discriminant) | 12.00 ns |
| `all_12` | `loop_floor` | 15.36 ns |
| `single_repeated` | `production` (Y / A / UHorn / ECircumflex / OHorn) | 111.8 / 94.7 / 113.2 / 100.4 / 112.2 ns |
| `single_repeated` | `loop_floor` | 85.3 / 102.8 / 102.6 / 98.6 / 101.4 ns |
| `large_1m` | `production` | 1.4507 ms (1.45 ns/elem) |
| `large_1m` | `shift_u16` | 1.4515 ms |
| `large_1m` | `lut_id` | 1.4708 ms |
| `large_1m` | `loop_floor` | 1.5454 ms |
| `max_id` (1M, compare-and-keep-max) | `production` | 844.2 µs |
| `u8_accumulator` (1M, `u8` sum) | `production` | 446.1 µs |

Reading: `production` is at or below `loop_floor` in every group, and
`shift_u16` — a copy of the same body — matches it to within 0.2% in `large_1m`.
A single shift is not the bottleneck, so **there is no headroom to win inside
`id()` itself**; a refactor can only match. The `all_12/lut_id` row being
*faster* than `loop_floor` is the layout artifact mentioned above, not a result.

Two notes for whoever refactors this:

- A `#[repr(u8)]` version cannot be A/B-tested before it lands: `(base as u8)`
  truncates away ID bits 8-9, so `(base as u8 >> 5)` is wrong today. Move the
  discriminants first, then re-run this bench.
- The `u8_accumulator` group exists because `id()` returns `u8` while the other
  groups widen to `usize`. A caller-side change there shows up in that group
  only.

## `BaseVowel::id()` — LUT vs `id_fast()` vs the old bit-packed version

Under `#[repr(u8)]` the discriminant no longer holds the ID, so `id()` has to
derive it. Two ways, both in the crate:

```rust
// A: one load from a 24-entry table (was `id()`)
const fn id(self) -> u8 { Self::ID_BY_ROOT_SHAPE[self as usize] }

// B: 12 comparison arms (`id_fast()`, and the source the table is built from)
const fn id_fast(self) -> u8 { match self { Self::Y => 0, /* ... */ Self::OHorn => 11 } }
```

C is the pre-refactor `#[repr(u16)]` layout from commit `8fd4a1b`, copied into
the bench as `OldBaseVowel`, where `id()` was `(self as u16 >> 5) as u8`.

The bench measures all three plus a bench-local copy of each, which turns out to
matter more than the candidates do: **the two copies of the same algorithm land
7.6% apart** (`lut_id` 1.5133 ms vs `production` 1.6289 ms) purely from inlining
context. Min of three runs pinned with `taskset -c 2`, `large_1m`:

| `large_1m` (1M calls) | time | vs floor |
| --- | --- | --- |
| `loop_floor` (no `id()` call, `u8` elements) | 1.5089 ms | — |
| A: LUT, bench-local copy | 1.5133 ms | +0.3% |
| B: `id_fast()`, real method | 1.5559 ms | +3.1% |
| B: `match`, bench-local copy | 1.5725 ms | +4.2% |
| A: LUT, real `id()` | 1.6289 ms | +8.0% |
| C: old `u16` bit shift | 1.6647 ms | +10.3% |
| `old_loop_floor` (`u16` elements) | 1.6497 ms | +9.3% |

A's two copies span 1.513–1.629 ms and B's span 1.556–1.573 ms, so the
distributions overlap: **no difference between the LUT and the match.** C is
within 2% of A's slowest copy, which matches the earlier run where C tied them
exactly. The only repeatable effect anywhere in this file is element size — the
`u8` floor beats the `u16` floor — which is the `#[repr(u8)]` win, not an `id()`
win.

### The LUT was returning wrong IDs

`RootVowel` discriminants run `Y=0, U=1, I=2, E=3, O=4, A=5`, but
`ID_BY_ROOT_SHAPE` was still laid out in `A, E, I, O, U, Y` row order, so
`root * 4 + shape` indexed the wrong row: 11 of its 12 entries were misplaced
(`Y` read slot 0 and got `A`'s ID 5, `UHorn` read slot 7 and got `INVALID_ID`,
and so on). The const check at the bottom of `vowel.rs` now fails to compile if
either row order, the mask, or the table ever disagree, and the table is fixed
to `Y, U, I, E, O, A`.

Worth keeping in mind: this is the argument for deleting the table rather than
fixing it. It is a hand-maintained 24-entry layout whose only failure mode is
silent, and it measured no faster than the `match` that generates it.

## `BaseVowel::from_parts()` — bench_base_vowel_from_parts

Resolving `(RootVowel, Shape)` into a `BaseVowel`. Four implementations:

- `valid_mask_transmute` — the shipped body: test the encoding
  `root * 4 + shape` against a 24-bit `VALID_MASK`, then `transmute` it, since
  under `#[repr(u8)]` the encoding *is* the discriminant. No table at all.
- `direct_variants_by_root_shape` — one load from a 24-entry
  `[Option<BaseVowel>; 24]`.
- `id_lut_then_variants_by_id` — the pre-`#[repr(u8)]` body: look up an
  `Option<u8>` ID, then index a 12-entry variant table. Two dependent loads.
- `production` — the shipped `BaseVowel::from_parts` itself, i.e. a *second*
  copy of `valid_mask_transmute`. It is the noise control: two copies of the
  same source that differ only in inlining context.

```sh
cargo bench -p vime-engine --bench bench_base_vowel_from_parts
```

Note Criterion takes a single filter, so pass one substring, not two.

### Mask vs table: no measurable difference

Min of three runs pinned with `taskset -c 2`, because single runs flip order:

| Group | `valid_mask_transmute` | `production` (same code) | `direct_variants_by_root_shape` | `id_lut_then_variants_by_id` |
| --- | --- | --- | --- | --- |
| `valid_1m` | 872.3 µs | 812.7 µs | 1026.8 µs | 1094.9 µs |
| `all_24_1m` | 1008.1 µs | 923.1 µs | 955.7 µs | 1274.4 µs |
| `valid_12` | 11.64 ns | 11.90 ns | 13.12 ns | 14.24 ns |
| `all_24_pairs` | 23.96 ns | 22.84 ns | 22.00 ns | 25.26 ns |
| `repeated/E_circumflex` (256x) | 168.0 ns | 172.9 ns | 166.3 ns | 241.2 ns |
| `repeated/O_horn` (256x) | 170.5 ns | 167.5 ns | 171.6 ns | 229.9 ns |
| `replace_shape/one_key_per_base` | 12.36 ns | — | 12.66 ns | 14.26 ns |

The `production` column is the same algorithm as `valid_mask_transmute`, and the
two differ by **7–9%** purely from inlining context. That is this file's noise
floor, and it is larger than every mask-vs-table gap here:

- `valid_1m` puts the table 18% behind the mask, `all_24_1m` puts it 5% *ahead*.
- `valid_12` puts the table 13% behind, `all_24_pairs` puts it 8% ahead.
- `repeated/*` and `replace_shape` agree within 3%.

So the mask and the table are the same speed, and the only defensible reading of
this file is: **the two-load `id_lut_then_variants_by_id` is genuinely slower**
(10–30% behind in every group, including both 1M groups and all four `repeated`
cases), which is why it was replaced. Everything since has been noise.

Keep the mask version. It is not measurably faster, but it deletes a 24-entry
table, needs no `Option` load, and states the invariant directly: an encoding is
valid exactly when its bit is set in `VALID_MASK`.

### Earlier history (pre-`#[repr(u8)]`, kept for reference)

The direct 24-entry LUT replaced the two-load version on this data, taken from
single runs at the time:

| Group | Candidate | Before | After |
| --- | --- | --- | --- |
| `valid_12` | `production` | 15.99 ns | 12.32 ns |
| `valid_12` | `id_lut_then_variants_by_id` | 16.19 ns | 16.59 ns |
| `all_24_pairs` | `production` | 22.74 ns | 28.14 ns |
| `repeated/A_circumflex` (256x) | `production` | 252.9 ns | 179.7 ns |
| `repeated/A_circumflex` (256x) | `id_lut_then_variants_by_id` | 216.9 ns | 209.6 ns |
| `replace_shape/one_key_per_base` | `direct` | 12.93 ns | 15.72 ns |
| `valid_1m` | `production` | 1.055 / 1.166 ms | 1.1225 ms |
| `all_24_1m` | `production` | 1.437 / 1.320 ms | 1.3865 ms |

Only the `repeated/*` and `replace_shape` rows exceeded the noise, which is where
the engine actually spends this function: one shape key on one base vowel.

## Comparing a future refactor

1. Pin the governor (`cpupower frequency-set -g performance`), close other work.
2. `cargo bench -p vime-engine --bench <name> -- --save-baseline before`
   (or copy `target/criterion` aside) so Criterion keeps both data sets.
3. Make the change; the benches assert correctness before they measure, so a
   wrong candidate fails the run instead of posting a fast number.
4. `cargo bench -p vime-engine --bench <name>` and compare against `before`.
5. Apply the noise rule: only accept a change that clears `loop_floor` and wins
   by more than ~20%, ideally in `large_1m` or `repeated`.

The original Criterion data for every run above is kept in
`target/criterion/`, so `cargo bench ... -- <group>` plus the saved
`estimates.json` files can re-derive any number here without re-measuring.

## `id()` formulation: popcount vs table (paired, 21 interleaved rounds)

Host: Intel i7-1195G7, `taskset -c 2`, median of 3 processes. Two builds:
`default` = stock target features (`fxsr,sse,sse2`, **no** `popcnt`);
`native` = `RUSTFLAGS="-C target-cpu=native"` (`+popcnt`). Candidate functions
are passed to `sum_bases_t` as fn *items*, not `fn` pointers in a slice: a slice
forces an indirect call per element worth ~0.45 ms/1M, which is larger than
every effect measured here and makes all candidates look identical.

Semantics differ on purpose, so the two groups are asserted separately. `match`
and `lut24` must equal `priority_id`; `src`, `rev32` and `rank_sub` must equal
each other (rank in *encoding* order). All asserts pass in both builds.

| comparison | default | native |
| --- | --- | --- |
| `match` / `src` | **0.366x** (table 2.7x faster) | **1.091x** (popcount 9% faster) |
| `src` / `rev32` | 1.006x (tie) | 1.000x (tie) |
| `src` / `rank_sub` | 1.002x (tie) | 1.000x (tie) |
| `rev32` / `lut24` | 0.366x | 0.805x |

Absolute medians per 1M bases, `default`: floor 0.586 ms, `match` 0.874 ms,
`lut24` 0.862 ms, all popcount forms 2.33-2.39 ms. `native`: floor 0.66-0.85 ms,
`match` 0.97-1.02 ms, `src` 0.89-0.96 ms, `rev32` 1.20-1.56 ms.

Conclusions:

1. **No popcount formulation beats the source one.** Reversing the mask so the
   shift count is a plain `31 - v`, and computing `12 - popcnt(mask >> v)`, are
   both exactly tied with `mask & ((1 << v) - 1)`. In a `popcnt` build all three
   compile to the same two instructions (`bzhi` + `popcnt`) - LLVM turns the
   shift/and/decrement into a single BMI2 `bzhi` - so there is nothing left to
   remove. The current `id()` is already optimal.
2. **The whole popcount approach is a portability bet.** Without `popcnt`,
   `count_ones` becomes a ~16-instruction SWAR sequence and the popcount forms
   are **2.7x slower** than the 23-byte table the `match` already compiles to.
   With `popcnt` they are only 9% faster. 9% upside against a 170% downside.
3. `match` and the explicit 24-byte `lut24` are indistinguishable (0.874 vs
   0.862 ms, and identical instruction shape), confirming the `match` is
   already a table lookup - a hand-written table buys nothing.

So `id()` cannot be made faster by rearranging the popcount. It is only worth
keeping if the answer must be *rank in encoding order*; if the answer can be
*priority id*, the `match` is both faster and portable. Note that all of
`id()`'s measured cost is hypothetical: it has no production callers, and
`Ord`, `encode_vowel` and the const guard all use `priority_id()`.

## `id()`: popcount vs const-derived table

The one remaining speed lever for `id()` is not rearranging the popcount but
removing it. `id_table` builds `[u8; 256]` from `NEW_DECLARED_MASK` in a const
block and indexes it, so it returns exactly the popcount's values - the bench
asserts the two agree for all 12 variants before timing anything. It is kept
bench-local rather than in `vowel.rs` so the comparison survives the source
revert.

| comparison | default | native |
| --- | --- | --- |
| `table` / `src` (popcount) | **0.358-0.361x** | 0.922-1.001x |

Per-process ratios, default: 2.784, 2.767, 2.790, 2.803. Native: 0.975, 1.001,
0.922. Absolute medians per 1M bases, default: popcount 1.37-2.14 ms, table
0.49-0.77 ms, floor 0.34-0.53 ms.

So the table is **2.78x faster than the popcount on the default build** and
within 0-8% of it on a `popcnt` build. The comparison is a coin flip: the
popcount is a large win only on hardware that has `popcnt`, and the table gives
up a few percent there to remove a 170% cliff everywhere else.

**`vowel.rs` keeps the popcount**, per decision, with this measurement recorded
on `id()` so the tradeoff is not re-litigated from scratch. If the default
target's SWAR cost ever matters, the table is the drop-in replacement and needs
no semantic change at all.

## The two versions of `BaseVowel::id()` — bench_base_vowel_id_versions

A/B of the committed `id() -> u8` API against the working-tree
`id() -> BaseVowelId` API. The HEAD bodies are copied verbatim into `mod head`
so both versions are measured in one binary, and `mod head_copy` is a second
byte-identical copy used as the noise control. **The bench changes no
production code.**

```sh
cargo bench -p vime-engine --bench bench_base_vowel_id_versions
```

Recorded 2026-09-28, same machine and toolchain as the rest of this file.
Min of 3 runs pinned with `taskset -c 2`; the source hash was verified identical
before and after all three runs (`SOURCE_STABLE=YES`), which matters because
`vowel.rs` was being edited during this work.

### Noise floor, measured rather than assumed

`head` vs `head_copy` is the same body compiled twice. Their gap is the floor
that every other number has to clear:

| Group | `head` | `head_copy` | spread |
| --- | --- | --- | --- |
| `all_12` | 8.58 ns | 8.69 ns | 1.3% |
| `single_repeated` (Y/A/UHorn/ÊCircumflex/OHorn) | 122.7 / 124.0 / 124.4 / 124.6 / 125.5 ns | 122.3 / 123.7 / 123.9 / 123.7 / 125.3 ns | 0.2–0.7% |
| `large_1m` | 559.75 µs | 575.21 µs | **2.8%** |

So **~3% is the resolution of this file.** The run-to-run spread of a single
benchmark on unchanged code is far larger: `large_1m/head` measured
559.75 / 785.32 / 659.05 µs across the three runs — **40%** — because the
governor is `powersave` and the desktop was busy. That is why this is min-of-3
and why no conclusion below rests on a single run.

### `id()` — the two versions are the same speed

| Group | `head` | `current` | `current_lookup` | `loop_floor` |
| --- | --- | --- | --- | --- |
| `all_12` | 8.58 ns | 8.86 ns (+3.3%) | 8.82 ns | 8.62 ns |
| `single_repeated` (5 vowels) | 122.7–125.5 ns | 122.4–133.5 ns | 121.3–147.3 ns | 83.9–90.3 ns |
| `large_1m` | 559.75 µs | 559.21 µs (**−0.1%**) | 567.79 µs (+1.4%) | 613.98 µs |

The `match` on a `#[repr(u8)]` discriminant is already a table load, so
returning a `#[repr(u8)]` enum instead of a `u8` changes nothing that survives
codegen — `current` lands within 0.1% of `head` in the group to quote, and
`current_lookup` within 1.4%, both inside the 2.8% floor. Every candidate sits
at or below `loop_floor`, so the call itself is free next to the harness. The
new `id_lookup()` 23-entry table buys nothing over the `match` either, which
re-confirms the earlier `match` vs `lut24` conclusion for this particular table.

### `from_id` — the extra `u8` → enum step is not free, but it is small

| Group | `head` | `current` | `current_unchecked` |
| --- | --- | --- | --- |
| `from_id_all_12` | tie | tie | tie |
| `from_id_large_1m` | 703.29 µs | 715.97 µs (+1.8%) | 657.44 µs (−6.5%) |

`current` is +1.8%, inside the 3% floor. `current_unchecked` skips the
`BaseVowelId::from_u8` range check that the new signature forces on a caller
holding a raw `u8`; at −6.5% it is the only delta in this file that clears the
floor, and it is the reason `from_u8_unchecked` is worth keeping. It is ~2x the
floor, so treat it as suggestive, not settled.

### `encode_vowel` / `to_char` — the only production caller

| Group | `head` | `current` | `current_lookup` |
| --- | --- | --- | --- |
| `encode_vowel_1m` | 1553.90 µs | 1538.00 µs (−1.0%) | 1874.30 µs (+20.6%) |
| `to_char_1m` | 1420.50 µs | 1379.10 µs (−2.9%) | — |

`current` ties `head` in both: the refactor costs nothing on the path users
actually hit. The `current_lookup` +20.6% is **not conclusive** — per run it is
2382.9 / 1874.3 / 1942.7 µs against `head`'s 1607.4 / 2192.8 / 1553.9 µs, so it
loses twice and wins once, and identical code varies 41% across runs in this
group. It needs a quiet machine with the governor pinned to resolve.

### Conclusion

1. **`u8` → `BaseVowelId` is free.** The strongest evidence is negative and it
   is consistent: the `match` already compiled to a table load, and every
   measurement of the new `id()` lands within the measured 3% noise floor.
2. **The safety and the ergonomics are therefore free too** — the enum removes
   the out-of-range `usize` that `from_id` used to take, at no measured cost.
3. **`id_lookup()` should not ship as the default.** It matches the `match` in
   `large_1m` and is the slower candidate in the one group where the table is
   actually on the hot path. It is also a hand-maintained 23-entry table whose
   failure mode is silent — the exact argument already recorded above for
   deleting `ID_BY_ROOT_SHAPE`.
4. **Keep `BaseVowelId::from_u8_unchecked`.** It is the only measurable win in
   the refactor and it is exactly the operation a caller with a `BaseVowelId` in
   hand no longer needs to perform.

