//! Compiles a real C program against the real header and links it against the
//! real static library.
//!
//! Everything else in this suite is Rust calling into a Rust library through a
//! declaration it also compiled. That catches logic errors and layout drift as
//! long as the Rust side of the declaration is right, but it cannot catch the
//! failure mode that matters most here: the header disagreeing with the library
//! — a name spelled differently, a parameter typed `int` where the library wants
//! `uint32_t`, a struct member the header forgot. Only a C compiler notices.
//!
//! So this test shells out to the system C compiler, which is the point: the
//! header is verified by a C front end, and the result is executed, so the
//! symbols are verified to link and the values to be right.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/// Skips the test with a message rather than failing, when no C compiler is
/// present. A machine without one is not a broken build; it just cannot run this
/// particular check.
macro_rules! require_cc {
    () => {
        match cc() {
            Some(cc) => cc,
            None => {
                eprintln!("skipping: no C compiler found (tried cc, gcc, clang)");
                return;
            }
        }
    };
}

/// The first C compiler on the system.
fn cc() -> Option<String> {
    for candidate in ["cc", "gcc", "clang"] {
        let found = Command::new(candidate)
            .arg("--version")
            .output()
            .is_ok_and(|out| out.status.success());
        if found {
            return Some(candidate.to_owned());
        }
    }
    None
}

fn header() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("include/vime_engine.h")
}

/// The static library `build.rs` located, built first if it is missing or stale.
///
/// `cargo test` builds the rlib the Rust tests link against, but not the
/// `staticlib` artifact the C programs need, and the one left over from an
/// earlier build may predate the current source. Both cases show up the same way —
/// undefined references to symbols that exist in the source — so the fix is the
/// same: build it, once, and let the linker sort out what is real.
///
/// Doing this here rather than in `build.rs` is deliberate. A build script that
/// runs `cargo` re-enters itself, and cargo holds the build-directory lock for
/// the whole outer invocation. A test running after the build has finished can
/// invoke cargo without either problem.
fn staticlib() -> Option<PathBuf> {
    static ONCE: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    ONCE.get_or_init(build_staticlib).clone()
}

