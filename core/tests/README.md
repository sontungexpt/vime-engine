# Integration tests (`core/tests/`)

One file per concern, each a standalone `[[test]]` target declared in
`core/Cargo.toml`:

| file | covers |
|------|--------|
| `config.rs` | engine construction, tone-placement init/updates |
| `discriminants.rs` | `repr(u8)` layout pins for `Coda`/`Onset` |
| `inline_vec.rs` | `InlineVec` grow/consume paths |
| `keymap.rs` | `Keymap` layer + `u128` bitmask lookups |
| `nucleus.rs` | exhaustive nucleus `WordState` classification (all 1884 sequences) |
| `phonotactics.rs` | spelling validator rules |
| `vowel.rs` | vowel codec, classification, precomposed table |

Conventions:

- Shared helpers go in `tests/common/` (never a target itself; only files
  directly under `tests/*.rs` are targets).
- Unit tests for the syllable builder live in
  `src/composition/syllable/tests/` (`common` harness + `corpus/` data +
  `push`/`insert`/`remove`/`render`/`lifecycle` entry points).
