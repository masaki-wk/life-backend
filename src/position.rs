use num_iter::range_inclusive;
use num_traits::{Bounded, One, ToPrimitive};
use std::fmt;
use std::hash::Hash;
use std::ops::{Add, Sub};

/// A position of a cell.
///
/// `Position<T>` is a tuple `(T, T)`. The first field is the x-coordinate value of the position and the second field is the y-coordinaate value of the potition.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position<T>(pub T, pub T);

impl<T> Position<T> {
    /// Creates an iterator over neighbour positions of the self, defined as [Moore neighbourhood](https://conwaylife.com/wiki/Moore_neighbourhood).
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::Position;
    /// use std::collections::HashSet;
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
    #[test]
    fn test_display() {
        let target = Position(1, 2);
        assert_eq!(format!("{target}"), "(1, 2)".to_string());
    }
}
