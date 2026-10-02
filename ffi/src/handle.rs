//! Opaque FFI handles for the VIME engine.
//!
//! The handles deliberately contain very little logic. The core engine remains
//! the source of truth for configuration, composition, cursor state, and input
//! processing. This module only provides the ownership boundary required by C.

use vime_engine::{syllable::InsertOutcome, Session, SessionFactory};

use crate::config::{FfiKeymap, FfiSessionConfig};

/// The engine session owned by one FFI session handle.
type FfiSession = Session<FfiKeymap>;

/// Opaque handle to a VIME session factory.
///
/// The factory owns the shared configuration. Individual sessions created from
/// it own their relationship to that shared configuration through the core
/// engine.
#[repr(C)]
pub struct VimeSessionFactoryHandle {
    factory: SessionFactory<FfiKeymap>,
}

impl VimeSessionFactoryHandle {
    /// Creates a factory with the supplied configuration.
    #[inline]
    pub(crate) fn new(config: FfiSessionConfig) -> Self {
        Self {
            factory: SessionFactory::new(config),
        }
    }

    /// Transfers ownership of the handle to a raw pointer for C.
    #[inline]
    pub(crate) fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    /// Borrows a factory handle mutably.
    ///
    /// Returns `None` for NULL.
    ///
    /// # Safety
    ///
    /// `ptr` must be NULL or a valid live factory handle.
    #[inline]
    pub(crate) unsafe fn from_raw<'a>(ptr: *mut Self) -> Option<&'a mut Self> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: Guaranteed by the caller according to this function's contract.
        Some(unsafe { &mut *ptr })
    }

    /// Borrows a factory handle immutably.
    ///
    /// Returns `None` for NULL.
    ///
    /// # Safety
    ///
    /// `ptr` must be NULL or a valid live factory handle.
    #[inline]
    pub(crate) unsafe fn from_raw_const<'a>(ptr: *const Self) -> Option<&'a Self> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: Guaranteed by the caller according to this function's contract.
        Some(unsafe { &*ptr })
    }

    /// Takes ownership of a factory handle back from C.
    ///
    /// # Safety
    ///
    /// `ptr` must be NULL or a pointer previously returned by [`Self::into_raw`]
    /// that has not already been destroyed.
    #[inline]
    pub(crate) unsafe fn into_box(ptr: *mut Self) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: Guaranteed by the caller according to this function's contract.
        Some(unsafe { Box::from_raw(ptr) })
    }

    /// Creates a session following the factory's shared configuration.
    #[inline]
    pub(crate) fn new_session(&self) -> FfiSession {
        self.factory.new_session()
    }

    /// Creates a session with a private configuration.
    #[inline]
    pub(crate) fn new_session_with(&self, config: FfiSessionConfig) -> FfiSession {
        self.factory.new_session_with(config)
    }

    /// Returns the factory's current shared configuration.
    #[inline]
    pub(crate) fn config(&self) -> FfiSessionConfig {
        self.factory.config()
    }

    /// Replaces the factory's shared configuration.
    #[inline]
    pub(crate) fn set_config(&self, config: FfiSessionConfig) {
        self.factory.set_config(config);
    }
}

#[repr(C)]
pub struct VimeInsertResult {
    pub kind: u32,
    pub first_changed: usize,
}
impl VimeInsertResult {
    pub fn invalid() -> Self {
        Self {
            kind: VIME_INSERT_INVALID,
            first_changed: 0,
        }
    }
}

const VIME_INSERT_EXTENDED: u32 = 1;
const VIME_INSERT_TRANSFORMED: u32 = 2;
const VIME_INSERT_INVALID: u32 = 3;

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
/// The session owns its core [`Session`] directly. Configuration sharing and
/// private configuration are handled by the core engine; the FFI handle does
/// not duplicate that state.
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
    #[inline]
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
    #[inline]
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
    #[inline]
    pub(crate) unsafe fn into_box(ptr: *mut Self) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: Guaranteed by the caller according to this function's contract.
        Some(unsafe { Box::from_raw(ptr) })
    }

    /// Resets the underlying engine session.
    #[inline]
    pub(crate) fn reset(&mut self) {
        self.session.reset();
    }

    /// Assigns a private configuration to the session.
    ///
    /// The session stops following the factory's shared configuration.
    #[inline]
    pub(crate) fn set_config(&mut self, config: FfiSessionConfig) {
        self.session.set_private_config(config);
    }

    /// Removes the session's private configuration.
    ///
    /// The session resumes following the factory's shared configuration.
    #[inline]
    pub(crate) fn clear_config(&mut self) {
        self.session.clear_private_config();
    }

    /// Moves the cursor left by up to `by` character positions.
    #[inline]
    pub(crate) fn move_cursor_left_by(&mut self, by: usize) -> bool {
        *self.session.move_cursor_left_by(by).rendered()
    }

    /// Moves the cursor right by up to `by` character positions.
    #[inline]
    pub(crate) fn move_cursor_right_by(&mut self, by: usize) -> bool {
        *self.session.move_cursor_right_by(by).rendered()
    }

    /// Inserts one Unicode scalar value at the current cursor.
    #[inline]
    pub(crate) fn insert(&mut self, character: u32) -> VimeInsertResult {
        let Some(character) = char::from_u32(character) else {
            return VimeInsertResult::invalid();
        };

        VimeInsertResult::from(*self.session.insert(character).rendered())
    }

    /// Performs Backspace at the current cursor.
    #[inline]
    pub(crate) fn backspace(&mut self) -> bool {
        *self.session.backspace().rendered()
    }

    /// Performs Delete at the current cursor.
    #[inline]
    pub(crate) fn delete(&mut self) -> bool {
        *self.session.delete().rendered()
    }
}
