use fnv::FnvBuildHasher;
use num_iter::range_inclusive;
use num_traits::{One, ToPrimitive, Zero};
use std::collections::hash_set;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::iter::FromIterator;

use crate::{BoardRange, Position};

/// A two-dimensional orthogonal grid map of live/dead cells.
///
/// The type parameter `T` is used as the type of the x- and y-coordinate values for each cell.
///
/// # Examples
///
/// ```
/// use life_backend::{Board, Position};
/// let pattern = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
/// let board: Board<i16> = pattern.iter().collect();
/// assert_eq!(board.contains(&Position(0, 0)), true);
/// assert_eq!(board.contains(&Position(0, 1)), false);
/// assert_eq!(board.iter().count(), 4);
/// ```
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Board<T>(HashSet<Position<T>, FnvBuildHasher>)
where
    T: Eq + Hash;

// Inherent methods

impl<T> Board<T>
where
    T: Eq + Hash,
{
    /// Creates an empty board.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::Board;
    /// let board = Board::<i16>::new();
    /// assert_eq!(board.iter().count(), 0);
    /// ```
    ///
    #[inline]
    pub fn new() -> Self {
        Self(HashSet::default())
    }

    /// Returns `true` if the board contains the specified position.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let board = Board::<i16>::new();
    /// assert_eq!(board.contains(&Position(0, 0)), false);
    /// ```
    ///
    #[inline]
    pub fn contains(&self, position: &Position<T>) -> bool {
        self.0.contains(position)
    }

    /// Adds the specified position to the board.
    ///
    /// Returns whether the position was newly inserted, like as [`insert()`] of [`HashSet`].
    ///
    /// [`insert()`]: std::collections::HashSet::insert
    /// [`HashSet`]: std::collections::HashSet
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// assert_eq!(board.insert(Position(0, 0)), true);
    /// assert_eq!(board.contains(&Position(0, 0)), true);
    /// ```
    ///
    #[inline]
    pub fn insert(&mut self, position: Position<T>) -> bool {
        self.0.insert(position)
    }

    /// Removes the specified position from the board.
    ///
    /// Returns whether the position was contained in the board, like as [`remove()`] of [`HashSet`].
    ///
    /// [`remove()`]: std::collections::HashSet::remove
    /// [`HashSet`]: std::collections::HashSet
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// assert_eq!(board.insert(Position(0, 0)), true);
    /// assert_eq!(board.remove(&Position(0, 0)), true);
    /// assert_eq!(board.contains(&Position(0, 0)), false);
    /// ```
    ///
    #[inline]
    pub fn remove(&mut self, position: &Position<T>) -> bool {
        self.0.remove(position)
    }

    /// Returns the minimum bounding box of all live cells on the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::new();
    /// board.insert(Position(-1, 2));
    /// board.insert(Position(3, -2));
    /// let bbox = board.bounding_box();
    /// assert_eq!(bbox.x(), &(-1..=3));
    /// assert_eq!(bbox.y(), &(-2..=2));
    /// ```
    ///
    #[inline]
    pub fn bounding_box(&self) -> BoardRange<T>
    where
        T: Copy + PartialOrd + Zero + One,
    {
        self.0.iter().collect::<BoardRange<_>>()
    }

    /// Removes all live cells in the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// board.insert(Position(0, 0));
    /// board.clear();
    /// assert_eq!(board.contains(&Position(0, 0)), false);
    /// ```
    ///
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Retains only the live cell positions specified by the predicate, similar as [`retain()`] of [`HashSet`].
    ///
    /// [`retain()`]: std::collections::HashSet::retain
    /// [`HashSet`]: std::collections::HashSet
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// board.insert(Position(0, 0));
    /// board.insert(Position(1, 0));
    /// board.insert(Position(0, 1));
    /// board.retain(|&pos| pos.0 == pos.1);
    /// assert_eq!(board.contains(&Position(0, 0)), true);
    /// assert_eq!(board.contains(&Position(1, 0)), false);
    /// assert_eq!(board.contains(&Position(0, 1)), false);
    /// ```
    ///
    #[inline]
    pub fn retain<F>(&mut self, pred: F)
    where
        F: FnMut(&Position<T>) -> bool,
    {
        self.0.retain(pred);
    }
}

