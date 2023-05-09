use fnv::FnvBuildHasher;
use num_iter::range_inclusive;
use num_traits::{One, ToPrimitive, Zero};
use std::collections::hash_set;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;

use crate::Position;

/// The default coordinate type of `Board`.
type DefaultCoordinateType = i16;

/// A representation of a two-dimensional orthogonal grid map of live/dead cells.
///
/// The type parameter `CoordinateType` is used as the type of the x- and y-coordinate values for each cell.
///
/// # Examples
///
/// ```
/// use life_backend::{Board, Position};
/// let pattern = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
/// let mut board: Board<i16> = pattern.iter().collect();
/// assert_eq!(board.get(&Position(0, 0)), true);
/// assert_eq!(board.get(&Position(0, 1)), false);
/// assert_eq!(board.iter().count(), 4);
/// board.clear();
/// board.set(&Position(1, 0), true);
/// board.set(&Position(0, 1), true);
/// assert_eq!(board.iter().count(), 2);
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board<CoordinateType = DefaultCoordinateType>(HashSet<Position<CoordinateType>, FnvBuildHasher>)
where
    CoordinateType: Eq + Hash;

// Inherent methods

impl<CoordinateType> Board<CoordinateType>
where
    CoordinateType: Eq + Hash,
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

    /// Returns the value of the specified position of the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let board = Board::<i16>::new();
    /// assert_eq!(board.get(&Position(0, 0)), false);
    /// ```
    ///
    #[inline]
    pub fn get(&self, position: &Position<CoordinateType>) -> bool {
        self.0.contains(position)
    }

    /// Set the specified value on the specified position of the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// board.set(&Position(0, 0), true);
    /// assert_eq!(board.get(&Position(0, 0)), true);
    /// ```
    ///
    pub fn set(&mut self, position: &Position<CoordinateType>, value: bool)
    where
        CoordinateType: Copy,
    {
        if value {
            self.0.insert(*position);
        } else {
            self.0.remove(position);
        }
    }

    /// Returns the minimum bounding box of all live cells on the board.
    /// If the board contains some live cells, `Some(x_min, x_max, y_min, y_max)` will be returned.
    /// Otherwise, `None` will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// assert_eq!(board.bounding_box(), None);
    /// board.set(&Position(-1, 2), true);
    /// board.set(&Position(3, -2), true);
    /// assert_eq!(board.bounding_box(), Some((-1, 3, -2, 2)));
    /// ```
    ///
    pub fn bounding_box(&self) -> Option<(CoordinateType, CoordinateType, CoordinateType, CoordinateType)>
    where
        CoordinateType: Copy + PartialOrd + Zero,
    {
        let mut iter = self.0.iter();
        if let Some(&Position(init_x, init_y)) = iter.next() {
            Some(iter.fold((init_x, init_x, init_y, init_y), |(x_min, x_max, y_min, y_max), &Position(x, y)| {
                (
                    if x_min < x { x_min } else { x },
                    if x_max > x { x_max } else { x },
                    if y_min < y { y_min } else { y },
                    if y_max > y { y_max } else { y },
                )
            }))
        } else {
            None
        }
    }

    /// Removes all live cells in the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// board.set(&Position(0, 0), true);
    /// board.clear();
    /// assert_eq!(board.get(&Position(0, 0)), false);
    /// ```
    ///
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Retains only the live cell positions specified by the predicate, similar as `retain()` of `HashSet`.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// board.set(&Position(0, 0), true);
    /// board.set(&Position(1, 0), true);
    /// board.set(&Position(0, 1), true);
    /// board.retain(|&pos| pos.0 == pos.1);
    /// assert_eq!(board.get(&Position(0, 0)), true);
    /// assert_eq!(board.get(&Position(1, 0)), false);
    /// assert_eq!(board.get(&Position(0, 1)), false);
    /// ```
    ///
    #[inline]
    pub fn retain<F>(&mut self, pred: F)
    where
        F: FnMut(&Position<CoordinateType>) -> bool,
    {
        self.0.retain(pred);
    }
}

impl<'a, CoordinateType> Board<CoordinateType>
where
    CoordinateType: Eq + Hash,
{
    /// Creates a non-owning iterator over the series of immutable live cell positions on the board in arbitrary order.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// use std::collections::HashSet;
    /// let mut board = Board::<i16>::new();
    /// board.set(&Position(1, 0), true);
    /// board.set(&Position(0, 1), true);
    /// let result: HashSet<_> = board.iter().collect();
    /// assert_eq!(result.len(), 2);
    /// assert!(result.contains(&Position(1, 0)));
    /// assert!(result.contains(&Position(0, 1)));
    /// ```
    ///
    #[inline]
    pub fn iter(&'a self) -> hash_set::Iter<'a, Position<CoordinateType>> {
        self.into_iter()
    }
}

// Trait implementations

impl<CoordinateType> Default for Board<CoordinateType>
where
    CoordinateType: Eq + Hash,
{
    /// Returns the default value of the type, same as the return value of `new()`.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<CoordinateType> fmt::Display for Board<CoordinateType>
where
    CoordinateType: Eq + Hash + Copy + PartialOrd + Zero + One + ToPrimitive,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some((x_min, x_max, y_min, y_max)) = self.bounding_box() {
            for y in range_inclusive(y_min, y_max) {
                let line: String = range_inclusive(x_min, x_max)
                    .map(|x| if self.get(&Position(x, y)) { 'O' } else { '.' })
                    .collect();
                writeln!(f, "{line}")?;
            }
        }
        Ok(())
    }
}