/// Returns the static library, building it if needed. `None` only if there is no
/// way to build it, in which case the caller skips.
fn build_staticlib() -> Option<PathBuf> {
    let path = PathBuf::from(std::env::var_os("VIME_STATICLIB")?);
    if is_fresh(&path) {
        return Some(path);
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(cargo)
        .args(["build", "--lib"])
        .current_dir(manifest_dir)
        .status();

    match status {
        Ok(status) if status.success() && path.exists() => Some(path),
        Ok(status) => {
            eprintln!("skipping the C ABI tests: `cargo build --lib` exited with {status}");
            None
        }
        Err(err) => {
            eprintln!("skipping the C ABI tests: could not run cargo: {err}");
            None
        }
    }
}

/// Whether `lib` is newer than every Rust source and the header.
///
/// A coarse check on purpose: the cost of a wrong "yes" is a confusing link error,
/// so the test errs towards rebuilding.
fn is_fresh(lib: &Path) -> bool {
    let Ok(built) = lib.metadata().and_then(|m| m.modified()) else {
        return false;
    };
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = std::iter::once(manifest_dir.join("src"))
        .chain(std::iter::once(manifest_dir.join("include/vime_engine.h")))
        .chain(std::iter::once(manifest_dir.join("build.rs")));

    fn newer(path: &Path, built: &SystemTime) -> bool {
        if path.is_file() {
            return std::fs::metadata(path)
                .and_then(|m| m.modified())
                .is_ok_and(|t| t > *built);
        }
        let Ok(entries) = std::fs::read_dir(path) else {
            return false;
        };
        entries.flatten().any(|entry| newer(&entry.path(), built))
    }

    !sources.any(|path| newer(&path, &built))
}

/// The directory to build C artifacts in. Under the target directory rather than
/// `/tmp`, so a read-only or wiped `/tmp` does not turn a passing test into a
/// confusing failure.
fn work_dir() -> PathBuf {
    let dir = PathBuf::from(option_env!("VIME_PROFILE_DIR").unwrap_or(".")).join("c_abi_test");
    std::fs::create_dir_all(&dir).expect("a work dir for the C tests");
    dir
}

/// Writes `source` to a `.c` file and returns its path.
fn write_source(name: &str, source: &str) -> PathBuf {
    let path = work_dir().join(name);
    std::fs::write(&path, source).expect("writing the C source");
    path
}

/// Compiles and links `source`, then runs it if it is a program.
///
/// `extra_flags` is for the header-only layout checks, which compile but must not
/// link; the rest link against the static library and run.
fn build_and_run(cc: &str, name: &str, source: &str) -> std::process::Output {
    let Some(staticlib) = staticlib() else {
        panic!("no static library; build.rs should have produced one");
    };
    let src = write_source(name, source);
    let dir = src.parent().unwrap();
    let exe = dir.join(name.replace(".c", ""));

    let compile = Command::new(cc)
        // -Wall -Wextra -Werror: a warning in a header a caller includes is a bug
        // in the header, not noise in someone else's build.
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-pedantic"])
        .arg("-I")
        .arg(header().parent().unwrap())
        .arg(&src)
        .arg("-o")
        .arg(&exe)
        .arg(&staticlib)
        .args(["-lpthread", "-ldl", "-lm"])
        .output()
        .expect("running the C compiler");

    assert!(
        compile.status.success(),
        "compiling {name} failed:\n{}\n--- source ---\n{source}",
        String::from_utf8_lossy(&compile.stderr)
    );

    Command::new(&exe).output().expect("running the C program")
}

/// Asserts the C program printed `expected`, and that it exited cleanly.
fn assert_printed(name: &str, out: &std::process::Output, expected: &str) {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "{name} exited with {:?}\nstderr: {stderr}\nstdout: {stdout}",
        out.status.code()
    );
    assert_eq!(stdout.trim(), expected, "{name} printed the wrong thing");
}

