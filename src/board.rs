use num_iter::range_inclusive;
use num_traits::{One, ToPrimitive, Zero};
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;

/// The default index type of boards.
pub type DefaultIndexType = i16;

/// A representation of boards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board<IndexType = DefaultIndexType>
where
    IndexType: Eq + Hash,
{
    live_cells: HashSet<(IndexType, IndexType)>,
}

// Inherent methods

impl<IndexType> Board<IndexType>
where
    IndexType: Eq + Hash,
{
    /// Creates an empty board.
    #[inline]
    pub fn new() -> Self {
        let live_cells = HashSet::new();
        Self { live_cells }
    }

    /// Returns the value of the specified position of the board.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let board = Board::new();
    /// assert_eq!(board.get(0, 0), false);
    /// ```
    ///
    #[inline]
    pub fn get(&self, x: IndexType, y: IndexType) -> bool {
        let pos = (x, y);
        self.live_cells.contains(&pos)
    }

    /// Set the specified value on the specified position of the board.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let mut board = Board::new();
    /// board.set(0, 0, true);
    /// assert_eq!(board.get(0, 0), true);
    /// ```
    ///
    pub fn set(&mut self, x: IndexType, y: IndexType, value: bool) {
        let pos = (x, y);
        if value {
            self.live_cells.insert(pos);
        } else {
            self.live_cells.remove(&pos);
        }
    }

    /// Returns the minimum bounding box of all live cells on the board.
    /// If the board is empty, returns (0, 0, 0, 0).
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let mut board = Board::new();
    /// assert_eq!((0, 0, 0, 0), board.bounding_box());
    /// board.set(-1, 2, true);
    /// board.set(3, -2, true);
    /// let (x_min, x_max, y_min, y_max) = board.bounding_box();
    /// assert_eq!(x_min, -1);
    /// assert_eq!(x_max, 3);
    /// assert_eq!(y_min, -2);
    /// assert_eq!(y_max, 2);
    /// ```
    ///
    pub fn bounding_box(&self) -> (IndexType, IndexType, IndexType, IndexType)
    where
        IndexType: Copy + PartialOrd + Zero,
    {
        let mut iter = self.live_cells.iter();
        if let Some(&(init_x, init_y)) = iter.next() {
            iter.fold((init_x, init_x, init_y, init_y), |(x_min, x_max, y_min, y_max), &(x, y)| {
                (
                    if x_min < x { x_min } else { x },
                    if x_max > x { x_max } else { x },
                    if y_min < y { y_min } else { y },
                    if y_max > y { y_max } else { y },
                )
            })
        } else {
            let zero = IndexType::zero();
            (zero, zero, zero, zero)
        }
    }
}