impl<'a, CoordinateType> IntoIterator for &'a Board<CoordinateType>
where
    CoordinateType: Eq + Hash,
{
    type Item = &'a Position<CoordinateType>;
    type IntoIter = hash_set::Iter<'a, Position<CoordinateType>>;

    /// Creates a non-owning iterator over the series of immutable live cell positions on the board in arbitrary order.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// use std::collections::HashSet;
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

impl<CoordinateType> IntoIterator for Board<CoordinateType>
where
    CoordinateType: Eq + Hash,
{
    type Item = Position<CoordinateType>;
    type IntoIter = hash_set::IntoIter<Self::Item>;

    /// Creates an owning iterator over the series of moved live cell positions on the board in arbitrary order.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// use std::collections::HashSet;
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let board: Board<i16> = pattern.iter().collect();
    /// let result: HashSet<_> = board.into_iter().collect();
    /// let expected: HashSet<_> = pattern.into_iter().collect();
    /// assert_eq!(result, expected);
    /// ```
    ///
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, CoordinateType> FromIterator<&'a Position<CoordinateType>> for Board<CoordinateType>
where
    CoordinateType: Eq + Hash + Copy + 'a,
{
    /// Conversion from a non-owning iterator over a series of `&Position<CoordinateType>`.
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let board: Board<i16> = pattern.iter().collect();
    /// assert_eq!(board.get(&Position(0, 0)), false);
    /// assert_eq!(board.get(&Position(1, 0)), true);
    /// assert_eq!(board.get(&Position(0, 1)), true);
    /// assert_eq!(board.get(&Position(1, 1)), false);
    /// ```
    ///
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a Position<CoordinateType>>,
    {
        let live_cells: HashSet<_, _> = iter.into_iter().copied().collect();
        Self(live_cells)
    }
}

impl<CoordinateType> FromIterator<Position<CoordinateType>> for Board<CoordinateType>
where
    CoordinateType: Eq + Hash,
{
    /// Conversion from an owning iterator over a series of `Position<CoordinateType>`.
    /// Each item in the series represents a moved live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut pattern = [Position(1, 0), Position(0, 1)];
    /// let board: Board<i16> = pattern.into_iter().collect();
    /// assert_eq!(board.get(&Position(0, 0)), false);
    /// assert_eq!(board.get(&Position(1, 0)), true);
    /// assert_eq!(board.get(&Position(0, 1)), true);
    /// assert_eq!(board.get(&Position(1, 1)), false);
    /// ```
    ///
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Position<CoordinateType>>,
    {
        let live_cells: HashSet<_, _> = iter.into_iter().collect();
        Self(live_cells)
    }
}

impl<'a, CoordinateType> Extend<&'a Position<CoordinateType>> for Board<CoordinateType>
where
    CoordinateType: Eq + Hash + Copy + 'a,
{
    /// Extend the board with the contents of the specified non-owning iterator over the series of `&Position<CoordinateType>`.
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// board.extend(pattern.iter());
    /// assert_eq!(board.get(&Position(0, 0)), false);
    /// assert_eq!(board.get(&Position(1, 0)), true);
    /// assert_eq!(board.get(&Position(0, 1)), true);
    /// assert_eq!(board.get(&Position(1, 1)), false);
    /// ```
    ///
    #[inline]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = &'a Position<CoordinateType>>,
    {
        self.0.extend(iter);
    }
}

impl<CoordinateType> Extend<Position<CoordinateType>> for Board<CoordinateType>
where
    CoordinateType: Eq + Hash,
{
    /// Extend the board with the contents of the specified owning iterator over the series of `Position<CoordinateType>`.
    /// Each item in the series represents a moved live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Position};
    /// let mut board = Board::<i16>::new();
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// board.extend(pattern.into_iter());
    /// assert_eq!(board.get(&Position(0, 0)), false);
    /// assert_eq!(board.get(&Position(1, 0)), true);
    /// assert_eq!(board.get(&Position(0, 1)), true);
    /// assert_eq!(board.get(&Position(1, 1)), false);
    /// ```
    ///
    #[inline]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Position<CoordinateType>>,
    {
        self.0.extend(iter);
    }
}
