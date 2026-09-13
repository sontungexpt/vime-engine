#pragma once

/* C ABI boundary between native frontend adapters (Fcitx5, IBus, macOS, Windows)
 * and the pure Rust Vietnamese IME engine. */

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Key event state — vime-engine's canonical KeyState bitmask.
 * Bitmask ORed into VimeKeyEvent.state. Frontends must translate native
 * modifier states into these exact values. */
#define VIME_KEY_STATE_CTRL      (1u << 0)
#define VIME_KEY_STATE_ALT       (1u << 1)
#define VIME_KEY_STATE_SHIFT     (1u << 2)
#define VIME_KEY_STATE_SUPER     (1u << 3)
#define VIME_KEY_STATE_CAPS_LOCK (1u << 4)
#define VIME_KEY_STATE_NUM_LOCK   (1u << 5)
#define VIME_KEY_STATE_HYPER     (1u << 6)
#define VIME_KEY_STATE_META      (1u << 7)

/* ========================================================================= */
/* Opaque Handles & Forward Declarations                                     */
/* ========================================================================= */

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
} VimeInputMethod;

/**
 * Discrete key codes. Values match the engine's internal Key enum.
 */
typedef enum VimeKey {
    VIME_KEY_NONE      = 0u,
    VIME_KEY_BACKSPACE = 1u,
    VIME_KEY_DELETE    = 2u,
    VIME_KEY_LEFT      = 3u,
    VIME_KEY_RIGHT     = 4u,
    VIME_KEY_ENTER     = 5u,
    VIME_KEY_ESCAPE    = 6u,
    VIME_KEY_TAB       = 7u,
    VIME_KEY_SPACE     = 8u,
} VimeKey;

/* ========================================================================= */
/* Structures                                                                */
/* ========================================================================= */

typedef struct VimeKeyEvent {
    VimeKey key;
    uint32_t character;
    uint32_t states; /* Engine-owned KeyState bitmask (VIME_KEY_STATE_*) */
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
/* Engine Lifecycle & Configuration APIs                                     */
/* ========================================================================= */

/** Creates a new engine instance. Returns NULL on allocation failure. */
VimeEngineHandle *vime_create(void);

/** Destroys an engine instance and frees associated memory. */
void vime_destroy(VimeEngineHandle *engine);

/** Sets the active input method engine (Telex / VNI). */
void vime_set_input_method(VimeEngineHandle *engine, VimeInputMethod method);

/** Resets the engine buffer state. */
VimeOutput vime_reset(VimeEngineHandle *engine);

/* ========================================================================= */
/* Event Processing & Utilities                                              */
/* ========================================================================= */

/** Processes a key event and returns output directives for the frontend adapter. */
VimeOutput vime_process_key(VimeEngineHandle *engine, VimeKeyEvent event);

#ifdef __cplusplus
}
#endif
