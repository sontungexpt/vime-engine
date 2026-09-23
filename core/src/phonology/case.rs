#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cased<T> {
    value: T,
    is_upper: bool,
}

impl<T> Cased<T> {
    #[inline(always)]
    pub const fn new(value: T, is_upper: bool) -> Self {
        Self { value, is_upper }
    }

    /// Creates an uppercased value.
    #[inline(always)]
    pub const fn upper(value: T) -> Self {
        Self::new(value, true)
    }

    /// Creates a lowercased value.
    #[inline(always)]
    pub const fn lower(value: T) -> Self {
        Self::new(value, false)
    }

    /// The cased value.
    #[inline(always)]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// `true` when the value carries the upper-case variant.
    #[inline(always)]
    pub const fn is_upper(&self) -> bool {
        self.is_upper
    }

    /// Replaces the value, keeping the case as-is.
    #[inline(always)]
    pub fn set_value(&mut self, value: T) {
        self.value = value;
    }

    /// Replaces the case flag, keeping the value as-is.
    #[inline(always)]
    pub fn set_upper(&mut self, is_upper: bool) {
        self.is_upper = is_upper;
    }
}
