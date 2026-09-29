#ifndef VIME_ENGINE_H
#define VIME_ENGINE_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================= */
/* Opaque Types                                                              */
/* ========================================================================= */

/**
 * Handle to one Vietnamese typing buffer, plus the text it hands back.
 *
 * A handle is a single session: it owns the keystrokes typed into it and the
 * word they parse to. A frontend that needs several buffers — one per window,
 * per conversation, or per test case — holds several handles, and they share
 * nothing: their buffers are independent even when they hold identical
 * settings.
 *
 * Because of that, vime_set_input_method and vime_set_tone_placement apply to
 * the handle they are given and to no other. A frontend that wants every
 * buffer to move together has to call them on each handle; there is no
 * handle-wide setting, and a handle never picks up a change made to another.
 *
 * Thread-safety: Not thread-safe. Synchronization is the caller's duty, and
 * covers all handles, since they are independent rather than isolated.
 */
typedef struct VimeEngineHandle VimeEngineHandle;

/* ========================================================================= */
/* Enumerations                                                              */
/* ========================================================================= */

/**
 * High-level action directive returned to native frontends.
 */
typedef enum VimeAction {
    VIME_ACTION_FORWARD        = 0, /* key ignored by IME; forward it to the app */
    VIME_ACTION_NOOP           = 1, /* key consumed; nothing visibly changed     */
    VIME_ACTION_CHANGED        = 2, /* buffer changed; re-read and redraw     */
    VIME_ACTION_COMMIT         = 3, /* text committed; clear the word, insert  */
    VIME_ACTION_CURSOR_MOVED   = 4, /* caret moved; the word is unchanged      */
} VimeAction;

/**
 * Exclusive input-method selection. Values reflect the ABI agreement with Rust backend.
 */
typedef enum VimeInputMethod {
    VIME_INPUT_METHOD_TELEX = 1u,
    VIME_INPUT_METHOD_VNI   = 2u,
    VIME_INPUT_METHOD_VIQR  = 3u,
} VimeInputMethod;

/**
 * Tone-placement scheme. Values reflect the ABI agreement with Rust backend.
 */
typedef enum VimeTonePlacement {
    VIME_TONE_PLACEMENT_MODERN = 1u, /* "hoá", "thuý"                       */
    VIME_TONE_PLACEMENT_OLD    = 2u, /* "hóa", "thúy"                       */
} VimeTonePlacement;

/**
 * Discrete key codes. Values match the engine's internal Key enum.
 */
typedef enum VimeKey {
    /* The event carries a character, in VimeKeyEvent::character. The core
     * Key::Character holds a char, which a flat struct cannot, so this
     * variant is payload-free and the character travels beside it. Every
     * other variant is a discrete key with no character. */
    VIME_KEY_CHARACTER = 0,
    VIME_KEY_BACKSPACE = 1,
    VIME_KEY_DELETE    = 2,
    VIME_KEY_LEFT      = 3,
    VIME_KEY_RIGHT     = 4,
    VIME_KEY_ENTER     = 5,
    VIME_KEY_ESCAPE    = 6,
    VIME_KEY_TAB       = 7,
    VIME_KEY_SPACE     = 8,
} VimeKey;

/* ========================================================================= */
/* Structures                                                                */
/* ========================================================================= */

/**
 * Modifier-state bits for `VimeKeyEvent::states`. Values match the engine's
 * internal KeyStates bitmask; frontends must translate native modifiers.
 */
#define VIME_KEY_STATE_CTRL      (1u << 0)
#define VIME_KEY_STATE_ALT       (1u << 1)
#define VIME_KEY_STATE_SHIFT     (1u << 2)
#define VIME_KEY_STATE_SUPER     (1u << 3)
#define VIME_KEY_STATE_CAPS_LOCK (1u << 4)
#define VIME_KEY_STATE_NUM_LOCK  (1u << 5)
#define VIME_KEY_STATE_HYPER     (1u << 6)
#define VIME_KEY_STATE_META      (1u << 7)

typedef struct VimeKeyEvent {
    VimeKey key;
    uint32_t character;
    uint32_t states;
} VimeKeyEvent;

