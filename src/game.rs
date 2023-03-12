use super::Board;
use num_iter::range_inclusive;
use num_traits::{Bounded, One, ToPrimitive, Zero};
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::ops::{Add, Sub};

/// The default index type of boards.
pub type DefaultIndexType = i16;

/// A representation of games.
#[derive(Debug)]
pub struct Game<IndexType = DefaultIndexType>
where
    IndexType: Eq + Hash,
{
    board: Board<IndexType>,
}

impl<IndexType> Game<IndexType>
where
    IndexType: Eq + Hash,
{
    /// Creates from the specified board.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::{Board, Game};
    /// let board: Board = [(1, 0), (0, 1)].iter().collect();
    /// let game = Game::new(board);
    /// ```
    ///
    #[inline]
    pub fn new(board: Board<IndexType>) -> Self {
        Self { board }
    }

    /// Returns the board.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::{Board, Game};
    /// let board: Board = [(1, 0), (0, 1)].iter().collect();
    /// let game = Game::new(board);
    /// let board = game.board();
    /// assert_eq!(board.bounding_box(), (0, 1, 0, 1));
    /// assert_eq!(board.get(0, 0), false);
    /// assert_eq!(board.get(1, 0), true);
    /// assert_eq!(board.get(0, 1), true);
    /// assert_eq!(board.get(1, 1), false);
    /// ```
    ///
    #[inline]
    pub fn board(&self) -> &Board<IndexType> {
        &self.board
    }

    // Returns positions of neighbourhoods of the specified position, and the specified position itself if requested.
    fn neighbourhood_plus_center(includes_center: bool, x: IndexType, y: IndexType) -> impl Iterator<Item = (IndexType, IndexType)>
    where
        IndexType: Copy + PartialEq + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        let min = IndexType::min_value();
        let max = IndexType::max_value();
        let one = IndexType::one();
        let mut buf = Vec::new();
        for v in range_inclusive(if y > min { y - one } else { y }, if y < max { y + one } else { y }) {
            for u in range_inclusive(if x > min { x - one } else { x }, if x < max { x + one } else { x }) {
                if includes_center || u != x || v != y {
                    buf.push((u, v))
                }
            }
        }
        buf.into_iter()
    }

    // Returns the next board of the specified board.
    fn next_board(board: &Board<IndexType>) -> Board<IndexType>
    where
        IndexType: Copy + PartialEq + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        let scan_positions: HashSet<(IndexType, IndexType)> = board.iter().flat_map(|&(x, y)| Self::neighbourhood_plus_center(true, x, y)).collect();
        let next_board: Board<IndexType> = scan_positions
            .into_iter()
            .filter(|&(x, y)| {
                let state = board.get(x, y);
                let count = Self::neighbourhood_plus_center(false, x, y).filter(|&(x, y)| board.get(x, y)).count();
                matches!((state, count), (true, 2) | (true, 3) | (false, 3))
            })
            .collect();
        next_board
    }

    /// Update the state of the game.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::{Board, Game};
    /// let board: Board = [(0, 1), (1, 1), (2, 1)].iter().collect(); // Blinker pattern
    /// let mut game = Game::new(board);
    /// game.update();
    /// let board = game.board();
    /// assert_eq!(board.bounding_box(), (1, 1, 0, 2));
    /// assert_eq!(board.get(0, 0), false);
    /// assert_eq!(board.get(1, 0), true);
    /// assert_eq!(board.get(2, 0), false);
    /// assert_eq!(board.get(0, 1), false);
    /// assert_eq!(board.get(1, 1), true);
    /// assert_eq!(board.get(2, 1), false);
    /// assert_eq!(board.get(0, 2), false);
    /// assert_eq!(board.get(1, 2), true);
    /// assert_eq!(board.get(2, 2), false);
    /// ```
    ///
    pub fn update(&mut self)
    where
        IndexType: Copy + PartialEq + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        self.board = Self::next_board(self.board())
    }
}

impl<IndexType> fmt::Display for Game<IndexType>
where
    IndexType: Eq + Hash + Copy + PartialOrd + Zero + One + ToPrimitive,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.board.fmt(f)
    }
}
