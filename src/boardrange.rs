use num_traits::{One, Zero};
use std::fmt;
use std::ops::RangeInclusive;

use crate::Position;

/// A range on a board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardRange<T>(RangeInclusive<T>, RangeInclusive<T>);

// Inherent methods

impl<T> BoardRange<T> {
    /// Creates an empty `BoardRange`.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::BoardRange;
    /// let range = BoardRange::<i32>::new();
    /// assert!(range.is_empty());
    /// ```
    ///
    pub fn new() -> Self
    where
        T: Zero + One,
    {
        Self(T::one()..=T::zero(), T::one()..=T::zero())
    }

    /// Creates a new `BoardRange` from the iterator over series of positions to be contained.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{BoardRange, Position};
    /// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    /// let range = BoardRange::new_from(positions.into_iter());
    /// assert!(!range.is_empty());
    /// assert_eq!(range.x(), &(0..=2));
    /// assert_eq!(range.y(), &(0..=1));
    /// ```
    ///
    pub fn new_from<U>(mut iter: U) -> Self
    where
        T: Copy + PartialOrd + Zero + One,
        U: Iterator<Item = Position<T>>,
    {
        if let Some(Position(init_x, init_y)) = iter.next() {
            iter.fold(Self(init_x..=init_x, init_y..=init_y), |acc, Position(x, y)| {
                let (range_x, range_y) = acc.into_inner();
                let (x_start, x_end) = range_x.into_inner();
                let (y_start, y_end) = range_y.into_inner();
                Self(
                    (if x_start < x { x_start } else { x })..=(if x_end > x { x_end } else { x }),
                    (if y_start < y { y_start } else { y })..=(if y_end > y { y_end } else { y }),
                )
            })
        } else {
            Self::new()
        }
    }

    /// Returns the range on the x-coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{BoardRange, Position};
    /// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    /// let range = BoardRange::new_from(positions.into_iter());
    /// assert_eq!(range.x(), &(0..=2));
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
    /// use life_backend::{BoardRange, Position};
    /// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    /// let range = BoardRange::new_from(positions.into_iter());
    /// assert_eq!(range.y(), &(0..=1));
    /// ```
    ///
    #[inline]
    pub const fn y(&self) -> &RangeInclusive<T> {
        &self.1
    }

    /// Destructures the `BoardRange` into (range-x, range-y).
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{BoardRange, Position};
    /// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    /// let range = BoardRange::new_from(positions.into_iter());
    /// let (range_x, range_y) = range.into_inner();
    /// assert_eq!(range_x, 0..=2);
    /// assert_eq!(range_y, 0..=1);
    /// ```
    ///
    #[inline]
    pub fn into_inner(self) -> (RangeInclusive<T>, RangeInclusive<T>) {
        (self.0, self.1)
    }

    /// Returns `true` if the range contains no area.
    ///
    /// If the range is empty, return values of methods are defined as the following:
    ///
    /// - `range.is_empty()` is `true`
    /// - `range.x().is_empty()` and `range.y().is_empty()` are `true`
    /// - `range.x().start()`, `range.x().end()`, `range.y().start()` and `range.y().end()` are unspecified
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{BoardRange, Position};
    /// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    /// let range = BoardRange::new_from(positions.into_iter());
    /// assert!(!range.is_empty());
    /// ```
    ///
    #[inline]
    pub fn is_empty(&self) -> bool
    where
        T: PartialOrd,
    {
        self.x().is_empty()
    }
}

// Trait implementations

impl<T> Default for BoardRange<T>
where
    T: Zero + One,
{
    /// Returns the default value of the type, same as the return value of [`new()`].
    ///
    /// [`new()`]: #method.new
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

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
