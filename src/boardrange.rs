use std::fmt;
use std::ops::RangeInclusive;

use crate::Position;

/// A range on a board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardRange<T>(RangeInclusive<T>, RangeInclusive<T>);

// Inherent methods

impl<T> BoardRange<T> {
    /// Creates a new `BoardRange`.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::BoardRange;
    /// let range = BoardRange::new(0..=3, 1..=2);
    /// assert_eq!(range.x(), &(0..=3));
    /// assert_eq!(range.y(), &(1..=2));
    /// ```
    ///
    #[inline]
    pub fn new(x: RangeInclusive<T>, y: RangeInclusive<T>) -> Self {
        Self(x, y)
    }

    /// Creates a new `BoardRange` from the iterator over series of positions to be contained.
    /// If the iterator contains any positions, `Some(BoardRange)` will be returned.
    /// Otherwise, `None` will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{BoardRange, Position};
    /// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    /// let range = BoardRange::new_from(positions.into_iter());
    /// assert_eq!(range, Some(BoardRange::new(0..=2, 0..=1)));
    /// ```
    ///
    pub fn new_from<U>(mut iter: U) -> Option<Self>
    where
        T: Copy + PartialOrd,
        U: Iterator<Item = Position<T>>,
    {
        if let Some(Position(init_x, init_y)) = iter.next() {
            Some(iter.fold(Self::new(init_x..=init_x, init_y..=init_y), |acc, Position(x, y)| {
                let x_start = *acc.x().start();
                let x_end = *acc.x().end();
                let y_start = *acc.y().start();
                let y_end = *acc.y().end();
                Self::new(
                    (if x_start < x { x_start } else { x })..=(if x_end > x { x_end } else { x }),
                    (if y_start < y { y_start } else { y })..=(if y_end > y { y_end } else { y }),
                )
            }))
        } else {
            None
        }
    }

    /// Returns the range on the x-coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::BoardRange;
    /// let range = BoardRange::new(0..=3, 1..=2);
    /// assert_eq!(range.x(), &(0..=3));
    /// ```
    ///
    #[inline]
    pub fn x(&self) -> &RangeInclusive<T> {
        &self.0
    }

    /// Returns the range on the y-coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::BoardRange;
    /// let range = BoardRange::new(0..=3, 1..=2);
    /// assert_eq!(range.y(), &(1..=2));
    /// ```
    ///
    #[inline]
    pub fn y(&self) -> &RangeInclusive<T> {
        &self.1
    }
}

// Trait implementations

impl<T> fmt::Display for BoardRange<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(x:[{}, {}] y:[{}, {}])", self.x().start(), self.x().end(), self.y().start(), self.y().end())?;
        Ok(())
    }
}
