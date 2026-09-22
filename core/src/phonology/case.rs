#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cased<T> {
    pub value: T,
    pub is_upper: bool,
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
}
