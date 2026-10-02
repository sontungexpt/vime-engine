//! Opaque handle to one typing session: the boundary around a core
//! [`Session`](vime_engine::Session).

use vime_engine::{syllable::InsertOutcome, Session};

use crate::config::{FfiKeymap, FfiSessionConfig};

/// The engine session owned by one FFI session handle.
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

/// Opaque handle to one VIME typing session.
///
/// The session owns its core [`Session`](vime_engine::Session) directly.
/// Configuration sharing and private configuration are handled by the core
/// engine; the FFI handle does not duplicate that state.
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

    /// Transfers ownership of the handle to a raw pointer for C.
    #[inline]
    pub(crate) fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    /// Borrows a session handle mutably.
    ///
    /// Returns `None` for NULL.
    ///
    /// # Safety
    ///
    /// `ptr` must be NULL or a valid pointer previously returned by
    /// [`Self::into_raw`] that has not been destroyed. No other mutable or
    /// immutable borrow of the handle may be active for the duration of the
    /// returned borrow.
    #[inline(always)]
    pub(crate) unsafe fn from_raw<'a>(ptr: *mut Self) -> Option<&'a mut Self> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: Guaranteed by the caller according to this function's contract.
        Some(unsafe { &mut *ptr })
    }

    /// Borrows a session handle immutably.
    ///
    /// Returns `None` for NULL.
    ///
    /// # Safety
    ///
    /// `ptr` must be NULL or a valid live session handle.
    #[inline(always)]
    pub(crate) unsafe fn from_raw_const<'a>(ptr: *const Self) -> Option<&'a Self> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: Guaranteed by the caller according to this function's contract.
        Some(unsafe { &*ptr })
    }

    /// Takes ownership of a session handle back from C.
    ///
    /// # Safety
    ///
    /// `ptr` must be NULL or a pointer previously returned by [`Self::into_raw`]
    /// that has not already been destroyed.
    #[inline(always)]
    pub(crate) unsafe fn into_box(ptr: *mut Self) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: Guaranteed by the caller according to this function's contract.
        Some(unsafe { Box::from_raw(ptr) })
    }

    /// Resets the underlying engine session.
    #[inline(always)]
    pub(crate) fn reset(&mut self) {
        self.session.reset();
    }

    /// Assigns a private configuration to the session.
    ///
    /// The session stops following the factory's shared configuration.
    #[inline(always)]
    pub(crate) fn set_private_config(&mut self, config: FfiSessionConfig) {
        self.session.set_private_config(config);
    }

    /// Removes the session's private configuration.
    ///
    /// The session resumes following the factory's shared configuration.
    #[inline(always)]
    pub(crate) fn clear_private_config(&mut self) {
        self.session.clear_private_config();
    }

    /// Moves the cursor left by up to `by` character positions.
    #[inline(always)]
    pub(crate) fn move_cursor_left_by(&mut self, by: usize) -> bool {
        *self.session.move_cursor_left_by(by).rendered()
    }

    /// Moves the cursor right by up to `by` character positions.
    #[inline(always)]
    pub(crate) fn move_cursor_right_by(&mut self, by: usize) -> bool {
        *self.session.move_cursor_right_by(by).rendered()
    }

    /// Whether the cursor can move one position left.
    #[inline(always)]
    pub(crate) fn can_move_cursor_left(&self) -> bool {
        *self.session.can_move_cursor_left().rendered()
    }

    /// Whether the cursor can move one position right.
    #[inline(always)]
    pub(crate) fn can_move_cursor_right(&self) -> bool {
        *self.session.can_move_cursor_right().rendered()
    }

    /// Inserts one Unicode scalar value at the current cursor.
    #[inline(always)]
    pub(crate) fn insert(&mut self, character: u32) -> VimeInsertResult {
        let Some(character) = char::from_u32(character) else {
            return VimeInsertResult::invalid();
        };

        VimeInsertResult::from(*self.session.insert(character).rendered())
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

    /// Whether the buffer spells a complete, valid Vietnamese syllable.
    #[inline(always)]
    pub(crate) fn is_valid_vietnamese(&mut self) -> bool {
        self.session.is_phonotactically_valid()
    }

    /// Caret position in the rendered buffer, in characters from the start.
    #[inline(always)]
    pub(crate) fn rendered_cursor(&self) -> usize {
        self.session.rendered_cursor()
    }

    /// Caret position in the raw keystroke buffer, in keystrokes from the start.
    #[inline(always)]
    pub(crate) fn raw_cursor(&self) -> usize {
        self.session.raw_cursor()
    }

    /// Length of the rendered buffer, in characters.
    #[inline(always)]
    pub(crate) fn rendered_len(&self) -> usize {
        self.session.rendered_len()
    }

    /// Length of the rendered buffer, in UTF-8 bytes.
    #[inline(always)]
    pub(crate) fn rendered_len_utf8(&self) -> usize {
        self.session.rendered_len_utf8()
    }

    /// Length of the raw keystroke buffer, in keystrokes.
    #[inline(always)]
    pub(crate) fn raw_len(&self) -> usize {
        self.session.raw_len()
    }

    /// Length of the raw keystroke buffer, in UTF-8 bytes.
    #[inline(always)]
    pub(crate) fn raw_len_utf8(&self) -> usize {
        self.session.raw_len_utf8()
    }
}