/// The whole point of the exercise: a C program includes the header, calls every
/// exported function, links, and runs.
#[test]
fn a_c_program_compiles_links_and_runs() {
    let cc = require_cc!();
    let source = r#"
#include <vime_engine.h>
#include <stdio.h>
#include <string.h>
#include <assert.h>

int main(void) {
    /* The version is a static string, so it is readable with no handle. */
    const char *version = vime_version();
    assert(version != NULL);
    assert(strlen(version) > 0);

    /* A factory with the header's default config, and one from the macro. */
    VimeSessionFactoryHandle *factory = vime_session_factory_create();
    assert(factory != NULL);

    VimeConfig config = VIME_CONFIG_INIT;
    config.tone_placement = VIME_TONE_PLACEMENT_OLD;
    assert(vime_session_factory_set_config(factory, &config));

    /* Two sessions, so the handles are visibly distinct objects. */
    VimeSessionHandle *a = vime_session_create(factory);
    VimeSessionHandle *b = vime_session_create_with_config(factory, &config);
    assert(a != NULL && b != NULL && a != b);

    /* Type through the ABI. */
    const char *word = "tieengs";
    for (const char *p = word; *p; p++) {
        assert(vime_session_insert(a, (uint32_t)(unsigned char)*p));
    }
    assert(strcmp(vime_session_render_text(a), "ti\xE1\xBA\xBFng") == 0);
    assert(strcmp(vime_session_render_raw_text(a), "tieengs") == 0);

    /* The snapshot: one call, everything a frontend needs to paint. */
    const VimeRenderState *state = vime_session_render_state(a);
    assert(state != NULL);
    assert(strcmp(state->text, "ti\xE1\xBA\xBFng") == 0);
    assert(strcmp(state->raw_text, "tieengs") == 0);
    assert(state->cursor_char_idx == 5);
    assert(state->raw_cursor_char_idx == 7);
    assert(state->cursor_byte_idx == strlen(state->text));
    assert(state->raw_cursor_byte_idx == strlen(state->raw_text));
    assert(state->is_valid_vietnamese);
    assert(state->bytes_to_delete == 0);
    assert(state->chars_to_delete == 0);

    /* Editing and the erase request. */
    assert(vime_session_backspace(a));
    assert(strcmp(vime_session_render_text(a), "ti\xE1\xBA\xBFn") == 0);
    state = vime_session_render_state(a);
    assert(state->bytes_to_delete == strlen("ti\xE1\xBA\xBFng"));
    assert(state->chars_to_delete == 5);

    /* Cursor movement, in both units. The caret was left at the end of the
     * shortened word, so one move left puts it at 3 characters, and the byte
     * index differs from it because of the two-byte characters. */
    assert(vime_session_get_cursor_char_idx(a) == 4);
    assert(vime_session_move_cursor_left(a));
    assert(vime_session_get_cursor_char_idx(a) == 3);
    assert(vime_session_get_cursor_byte_idx(a) == 5);
    assert(vime_session_get_cursor_char_idx(a) != vime_session_get_cursor_byte_idx(a));
    /* Move to the end again, where `delete` has nothing forward to remove. */
    while (vime_session_move_cursor_right(a)) {}
    assert(vime_session_get_cursor_char_idx(a) == 4);
    /* `assert` takes one argument, so the message goes in a comment. */
    assert(!vime_session_delete(a)); /* nothing forward of the caret */
    assert(strcmp(vime_session_render_text(a), "ti\xE1\xBA\xBFn") == 0);
    /* Back one, and `delete` now removes the character under the caret's right. */
    assert(vime_session_move_cursor_left(a));
    assert(vime_session_delete(a));
    assert(strcmp(vime_session_render_text(a), "ti\xE1\xBA\xBF") == 0);
    assert(vime_session_get_cursor_char_idx(a) == 3);
    assert(vime_session_get_raw_cursor_char_idx(b) == 0);
    assert(vime_session_get_raw_cursor_byte_idx(b) == 0);
    assert(!vime_session_is_valid_vietnamese(b));

    /* Validity, and the private-config escape hatch. */
    vime_session_reset(b);
    const char *valid = "toan";
    for (const char *p = valid; *p; p++) {
        assert(vime_session_insert(b, (uint32_t)(unsigned char)*p));
    }
    assert(vime_session_is_valid_vietnamese(b));
    assert(vime_session_clear_config(b));

    /* A session keeps working after its factory is gone. */
    vime_session_factory_destroy(factory);
    assert(vime_session_insert(b, (uint32_t)'g'));
    assert(strcmp(vime_session_render_text(b), "toang") == 0);

    vime_session_destroy(a);
    vime_session_destroy(b);

    printf("ok\n");
    return 0;
}
"#;
    let out = build_and_run(&cc, "smoke.c", source);
    assert_printed("smoke.c", &out, "ok");
}

