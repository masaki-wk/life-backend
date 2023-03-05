use anyhow::{ensure, Result};
use std::fmt;

/// the index type of boards.
pub type BoardIndexType = u16;

// the position type on boards.
type Position = (BoardIndexType, BoardIndexType);

/// A representation of boards.
#[derive(Debug)]
pub struct Board {
    width: BoardIndexType,
    height: BoardIndexType,
    grids: Vec<bool>,
}

impl Board {
    /// Creates a board with specified width and height.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let board = Board::with_size(3, 2);
    /// assert_eq!(board.width(), 3);
    /// assert_eq!(board.height(), 2);
    /// ```
    ///
    pub fn with_size(width: BoardIndexType, height: BoardIndexType) -> Self {
        let grids = if width > 0 && height > 0 {
            vec![false; width as usize * height as usize]
        } else {
            Vec::new()
        };
        Self {
            width,
            height,
            grids,
        }
    }

    /// Returns the width of the board.
    pub fn width(&self) -> BoardIndexType {
        self.width
    }

    /// Returns the height of the board.
    pub fn height(&self) -> BoardIndexType {
        self.height
    }

    // Converts the specified position (x, y) into the index for self.grids.
    // If the position is out of the board, returns an error.
    fn position_to_index(&self, x: BoardIndexType, y: BoardIndexType) -> Result<usize> {
        ensure!(x < self.width && y < self.height, "index out of bounds");
        let index = y as usize * self.width as usize + x as usize;
        Ok(index)
    }

    /// Returns the value of the specified position of the board.
    /// If the position is out of the board, returns an error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let board = Board::with_size(3, 2);
    /// assert!(matches!(board.get(0, 0), Ok(false)));
    /// assert!(matches!(board.get(3, 0), Err(_)));
    /// ```
    ///
    pub fn get(&self, x: BoardIndexType, y: BoardIndexType) -> Result<bool> {
        let index = self.position_to_index(x, y)?;
        Ok(self.grids[index])
    }

    /// Set the specified value on the specified position of the board.
    /// If the position is out of the board, changes nothing and returns an error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let mut board = Board::with_size(3, 2);
    /// assert!(matches!(board.set(0, 0, true), Ok(_)));
    /// assert!(matches!(board.get(0, 0), Ok(true)));
    /// assert!(matches!(board.set(3, 0, true), Err(_)));
    /// ```
    ///
    pub fn set(&mut self, x: BoardIndexType, y: BoardIndexType, value: bool) -> Result<()> {
        let index = self.position_to_index(x, y)?;
        self.grids[index] = value;
        Ok(())
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.height() {
            let line: String = (0..self.width())
                .map(|x| if self.get(x, y).unwrap() { 'O' } else { '.' })
                .collect();
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

impl<'a> FromIterator<&'a Position> for Board {
    /// Conversion from an Iterator over a sequence of &(BoardIndexType, BoardIndexType).
    /// Each item in the sequence represents a reference of a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Board;
    /// let pattern = [(0, 0), (1, 0), (2, 0), (1, 1)];  // T-tetromino pattern
    /// let board: Board = pattern.iter().collect();
    /// assert_eq!(board.width(), 3);
    /// assert_eq!(board.height(), 2);
    /// assert_eq!(board.get(0, 0).unwrap(), true);
    /// assert_eq!(board.get(1, 0).unwrap(), true);
    /// assert_eq!(board.get(2, 0).unwrap(), true);
    /// assert_eq!(board.get(0, 1).unwrap(), false);
    /// assert_eq!(board.get(1, 1).unwrap(), true);
    /// assert_eq!(board.get(2, 1).unwrap(), false);
    /// ```
    ///
    fn from_iter<T: IntoIterator<Item = &'a Position>>(iter: T) -> Self {
        let mut buf = Vec::new();
        let mut width: BoardIndexType = 0;
        let mut height: BoardIndexType = 0;
        for &pos in iter {
            let (x, y) = pos;
            if x >= width {
                if x == BoardIndexType::MAX {
                    panic!("x coordinate of the position is out of bounds");
                }
                width = x + 1;
            }
            if y >= height {
                if y == BoardIndexType::MAX {
                    panic!("y coordinate of the position is out of bounds");
                }
                height = y + 1;
            }
            buf.push(pos);
        }
        let mut board = Self::with_size(width, height);
        for pos in buf {
            board.set(pos.0, pos.1, true).unwrap(); // this never panics
        }
        board
    }
}
