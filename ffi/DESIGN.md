# Design notes

Plans that are deliberately **not** implemented. Each records the decision and
the reasoning, so the next person can pick it up without re-deriving it.

---

## Multiple compositions / sessions, addressed by id

**Status: done in the core, not done in the ABI.** `vime_engine` has
`Session` (one buffer), `Engine` (shared settings plus a factory) and
`Sessions` (a registry that holds sessions and names them). One
`VimeEngineHandle` still owns exactly one `Session`, and there is still no
registry and no id in the ABI.

### What the core already settled

The questions that came up while building this are answered in the core:

1. **Does a session need an id?** Yes, but only for a *registry*. A caller that
   owns its buffer outright never needs one, which is why `Engine::new_session`
   hands the session out by value and `Sessions` is opt-in. Identity is a
   property of *who holds the buffer*; a bare pointer is already one, and a
   number only earns its keep once something else has to name it.
2. **What is shared between sessions?** Settings only. `Session` owns its own
   `Composition`, so two handles cannot alias a buffer even by accident.
3. **How does one settings change reach every session?** `SharedConfig` is
   replaced behind an atomic generation counter, and each session notices on its
   next keystroke or `refresh()`. So the *core* already has the "change one
   thing, every session sees it" property — and it does **not** come from
   walking the registry, which is what keeps a session that was idle, or one
   opened a moment later, from being left behind.
4. **What does holding a session cost?** A registered session is an
   `Arc<Mutex<Session>>`. That is not a free choice: `Arc<T>` hands out `&T`,
   so two owners of a buffer need interior mutability. What it buys is that the
   keystroke path never touches the *registry* lock — a caller types through its
   own handle — so no input context can block another. The per-session lock is
   uncontended whenever a session has the single owner it is meant to have.

### Why one handle is right today

The engine is not thread-safe, and nothing in the C ABI creates a second owner.
`grep` for `Send`/`Sync`/`Arc` finds nothing in the FFI crate; the handle is a
plain `Box<VimeEngineHandle>` whose text pointers are invalidated by the next
call on that handle.

A frontend with several input fields wants one buffer per field, and the
cheapest correct way to get that today is **one handle per field**. Each is
independent, and a null-handle guard is all the failure mode a frontend has to
handle.

### What would change, and what would not

The good news is that the expensive parts are already in the right place:

- `Engine`, `Session`, `Sessions`, `Composition` and `SyllableBuilder` know
  nothing about handles or the C ABI. They stay exactly as they are.
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

On the Rust side, one `VimeRegistry` wrapping `Sessions<DefaultKeymap>`, where a
registered `SessionRef` is what each `VimeEngineHandle` holds. A missing id
returns the same neutral values a null handle does today.

### Decisions to make before writing any of it

1. **Where do ids come from?** The core uses a monotonic counter and never
   reuses a value, so a stale id is always "not found" rather than silently
   addressing a different session. A caller-supplied id avoids the lookup but
   makes reuse the caller's problem.
2. **Does an id replace the handle, or sit beside it?** Beside it, for now:
   `vime_create`/`vime_destroy` keep working, and the registry is additive. That
   keeps the three ABI breaks already in flight from compounding.
3. **Invalidation semantics must stay identical.** A returned pointer stays
   valid until the *next call on the same session*. With a registry, a lookup on
   session A must never disturb session B's buffers — which it cannot, because
   the buffers live in the `Session`, not the registry. Worth a test.
4. **Thread safety.** The core answer is that a *registered* session is an
   `Arc<Mutex<Session>>`, so a `Mutex<Registry>` per lookup is not needed for
   correctness — only for the registry's own `Vec`. Whether the ABI exposes that
   at all is the real question: keeping handles single-threaded and leaving the
   registry to the frontend preserves the existing "not thread-safe, caller's
   duty" contract and changes nothing about it. Do not pick this by accident.
5. **Failure mode.** An unknown id must be distinguishable from an empty
   preedit. Today a null handle and an empty buffer both mean "nothing to show",
   which is fine for a single handle and too lossy for many.
6. **Where does "apply to every session" go?** The core has already answered
   this: a settings change is a `SharedConfig::replace`, not a walk. A
   `vime_config_set_all` should therefore be a call on a *settings* handle
   mirroring `SharedConfig`, and it should not touch the session registry at all.
   Only genuinely per-buffer operations — "clear every window" — belong in
   `for_each`. Prefer the first: it keeps the per-session path free of registry
   work, and the registry stays a dumb list of names.

### What not to do

Do not put a `session_id` field on `Session` or thread one through
`Composition`. That pushes an FFI concern into the core and buys nothing: the
identity of a session is a property of *who owns the buffer*, and the owner is
the handle.
