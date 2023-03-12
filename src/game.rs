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

    // Returns positions of neighbourhoods of the specified position
    fn neighbourhoods_positions(x: IndexType, y: IndexType) -> impl Iterator<Item = (IndexType, IndexType)>
    where
        IndexType: Copy + PartialEq + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        let min = IndexType::min_value();
        let max = IndexType::max_value();
        let one = IndexType::one();
        let mut buf = Vec::new();
        for v in range_inclusive(if y > min { y - one } else { y }, if y < max { y + one } else { y }) {
            for u in range_inclusive(if x > min { x - one } else { x }, if x < max { x + one } else { x }) {
                if u != x || v != y {
                    buf.push((u, v))
                }
            }
        }
        buf.into_iter()
    }

    // Returns the count of neighbourhoods of the specified position
    fn neighbourhoods_count(x: IndexType, y: IndexType, board: &Board<IndexType>) -> usize
    where
        IndexType: Copy + PartialEq + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        Self::neighbourhoods_positions(x, y).filter(|&(u, v)| board.get(u, v)).count()
    }

    // Returns the next board of the specified board.
    fn next_board(board: &Board<IndexType>) -> Board<IndexType>
    where
        IndexType: Copy + PartialEq + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        let scanpos_for_deadcells: HashSet<(IndexType, IndexType)> = board
            .iter()
            .flat_map(|&(x, y)| Self::neighbourhoods_positions(x, y))
            .filter(|&(x, y)| !board.get(x, y))
            .collect();
        let scanpos_for_livecells = board.iter();
        let mut livecells = Vec::new();
        scanpos_for_livecells
            .filter(|&&(x, y)| {
                let count = Self::neighbourhoods_count(x, y, board);
                count == 2 || count == 3
            })
            .for_each(|&pos| livecells.push(pos));
        scanpos_for_deadcells
            .into_iter()
            .filter(|&(x, y)| {
                let count = Self::neighbourhoods_count(x, y, board);
                count == 3
            })
            .for_each(|pos| livecells.push(pos));
        livecells.into_iter().collect()
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
        IndexType: Copy + PartialEq + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive + std::fmt::Debug,
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
