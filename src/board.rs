use std::collections::HashSet;
use std::fmt;

/// the index type of boards.
pub type IndexType = i16;

// the position type of boards.
type Position = (IndexType, IndexType);

/// A representation of boards.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    live_cells: HashSet<Position>,
}

impl Board {
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let mut board = Board::new();
    /// board.set(-1, 2, true);
    /// board.set(3, -2, true);
    /// let (x_min, x_max, y_min, y_max) = board.bounding_box();
    /// assert_eq!(x_min, -1);
    /// assert_eq!(x_max, 3);
    /// assert_eq!(y_min, -2);
    /// assert_eq!(y_max, 2);
    /// ```
    ///
    pub fn bounding_box(&self) -> (IndexType, IndexType, IndexType, IndexType) {
        let mut x_min = 0;
        let mut x_max = 0;
        let mut y_min = 0;
        let mut y_max = 0;
        for &(x, y) in &self.live_cells {
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
        }
        (x_min, x_max, y_min, y_max)
    }
}

impl Default for Board {
    /// Same as new().
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (x_min, x_max, y_min, y_max) = self.bounding_box();
        for y in y_min..=y_max {
            let line: String = (x_min..=x_max)
                .map(|x| if self.get(x, y) { 'O' } else { '.' })
                .collect();
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

impl<'a> FromIterator<&'a Position> for Board {
    /// Conversion from a non-consuming iterator over a series of &(IndexType, IndexType).
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
    fn from_iter<T: IntoIterator<Item = &'a Position>>(iter: T) -> Self {
        let live_cells: HashSet<Position> = iter.into_iter().copied().collect();
        Self { live_cells }
    }
}

impl<'a> FromIterator<&'a mut Position> for Board {
    /// Conversion from a non-consuming iterator over a series of &mut (IndexType, IndexType).
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
    fn from_iter<T: IntoIterator<Item = &'a mut Position>>(iter: T) -> Self {
        let live_cells: HashSet<Position> = iter.into_iter().map(|&mut x| x).collect();
        Self { live_cells }
    }
}

impl FromIterator<Position> for Board {
    /// Conversion from a consuming iterator over a series of (IndexType, IndexType).
    /// Each item in the series represents a live cell position.
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
    fn from_iter<T: IntoIterator<Item = Position>>(iter: T) -> Self {
        let live_cells: HashSet<Position> = iter.into_iter().collect();
        Self { live_cells }
    }
}

impl<'a> Extend<&'a Position> for Board {
    /// Extend the board with the contents of the specified non-consuming iterator over the series of &(IndexType, IndexType).
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
    fn extend<T: IntoIterator<Item = &'a Position>>(&mut self, iter: T) {
        self.live_cells.extend(iter);
    }
}

impl Extend<Position> for Board {
    /// Extend the board with the contents of the specified consuming iterator over the series of (IndexType, IndexType).
    /// Each item in the series represents a live cell position.
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
    fn extend<T: IntoIterator<Item = Position>>(&mut self, iter: T) {
        self.live_cells.extend(iter);
    }
}

impl<'a> IntoIterator for &'a Board {
    type Item = &'a Position;
    type IntoIter = std::collections::hash_set::Iter<'a, Position>;

    /// Creates a non-consuming iterator over the series of immutable live cell positions on the board in arbitrary order.
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

impl<'a> Board {
    /// Creates a non-consuming iterator over the series of immutable live cell positions on the board in arbitrary order.
    pub fn iter(&'a self) -> std::collections::hash_set::Iter<'a, Position> {
        self.into_iter()
    }
}

impl IntoIterator for Board {
    type Item = Position;
    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;

    /// Creates a consuming iterator over the series of moved live cell positions on the board in arbitrary order.
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
