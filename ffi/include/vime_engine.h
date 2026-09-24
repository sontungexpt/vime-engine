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
    VIME_ACTION_UPDATE_PREEDIT = 2, /* preedit updated; refresh the window       */
    VIME_ACTION_COMMIT         = 3, /* text committed; clear preedit, insert it  */
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
    VIME_KEY_NONE      = 0,
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
    /* preedit text (UTF-8), NULL if empty/unchanged;
     * owned by the handle, valid until the next call on
     * the same handle or vime_destroy */
    const char *rendered;
    /* text to commit (UTF-8), NULL if none; owned by the
     * handle, valid until the next call on the same
     * handle or vime_destroy */
    const char *commit;
} VimeOutput;

/* ========================================================================= */
/* Engine Lifecycle                                                          */
/* ========================================================================= */

/** Creates a new engine instance. Returns NULL on allocation failure. */
VimeEngineHandle *vime_create(void);

/** Creates an engine for the given input method and tone-placement scheme. */
VimeEngineHandle *vime_create_with(VimeInputMethod method, VimeTonePlacement tone_placement);

/** Destroys an engine instance and frees associated memory. */
void vime_destroy(VimeEngineHandle *engine);

/** Sets the active input method engine, clearing the buffer. */
VimeOutput vime_set_input_method(VimeEngineHandle *engine, VimeInputMethod method);

/** Switches the tone-placement scheme, re-rendering the current preedit. */
VimeOutput vime_set_tone_placement(VimeEngineHandle *engine, VimeTonePlacement tone_placement);

/** Resets the engine buffer state. */
VimeOutput vime_reset(VimeEngineHandle *engine);

/** Commits the pending buffer as text and clears the preedit. */
VimeOutput vime_commit(VimeEngineHandle *engine);

/* ========================================================================= */
/* Event Processing & Utilities                                              */
/* ========================================================================= */

/** Processes a key event and returns output directives for the frontend adapter. */
VimeOutput vime_process_key(VimeEngineHandle *engine, VimeKeyEvent event);

#ifdef __cplusplus
}
#endif

#endif /* VIME_ENGINE_H */
