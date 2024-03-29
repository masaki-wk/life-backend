use num_traits::{Bounded, One, ToPrimitive, Zero};
use std::fmt;
use std::hash::Hash;
use std::mem;
use std::ops::{Add, Sub};

use crate::{Board, Position, Rule};

/// A representation of a game.
///
/// The type parameter `T` is used as the type of the x- and y-coordinate values for each cell.
///
/// The following operations are supported:
///
/// - Constructing from [`Rule`] and [`Board`]
/// - Advancing a generation
/// - Returning the current state information
///
/// # Examples
///
/// ```
/// use life_backend::{Board, Game, Position, Rule};
/// let rule = Rule::conways_life();
/// let board: Board<_> = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)] // Glider pattern
///     .iter()
///     .copied()
///     .map(|(x, y)| Position(x, y))
///     .collect();
/// let mut game = Game::new(rule, board);
/// game.advance();
/// let bbox = game.board().bounding_box();
/// assert_eq!(bbox.x(), &(0..=2));
/// assert_eq!(bbox.y(), &(1..=3));
/// ```
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Game<T>
where
    T: Eq + Hash,
{
    rule: Rule,
    curr_board: Board<T>,
    prev_board: Board<T>,
}

// Inherent methods

impl<T> Game<T>
where
    T: Eq + Hash,
{
    /// Creates from the specified rule and the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Game, Position, Rule};
    /// let rule = Rule::conways_life();
    /// let board: Board<_> = [Position(1, 0), Position(0, 1)].iter().collect();
    /// let game = Game::new(rule, board);
    /// ```
    ///
    pub fn new(rule: Rule, board: Board<T>) -> Self {
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
    /// let board: Board<_> = [Position(1, 0), Position(0, 1)].iter().collect();
    /// let game = Game::new(rule.clone(), board);
    /// assert_eq!(game.rule(), &rule);
    /// ```
    ///
    #[inline]
    pub const fn rule(&self) -> &Rule {
        &self.rule
    }

    /// Returns the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Game, Position, Rule};
    /// let rule = Rule::conways_life();
    /// let board: Board<_> = [Position(1, 0), Position(0, 1)].iter().collect();
    /// let game = Game::new(rule, board);
    /// let board = game.board();
    /// let bbox = board.bounding_box();
    /// assert_eq!(bbox.x(), &(0..=1));
    /// assert_eq!(bbox.y(), &(0..=1));
    /// assert_eq!(board.contains(&Position(0, 0)), false);
    /// assert_eq!(board.contains(&Position(1, 0)), true);
    /// assert_eq!(board.contains(&Position(0, 1)), true);
    /// assert_eq!(board.contains(&Position(1, 1)), false);
    /// ```
    ///
    #[inline]
    pub const fn board(&self) -> &Board<T> {
        &self.curr_board
    }

    // Returns the count of live neighbours of the specified position.
    fn live_neighbour_count(board: &Board<T>, position: &Position<T>) -> usize
    where
        T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T> + One + Bounded + ToPrimitive,
    {
        position.moore_neighborhood_positions().filter(|pos| board.contains(pos)).count()
    }

    /// Advance the game by one generation.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Game, Position, Rule};
    /// let rule = Rule::conways_life();
    /// let board: Board<_> = [Position(0, 1), Position(1, 1), Position(2, 1)].iter().collect(); // Blinker pattern
    /// let mut game = Game::new(rule, board);
    /// game.advance();
    /// let board = game.board();
    /// let bbox = board.bounding_box();
    /// assert_eq!(bbox.x(), &(1..=1));
    /// assert_eq!(bbox.y(), &(0..=2));
    /// assert_eq!(board.contains(&Position(1, 0)), true);
    /// assert_eq!(board.contains(&Position(1, 1)), true);
    /// assert_eq!(board.contains(&Position(1, 2)), true);
    /// ```
    ///
    pub fn advance(&mut self)
    where
        T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T> + One + Bounded + ToPrimitive,
    {
        mem::swap(&mut self.curr_board, &mut self.prev_board);
        let prev_board = &self.prev_board;
        let rule = &self.rule;
        self.curr_board.clear();
        self.curr_board.extend(
            self.prev_board
                .iter()
                .flat_map(|pos| pos.moore_neighborhood_positions())
                .filter(|pos| !prev_board.contains(pos)),
        );
        self.curr_board.retain(|pos| {
            let count = Self::live_neighbour_count(prev_board, pos);
            rule.is_born(count)
        });
        self.curr_board.extend(self.prev_board.iter().copied().filter(|pos| {
            let count = Self::live_neighbour_count(prev_board, pos);
            rule.is_survive(count)
        }));
    }
}

// Trait implementations

impl<T> fmt::Display for Game<T>
where
    T: Eq + Hash + Copy + PartialOrd + Zero + One + ToPrimitive,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.board().fmt(f)
    }
}

// Unit tests

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display() {
        let rule = Rule::conways_life();
        let board: Board<_> = [Position(1, 0), Position(0, 1)].iter().collect();
        let target = Game::new(rule, board);
        println!("{target}");
    }
}
