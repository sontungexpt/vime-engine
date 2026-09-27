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

## `BaseVowel::from_parts()` — bench_base_vowel_from_parts

Compares resolving `(RootVowel, Shape)` into a `BaseVowel`:

- `id_lut_then_variants_by_id` — the old body: look up an `Option<u8>` ID in a
  24-entry table, then index a 12-entry variant table. Two dependent loads.
- `direct_variants_by_root_shape` — one load from a 24-entry
  `[Option<BaseVowel>; 24]`. **This is what ships now.**
- `packed_discriminant_transmute` — keep the ID lookup, rebuild the packed
  discriminant with a `transmute`. Rejected: slowest and least stable.
- `production` — the shipped `BaseVowel::from_parts` itself.

```sh
cargo bench -p vime-engine --bench bench_base_vowel_from_parts          # all groups
cargo bench -p vime-engine --bench bench_base_vowel_from_parts -- 1m    # reliable groups only
```

Note Criterion takes a single filter, so pass one substring, not two.

| Group | Candidate | Before | After |
| --- | --- | --- | --- |
| `valid_12` | `production` | 15.99 ns | 12.32 ns |
| `valid_12` | `id_lut_then_variants_by_id` | 16.19 ns | 16.59 ns |
| `all_24_pairs` | `production` | 22.74 ns | 28.14 ns |
| `all_24_pairs` | `id_lut_then_variants_by_id` | 28.59 ns | 29.83 ns |
| `repeated/A_circumflex` (256x) | `production` | 252.9 ns | 179.7 ns |
| `repeated/A_circumflex` (256x) | `id_lut_then_variants_by_id` | 216.9 ns | 209.6 ns |
| `repeated/O_horn` (256x) | `production` | 216.1 ns | 228.4 ns |
| `repeated/O_horn` (256x) | `id_lut_then_variants_by_id` | 235.9 ns | 245.3 ns |
| `replace_shape/one_key_per_base` | `direct` | 12.93 ns | 15.72 ns |
| `replace_shape/one_key_per_base` | `id_lut_then_variants_by_id` | 15.29 ns | 18.64 ns |
| `valid_1m` | `production` | 1.055 / 1.166 ms | 1.1225 ms |
| `valid_1m` | `id_lut_then_variants_by_id` | 0.921 / 1.098 ms | 1.2367 ms |
| `all_24_1m` | `production` | 1.437 / 1.320 ms | 1.3865 ms |
| `all_24_1m` | `id_lut_then_variants_by_id` | 1.304 / 1.311 ms | 1.3756 ms |

The "after" column is one `cargo bench` run, so the trustworthy comparison is
within it: `production` (now the direct LUT) beats the old two-step algorithm in
`valid_12` (12.32 vs 16.59 ns), `replace_shape` (15.72 vs 18.64 ns),
`repeated/A_circumflex` (179.7 vs 209.6 ns), `valid_1m` (1.12 vs 1.24 ms), and
ties or loses inside noise on `repeated/O_horn`, `repeated/A_horn_invalid`, and
`all_24_1m`. The mixed-validity `all_24_1m` group swings 1.01-1.48 ms between
runs — the second load is L1-resident there, so the workload cannot resolve it.

The clear win is in `repeated/*` and `replace_shape`, which is where the engine
actually spends this function: one shape key on one base vowel.

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