typedef struct VimeOutput {
    /* high-level action for the frontend state machine */
    VimeAction action;
    /* Text to commit (UTF-8, NUL-terminated), or NULL if there is none.
     *
     * This is the only way to obtain the commit text: it is set when
     * `action` is VIME_ACTION_COMMIT and NULL for every other action, so
     * there is no separate accessor to call and no window in which a
     * frontend can read a stale commit.
     *
     * Owned by the handle, valid until the next call that changes the state
     * on the same handle, or vime_destroy. Do not free it. */
    const char *commit;
} VimeOutput;

/* ========================================================================= */
/* Engine Lifecycle                                                          */
/* ========================================================================= */

/**
 * Engine settings, in the layout a C caller sees.
 *
 * Each field mirrors one field of the engine's own configuration, so a
 * frontend sets its behaviour without a bespoke call per setting. The input
 * method and tone-placement scheme are not here: they are runtime choices
 * with their own entry points.
 *
 * The library and this header are installed together, so a caller cannot end
 * up with a struct from one revision and a libvime from another. Adding a
 * field is therefore a normal breaking change.
 */
typedef struct VimeConfig {
    /* Restore English when the word contains characters that are not
     * Vietnamese, committing the literal text instead of interpreting it.
     * Defaults to true, which is not the C zero, so a caller that zero-initialises
     * the struct gets a different value from one that passes it zeroed. */
    bool auto_restore_english;
} VimeConfig;

/* The defaults: what a NULL config selects. */
#define VIME_CONFIG_INIT ((VimeConfig){ .auto_restore_english = true })

/** Creates a new Telex engine with the default configuration. */
VimeEngineHandle *vime_create(void);

/**
 * Creates an engine for the given input method and tone-placement scheme,
 * with every setting at its default.
 */
VimeEngineHandle *vime_create_with(VimeInputMethod method, VimeTonePlacement tone_placement);

/**
 * Creates an engine for the given input method, tone-placement scheme and
 * settings.
 *
 * config may be NULL, which means every default. Returns NULL for an
 * unknown input method.
 */
VimeEngineHandle *vime_create_with_config(const VimeConfig *config,
                                          VimeInputMethod method,
                                          VimeTonePlacement tone_placement);

/** Destroys an engine instance and frees associated memory. */
void vime_destroy(VimeEngineHandle *engine);

/**
 * Sets the active input method, clearing the buffer.
 *
 * Returns true on success, false for a NULL handle or an unknown method (in
 * which case the engine is untouched).
 *
 * On success the buffer was cleared, so the word has changed: call
 * vime_parsed and redraw. No key was consumed and no text is committed, so
 * there is no action to dispatch.
 */
bool vime_set_input_method(VimeEngineHandle *engine, VimeInputMethod method);

/**
 * Switches the tone-placement scheme, re-rendering the current word.
 *
 * Returns true on success, false for a NULL handle or an unknown scheme (in
 * which case the engine is untouched).
 *
 * On success the pending vowels render under the new scheme, so the word has
 * changed: call vime_parsed and redraw.
 */
bool vime_set_tone_placement(VimeEngineHandle *engine, VimeTonePlacement tone_placement);

/**
 * Returns the word currently parsed (UTF-8, NUL-terminated), or NULL if the
 * engine has no valid handle.
 *
 * Rendered lazily: the text is produced on the first call after a state change
 * and cached until the next call that changes the state, so a frontend that
 * only reacts to VIME_ACTION_COMMIT never pays for it. Call this whenever the
 * action is VIME_ACTION_CHANGED or VIME_ACTION_CURSOR_MOVED.
 *
 * The pointer is owned by the handle and stays valid until the next call that
 * changes the state, or vime_destroy. Do not free it.
 */
const char *vime_parsed(VimeEngineHandle *engine);

/**
 * Clears the buffer.
 *
 * Returns true on success, false for a NULL handle (in which case the engine
 * is untouched) — the same shape as vime_set_input_method and
 * vime_set_tone_placement.
 *
 * There is no action to dispatch: reset consumes no key and commits no text,
 * and on success the word is empty, so the frontend clears its preedit and
 * repaints.
 */
bool vime_reset(VimeEngineHandle *engine);

/* ========================================================================= */
/* Event Processing & Utilities                                              */
/* ========================================================================= */

/** Processes a key event and returns output directives for the frontend adapter. */
VimeOutput vime_process_key(VimeEngineHandle *engine, VimeKeyEvent event);

#ifdef __cplusplus
}
#endif

#endif /* VIME_ENGINE_H */