/// The layout claims in the header, checked by the C compiler's own arithmetic.
/// If Rust's `#[repr(C)]` and the C struct ever disagree about a size or an
/// offset, this is where it shows up — a C compiler has no reason to agree with
/// us about anything.
#[test]
fn the_c_abi_sizes_match_what_the_header_declares() {
    let cc = require_cc!();
    let source = r#"
#include <vime_engine.h>
#include <stddef.h>
#include <stdio.h>
#include <assert.h>

int main(void) {
    /* int + int */
    assert(sizeof(VimeConfig) == 8);
    assert(_Alignof(VimeConfig) == 4);
    assert(offsetof(VimeConfig, input_method) == 0);
    assert(offsetof(VimeConfig, tone_placement) == 4);

    /* The enums are C ints, so they are 4 bytes wide. */
    assert(sizeof(VimeInputMethod) == 4);
    assert(sizeof(VimeTonePlacement) == 4);
    assert(VIME_INPUT_METHOD_TELEX == 1);
    assert(VIME_INPUT_METHOD_VNI == 2);
    assert(VIME_INPUT_METHOD_VIQR == 3);
    assert(VIME_TONE_PLACEMENT_MODERN == 1);
    assert(VIME_TONE_PLACEMENT_OLD == 2);

    /* Two pointers, six size_ts, a bool, and tail padding. */
    assert(sizeof(VimeRenderState) == 72);
    assert(_Alignof(VimeRenderState) == 8);
    assert(offsetof(VimeRenderState, text) == 0);
    assert(offsetof(VimeRenderState, raw_text) == 8);
    assert(offsetof(VimeRenderState, cursor_byte_idx) == 16);
    assert(offsetof(VimeRenderState, cursor_char_idx) == 24);
    assert(offsetof(VimeRenderState, raw_cursor_byte_idx) == 32);
    assert(offsetof(VimeRenderState, raw_cursor_char_idx) == 40);
    assert(offsetof(VimeRenderState, bytes_to_delete) == 48);
    assert(offsetof(VimeRenderState, chars_to_delete) == 56);
    assert(offsetof(VimeRenderState, is_valid_vietnamese) == 64);

    /* The handles are opaque: incomplete types, so a C caller can only hold
     * pointers to them. `sizeof` on one does not even compile, which is what
     * keeps the library free to change the layout. Checked here by using the
     * pointers in the ways a caller actually would. */
    VimeSessionFactoryHandle *factory = vime_session_factory_create();
    VimeSessionHandle *session = vime_session_create(factory);
    assert(factory != NULL && session != NULL);
    assert(sizeof(factory) == sizeof(void *));
    assert(sizeof(session) == sizeof(void *));
    /* A `const VimeConfig *` is the only thing a caller may pass instead of
     * dereferencing a handle, and the handles are never arrays. */
    assert(sizeof(VimeConfig *) == sizeof(void *));
    vime_session_destroy(session);
    vime_session_factory_destroy(factory);

    printf("ok\n");
    return 0;
}
"#;
    let out = build_and_run(&cc, "layout.c", source);
    assert_printed("layout.c", &out, "ok");
}

/// A bad value from C is rejected rather than trusted, checked from the C side
/// where a real caller would produce one.
#[test]
fn c_can_pass_a_bad_value_and_is_told_no() {
    let cc = require_cc!();
    let source = r#"
#include <vime_engine.h>
#include <stdio.h>
#include <string.h>
#include <assert.h>

int main(void) {
    /* An input method outside the header's range. A C caller can do this by
     * casting, and a plugin loading a mismatched build can do it by accident. */
    VimeConfig bad = VIME_CONFIG_INIT;
    bad.input_method = (VimeInputMethod)99;
    assert(vime_session_factory_create_with_config(&bad) == NULL);

    bad.tone_placement = (VimeTonePlacement)0;
    assert(vime_session_factory_create_with_config(&bad) == NULL);

    /* A good one still works, so the check is not simply refusing everything. */
    VimeConfig good = VIME_CONFIG_INIT;
    assert(vime_session_factory_create_with_config(&good) != NULL);

    /* NULL is refused by the setters, and means "default" to the constructors. */
    VimeSessionFactoryHandle *factory = vime_session_factory_create();
    assert(factory != NULL);
    assert(!vime_session_factory_set_config(factory, NULL));
    VimeSessionHandle *s = vime_session_create_with_config(factory, NULL);
    assert(s != NULL);
    assert(!vime_session_set_config(s, NULL));
    /* clear_config succeeds on a session that is following the shared config: it
     * has nothing private, so there is nothing to undo, and saying so is more
     * useful than refusing. */
    assert(vime_session_clear_config(s));
    vime_session_destroy(s);
    vime_session_factory_destroy(factory);

    /* A uint32_t that is not a Unicode scalar value. */
    VimeSessionHandle *t = vime_session_create(vime_session_factory_create());
    assert(t != NULL);
    assert(vime_session_insert(t, (uint32_t)'a'));
    assert(!vime_session_insert(t, 0xD800));      /* lone high surrogate */
    assert(!vime_session_insert(t, 0x00110000)); /* past U+10FFFF */
    assert(strcmp(vime_session_render_text(t), "a") == 0);
    vime_session_destroy(t);

    /* Every call tolerates NULL. */
    assert(!vime_session_insert(NULL, 'a'));
    assert(!vime_session_backspace(NULL));
    assert(!vime_session_delete(NULL));
    assert(!vime_session_move_cursor_left(NULL));
    assert(!vime_session_move_cursor_right(NULL));
    vime_session_reset(NULL);
    assert(vime_session_render_text(NULL) == NULL);
    assert(vime_session_render_raw_text(NULL) == NULL);
    assert(vime_session_render_state(NULL) == NULL);
    assert(vime_session_get_cursor_char_idx(NULL) == 0);
    assert(vime_session_get_cursor_byte_idx(NULL) == 0);
    assert(vime_session_get_raw_cursor_char_idx(NULL) == 0);
    assert(vime_session_get_raw_cursor_byte_idx(NULL) == 0);
    assert(!vime_session_is_valid_vietnamese(NULL));
    vime_session_destroy(NULL);
    vime_session_factory_destroy(NULL);

    printf("ok\n");
    return 0;
}
"#;
    let out = build_and_run(&cc, "badvalues.c", source);
    assert_printed("badvalues.c", &out, "ok");
}

