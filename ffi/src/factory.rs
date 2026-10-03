//! Opaque handle to a session factory: owner of shared configuration.

use crate::config::{FfiKeymap, FfiSessionConfig};
use crate::session::FfiSession;
use vime_engine::SessionFactory;

/// Opaque handle to a VIME session factory. Owns shared config; sessions manage their relationship via core.
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
    /// Transfers ownership to a raw pointer for C.
    #[inline]
    pub(crate) fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    /// Borrows mutably. None for NULL. # Safety: `ptr` NULL or valid live handle.
    #[inline]
    pub(crate) unsafe fn from_raw<'a>(ptr: *mut Self) -> Option<&'a mut Self> {
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &mut *ptr })
    }
    /// Borrows immutably. None for NULL. # Safety: `ptr` NULL or valid live handle.
    #[inline]
    pub(crate) unsafe fn from_raw_const<'a>(ptr: *const Self) -> Option<&'a Self> {
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &*ptr })
    }
    /// Takes ownership back from C. # Safety: `ptr` NULL or from `into_raw` not destroyed.
    #[inline]
    pub(crate) unsafe fn into_box(ptr: *mut Self) -> Option<Box<Self>> {
        if ptr.is_null() {
            return None;
        }
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
