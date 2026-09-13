# Vime Engine

Pure Rust Vietnamese input-method engine, packaged with its C ABI boundary.

## Layout

```
vime-engine/
├── core/     # vime-engine — pure logic (Engine, Buffer, Parser, phonology,
│             #   Telex/VNI/VIQR rule engines, renderer)
└── ffi/      # vime-engine-ffi — C ABI boundary (vime_engine.h, libvime.so / libvime.a)
```

## Build

```sh
cargo build --release     # produces libvime.so / libvime.a, libvime_engine.rlib
cargo test --workspace
```

The FFI crate emits `libvime.so` (shared), `libvime.a` (static), and a
`vime_engine.h` header (in `ffi/include/`) for C/C++ frontends. Adapters link
this library and do not depend on the Rust toolchain.

## Public API

- `vime_create` / `vime_destroy` — engine lifecycle
- `vime_process_key` — feed keystrokes, get a `VimeAction` plus preedit/commit text
- `vime_reset` — clear engine state
- `vime_set_input_method` — select Telex / VNI

`VimeOutput` carries a `VimeAction` (`Forward`, `Noop`, `UpdatePreedit`,
`Commit`) and `rendered`/`commit` C strings owned by the engine handle — valid
until the next call on that handle or `vime_destroy` (no manual free needed).

See `ffi/include/vime_engine.h` for the full ABI contract.