impl<'a, T> Board<T>
where
    T: Eq + Hash,
{
    /// Creates a non-owning iterator over the series of immutable live cell positions on the board in arbitrary order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// board.insert(Position(1, 0));
    /// board.insert(Position(0, 1));
    /// let result: HashSet<_> = board.iter().collect();
    /// assert_eq!(result.len(), 2);
    /// assert!(result.contains(&Position(1, 0)));
    /// assert!(result.contains(&Position(0, 1)));
    /// ```
    ///
    #[inline]
    pub fn iter(&'a self) -> hash_set::Iter<'a, Position<T>> {
        self.into_iter()
    }
}

// Trait implementations

impl<T> Default for Board<T>
where
    T: Eq + Hash,
{
    /// Returns the default value of the type, same as the return value of [`new()`].
    ///
    /// [`new()`]: #method.new
    ///
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> fmt::Display for Board<T>
where
    T: Eq + Hash + Copy + PartialOrd + Zero + One + ToPrimitive,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bbox = self.bounding_box();
        for y in range_inclusive(*bbox.y().start(), *bbox.y().end()) {
            let line: String = range_inclusive(*bbox.x().start(), *bbox.x().end())
                .map(|x| if self.contains(&Position(x, y)) { 'O' } else { '.' })
                .collect();
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

impl<'a, T> IntoIterator for &'a Board<T>
where
    T: Eq + Hash,
{
    type Item = &'a Position<T>;
    type IntoIter = hash_set::Iter<'a, Position<T>>;

    /// Creates a non-owning iterator over the series of immutable live cell positions on the board in arbitrary order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use life_backend::{Board, Position};
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let board: Board<i16> = pattern.iter().collect();
    /// let result: HashSet<_> = (&board).into_iter().collect();
    /// let expected: HashSet<_> = pattern.iter().collect();
    /// assert_eq!(result, expected);
    /// ```
    ///
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T> IntoIterator for Board<T>
where
    T: Eq + Hash,
{
    type Item = Position<T>;
    type IntoIter = hash_set::IntoIter<Self::Item>;

    /// Creates an owning iterator over the series of moved live cell positions on the board in arbitrary order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use life_backend::{Board, Position};
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let board: Board<i16> = pattern.iter().collect();
    /// let result: HashSet<_> = board.into_iter().collect();
    /// let expected: HashSet<_> = pattern.iter().copied().collect();
    /// assert_eq!(result, expected);
    /// ```
    ///
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> FromIterator<&'a Position<T>> for Board<T>
where
    T: Eq + Hash + Copy + 'a,
{
    /// Creates a value from a non-owning iterator over a series of [`&Position<T>`].
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// [`&Position<T>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let board: Board<i16> = pattern.iter().collect();
    /// assert_eq!(board.contains(&Position(0, 0)), false);
    /// assert_eq!(board.contains(&Position(1, 0)), true);
    /// assert_eq!(board.contains(&Position(0, 1)), true);
    /// assert_eq!(board.contains(&Position(1, 1)), false);
    /// ```
    ///
    #[inline]
    fn from_iter<U>(iter: U) -> Self
    where
        U: IntoIterator<Item = &'a Position<T>>,
    {
        Self::from_iter(iter.into_iter().copied())
    }
}

impl<T> FromIterator<Position<T>> for Board<T>
where
    T: Eq + Hash,
{
    /// Creates a value from an owning iterator over a series of [`Position<T>`].
    /// Each item in the series represents a moved live cell position.
    ///
    /// [`Position<T>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut pattern = [Position(1, 0), Position(0, 1)];
    /// let board: Board<i16> = pattern.into_iter().collect();
    /// assert_eq!(board.contains(&Position(0, 0)), false);
    /// assert_eq!(board.contains(&Position(1, 0)), true);
    /// assert_eq!(board.contains(&Position(0, 1)), true);
    /// assert_eq!(board.contains(&Position(1, 1)), false);
    /// ```
    ///
    #[inline]
    fn from_iter<U>(iter: U) -> Self
    where
        U: IntoIterator<Item = Position<T>>,
    {
        Self(HashSet::<Position<T>, _>::from_iter(iter))
    }
}

impl<'a, T> Extend<&'a Position<T>> for Board<T>
where
    T: Eq + Hash + Copy + 'a,
{
    /// Extends the board with the contents of the specified non-owning iterator over the series of [`&Position<T>`].
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// [`&Position<T>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// board.extend(pattern.iter());
    /// assert_eq!(board.contains(&Position(0, 0)), false);
    /// assert_eq!(board.contains(&Position(1, 0)), true);
    /// assert_eq!(board.contains(&Position(0, 1)), true);
    /// assert_eq!(board.contains(&Position(1, 1)), false);
    /// ```
    ///
    #[inline]
    fn extend<U>(&mut self, iter: U)
    where
        U: IntoIterator<Item = &'a Position<T>>,
    {
        self.0.extend(iter);
    }
}

impl<T> Extend<Position<T>> for Board<T>
where
    T: Eq + Hash,
{
    /// Extends the board with the contents of the specified owning iterator over the series of [`Position<T>`].
    /// Each item in the series represents a moved live cell position.
    ///
    /// [`Position<T>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// board.extend(pattern.into_iter());
    /// assert_eq!(board.contains(&Position(0, 0)), false);
    /// assert_eq!(board.contains(&Position(1, 0)), true);
    /// assert_eq!(board.contains(&Position(0, 1)), true);
    /// assert_eq!(board.contains(&Position(1, 1)), false);
    /// ```
    ///
    #[inline]
    fn extend<U>(&mut self, iter: U)
    where
        U: IntoIterator<Item = Position<T>>,
    {
        self.0.extend(iter);
    }
}

// Unit tests

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default() {
        let target = Board::<i16>::default();
        let expected = Board::<i16>::new();
        assert_eq!(target, expected);
    }
}
