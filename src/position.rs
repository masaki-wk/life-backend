use std::hash::Hash;

/// A position of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position<T>(pub T, pub T);
