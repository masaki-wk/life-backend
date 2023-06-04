use num_iter::range_inclusive;
use num_traits::{Bounded, One, ToPrimitive};
use std::fmt;
use std::hash::Hash;
use std::ops::{Add, Sub};

/// A position of a cell.
///
/// `Position<T>` is a tuple `(T, T)`.
/// The first field is the x-coordinate value of the position and the second field is the y-coordinate value of the potition.
/// The type parameter `T` is used as the type of the x- and y-coordinate values of positions.
///
/// # Examples
///
/// ```
/// use life_backend::Position;
/// let pos = Position(2, 3);
/// let pos_x = pos.0;
/// let pos_y = pos.1;
/// assert_eq!(pos_x, 2);
/// assert_eq!(pos_y, 3);
/// ```
///
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Position<T>(pub T, pub T);

impl<T> Position<T> {
    /// Converts from `Position<U>` to `Position<T>`.
    ///
    /// This operation converts the type of the x- and y-coordinate values of the position from `U` to `T`.
    /// If an error occurs in converting from `U` to `T`, returns that error.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let base: Position<usize> = Position(0, 0);
    /// let pos = Position::<i16>::try_from(base)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn try_from<U>(value: Position<U>) -> Result<Position<T>, T::Error>
    where
        T: TryFrom<U>,
    {
        Ok(Position(T::try_from(value.0)?, T::try_from(value.1)?))
    }

    /// Converts from `Position<T>` to `Position<U>`.
    ///
    /// `base.try_into::<U>()` is the same as `Position::<U>::try_from(base)`, see [`try_from()`].
    ///
    /// [`try_from()`]: #method.try_from
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let base: Position<usize> = Position(0, 0);
    /// let pos: Position<i16> = base.try_into()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    #[inline]
    pub fn try_into<U>(self) -> Result<Position<U>, U::Error>
    where
        U: TryFrom<T>,
    {
        Position::<U>::try_from(self)
    }

    /// Creates an owning iterator over neighbour positions of the self position in arbitrary order.
    /// The neighbour positions are defined in [Moore neighbourhood](https://conwaylife.com/wiki/Moore_neighbourhood).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use life_backend::Position;
    /// let pos = Position(2, 3);
    /// let result: HashSet<_> = pos
    ///     .moore_neighborhood_positions()
    ///     .collect();
    /// let expected: HashSet<_> = [(1, 2), (2, 2), (3, 2), (1, 3), (3, 3), (1, 4), (2, 4), (3, 4)]
    ///     .into_iter()
    ///     .map(|(x, y)| Position(x, y))
    ///     .collect();
    /// assert_eq!(result, expected);
    /// ```
    ///
    pub fn moore_neighborhood_positions(&self) -> impl Iterator<Item = Self>
    where
        T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T> + One + Bounded + ToPrimitive,
    {
        let Position(x, y) = *self;
        let min = T::min_value();
        let max = T::max_value();
        let one = T::one();
        let x_start = if x > min { x - one } else { x };
        let x_stop = if x < max { x + one } else { x };
        let y_start = if y > min { y - one } else { y };
        let y_stop = if y < max { y + one } else { y };
        range_inclusive(y_start, y_stop)
            .flat_map(move |v| range_inclusive(x_start, x_stop).map(move |u| Position(u, v)))
            .filter(move |&pos| pos != Position(x, y))
    }
}

impl<T> fmt::Display for Position<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use i32 as I;
    use std::collections::HashSet;
    #[test]
    fn display() {
        let target = Position(1, 2);
        assert_eq!(format!("{target}"), "(1, 2)".to_string());
    }
    #[test]
    fn try_from_infallible() {
        let base: Position<i8> = Position(0, 0);
        let target = Position::<i16>::try_from(base);
        assert!(target.is_ok());
    }
    #[test]
    fn try_from_fallible_no_error() {
        let base: Position<i16> = Position(0, 0);
        let target = Position::<i8>::try_from(base);
        assert!(target.is_ok());
    }
    #[test]
    fn try_from_fallible_with_error() {
        let base: Position<i8> = Position(-1, 0);
        let target = Position::<u8>::try_from(base);
        assert!(target.is_err());
    }
    #[test]
    fn try_into_infallible() {
        let base: Position<i8> = Position(0, 0);
        let target = base.try_into::<i16>();
        assert!(target.is_ok());
    }
    #[test]
    fn try_into_fallible_no_error() {
        let base: Position<i16> = Position(0, 0);
        let target = base.try_into::<i8>();
        assert!(target.is_ok());
    }
    #[test]
    fn try_into_fallible_with_error() {
        let base: Position<i8> = Position(-1, 0);
        let target = base.try_into::<u8>();
        assert!(target.is_err());
    }
    #[test]
    fn moore_neighborhood_positions_basic() {
        let target: Position<I> = Position(0, 0);
        let result: HashSet<_> = target.moore_neighborhood_positions().collect();
        assert_eq!(
            result,
            [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)]
                .into_iter()
                .map(|(x, y)| Position(x, y))
                .collect::<HashSet<_>>()
        );
    }
    #[test]
    fn moore_neighborhood_positions_bounds() {
        let min = I::min_value();
        let max = I::max_value();
        let zero: I = 0;
        for (pos_tuple, expected_count) in [
            ((min, min), 3),
            ((min, zero), 5),
            ((min, max), 3),
            ((zero, min), 5),
            ((zero, zero), 8),
            ((zero, max), 5),
            ((max, min), 3),
            ((max, zero), 5),
            ((max, max), 3),
        ] {
            let pos = Position(pos_tuple.0, pos_tuple.1);
            assert_eq!(pos.moore_neighborhood_positions().count(), expected_count);
        }
    }
}
