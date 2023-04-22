use std::error::Error;
use std::fmt;
use std::result::Result;
use std::str::FromStr;

/// A representation of the rules of [Life-like cellular automatons](https://conwaylife.com/wiki/Life-like_cellular_automaton).
/// It only supports the birth/survival notation such as "B3/S23", see <https://conwaylife.com/wiki/Rulestring>.
#[derive(Debug)]
pub struct Rule {
    birth: [bool; 9],
    survival: [bool; 9],
}

// Inherent methods

impl Rule {
    /// Returns whether a new cell will be born from the specified number of alive neighbors.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Rule;
    /// let rule = Rule::conways_life();
    /// let b = [3];
    /// for i in 0..=8 {
    ///     assert_eq!(rule.is_born(i), b.iter().any(|&x| x == i));
    /// }
    /// ```
    ///
    pub fn is_born(&self, count: usize) -> bool {
        self.birth[count]
    }

    /// Returns whether the cell surrounded by a specified number of alive neighbors will survive.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::Rule;
    /// let rule = Rule::conways_life();
    /// let s = [2, 3];
    /// for i in 0..=8 {
    ///     assert_eq!(rule.is_survive(i), s.iter().any(|&x| x == i));
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
    ///     assert_eq!(rule.is_born(i), b.iter().any(|&x| x == i));
    ///     assert_eq!(rule.is_survive(i), s.iter().any(|&x| x == i));
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
    ///     assert_eq!(rule.is_born(i), b.iter().any(|&x| x == i));
    ///     assert_eq!(rule.is_survive(i), s.iter().any(|&x| x == i));
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
        fn convert_slice_to_string(slice: &[bool]) -> String {
            slice
                .iter()
                .enumerate()
                .filter_map(|(i, &x)| if x { Some(i) } else { None })
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("")
        }
        let s = ["B", &convert_slice_to_string(&self.birth), "/S", &convert_slice_to_string(&self.survival)].join("");
        f.write_str(&s)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ParseRuleError;

impl Error for ParseRuleError {}

impl fmt::Display for ParseRuleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("cannot parse rule from the string")
    }
}

impl FromStr for Rule {
    type Err = ParseRuleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.chars();
        if !matches!(iter.next(), Some('B')) {
            return Err(ParseRuleError);
        }
        let mut birth = [false; 9];
        loop {
            let Some(c) = iter.next() else { return Err(ParseRuleError); };
            if c == '/' {
                break;
            }
            let Some(n) = c.to_digit(10) else { return Err(ParseRuleError); };
            if n == 9 {
                return Err(ParseRuleError);
            }
            birth[n as usize] = true;
        }
        if !matches!(iter.next(), Some('S')) {
            return Err(ParseRuleError);
        }
        let mut survival = [false; 9];
        loop {
            let Some(c) = iter.next() else { break; };
            let Some(n) = c.to_digit(10) else { return Err(ParseRuleError); };
            if n == 9 {
                return Err(ParseRuleError);
            }
            survival[n as usize] = true;
        }
        Ok(Self { birth, survival })
    }
}

// Unit tests

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    fn check_value(target: &Rule, expected_birth: &[usize], expected_survival: &[usize]) {
        for i in 0..=8 {
            assert_eq!(target.is_born(i), expected_birth.iter().any(|&x| x == i));
            assert_eq!(target.is_survive(i), expected_survival.iter().any(|&x| x == i));
        }
    }
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
    #[test]
    fn test_from_str() -> Result<()> {
        let target: Rule = "B3/S23".parse()?;
        check_value(&target, &[3], &[2, 3]);
        Ok(())
    }
}
