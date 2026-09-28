# Design notes

Plans that are deliberately **not** implemented. Each records the decision and
the reasoning, so the next person can pick it up without re-deriving it.

---

## Multiple compositions / sessions, addressed by id

**Status: not implemented, and not currently needed.** One `VimeEngineHandle`
owns exactly one `Engine`, which owns one `Composition`, which owns one
`SyllableBuilder`. There is no registry and no id anywhere in the ABI.

### Why one handle is right today

The engine is not thread-safe, and nothing in the C ABI creates a second owner.
`grep` for `Send`/`Sync`/`Arc` in `core` finds nothing; the FFI handle is a plain
`Box<VimeEngineHandle>` whose text pointers are invalidated by the next call on
that handle.

A frontend with several input fields wants one engine per field, and the
cheapest correct way to get that today is **one handle per field**. Each is
independent, and a null-handle guard is all the failure mode a frontend has to
handle.

### What would change, and what would not

The good news is that the expensive parts are already in the right place:

- `Engine`, `Composition` and `SyllableBuilder` know nothing about handles,
  sessions or ids. They stay exactly as they are.
- `VimeEngineHandle` is the *only* type that owns C-visible state. A session
  registry would sit beside it, not inside it.
- The preedit/commit buffers are per-handle, so sessions cannot alias them by
  construction.

So the change is confined to the FFI layer.

### The shape I would use

```c
typedef uint32_t VimeSessionId;

/* Create/destroy a session; the returned id is valid until destroyed. */
VimeSessionId vime_session_create(VimeInputMethod, VimeTonePlacement);
void          vime_session_destroy(VimeSessionId);

/* The same two text accessors, now per session. */
const char *vime_session_preedit(VimeSessionId);
const char *vime_session_committed(VimeSessionId);
```

On the Rust side, one `VimeRegistry` owning `HashMap<VimeSessionId, Session>`,
where a `Session` is today's `VimeEngineHandle` renamed. A missing id returns
the same neutral values a null handle does today.

### Decisions to make before writing any of it

1. **Where do ids come from?** A monotonically increasing counter is simplest
   and never reuses a value, so a stale id is always "not found" rather than
   silently addressing a different session. A caller-supplied id avoids the
   lookup but makes reuse the caller's problem.
2. **Does an id replace the handle, or sit beside it?** Beside it, for now:
   `vime_create`/`vime_destroy` keep working, and the registry is additive. That
   keeps the three ABI breaks already in flight from compounding.
3. **Invalidation semantics must stay identical.** A returned pointer stays
   valid until the *next call on the same session*. With a registry, a lookup on
   session A must never disturb session B's buffers — which it cannot, because
   the buffers live in the `Session`, not the registry. Worth a test.
4. **Thread safety is a separate decision.** A registry implies concurrent
   access is at least conceivable. A `Mutex<Registry>` per lookup is the safe
   default, but it is a large hammer for an IME hot path; a per-session
   `Arc<Mutex<Session>>` and a lock-free registry read is the careful version.
   Do not pick this by accident.
5. **Failure mode.** An unknown id must be distinguishable from an empty
   preedit. Today a null handle and an empty buffer both mean "nothing to show",
   which is fine for a single handle and too lossy for many.

### What not to do

Do not put a `session_id` field on `Engine` or thread one through
`Composition`. That pushes an FFI concern into the core and buys nothing: the
identity of a session is a property of *who owns the buffer*, and the owner is
the handle.
