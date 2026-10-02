#ifndef VIME_ENGINE_H
#define VIME_ENGINE_H

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================= */
/* Opaque Handles                                                            */
/* ========================================================================= */

typedef struct VimeSessionFactoryHandle VimeSessionFactoryHandle;
typedef struct VimeSessionHandle VimeSessionHandle;

/* ========================================================================= */
/* Enums & Config Struct                                                     */
/* ========================================================================= */

typedef uint32_t VimeInputMethod;

#define VIME_INPUT_METHOD_TELEX ((VimeInputMethod)1u)
#define VIME_INPUT_METHOD_VNI   ((VimeInputMethod)2u)
#define VIME_INPUT_METHOD_VIQR  ((VimeInputMethod)3u)

typedef uint32_t VimeTonePlacement;

#define VIME_TONE_PLACEMENT_MODERN ((VimeTonePlacement)1u)
#define VIME_TONE_PLACEMENT_OLD    ((VimeTonePlacement)2u)

typedef struct VimeConfig {
    VimeInputMethod input_method;
    VimeTonePlacement tone_placement;
} VimeConfig;

/**
 * Helper macro for default C-style configuration initialization.
 */
#define VIME_CONFIG_DEFAULT \
    ((VimeConfig){ \
        VIME_INPUT_METHOD_TELEX, \
        VIME_TONE_PLACEMENT_MODERN \
    })

/* ========================================================================= */
/* Session Factory APIs                                                      */
/* ========================================================================= */

/**
 * Creates a new Session Factory with default configuration (Telex, Modern tone).
 * @return Pointer to factory handle, or NULL on memory allocation failure.
 */
VimeSessionFactoryHandle *vime_session_factory_create(void);

/**
 * Creates a new Session Factory with custom initial configuration.
 * @param config Custom configuration parameters.
 * @return Pointer to factory handle, or NULL on failure.
 */
VimeSessionFactoryHandle *vime_session_factory_create_with_config(const VimeConfig *config);

/**
 * Destroys a Session Factory and frees associated memory.
 * @param factory Pointer to factory handle.
 */
void vime_session_factory_destroy(VimeSessionFactoryHandle *factory);

/**
 * Updates the SharedConfig of the Factory.
 * All active sessions sharing this config (without private overrides)
 * will automatically update on their next operation.
 * @return true if successfully updated, false if factory is NULL.
 */
bool vime_session_factory_set_config(VimeSessionFactoryHandle *factory, const VimeConfig *config);

/**
 * Returns the current shared configuration.
 *
 * The configuration is returned by value.
 */
VimeConfig vime_session_factory_get_config(const VimeSessionFactoryHandle *factory);


/* ========================================================================= */
/* Session Lifecycle APIs                                                    */
/* ========================================================================= */

/**
 * Creates a new Session bound to the Factory's SharedConfig.
 * @param factory Parent factory handle.
 * @return Pointer to session handle, or NULL on failure.
 */
VimeSessionHandle *vime_session_create(VimeSessionFactoryHandle *factory);

/**
 * Creates a new Session with a private config override.
 * @param factory Parent factory handle.
 * @param config Private configuration for this session.
 * @return Pointer to session handle, or NULL on failure.
 */
VimeSessionHandle *vime_session_create_with_config(VimeSessionFactoryHandle *factory, const VimeConfig *config);

/**
 * Destroys a Session and frees its associated rendering caches.
 * @param session Pointer to session handle.
 */
void vime_session_destroy(VimeSessionHandle *session);

/**
 * Clears input buffers and resets session state to initial conditions.
 * Safe to call with NULL pointer (no-op).
 */
void vime_session_reset(VimeSessionHandle *session);

/* ========================================================================= */
/* Session Config Isolation APIs                                             */
/* ========================================================================= */

/**
 * Sets a private config for the session, unlinking it from Factory SharedConfig.
 * @return true on success, false if session or config is NULL.
 */
bool vime_session_set_config(VimeSessionHandle *session, const VimeConfig *config);

/**
 * Removes private config, reverting session to Factory SharedConfig.
 * @return true on success, false if session is NULL.
 */
bool vime_session_clear_config(VimeSessionHandle *session);

/* ========================================================================= */
/* Session Input & Editing APIs                                              */
/* ========================================================================= */

/**
 * Moves cursor left by 1 character.
 * @return true on success, false if at beginning of buffer or session is NULL.
 */
bool vime_session_move_cursor_left(VimeSessionHandle *session, uint32_t by);

