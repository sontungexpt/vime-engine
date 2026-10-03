//! Opaque handle to one typing session: boundary around core [`Session`].

use crate::config::{FfiKeymap, FfiSessionConfig};
use vime_engine::{syllable::InsertOutcome, Session};

pub(crate) type FfiSession = Session<FfiKeymap>;

#[repr(C)]
pub struct VimeInsertResult {
    pub kind: u32,
    pub first_changed: usize,
}

const VIME_INSERT_EXTENDED: u32 = 1;
const VIME_INSERT_TRANSFORMED: u32 = 2;
const VIME_INSERT_INVALID: u32 = 3;

impl VimeInsertResult {
    pub fn invalid() -> Self {
        Self {
            kind: VIME_INSERT_INVALID,
            first_changed: 0,
        }
    }
}

impl From<InsertOutcome> for VimeInsertResult {
    #[inline]
    fn from(outcome: InsertOutcome) -> Self {
        match outcome {
            InsertOutcome::Extended => Self {
                kind: VIME_INSERT_EXTENDED,
                first_changed: 0,
            },
            InsertOutcome::Transformed { first_changed } => Self {
                kind: VIME_INSERT_TRANSFORMED,
                first_changed,
            },
        }
    }
}

/// Opaque handle to one VIME typing session. Owns core `Session`. Config sharing handled by core.
#[repr(C)]
pub struct VimeSessionHandle {
    session: FfiSession,
}

impl VimeSessionHandle {
    /// Wraps an engine session in an FFI handle.
    #[inline]
    pub(crate) fn new(session: FfiSession) -> Self {
        Self { session }
    }
    /// Transfers ownership to a raw pointer for C.
    #[inline]
    pub(crate) fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    /// Borrows handle mutably. None for NULL.
    /// # Safety: `ptr` must be NULL or valid pointer from `into_raw` not destroyed.
    #[inline(always)]
    pub(crate) unsafe fn from_raw<'a>(ptr: *mut Self) -> Option<&'a mut Self> {
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &mut *ptr })
    }
    /// Borrows handle immutably. None for NULL.
    /// # Safety: `ptr` must be NULL or valid live handle.
    #[inline(always)]
    pub(crate) unsafe fn from_raw_const<'a>(ptr: *const Self) -> Option<&'a Self> {
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &*ptr })
    }
    /// Takes ownership back from C.
    /// # Safety: `ptr` must be NULL or from `into_raw` not destroyed.
    #[inline(always)]
    pub(crate) unsafe fn into_box(ptr: *mut Self) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { Box::from_raw(ptr) })
    }

    // Session management
    /// Resets the underlying engine session.
    #[inline(always)]
    pub(crate) fn reset(&mut self) {
        self.session.reset();
    }
    /// Assigns a private configuration to the session (stops following shared config).
    #[inline(always)]
    pub(crate) fn set_private_config(&mut self, config: FfiSessionConfig) {
        self.session.set_private_config(config);
    }
    /// Removes private config; session resumes following factory's shared config.
    #[inline(always)]
    pub(crate) fn clear_private_config(&mut self) {
        self.session.clear_private_config();
    }

    // Cursor movement
    /// Moves cursor left by up to `by` character positions. Returns true if moved.
    #[inline(always)]
    pub(crate) fn move_cursor_left_by(&mut self, by: usize) -> bool {
        *self.session.move_cursor_left_by(by).rendered()
    }
    /// Moves cursor right by up to `by` character positions. Returns true if moved.
    #[inline(always)]
    pub(crate) fn move_cursor_right_by(&mut self, by: usize) -> bool {
        *self.session.move_cursor_right_by(by).rendered()
    }

    // Cursor queries
    /// Returns true if cursor can move one position left.
    #[inline(always)]
    pub(crate) fn can_move_cursor_left(&self) -> bool {
        *self.session.can_move_cursor_left().rendered()
    }
    /// Returns true if cursor can move one position right.
    #[inline(always)]
    pub(crate) fn can_move_cursor_right(&self) -> bool {
        *self.session.can_move_cursor_right().rendered()
    }

    // Editing operations
    /// Inserts one Unicode scalar (UTF-32) at the current cursor.
    #[inline(always)]
    pub(crate) fn insert(&mut self, character: u32) -> VimeInsertResult {
        let Some(c) = char::from_u32(character) else {
            return VimeInsertResult::invalid();
        };
        VimeInsertResult::from(*self.session.insert(c).rendered())
    }
    /// Performs Backspace at the current cursor.
    #[inline(always)]
    pub(crate) fn backspace(&mut self) -> bool {
        *self.session.backspace().rendered()
    }
    /// Performs Delete at the current cursor.
    #[inline(always)]
    pub(crate) fn delete(&mut self) -> bool {
        *self.session.delete().rendered()
    }

    // Validation
    /// Returns true if buffer spells a valid Vietnamese syllable.
    #[inline(always)]
    pub(crate) fn is_valid_vietnamese(&mut self) -> bool {
        self.session.is_phonotactically_valid()
    }

    // Getters / queries
    /// Caret position in rendered buffer (chars from start).
    #[inline(always)]
    pub(crate) fn rendered_cursor(&self) -> usize {
        self.session.rendered_cursor()
    }
    /// Caret position in raw keystroke buffer (keystrokes from start).
    #[inline(always)]
    pub(crate) fn raw_cursor(&self) -> usize {
        self.session.raw_cursor()
    }
    /// Length of rendered buffer in characters.
    #[inline(always)]
    pub(crate) fn rendered_len(&self) -> usize {
        self.session.rendered_len()
    }
    /// Length of rendered buffer in UTF-8 bytes.
    #[inline(always)]
    pub(crate) fn rendered_len_utf8(&self) -> usize {
        self.session.rendered_len_utf8()
    }
    /// Length of raw keystroke buffer in keystrokes.
    #[inline(always)]
    pub(crate) fn raw_len(&self) -> usize {
        self.session.raw_len()
    }
    /// Length of raw keystroke buffer in UTF-8 bytes.
    #[inline(always)]
    pub(crate) fn raw_len_utf8(&self) -> usize {
        self.session.raw_len_utf8()
    }
}
