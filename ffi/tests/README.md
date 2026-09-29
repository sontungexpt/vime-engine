# FFI integration tests (`ffi/tests/`)

| file | covers |
|------|--------|
| `abi.rs` | C ABI layout / enum-value locks |
| `lifecycle.rs` | create/destroy, method/tone switching, reset, commit |
| `typing.rs` | typing round-trips per input method |
| `editing.rs` | backspace/delete, caret navigation |
| `safety.rs` | NULL handles, invalid Unicode payloads |
| `common/` | shared harness (`Engine` RAII driver, event builders) |

Each test file declares `mod common;` to reuse the harness. `common/` is a
helper directory, never a test target.
