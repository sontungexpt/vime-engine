# Vime Engine

Pure Rust Vietnamese input-method engine, packaged with its C ABI boundary.

## Layout

```
vime-engine/
├── core/     # vime-engine — pure logic (Engine, Buffer, Parser, phonology,
│             #   Telex/VNI/VIQR rule engines, renderer)
└── ffi/      # vime-ffi — C ABI boundary (vime.h, libvime.so / libvime.a)
```

## Build

```sh
cargo build --release     # produces libvime.so / libvime.a, libvime_engine.rlib
cargo test --workspace
```

The FFI crate emits `libvime.so` (shared), `libvime.a` (static), and a
`vime.h` header (in `ffi/include/`) for C/C++ frontends. Adapters link this
library and do not depend on the Rust toolchain.

## Public API

- `vime_create` / `vime_destroy` — engine lifecycle
- `vime_process_key` — feed keystrokes, get preedit/commit directives
- `vime_reset` — clear engine state
- `vime_set_input_method` — select Telex / VNI
- `vime_free_string` — free Rust-allocated strings

See `ffi/include/vime.h` for the full ABI contract.