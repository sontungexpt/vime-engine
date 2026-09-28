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
 * Handle to an active Vietnamese input engine instance.
 * Thread-safety: Not thread-safe. Synchronization is the caller's duty.
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
 * ## Versioning
 *
 * struct_size is the size the caller compiled against. A library that gains a
 * field appends it and does not change the leading fields, so an older caller
 * still passes the smaller size and the library reads only the fields that
 * were present. This is the same convention as XkbGetRules and FcConfigSet.
 *
 * A zero size means "no configuration": every field takes its default. That is
 * also what a NULL pointer means, so the two are interchangeable.
 *
 * Initialise with VIME_CONFIG_INIT so a caller that sets no fields still
 * passes a valid size.
 */
typedef struct VimeConfig {
    /* sizeof(VimeConfig) as the caller knows it; 0 selects every default. */
    uint32_t struct_size;

    /* Restore English when the word is empty. Mirrors the engine's
     * Config::auto_restore_english.
     *
     * Added in revision 2. A caller compiled against revision 1 reports
     * struct_size == 4 and never wrote this byte; the library ignores it in
     * that case rather than reading whatever the padding held, and uses the
     * engine default. The default is true, which is not the C zero, so a
     * caller that says nothing must not be answered with false. */
    bool auto_restore_english;

    /* Fields are appended below in later revisions. Read one only after
     * checking that struct_size covers it. */
} VimeConfig;

/* The value a caller must store in VimeConfig::struct_size. */
#define VIME_CONFIG_INIT \
    ((VimeConfig){ .struct_size = (uint32_t)sizeof(VimeConfig) })

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
 * config may be NULL, which means every default; see VimeConfig for the
 * versioning rules. Returns NULL for an unknown input method or a
 * struct_size this library does not recognise.
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