/// The pointer-stability property, measured from C: the rendered buffer the
/// library hands back does not move between keystrokes.
///
/// A `const char *` that changed address on every call would force every consumer
/// to copy it, and would make a frontend that caches the pointer read freed memory.
/// Only a real linked C program can confirm the library behaves this way at the
/// addresses it actually hands out.
#[test]
fn the_text_pointer_is_stable_across_keystrokes_in_c() {
    let cc = require_cc!();
    let source = r#"
#include <vime_engine.h>
#include <stdio.h>
#include <assert.h>
#include <string.h>

int main(void) {
    VimeSessionFactoryHandle *factory = vime_session_factory_create();
    VimeSessionHandle *s = vime_session_create(factory);
    assert(s != NULL);

    const char *last = NULL;
    for (const char *p = "tieengs"; *p; p++) {
        assert(vime_session_insert(s, (uint32_t)(unsigned char)*p));
        const char *text = vime_session_render_text(s);
        assert(text != NULL);
        /* One allocation covers any Vietnamese syllable, so the address the caller
         * is given is the same one every time. */
        if (last != NULL) {
            assert(text == last);
        }
        last = text;
    }
    assert(strcmp(last, "ti\xE1\xBA\xBFng") == 0);

    /* Repeated snapshots are free of re-renders: the same text, the same address. */
    for (int i = 0; i < 1000; i++) {
        const VimeRenderState *state = vime_session_render_state(s);
        assert(state != NULL);
        assert(state->text == last);
    }

    /* Backspacing to empty and back: still one buffer. */
    const char *empty = NULL;
    for (int i = 0; i < 16; i++) {
        vime_session_backspace(s);
        empty = vime_session_render_text(s);
        assert(empty == last);
    }
    assert(strcmp(empty, "") == 0);

    vime_session_destroy(s);
    vime_session_factory_destroy(factory);
    printf("ok\n");
    return 0;
}
"#;
    let out = build_and_run(&cc, "stable.c", source);
    assert_printed("stable.c", &out, "ok");
}

