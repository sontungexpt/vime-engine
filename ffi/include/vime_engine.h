#ifndef VIME_ENGINE_H
#define VIME_ENGINE_H

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================= */
/* Opaque Types                                                              */
/* ========================================================================= */

/**
 * Factory that owns the shared configuration and creates sessions.
 *
 * Sessions created from the factory follow its shared configuration unless
 * they have a private configuration.
 *
 * Thread-safety: not thread-safe. Synchronization is the caller's
 * responsibility.
 */
typedef struct VimeSessionFactoryHandle VimeSessionFactoryHandle;

/**
 * One independent typing session.
 *
 * Owns its input buffer, composition state, cursor state, and optionally
 * a private configuration.
 */
typedef struct VimeSessionHandle VimeSessionHandle;


/* ========================================================================= */
/* Enumerations                                                              */
/* ========================================================================= */

/**
 * Action returned after processing a key.
 */
typedef enum VimeAction {
    /**
     * The key was not consumed by the IME.
     * The frontend should forward it to the application.
     */
    VIME_ACTION_FORWARD = 0,

    /**
     * The key was consumed, but no visible state changed.
     */
    VIME_ACTION_NOOP = 1,

    /**
     * The composition changed.
     * The frontend should re-read the rendered composition.
     */
    VIME_ACTION_CHANGED = 2,

    /**
     * Text was committed.
     *
     * `VimeOutput.commit` contains the UTF-8 text to commit.
     */
    VIME_ACTION_COMMIT = 3,

    /**
     * The composition cursor moved without changing the composition text.
     */
    VIME_ACTION_CURSOR_MOVED = 4,
} VimeAction;


/**
 * Vietnamese input method.
 */
typedef enum VimeInputMethod {
    VIME_INPUT_METHOD_TELEX = 1u,
    VIME_INPUT_METHOD_VNI   = 2u,
    VIME_INPUT_METHOD_VIQR  = 3u,
} VimeInputMethod;


/**
 * Vietnamese tone-placement convention.
 */
typedef enum VimeTonePlacement {
    VIME_TONE_PLACEMENT_MODERN = 1u,
    VIME_TONE_PLACEMENT_OLD    = 2u,
} VimeTonePlacement;


/**
 * Discrete key codes.
 *
 * VIME_KEY_CHARACTER uses `VimeKeyEvent.character`.
 */
