use num_traits::{One, Zero};
use std::fmt;
use std::ops::RangeInclusive;

use crate::Position;

/// A range on a board.
///
/// This range consists of four pieces of information: the minimum and maximum x-coordinate values and the minimum and maximum y-coordinate values.
/// The type parameter `T` is used as the type of the x- and y-coordinate values.
///
/// # Examples
///
/// ```
/// use life_backend::{BoardRange, Position};
/// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
/// let range: BoardRange<_> = positions.iter().collect();
/// let min_x = range.x().start();
/// let max_x = range.x().end();
/// let min_y = range.y().start();
/// let max_y = range.y().end();
/// assert_eq!(min_x, &0);
/// assert_eq!(max_x, &2);
/// assert_eq!(min_y, &0);
/// assert_eq!(max_y, &1);
/// ```
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BoardRange<T>(RangeInclusive<T>, RangeInclusive<T>);

// Inherent methods

impl<T> BoardRange<T> {
    /// Creates an empty range.
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

    // Implementation of public extend().
    fn extend<U>(self, iter: U) -> Self
    where
        T: Copy + PartialOrd + Zero + One,
        U: Iterator<Item = Position<T>>,
    {
        iter.fold(self, |acc, Position(x, y)| {
            if acc.is_empty() {
                Self(x..=x, y..=y)
            } else {
                let (range_x, range_y) = acc.into_inner();
                let (x_start, x_end) = range_x.into_inner();
                let (y_start, y_end) = range_y.into_inner();
                Self(
                    (if x_start < x { x_start } else { x })..=(if x_end > x { x_end } else { x }),
                    (if y_start < y { y_start } else { y })..=(if y_end > y { y_end } else { y }),
                )
            }
        })
    }

    // Implementation of public from_iter().
    fn from_iter<U>(iter: U) -> Self
    where
        T: Copy + PartialOrd + Zero + One,
        U: Iterator<Item = Position<T>>,
    {
        Self::new().extend(iter)
    }

    /// Returns the range on the x-coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{BoardRange, Position};
    /// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    /// let range: BoardRange<_> = positions.iter().collect();
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
    /// let range: BoardRange<_> = positions.iter().collect();
    /// assert_eq!(range.y(), &(0..=1));
    /// ```
    ///
    #[inline]
    pub const fn y(&self) -> &RangeInclusive<T> {
        &self.1
    }

    /// Destructures [`BoardRange`] into (range-x, range-y).
    ///
    /// [`BoardRange`]: BoardRange
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{BoardRange, Position};
    /// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    /// let range: BoardRange<_> = positions.iter().collect();
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
    /// let range: BoardRange<_> = positions.iter().collect();
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
    ///
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
            write!(
                f,
                "(x:[{}, {}], y:[{}, {}])",
                self.x().start(),
                self.x().end(),
                self.y().start(),
                self.y().end()
            )?;
        }
        Ok(())
    }
}

impl<'a, T> FromIterator<&'a Position<T>> for BoardRange<T>
where
    T: Copy + PartialOrd + Zero + One + 'a,
{
    /// Creates a value from a non-owning iterator over a series of [`&Position<T>`].
    /// Each item in the series represents an immutable reference of a position to be contained to the range.
    ///
    /// [`&Position<T>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{BoardRange, Position};
    /// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    /// let range: BoardRange<_> = positions.iter().collect();
    /// assert!(!range.is_empty());
    /// assert_eq!(range.x(), &(0..=2));
    /// assert_eq!(range.y(), &(0..=1));
    /// ```
    ///
    #[inline]
    fn from_iter<U>(iter: U) -> Self
    where
        U: IntoIterator<Item = &'a Position<T>>,
    {
        Self::from_iter(iter.into_iter().copied())
    }
}

impl<T> FromIterator<Position<T>> for BoardRange<T>
where
    T: Copy + PartialOrd + Zero + One,
{
    /// Creates a value from an owning iterator over a series of [`Position<T>`].
    /// Each item in the series represents a moved position to be contained to the range.
    ///
    /// [`Position<T>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{BoardRange, Position};
    /// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    /// let range: BoardRange<_> = positions.into_iter().collect();
    /// assert!(!range.is_empty());
    /// assert_eq!(range.x(), &(0..=2));
    /// assert_eq!(range.y(), &(0..=1));
    /// ```
    ///
    #[inline]
    fn from_iter<U>(iter: U) -> Self
    where
        U: IntoIterator<Item = Position<T>>,
    {
        Self::from_iter(iter.into_iter())
    }
}

impl<'a, T> Extend<&'a Position<T>> for BoardRange<T>
where
    T: Copy + PartialOrd + Zero + One + 'a,
{
    /// Extends the range with the contents of the specified non-owning iterator over the series of [`&Position<T>`].
    /// Each item in the series represents an immutable reference of a position.
    ///
    /// [`&Position<T>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{BoardRange, Position};
    /// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    /// let mut range = BoardRange::new();
    /// range.extend(positions.iter());
    /// assert!(!range.is_empty());
    /// assert_eq!(range.x(), &(0..=2));
    /// assert_eq!(range.y(), &(0..=1));
    /// ```
    ///
    fn extend<U>(&mut self, iter: U)
    where
        U: IntoIterator<Item = &'a Position<T>>,
    {
        *self = self.clone().extend(iter.into_iter().copied())
    }
}

impl<T> Extend<Position<T>> for BoardRange<T>
where
    T: Copy + PartialOrd + Zero + One,
{
    /// Extends the range with the contents of the specified owning iterator over the series of [`Position<T>`].
    /// Each item in the series represents a moved position.
    ///
    /// [`Position<T>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{BoardRange, Position};
    /// let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    /// let mut range = BoardRange::new();
    /// range.extend(positions.into_iter());
    /// assert!(!range.is_empty());
    /// assert_eq!(range.x(), &(0..=2));
    /// assert_eq!(range.y(), &(0..=1));
    /// ```
    ///
    fn extend<U>(&mut self, iter: U)
    where
        U: IntoIterator<Item = Position<T>>,
    {
        *self = self.clone().extend(iter.into_iter())
    }
}

// Unit tests

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default() {
        let target = BoardRange::<i32>::default();
        let expected = BoardRange::<i32>::new();
        assert_eq!(target, expected);
    }
    #[test]
    fn display_empty() {
        let target = BoardRange::<i32>::new();
        assert_eq!(format!("{target}"), "(empty)".to_string());
    }
    #[test]
    fn display_notempty() {
        let positions = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
        let target: BoardRange<_> = positions.iter().collect();
        assert_eq!(format!("{target}"), "(x:[0, 2], y:[0, 1])".to_string());
    }
}