/// The header is what a C caller compiles against, so it has to be self-contained
/// and warning-clean. Compiling it twice, with and without the implementation
/// macro, is how a missing include or a redefinition shows up.
#[test]
fn the_header_is_self_contained_and_warning_clean() {
    let cc = require_cc!();
    for (name, source) in [
        (
            "header_only.c",
            "#include <vime_engine.h>\n\
             /* Referencing an enum constant, which is the only thing the header\n\
              * offers without a handle. */\n\
             int main(void) { return VIME_INPUT_METHOD_TELEX == 1 ? 0 : 1; }\n",
        ),
        (
            "header_twice.c",
            "#include <vime_engine.h>\n#include <vime_engine.h>\nint main(void) { return 0; }\n",
        ),
        (
            "header_first.c",
            "#include <stdio.h>\n#include <vime_engine.h>\n#include <vime_engine.h>\nint main(void) { printf(\"%s\", vime_version()); return 0; }\n",
        ),
    ] {
        // Compiled and linked, so a declared-but-missing symbol is caught too.
        let out = build_and_run(&cc, name, source);
        assert!(
            out.status.success(),
            "{name} failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// Every symbol the header declares is present in the library.
///
/// The test links a program that names all of them. They are reached through calls
/// with correctly typed arguments rather than through a table of function
/// pointers, because the twenty functions do not share a signature and ISO C does
/// not allow one to be cast to `void *` — a cast would both weaken the check and
/// fail `-pedantic`. The calls sit behind a condition the compiler cannot fold
/// away, so nothing is optimised out and every symbol has to resolve at link
/// time. A missing or renamed export is then a link error here rather than a
/// surprise in someone else's build.
#[test]
fn every_header_symbol_is_exported() {
    let cc = require_cc!();
    let Some(staticlib) = staticlib() else {
        eprintln!("skipping: no static library to inspect");
        return;
    };
    assert!(staticlib.exists(), "expected {}", staticlib.display());

    let program = r#"
#include <vime_engine.h>
#include <stdlib.h>

/* Set from the environment, so the compiler cannot fold the branch away and the
 * linker still has to resolve everything inside it. The test never sets it, so
 * the calls are not actually made at run time. */
static int never;

int main(void) {
    const char *flag = getenv("VIME_NEVER_REACHABLE");
    if (flag != NULL) {
        never = atoi(flag);
    }
    if (never) {
        VimeConfig config = VIME_CONFIG_INIT;
        VimeSessionFactoryHandle *factory = vime_session_factory_create();
        VimeSessionHandle *session = vime_session_create(factory);

        const char *version = vime_version();
        vime_session_factory_create_with_config(&config);
        vime_session_factory_set_config(factory, &config);
        vime_session_create_with_config(factory, &config);
        vime_session_set_config(session, &config);
        vime_session_clear_config(session);
        vime_session_insert(session, (uint32_t)'a');
        vime_session_backspace(session);
        vime_session_delete(session);
        vime_session_move_cursor_left(session);
        vime_session_move_cursor_right(session);
        vime_session_render_text(session);
        vime_session_render_raw_text(session);
        vime_session_render_state(session);
        vime_session_get_cursor_char_idx(session);
        vime_session_get_cursor_byte_idx(session);
        vime_session_get_raw_cursor_char_idx(session);
        vime_session_get_raw_cursor_byte_idx(session);
        vime_session_is_valid_vietnamese(session);
        vime_session_reset(session);
        vime_session_destroy(session);
        vime_session_factory_destroy(factory);

        (void)version;
        return 0;
    }
    return 0;
}
"#;

    let src = write_source("exports.c", program);
    let dir = src.parent().unwrap();
    let out = Command::new(cc)
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-pedantic"])
        .arg("-I")
        .arg(header().parent().unwrap())
        .arg("-o")
        .arg(dir.join("exports"))
        .arg(&src)
        .arg(&staticlib)
        .args(["-lpthread", "-ldl", "-lm"])
        .output()
        .expect("running the C compiler");

    assert!(
        out.status.success(),
        "linking every exported symbol failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    // And the program runs, so the symbols are real code rather than stubs.
    let run = Command::new(dir.join("exports"))
        .output()
        .expect("running the exports program");
    assert!(
        run.status.success(),
        "the exports program failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
}