/**
 * Moves cursor right by 1 character.
 * @return true on success, false if at end of buffer or session is NULL.
 */
bool vime_session_move_cursor_right(VimeSessionHandle *session, uint32_t by);

/**
 * Reports whether the cursor can move one character left.
 * @return true if the cursor can move left, false if at the start of the buffer
 *         or session is NULL.
 */
bool vime_session_can_move_cursor_left(VimeSessionHandle *session);

/**
 * Reports whether the cursor can move one character right.
 * @return true if the cursor can move right, false if at the end of the buffer
 *         or session is NULL.
 */
bool vime_session_can_move_cursor_right(VimeSessionHandle *session);

typedef uint32_t VimeInsertKind;
#define VIME_INSERT_EXTENDED    ((VimeInsertKind)1u)
#define VIME_INSERT_TRANSFORMED ((VimeInsertKind)2u)
#define VIME_INSERT_INVALID     ((VimeInsertKind)3u)

typedef struct VimeInsertResult {
    VimeInsertKind kind;
    size_t first_changed; // The first position in unicode buffer that was transformed by the insert.
} VimeInsertResult;

/**
 * Inserts a Unicode scalar value at the current cursor position.
 *
 * @return The rendered-buffer change.
 */
VimeInsertResult vime_session_insert(VimeSessionHandle *session, uint32_t character);

/**
 * Performs a Backspace operation at the current cursor position.
 * @return true if character was deleted, false if buffer is empty or session is NULL.
 */
bool vime_session_backspace(VimeSessionHandle *session);

/**
 * Performs a Delete operation at the current cursor position.
 * @return true if character was deleted, false if buffer is empty or session is NULL.
 */
bool vime_session_delete(VimeSessionHandle *session);


/**
 * [VALIDATION] Checks if current buffer conforms to Vietnamese orthography rules.
 * @note Returns false if session is NULL, buffer is empty, or feature is unimplemented.
 */
bool vime_session_is_valid_vietnamese(VimeSessionHandle *session);

/* ========================================================================= */
/* Session Render & State APIs                                               */
/* ========================================================================= */

/**
 * Gets the caret position in the rendered buffer.
 * @return Offset in characters from the start of the rendered buffer, or 0 if
 *         session is NULL or the buffer is empty.
 */
size_t vime_session_get_rendered_cursor(VimeSessionHandle *session);

/**
 * Gets the caret position in the raw keystroke buffer.
 * @note Not interchangeable with vime_session_get_rendered_cursor(): a transform
 *       consumes a keystroke without lengthening the rendered word, so the two
 *       positions need not agree.
 * @return Offset in keystrokes from the start of the raw buffer, or 0 if
 *         session is NULL or the buffer is empty.
 */
size_t vime_session_get_raw_cursor(VimeSessionHandle *session);

/**
 * Gets the length of the composed character buffer.
 *
 * @param session Pointer to the active Vime session handle.
 * @return Number of composed Unicode characters (or UTF-8 bytes) in the buffer,
 *         or 0 if session is NULL or the buffer is empty.
 */
size_t vime_session_get_rendered_len(VimeSessionHandle *session);

size_t vime_session_get_rendered_len_utf8(VimeSessionHandle *session);

/**
 * Gets the length of the raw character buffer.
 *
 * @param session Pointer to the active Vime session handle.
 * @return Number of raw Unicode characters (or UTF-8 bytes) in the buffer,
 *         or 0 if session is NULL or the buffer is empty.
 */
size_t vime_session_get_raw_len(VimeSessionHandle *session);

size_t vime_session_get_raw_len_utf8(VimeSessionHandle *session);


// /**
//  * [RENDER TEXT] Gets the transformed Vietnamese UTF-8 display string.
//  * Returned pointer is managed by Session and remains valid until next session call.
//  * @note Returns empty string ("") if uninitialized or unimplemented.
//  */
// const char *vime_session_render_text(VimeSessionHandle *session);
//
// /**
//  * [RAW TEXT] Gets the raw UTF-8 sequence typed by user.
//  * Returned pointer is managed by Session and remains valid until next session call.
//  * @note Returns empty string ("") if uninitialized or unimplemented.
//  */
// const char *vime_session_render_raw_text(VimeSessionHandle *session);


/* ========================================================================= */
/* System Info                                                               */
/* ========================================================================= */

/**
 * Gets the semver string of the VIME Engine C-FFI library.
 */
const char *vime_version(void);

#ifdef __cplusplus
}
#endif

#endif /* VIME_ENGINE_H */
