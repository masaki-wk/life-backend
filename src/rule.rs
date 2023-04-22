use std::fmt;

/// A representation of the rules of [Life-like cellular automatons](https://conwaylife.com/wiki/Life-like_cellular_automaton).
#[derive(Debug)]
pub struct Rule {
    birth: [bool; 9],
    survival: [bool; 9],
}

// Inherent methods

impl Rule {
    /// Returns whether the cell will be born from the specified number of alive neighbors.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Rule;
    /// let rule = Rule::conways_life();
    /// let b = [3];
    /// for i in 0..=8 {
    ///     assert_eq!(rule.is_born(i), b.iter().find(|&&x| x == i).is_some());
    /// }
    /// ```
    ///
    pub fn is_born(&self, count: usize) -> bool {
        self.birth[count]
    }

    /// Returns whether the cell will be survive from the specified number of alive neighbors.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Rule;
    /// let rule = Rule::conways_life();
    /// let s = [2, 3];
    /// for i in 0..=8 {
    ///     assert_eq!(rule.is_survive(i), s.iter().find(|&&x| x == i).is_some());
    /// }
    /// ```
    ///
    pub fn is_survive(&self, count: usize) -> bool {
        self.survival[count]
    }

    /// Returns the rule of [Conway's Game of Life](https://conwaylife.com/wiki/Conway%27s_Game_of_Life).
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Rule;
    /// let rule = Rule::conways_life();
    /// let b = [3];
    /// let s = [2, 3];
    /// for i in 0..=8 {
    ///     assert_eq!(rule.is_born(i), b.iter().find(|&&x| x == i).is_some());
    ///     assert_eq!(rule.is_survive(i), s.iter().find(|&&x| x == i).is_some());
    /// }
    /// ```
    ///
    pub const fn conways_life() -> Self {
        Self {
            birth: [false, false, false, true, false, false, false, false, false],
            survival: [false, false, true, true, false, false, false, false, false],
        }
    }

    /// Returns the rule of [HighLife](https://conwaylife.com/wiki/OCA:HighLife).
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Rule;
    /// let rule = Rule::highlife();
    /// let b = [3, 6];
    /// let s = [2, 3];
    /// for i in 0..=8 {
    ///     assert_eq!(rule.is_born(i), b.iter().find(|&&x| x == i).is_some());
    ///     assert_eq!(rule.is_survive(i), s.iter().find(|&&x| x == i).is_some());
    /// }
    /// ```
    ///
    pub const fn highlife() -> Self {
        Self {
            birth: [false, false, false, true, false, false, true, false, false],
            survival: [false, false, true, true, false, false, false, false, false],
        }
    }
}

// Trait implementations of Rule

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let convert_slice_to_string = |slice: &[bool]| {
            slice
                .iter()
                .enumerate()
                .filter_map(|(i, &x)| if x { Some(i) } else { None })
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("")
        };
        let mut buf = String::new();
        buf += "B";
        buf += &convert_slice_to_string(&self.birth);
        buf += "/S";
        buf += &convert_slice_to_string(&self.survival);
        write!(f, "{buf}")?;
        Ok(())
    }
}

// Unit tests

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_display_conways_life() {
        let target = Rule::conways_life();
        assert_eq!(target.to_string(), "B3/S23");
    }
    #[test]
    fn test_display_highlife() {
        let target = Rule::highlife();
        assert_eq!(target.to_string(), "B36/S23");
    }
}
