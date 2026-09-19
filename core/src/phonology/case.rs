#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cased<T> {
    pub value: T,
    pub uppercase: bool,
}

impl<T> Cased<T> {
    #[inline(always)]
    pub const fn new(value: T, uppercase: bool) -> Self {
        Self { value, uppercase }
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

    /// Whether the value is uppercased.
    #[inline(always)]
    pub const fn is_upper(&self) -> bool {
        self.uppercase
    }

    /// Whether the value is lowercased.
    #[inline(always)]
    pub const fn is_lower(&self) -> bool {
        !self.uppercase
    }
}
