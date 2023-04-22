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
