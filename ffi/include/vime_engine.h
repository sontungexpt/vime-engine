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

typedef enum VimeInputMethod {
    VIME_INPUT_METHOD_TELEX = 1u,
    VIME_INPUT_METHOD_VNI   = 2u,
    VIME_INPUT_METHOD_VIQR  = 3u,
} VimeInputMethod;

typedef enum VimeTonePlacement {
    VIME_TONE_PLACEMENT_MODERN = 1u,
    VIME_TONE_PLACEMENT_OLD    = 2u,
} VimeTonePlacement;

typedef struct VimeConfig {
    VimeInputMethod input_method;
    VimeTonePlacement tone_placement;
} VimeConfig;

/**
 * Helper macro for default C-style configuration initialization.
 */
#define VIME_CONFIG_INIT \
    ((VimeConfig){ \
        .input_method = VIME_INPUT_METHOD_TELEX, \
        .tone_placement = VIME_TONE_PLACEMENT_MODERN \
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
 * Inserts a single Unicode scalar value (UTF-32) at the current cursor position.
 * @return true if input was processed, false if session is NULL or invalid.
 */
bool vime_session_insert(VimeSessionHandle *session, uint32_t character);

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
 * Moves cursor left by 1 character.
 * @return true on success, false if at beginning of buffer or session is NULL.
 */
bool vime_session_move_cursor_left(VimeSessionHandle *session);

/**
 * Moves cursor right by 1 character.
 * @return true on success, false if at end of buffer or session is NULL.
 */
bool vime_session_move_cursor_right(VimeSessionHandle *session);

/* ========================================================================= */
/* Session Render & State APIs                                               */
/* ========================================================================= */

/**
 * Detailed render state snapshot for UI, uinput, and IME frameworks.
 */
typedef struct VimeRenderState {
    const char *text;             /* Transformed Vietnamese UTF-8 text (e.g., "viê") */
    const char *raw_text;         /* Raw UTF-8 key sequence entered by user (e.g., "viee") */

    /* Cursor indicators for Rendered (Display) string */
    size_t cursor_byte_idx;       /* Rendered cursor index in Bytes */
    size_t cursor_char_idx;       /* Rendered cursor index in CodePoints (Chars) */

    /* Cursor indicators for Raw string */
    size_t raw_cursor_byte_idx;   /* Raw cursor index in Bytes */
    size_t raw_cursor_char_idx;   /* Raw cursor index in CodePoints (Chars) */

    /* uinput & IME Integration indicators */
    size_t bytes_to_delete;       /* Number of UTF-8 bytes to remove from previous render */
    size_t chars_to_delete;       /* Number of CodePoints (Backspaces) to delete from host buffer */

    bool is_valid_vietnamese;     /* True if current buffer forms a valid Vietnamese word */
} VimeRenderState;

/**
 * [RENDER TEXT] Gets the transformed Vietnamese UTF-8 display string.
 * Returned pointer is managed by Session and remains valid until next session call.
 * @note Returns empty string ("") if uninitialized or unimplemented.
 */
const char *vime_session_render_text(VimeSessionHandle *session);

/**
 * [RAW TEXT] Gets the raw UTF-8 sequence typed by user.
 * Returned pointer is managed by Session and remains valid until next session call.
 * @note Returns empty string ("") if uninitialized or unimplemented.
 */
const char *vime_session_render_raw_text(VimeSessionHandle *session);

/**
 * [VALIDATION] Checks if current buffer conforms to Vietnamese orthography rules.
 * @note Returns false if session is NULL, buffer is empty, or feature is unimplemented.
 */
bool vime_session_is_valid_vietnamese(VimeSessionHandle *session);

/**
 * [RENDER CURSOR] Gets rendered cursor position in CodePoints (Unicode characters).
 * @note Returns 0 if session is NULL or unimplemented in core engine.
 */
size_t vime_session_get_cursor_char_idx(VimeSessionHandle *session);

/**
 * [RENDER CURSOR] Gets rendered cursor position in UTF-8 Byte offset.
 * @note Returns 0 if session is NULL or unimplemented in core engine.
 */
size_t vime_session_get_cursor_byte_idx(VimeSessionHandle *session);

/**
 * [RAW CURSOR] Gets raw cursor position in CodePoints (Unicode characters).
 * @note Returns 0 if session is NULL or unimplemented in core engine.
 */
size_t vime_session_get_raw_cursor_char_idx(VimeSessionHandle *session);

/**
 * [RAW CURSOR] Gets raw cursor position in UTF-8 Byte offset.
 * @note Returns 0 if session is NULL or unimplemented in core engine.
 */
size_t vime_session_get_raw_cursor_byte_idx(VimeSessionHandle *session);

/**
 * [ADVANCED STATE] Retrieves a complete snapshot of current session render state.
 * Returned pointer is managed by Session and remains valid until next session call.
 * @note Fields for unimplemented core features will fallback to zeroed/empty defaults.
 */
const VimeRenderState *vime_session_render_state(VimeSessionHandle *session);

/**
 * Clears input buffers and resets session state to initial conditions.
 * Safe to call with NULL pointer (no-op).
 */
void vime_session_reset(VimeSessionHandle *session);

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
