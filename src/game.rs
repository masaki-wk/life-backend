use super::Board;
use num_iter::range_inclusive;
use num_traits::{Bounded, One, ToPrimitive, Zero};
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::mem;
use std::ops::{Add, Sub};

/// The default index type of boards.
type DefaultIndexType = i16;

/// A representation of games.
#[derive(Debug)]
pub struct Game<IndexType = DefaultIndexType>
where
    IndexType: Eq + Hash,
{
    curr_board: Board<IndexType>,
    prev_board: Board<IndexType>,
    birth_candidates: HashSet<(IndexType, IndexType)>,
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
    pub fn new(board: Board<IndexType>) -> Self {
        Self {
            curr_board: board,
            prev_board: Board::new(),
            birth_candidates: HashSet::new(),
        }
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
        &self.curr_board
    }

    // Creates an iterator over neighbour positions of the specified position, defined as Moore neighbourhood.
    //
    // - _with_boundary_check: Maximum and minimum bounds of x and y are checked
    // - _no_boundary_check: Maximum and minimum bounds of x and y are NOT checked
    //
    fn neighbour_positions_with_boundary_check(x: IndexType, y: IndexType) -> impl Iterator<Item = (IndexType, IndexType)>
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        let min = IndexType::min_value();
        let max = IndexType::max_value();
        let one = IndexType::one();
        let x_start = if x > min { x - one } else { x };
        let x_stop = if x < max { x + one } else { x };
        let y_start = if y > min { y - one } else { y };
        let y_stop = if y < max { y + one } else { y };
        range_inclusive(y_start, y_stop)
            .flat_map(move |v| range_inclusive(x_start, x_stop).map(move |u| (u, v)))
            .filter(move |&(u, v)| u != x || v != y)
    }
    fn neighbour_positions_no_boundary_check(x: IndexType, y: IndexType) -> impl Iterator<Item = (IndexType, IndexType)>
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + ToPrimitive,
    {
        let one = IndexType::one();
        let x_start = x - one;
        let x_stop = x + one;
        let y_start = y - one;
        let y_stop = y + one;
        range_inclusive(y_start, y_stop)
            .flat_map(move |v| range_inclusive(x_start, x_stop).map(move |u| (u, v)))
            .filter(move |&(u, v)| u != x || v != y)
    }

    // Returns the count of live neighbours of the specified position.
    //
    // - _with_boundary_check: Maximum and minimum bounds of x and y are checked
    // - _no_boundary_check: Maximum and minimum bounds of x and y are NOT checked
    //
    fn live_neighbour_count_with_boundary_check(board: &Board<IndexType>, x: IndexType, y: IndexType) -> usize
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        Self::neighbour_positions_with_boundary_check(x, y).filter(|&(u, v)| board.get(u, v)).count()
    }
    fn live_neighbour_count_no_boundary_check(board: &Board<IndexType>, x: IndexType, y: IndexType) -> usize
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + ToPrimitive,
    {
        Self::neighbour_positions_no_boundary_check(x, y).filter(|&(u, v)| board.get(u, v)).count()
    }

    // Returns candidate cells that will be born in the next generation of the specific board. These candidates may contain duplicate positions.
    //
    // - _with_boundary_check: Maximum and minimum bounds of x and y are checked
    // - _no_boundary_check: Maximum and minimum bounds of x and y are NOT checked
    //
    fn birth_candidate_cells_with_boundary_check(board: &Board<IndexType>) -> impl Iterator<Item = (IndexType, IndexType)> + '_
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        board
            .iter()
            .flat_map(|&(x, y)| Self::neighbour_positions_with_boundary_check(x, y))
            .filter(|&(x, y)| !board.get(x, y))
    }
    fn birth_candidate_cells_no_boundary_check(board: &Board<IndexType>) -> impl Iterator<Item = (IndexType, IndexType)> + '_
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + ToPrimitive,
    {
        board
            .iter()
            .flat_map(|&(x, y)| Self::neighbour_positions_no_boundary_check(x, y))
            .filter(|&(x, y)| !board.get(x, y))
    }

    // Returns cells that will be survive in the next generation of the specific board.
    //
    // - _with_boundary_check: Maximum and minimum bounds of x and y are checked
    // - _no_boundary_check: Maximum and minimum bounds of x and y are NOT checked
    //
    fn survive_cells_with_boundary_check(board: &Board<IndexType>) -> impl Iterator<Item = (IndexType, IndexType)> + '_
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        board.iter().copied().filter(|&(x, y)| {
            let count = Self::live_neighbour_count_with_boundary_check(board, x, y);
            count == 2 || count == 3
        })
    }
    fn survive_cells_no_boundary_check(board: &Board<IndexType>) -> impl Iterator<Item = (IndexType, IndexType)> + '_
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + ToPrimitive,
    {
        board.iter().copied().filter(|&(x, y)| {
            let count = Self::live_neighbour_count_no_boundary_check(board, x, y);
            count == 2 || count == 3
        })
    }

    // Selects the cells that will actually be born from the specific candidate birth cells.
    //
    // - _with_boundary_check: Maximum and minimum bounds of x and y are checked
    // - _no_boundary_check: Maximum and minimum bounds of x and y are NOT checked
    //
    fn birth_cells_with_boundary_check<'a, 'b>(
        board: &'a Board<IndexType>,
        candidates: &'b HashSet<(IndexType, IndexType)>,
    ) -> impl Iterator<Item = (IndexType, IndexType)> + 'b
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
        'a: 'b,
    {
        candidates.iter().copied().filter(|&(x, y)| {
            let count = Self::live_neighbour_count_with_boundary_check(board, x, y);
            count == 3
        })
    }
    fn birth_cells_no_boundary_check<'a, 'b>(
        board: &'a Board<IndexType>,
        candidates: &'b HashSet<(IndexType, IndexType)>,
    ) -> impl Iterator<Item = (IndexType, IndexType)> + 'b
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + ToPrimitive,
        'a: 'b,
    {
        candidates.iter().copied().filter(|&(x, y)| {
            let count = Self::live_neighbour_count_no_boundary_check(board, x, y);
            count == 3
        })
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
    /// assert_eq!(board.get(1, 0), true);
    /// assert_eq!(board.get(1, 1), true);
    /// assert_eq!(board.get(1, 2), true);
    /// ```
    ///
    pub fn update(&mut self)
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + Zero + One + Bounded + ToPrimitive,
    {
        mem::swap(&mut self.curr_board, &mut self.prev_board);
        self.curr_board.clear();
        self.birth_candidates.clear();
        let boundary_check = {
            let min = IndexType::min_value();
            let max = IndexType::max_value();
            let one = IndexType::one();
            let near_min = min + one;
            let near_max = max - one;
            let (min_x, max_x, min_y, max_y) = self.prev_board.bounding_box();
            (min_x <= near_min) || (max_x >= near_max) || (min_y <= near_min) || (max_y >= near_max)
        };
        if boundary_check {
            self.birth_candidates.extend(Self::birth_candidate_cells_with_boundary_check(&self.prev_board));
            self.curr_board.extend(Self::survive_cells_with_boundary_check(&self.prev_board));
            self.curr_board
                .extend(Self::birth_cells_with_boundary_check(&self.prev_board, &self.birth_candidates));
        } else {
            self.birth_candidates.extend(Self::birth_candidate_cells_no_boundary_check(&self.prev_board));
            self.curr_board.extend(Self::survive_cells_no_boundary_check(&self.prev_board));
            self.curr_board
                .extend(Self::birth_cells_no_boundary_check(&self.prev_board, &self.birth_candidates));
        }
    }
}

impl<IndexType> fmt::Display for Game<IndexType>
where
    IndexType: Eq + Hash + Copy + PartialOrd + Zero + One + ToPrimitive,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.board().fmt(f)
    }
}
