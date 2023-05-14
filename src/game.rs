use num_traits::{Bounded, One, ToPrimitive, Zero};
use std::fmt;
use std::hash::Hash;
use std::mem;
use std::ops::{Add, Sub};

use crate::{Board, Position, Rule};

/// The default coordinate type of `Game`.
type DefaultCoordinateType = i16;

/// A representation of games.
///
/// The type parameter `T` is used as the type of the x- and y-coordinate values for each cell.
/// The following operations are supported:
///
/// - Constructing from `Rule` and `Board`
/// - Advancing a generation
/// - Returning the current state
///
/// # Examples
///
/// ```
/// use life_backend::{Board, Game, Position, Rule};
/// let rule = Rule::conways_life();
/// let board: Board<_> = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)] // Glider pattern
///     .into_iter()
///     .map(|(x, y)| Position(x, y))
///     .collect();
/// let mut game = Game::new(rule, board);
/// game.update();
/// let board = game.board();
/// let bbox = board.bounding_box();
/// assert_eq!(bbox.x(), &(0..=2));
/// assert_eq!(bbox.y(), &(1..=3));
/// ```
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game<T = DefaultCoordinateType>
where
    T: Eq + Hash,
{
    rule: Rule,
    curr_board: Board<T>,
    prev_board: Board<T>,
}

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
    /// assert_eq!(board.get(&Position(0, 0)), false);
    /// assert_eq!(board.get(&Position(1, 0)), true);
    /// assert_eq!(board.get(&Position(0, 1)), true);
    /// assert_eq!(board.get(&Position(1, 1)), false);
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
        position.moore_neighborhood_positions().filter(|pos| board.get(pos)).count()
    }

    /// Update the state of the game.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Board, Game, Position, Rule};
    /// let rule = Rule::conways_life();
    /// let board: Board<_> = [Position(0, 1), Position(1, 1), Position(2, 1)].iter().collect(); // Blinker pattern
    /// let mut game = Game::new(rule, board);
    /// game.update();
    /// let board = game.board();
    /// let bbox = board.bounding_box();
    /// assert_eq!(bbox.x(), &(1..=1));
    /// assert_eq!(bbox.y(), &(0..=2));
    /// assert_eq!(board.get(&Position(1, 0)), true);
    /// assert_eq!(board.get(&Position(1, 1)), true);
    /// assert_eq!(board.get(&Position(1, 2)), true);
    /// ```
    ///
    pub fn update(&mut self)
    where
        T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T> + One + Bounded + ToPrimitive,
    {
        mem::swap(&mut self.curr_board, &mut self.prev_board);
        self.curr_board.clear();
        self.curr_board.extend(
            self.prev_board
                .iter()
                .flat_map(|pos| pos.moore_neighborhood_positions())
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

impl<T> fmt::Display for Game<T>
where
    T: Eq + Hash + Copy + PartialOrd + Zero + One + ToPrimitive,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.board().fmt(f)
    }
}
