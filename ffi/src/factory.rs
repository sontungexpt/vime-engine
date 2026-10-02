//! Opaque handle to a session factory: the owner of the shared configuration.

use vime_engine::SessionFactory;

use crate::config::{FfiKeymap, FfiSessionConfig};
use crate::session::FfiSession;

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