typedef enum VimeKey {
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
/* Modifier State                                                             */
/* ========================================================================= */

#define VIME_KEY_STATE_CTRL      (1u << 0)
#define VIME_KEY_STATE_ALT       (1u << 1)
#define VIME_KEY_STATE_SHIFT     (1u << 2)
#define VIME_KEY_STATE_SUPER     (1u << 3)
#define VIME_KEY_STATE_CAPS_LOCK (1u << 4)
#define VIME_KEY_STATE_NUM_LOCK  (1u << 5)
#define VIME_KEY_STATE_HYPER     (1u << 6)
#define VIME_KEY_STATE_META      (1u << 7)


/* ========================================================================= */
/* Structures                                                                 */
/* ========================================================================= */

/**
 * Key event passed to a session.
 */
typedef struct VimeKeyEvent {
    VimeKey key;

    /**
     * Unicode scalar value for VIME_KEY_CHARACTER.
     *
     * Ignored for other key types.
     */
    uint32_t character;

    /**
     * Combination of VIME_KEY_STATE_* flags.
     */
    uint32_t states;
} VimeKeyEvent;


/**
 * Result of processing one key.
 *
 * `commit` is non-NULL only when action == VIME_ACTION_COMMIT.
 *
 * The returned pointer is owned by the session and must not be freed.
 * It remains valid until the next operation that changes the session state
 * or until vime_session_destroy().
 */
typedef struct VimeOutput {
    VimeAction action;
    const char *commit;
} VimeOutput;


/**
 * Complete configuration used by sessions.
 *
 * The factory stores one shared configuration. Sessions normally follow it,
 * unless a session has explicitly taken a private configuration.
 */
typedef struct VimeConfig {
    bool auto_restore_english;
    VimeInputMethod input_method;
    VimeTonePlacement tone_placement;
} VimeConfig;


/**
 * Default configuration.
 */
#define VIME_CONFIG_INIT \
    ((VimeConfig){ \
        .auto_restore_english = true, \
        .input_method = VIME_INPUT_METHOD_TELEX, \
        .tone_placement = VIME_TONE_PLACEMENT_MODERN \
    })


/* ========================================================================= */
/* Session Factory — Lifecycle                                               */
/* ========================================================================= */

/**
 * Creates a factory with the default configuration.
 *
 * Equivalent to:
 *
 *     vime_session_factory_create_with_config(&VIME_CONFIG_INIT)
 */
VimeSessionFactoryHandle *vime_session_factory_create(void);


/**
 * Creates a factory with the supplied configuration.
 *
 * `config` may be NULL, in which case VIME_CONFIG_INIT is used.
 *
 * Returns NULL if the configuration contains an invalid enum value.
 */
VimeSessionFactoryHandle *vime_session_factory_create_with_config(
    const VimeConfig *config
);


/**
 * Destroys a factory.
 *
 * Existing sessions created by the factory remain valid if the implementation
 * retains their shared configuration independently.
 */
void vime_session_factory_destroy(
    VimeSessionFactoryHandle *factory
);


/* ========================================================================= */
/* Session Factory — Shared Configuration                                    */
/* ========================================================================= */

/**
 * Replaces the shared configuration.
 *
 * Existing sessions that follow the shared configuration observe the change
 * on their next operation.
 *
 * Sessions with a private configuration are unaffected.
 *
 * Returns false for NULL handles or invalid enum values.
 */
bool vime_session_factory_set_config(
    VimeSessionFactoryHandle *factory,
    const VimeConfig *config
);


/**
 * Changes the shared input method.
 *
 * Sessions following the shared configuration observe the new method on
 * their next operation.
 */
bool vime_session_factory_set_input_method(
    VimeSessionFactoryHandle *factory,
    VimeInputMethod method
);


/**
 * Changes the shared tone-placement convention.
 *
 * Sessions following the shared configuration observe the new convention on
 * their next operation.
 */
bool vime_session_factory_set_tone_placement(
    VimeSessionFactoryHandle *factory,
    VimeTonePlacement tone_placement
);


/**
 * Changes the shared English auto-restore setting.
 */
bool vime_session_factory_set_auto_restore_english(
    VimeSessionFactoryHandle *factory,
    bool enabled
);


/* ========================================================================= */
/* Session — Lifecycle                                                       */
/* ========================================================================= */

/**
 * Creates a new empty session following the factory's shared configuration.
 */
VimeSessionHandle *vime_session_create(
    VimeSessionFactoryHandle *factory
);


/**
 * Creates a new empty session with a private configuration.
 *
 * The session does not follow subsequent shared configuration changes until
 * its private configuration is cleared.
 */
VimeSessionHandle *vime_session_create_with_config(
    VimeSessionFactoryHandle *factory,
    const VimeConfig *config
);


/**
 * Destroys a session.
 */
void vime_session_destroy(
    VimeSessionHandle *session
);


/* ========================================================================= */
/* Session — Configuration                                                   */
/* ========================================================================= */

/**
 * Replaces this session's configuration with a private configuration.
 *
 * After this call, changes to the factory's shared configuration no longer
 * affect this session.
 */
bool vime_session_set_config(
    VimeSessionHandle *session,
    const VimeConfig *config
);


/**
 * Clears the session's private configuration.
 *
 * The session resumes following the factory's current shared configuration.
 */
bool vime_session_clear_config(
    VimeSessionHandle *session
);


/* ========================================================================= */
/* Session — Input Processing                                                */
/* ========================================================================= */

/**
 * Processes one key event.
 *
 * The returned action tells the frontend what to do next.
 *
 * For VIME_ACTION_COMMIT, `output.commit` contains the text to commit.
 */
VimeOutput vime_session_process_key(
    VimeSessionHandle *session,
    VimeKeyEvent event
);


/* ========================================================================= */
/* Session — Composition                                                     */
/* ========================================================================= */

/**
 * Returns the currently rendered composition.
 *
 * Returns:
 *
 *     NULL    invalid session
 *     ""      empty composition
 *     text    current rendered composition
 *
 * The returned UTF-8 string is owned by the session and must not be freed.
 *
 * The pointer remains valid until the next operation that changes the
 * session's rendered state or until vime_session_destroy().
 */
const char *vime_session_render(
    VimeSessionHandle *session
);


/**
 * Clears the current composition.
 *
 * Returns false for a NULL session.
 */
bool vime_session_reset(
    VimeSessionHandle *session
);


/* ========================================================================= */
/* Session — Cursor                                                          */
/* ========================================================================= */

/**
 * Returns the current composition cursor position.
 *
 * The position is expressed in rendered Unicode characters, not UTF-8 bytes.
 *
 * Returns SIZE_MAX for an invalid session.
 */
size_t vime_session_cursor(
    const VimeSessionHandle *session
);


/**
 * Returns the rendered composition length in Unicode characters.
 *
 * Returns SIZE_MAX for an invalid session.
 */
size_t vime_session_length(
    const VimeSessionHandle *session
);


/* ========================================================================= */
/* Version                                                                     */
/* ========================================================================= */

/**
 * Returns the VIME ABI version.
 *
 * The returned string is static and must not be freed.
 */
const char *vime_version(void);


#ifdef __cplusplus
}
#endif

#endif /* VIME_ENGINE_H */
