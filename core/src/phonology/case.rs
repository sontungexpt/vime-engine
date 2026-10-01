/// Whether a value should be rendered with uppercase or lowercase letters.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Lower = 0,
    Upper = 1,
}

impl Case {
    #[inline(always)]
    pub const fn is_upper(self) -> bool {
        matches!(self, Self::Upper)
    }
}

impl From<Case> for bool {
    #[inline(always)]
    fn from(case: Case) -> Self {
        case.is_upper()
    }
}

impl From<bool> for Case {
    #[inline(always)]
    fn from(upper: bool) -> Self {
        // SAFETY: `bool` is 0 or 1, matching the `repr(u8)` discriminants.
        unsafe { std::mem::transmute::<bool, Self>(upper) }
    }
}

/// A value paired with the case it should render in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cased<T> {
    value: T,
    case: Case,
}

impl<T> Cased<T> {
    #[inline(always)]
    pub const fn new(value: T, case: Case) -> Self {
        Self { value, case }
    }

    /// Creates an uppercased value.
    #[inline(always)]
    pub const fn upper(value: T) -> Self {
        Self::new(value, Case::Upper)
    }

    /// Creates a lowercased value.
    #[inline(always)]
    pub const fn lower(value: T) -> Self {
        Self::new(value, Case::Lower)
    }

    #[inline(always)]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// `true` when the value carries the upper-case variant.
    #[inline(always)]
    pub const fn is_upper(&self) -> bool {
        self.case.is_upper()
    }

    #[inline(always)]
    pub const fn case(&self) -> Case {
        self.case
    }

    /// Replaces the value, keeping the case as-is.
    #[inline(always)]
    pub fn set_value(&mut self, value: T) {
        self.value = value;
    }

    /// Replaces the case, keeping the value as-is.
    #[inline(always)]
    pub fn set_case(&mut self, case: Case) {
        self.case = case;
    }

    /// Replaces the case from an uppercase flag, keeping the value as-is.
    #[inline(always)]
    pub fn set_upper(&mut self, is_upper: bool) {
        self.set_case(is_upper.into());
    }
}

impl<T: Copy> Cased<T> {
    #[inline(always)]
    pub const fn get(&self) -> T {
        self.value
    }
}
