use num_traits::{One, Zero};
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
    pub const fn new(x: RangeInclusive<T>, y: RangeInclusive<T>) -> Self {
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
    /// assert_eq!(range, BoardRange::new(0..=2, 0..=1));
    /// ```
    ///
    pub fn new_from<U>(mut iter: U) -> Self
    where
        T: Copy + PartialOrd + Zero + One,
        U: Iterator<Item = Position<T>>,
    {
        if let Some(Position(init_x, init_y)) = iter.next() {
            iter.fold(Self::new(init_x..=init_x, init_y..=init_y), |acc, Position(x, y)| {
                let x_start = *acc.x().start();
                let x_end = *acc.x().end();
                let y_start = *acc.y().start();
                let y_end = *acc.y().end();
                Self::new(
                    (if x_start < x { x_start } else { x })..=(if x_end > x { x_end } else { x }),
                    (if y_start < y { y_start } else { y })..=(if y_end > y { y_end } else { y }),
                )
            })
        } else {
            Self::new(T::one()..=T::zero(), T::one()..=T::zero())
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
    pub const fn x(&self) -> &RangeInclusive<T> {
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
    pub const fn y(&self) -> &RangeInclusive<T> {
        &self.1
    }

    /// Returns `true` if the range contains no area.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::BoardRange;
    /// let range = BoardRange::<i16>::new_from([].into_iter());
    /// assert!(range.is_empty());
    /// ```
    ///
    pub fn is_empty(&self) -> bool
    where
        T: PartialOrd,
    {
        self.x().is_empty() || self.y().is_empty()
    }
}

// Trait implementations

impl<T> fmt::Display for BoardRange<T>
where
    T: PartialOrd + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            write!(f, "(empty)")?;
        } else {
            write!(f, "(x:[{}, {}] y:[{}, {}])", self.x().start(), self.x().end(), self.y().start(), self.y().end())?;
        }
        Ok(())
    }
}
