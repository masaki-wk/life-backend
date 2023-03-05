use num::One;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::ops::Add;

/// the default index type of boards.
pub type DefaultIndexType = i16;

/// A representation of boards.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board<IndexType = DefaultIndexType>
where
    IndexType: Eq + Hash,
{
    live_cells: HashSet<(IndexType, IndexType)>,
}

impl<IndexType> Board<IndexType>
where
    IndexType: Eq + Hash,
{
    /// Creates an empty board.
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
        IndexType: Copy + Default + PartialOrd,
    {
        let mut iter = self.live_cells.iter();
        if let Some(&(init_x, init_y)) = iter.next() {
            iter.fold(
                (init_x, init_x, init_y, init_y),
                |(mut x_min, mut x_max, mut y_min, mut y_max), &(x, y)| {
                    if x < x_min {
                        x_min = x
                    };
                    if x > x_max {
                        x_max = x
                    };
                    if y < y_min {
                        y_min = y
                    };
                    if y > y_max {
                        y_max = y
                    };
                    (x_min, x_max, y_min, y_max)
                },
            )
        } else {
            (
                IndexType::default(),
                IndexType::default(),
                IndexType::default(),
                IndexType::default(),
            )
        }
    }
}

impl<IndexType> Default for Board<IndexType>
where
    IndexType: Eq + Hash,
{
    /// Same as new().
    fn default() -> Self {
        Self::new()
    }
}

impl<IndexType> fmt::Display for Board<IndexType>
where
    IndexType: Add<Output = IndexType> + Copy + Default + Eq + Hash + One + PartialOrd,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (x_min, x_max, y_min, y_max) = self.bounding_box();
        let mut y = y_min;
        while y <= y_max {
            let mut x = x_min;
            let mut line = String::new();
            while x <= x_max {
                line.push(if self.get(x, y) { 'O' } else { '.' });
                x = x + IndexType::one();
            }
            writeln!(f, "{line}")?;
            y = y + IndexType::one();
        }
        Ok(())
    }
}

impl<'a, IndexType> FromIterator<&'a (IndexType, IndexType)> for Board<IndexType>
where
    IndexType: Copy + Eq + Hash + 'a,
{
    /// Conversion from a non-owning iterator over a series of &(IndexType, IndexType).
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let pattern = [(0, 0), (1, 0), (2, 0), (1, 1)];
    /// let board: Board = pattern.iter().collect();
    /// assert_eq!(board.get(0, 0), true);
    /// assert_eq!(board.get(1, 0), true);
    /// assert_eq!(board.get(2, 0), true);
    /// assert_eq!(board.get(0, 1), false);
    /// assert_eq!(board.get(1, 1), true);
    /// assert_eq!(board.get(2, 1), false);
    /// ```
    ///
    fn from_iter<T: IntoIterator<Item = &'a (IndexType, IndexType)>>(iter: T) -> Self {
        let live_cells: HashSet<(IndexType, IndexType)> = iter.into_iter().copied().collect();
        Self { live_cells }
    }
}

impl<'a, IndexType> FromIterator<&'a mut (IndexType, IndexType)> for Board<IndexType>
where
    IndexType: Copy + Eq + Hash,
{
    /// Conversion from a non-owning iterator over a series of &mut (IndexType, IndexType).
    /// Each item in the series represents a mutable reference of a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let mut pattern = [(0, 0), (1, 0), (2, 0), (1, 1)];
    /// let board: Board = pattern.iter_mut().collect();
    /// assert_eq!(board.get(0, 0), true);
    /// assert_eq!(board.get(1, 0), true);
    /// assert_eq!(board.get(2, 0), true);
    /// assert_eq!(board.get(0, 1), false);
    /// assert_eq!(board.get(1, 1), true);
    /// assert_eq!(board.get(2, 1), false);
    /// ```
    ///
    fn from_iter<T: IntoIterator<Item = &'a mut (IndexType, IndexType)>>(iter: T) -> Self {
        let live_cells: HashSet<(IndexType, IndexType)> =
            iter.into_iter().map(|&mut x| x).collect();
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
    /// let pattern = [(0, 0), (1, 0), (2, 0), (1, 1)];
    /// let board: Board = pattern.into_iter().collect();
    /// assert_eq!(board.get(0, 0), true);
    /// assert_eq!(board.get(1, 0), true);
    /// assert_eq!(board.get(2, 0), true);
    /// assert_eq!(board.get(0, 1), false);
    /// assert_eq!(board.get(1, 1), true);
    /// assert_eq!(board.get(2, 1), false);
    /// ```
    ///
    fn from_iter<T: IntoIterator<Item = (IndexType, IndexType)>>(iter: T) -> Self {
        let live_cells: HashSet<(IndexType, IndexType)> = iter.into_iter().collect();
        Self { live_cells }
    }
}

impl<'a, IndexType> Extend<&'a (IndexType, IndexType)> for Board<IndexType>
where
    IndexType: Copy + Eq + Hash + 'a,
{
    /// Extend the board with the contents of the specified non-owning iterator over the series of &(IndexType, IndexType).
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let mut board = Board::new();
    /// let pattern = [(0, 0), (1, 0), (2, 0), (1, 1)];
    /// board.extend(pattern.iter());
    /// assert_eq!(board.get(0, 0), true);
    /// assert_eq!(board.get(1, 0), true);
    /// assert_eq!(board.get(2, 0), true);
    /// assert_eq!(board.get(0, 1), false);
    /// assert_eq!(board.get(1, 1), true);
    /// assert_eq!(board.get(2, 1), false);
    /// ```
    ///
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
    /// let pattern = [(0, 0), (1, 0), (2, 0), (1, 1)];
    /// board.extend(pattern.into_iter());
    /// assert_eq!(board.get(0, 0), true);
    /// assert_eq!(board.get(1, 0), true);
    /// assert_eq!(board.get(2, 0), true);
    /// assert_eq!(board.get(0, 1), false);
    /// assert_eq!(board.get(1, 1), true);
    /// assert_eq!(board.get(2, 1), false);
    /// ```
    ///
    fn extend<T: IntoIterator<Item = (IndexType, IndexType)>>(&mut self, iter: T) {
        self.live_cells.extend(iter);
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
    /// let pattern = [(0, 0), (1, 0), (2, 0), (1, 1)];
    /// let board: Board = pattern.iter().collect();
    /// let result: HashSet<_> = (&board).into_iter().collect();
    /// let expected: HashSet<_> = pattern.iter().collect();
    /// assert_eq!(result, expected);
    /// ```
    ///
    fn into_iter(self) -> Self::IntoIter {
        self.live_cells.iter()
    }
}

impl<'a, IndexType> Board<IndexType>
where
    IndexType: Eq + Hash,
{
    /// Creates a non-owning iterator over the series of immutable live cell positions on the board in arbitrary order.
    pub fn iter(&'a self) -> std::collections::hash_set::Iter<'a, (IndexType, IndexType)> {
        self.into_iter()
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
    /// let pattern = [(0, 0), (1, 0), (2, 0), (1, 1)];
    /// let board: Board = pattern.iter().collect();
    /// let result: HashSet<_> = board.into_iter().collect();
    /// let expected: HashSet<_> = pattern.into_iter().collect();
    /// assert_eq!(result, expected);
    /// ```
    ///
    fn into_iter(self) -> Self::IntoIter {
        self.live_cells.into_iter()
    }
}
