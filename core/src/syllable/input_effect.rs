#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEffect {
    /// Mutated an existing character into a marked form (e.g. `a` + `w` -> `ă`).
    Transformed,
    /// Changed the buffer structure (inserted a new character or removed one).
    StructurallyChanged,
}
