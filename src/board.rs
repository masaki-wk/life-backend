use std::collections::HashSet;
use std::fmt;

/// the index type of boards.
pub type IndexType = i16;

// the position type of boards.
type Position = (IndexType, IndexType);

/// A representation of boards.
#[derive(Debug)]
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
    /// Conversion from an Iterator over a sequence of &(IndexType, IndexType).
    /// Each item in the sequence represents a reference of a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let pattern = [(0, 0), (1, 0), (2, 0), (1, 1)];  // T-tetromino pattern
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

impl FromIterator<Position> for Board {
    /// Conversion from an Iterator over a sequence of (IndexType, IndexType).
    /// Each item in the sequence represents a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let pattern = [(0, 0), (1, 0), (2, 0), (1, 1)];  // T-tetromino pattern
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