impl<'a, IndexType> Board<IndexType>
where
    IndexType: Eq + Hash,
{
    /// Creates a non-owning iterator over the series of immutable live cell positions on the board in arbitrary order.
    #[inline]
    pub fn iter(&'a self) -> std::collections::hash_set::Iter<'a, (IndexType, IndexType)> {
        self.into_iter()
    }
}

// Trait implementations

impl<IndexType> Default for Board<IndexType>
where
    IndexType: Eq + Hash,
{
    /// Same as new().
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<IndexType> fmt::Display for Board<IndexType>
where
    IndexType: Eq + Hash + Copy + PartialOrd + Zero + One + ToPrimitive,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (x_min, x_max, y_min, y_max) = self.bounding_box();
        for y in range_inclusive(y_min, y_max) {
            let mut line = String::new();
            for x in range_inclusive(x_min, x_max) {
                line.push(if self.get(x, y) { 'O' } else { '.' });
            }
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

impl<'a, IndexType> IntoIterator for &'a Board<IndexType>
where
    IndexType: Eq + Hash,
{
    type Item = &'a (IndexType, IndexType);
    type IntoIter = std::collections::hash_set::Iter<'a, (IndexType, IndexType)>;

    /// Creates a non-owning iterator over the series of immutable live cell positions on the board in arbitrary order.
    ///
    /// ```
    /// # use life_backend::Board;
    /// # use std::collections::HashSet;
    /// let pattern = [(1, 0), (0, 1)];
    /// let board: Board = pattern.iter().collect();
    /// let result: HashSet<_> = (&board).into_iter().collect();
    /// let expected: HashSet<_> = pattern.iter().collect();
    /// assert_eq!(result, expected);
    /// ```
    ///
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.live_cells.iter()
    }
}

impl<IndexType> IntoIterator for Board<IndexType>
where
    IndexType: Eq + Hash,
{
    type Item = (IndexType, IndexType);
    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;

    /// Creates an owning iterator over the series of moved live cell positions on the board in arbitrary order.
    ///
    /// ```
    /// # use life_backend::Board;
    /// # use std::collections::HashSet;
    /// let pattern = [(1, 0), (0, 1)];
    /// let board: Board = pattern.iter().collect();
    /// let result: HashSet<_> = board.into_iter().collect();
    /// let expected: HashSet<_> = pattern.into_iter().collect();
    /// assert_eq!(result, expected);
    /// ```
    ///
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.live_cells.into_iter()
    }
}

impl<'a, IndexType> FromIterator<&'a (IndexType, IndexType)> for Board<IndexType>
where
    IndexType: Eq + Hash + Copy + 'a,
{
    /// Conversion from a non-owning iterator over a series of &(IndexType, IndexType).
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let pattern = [(1, 0), (0, 1)];
    /// let board: Board = pattern.iter().collect();
    /// assert_eq!(board.get(0, 0), false);
    /// assert_eq!(board.get(1, 0), true);
    /// assert_eq!(board.get(0, 1), true);
    /// assert_eq!(board.get(1, 1), false);
    /// ```
    ///
    fn from_iter<T: IntoIterator<Item = &'a (IndexType, IndexType)>>(iter: T) -> Self {
        let live_cells: HashSet<(IndexType, IndexType)> = iter.into_iter().copied().collect();
        Self { live_cells }
    }
}

impl<IndexType> FromIterator<(IndexType, IndexType)> for Board<IndexType>
where
    IndexType: Eq + Hash,
{
    /// Conversion from an owning iterator over a series of (IndexType, IndexType).
    /// Each item in the series represents a moved live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let mut pattern = [(1, 0), (0, 1)];
    /// let board: Board = pattern.into_iter().collect();
    /// assert_eq!(board.get(0, 0), false);
    /// assert_eq!(board.get(1, 0), true);
    /// assert_eq!(board.get(0, 1), true);
    /// assert_eq!(board.get(1, 1), false);
    /// ```
    ///
    fn from_iter<T: IntoIterator<Item = (IndexType, IndexType)>>(iter: T) -> Self {
        let live_cells: HashSet<(IndexType, IndexType)> = iter.into_iter().collect();
        Self { live_cells }
    }
}

impl<'a, IndexType> Extend<&'a (IndexType, IndexType)> for Board<IndexType>
where
    IndexType: Eq + Hash + Copy + 'a,
{
    /// Extend the board with the contents of the specified non-owning iterator over the series of &(IndexType, IndexType).
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let mut board = Board::new();
    /// let pattern = [(1, 0), (0, 1)];
    /// board.extend(pattern.iter());
    /// assert_eq!(board.get(0, 0), false);
    /// assert_eq!(board.get(1, 0), true);
    /// assert_eq!(board.get(0, 1), true);
    /// assert_eq!(board.get(1, 1), false);
    /// ```
    ///
    #[inline]
    fn extend<T: IntoIterator<Item = &'a (IndexType, IndexType)>>(&mut self, iter: T) {
        self.live_cells.extend(iter);
    }
}

impl<IndexType> Extend<(IndexType, IndexType)> for Board<IndexType>
where
    IndexType: Eq + Hash,
{
    /// Extend the board with the contents of the specified owning iterator over the series of (IndexType, IndexType).
    /// Each item in the series represents a moved live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let mut board = Board::new();
    /// let pattern = [(1, 0), (0, 1)];
    /// board.extend(pattern.into_iter());
    /// assert_eq!(board.get(0, 0), false);
    /// assert_eq!(board.get(1, 0), true);
    /// assert_eq!(board.get(0, 1), true);
    /// assert_eq!(board.get(1, 1), false);
    /// ```
    ///
    #[inline]
    fn extend<T: IntoIterator<Item = (IndexType, IndexType)>>(&mut self, iter: T) {
        self.live_cells.extend(iter);
    }
}
