use num_iter::range_inclusive;
use num_traits::{Bounded, One, ToPrimitive, Zero};
use std::fmt;
use std::hash::Hash;
use std::mem;
use std::ops::{Add, Sub};

use crate::{Board, Position, Rule};

/// The default index type of boards.
type DefaultIndexType = i16;

/// A representation of games.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game<IndexType = DefaultIndexType>
where
    IndexType: Eq + Hash,
{
    rule: Rule,
    curr_board: Board<IndexType>,
    prev_board: Board<IndexType>,
}

impl<IndexType> Game<IndexType>
where
    IndexType: Eq + Hash,
{
    /// Creates from the specified rule and the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Game, Position, Rule};
    /// let rule = Rule::conways_life();
    /// let board: Board = [Position(1, 0), Position(0, 1)].iter().collect();
    /// let game = Game::new(rule, board);
    /// ```
    ///
    pub fn new(rule: Rule, board: Board<IndexType>) -> Self {
        Self {
            rule,
            curr_board: board,
            prev_board: Board::new(),
        }
    }

    /// Returns the rule.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Game, Position, Rule};
    /// let rule = Rule::conways_life();
    /// let board: Board = [Position(1, 0), Position(0, 1)].iter().collect();
    /// let game = Game::new(rule.clone(), board);
    /// assert_eq!(game.rule(), &rule);
    /// ```
    ///
    #[inline]
    pub fn rule(&self) -> &Rule {
        &self.rule
    }

    /// Returns the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Game, Position, Rule};
    /// let rule = Rule::conways_life();
    /// let board: Board = [Position(1, 0), Position(0, 1)].iter().collect();
    /// let game = Game::new(rule, board);
    /// let board = game.board();
    /// assert_eq!(board.bounding_box(), Some((0, 1, 0, 1)));
    /// assert_eq!(board.get(&Position(0, 0)), false);
    /// assert_eq!(board.get(&Position(1, 0)), true);
    /// assert_eq!(board.get(&Position(0, 1)), true);
    /// assert_eq!(board.get(&Position(1, 1)), false);
    /// ```
    ///
    #[inline]
    pub fn board(&self) -> &Board<IndexType> {
        &self.curr_board
    }

    // Creates an iterator over neighbour positions of the specified position, defined as Moore neighbourhood.
    fn neighbour_positions(position: &Position<IndexType>) -> impl Iterator<Item = Position<IndexType>>
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        let Position(x, y) = *position;
        let min = IndexType::min_value();
        let max = IndexType::max_value();
        let one = IndexType::one();
        let x_start = if x > min { x - one } else { x };
        let x_stop = if x < max { x + one } else { x };
        let y_start = if y > min { y - one } else { y };
        let y_stop = if y < max { y + one } else { y };
        range_inclusive(y_start, y_stop)
            .flat_map(move |v| range_inclusive(x_start, x_stop).map(move |u| Position(u, v)))
            .filter(move |&Position(u, v)| u != x || v != y)
    }

    // Returns the count of live neighbours of the specified position.
    fn live_neighbour_count(board: &Board<IndexType>, position: &Position<IndexType>) -> usize
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        Self::neighbour_positions(position).filter(|pos| board.get(pos)).count()
    }

    /// Update the state of the game.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Game, Position, Rule};
    /// let rule = Rule::conways_life();
    /// let board: Board = [Position(0, 1), Position(1, 1), Position(2, 1)].iter().collect(); // Blinker pattern
    /// let mut game = Game::new(rule, board);
    /// game.update();
    /// let board = game.board();
    /// assert_eq!(board.bounding_box(), Some((1, 1, 0, 2)));
    /// assert_eq!(board.get(&Position(1, 0)), true);
    /// assert_eq!(board.get(&Position(1, 1)), true);
    /// assert_eq!(board.get(&Position(1, 2)), true);
    /// ```
    ///
    pub fn update(&mut self)
    where
        IndexType: Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + One + Bounded + ToPrimitive,
    {
        mem::swap(&mut self.curr_board, &mut self.prev_board);
        self.curr_board.clear();
        self.curr_board.extend(
            self.prev_board
                .iter()
                .flat_map(|pos| Self::neighbour_positions(pos))
                .filter(|pos| !self.prev_board.get(pos)),
        );
        self.curr_board.retain(|pos| {
            let count = Self::live_neighbour_count(&self.prev_board, pos);
            self.rule.is_born(count)
        });
        self.curr_board.extend(self.prev_board.iter().copied().filter(|pos| {
            let count = Self::live_neighbour_count(&self.prev_board, pos);
            self.rule.is_survive(count)
        }));
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
