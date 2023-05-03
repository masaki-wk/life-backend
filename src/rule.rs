use std::error::Error;
use std::fmt;
use std::result::Result;
use std::str::FromStr;

/// A representation of the rules of [Life-like cellular automatons](https://conwaylife.com/wiki/Life-like_cellular_automaton).
///
/// # Examples
///
/// ```
/// # use life_backend::Rule;
/// let rule = "B3/S23".parse::<Rule>().unwrap();
/// for i in 0..=8 {
///     assert_eq!(rule.is_born(i), [3].iter().any(|&x| x == i));
///     assert_eq!(rule.is_survive(i), [2, 3].iter().any(|&x| x == i));
/// }
/// assert_eq!(format!("{rule}"), "B3/S23");
/// ```
///
/// Converting from a Rule value into a String value via `format!("{}", ...)` only supports the birth/survival notation. (ex. "B3/S23")
///
/// Parsing from a string slice into a Rule value via `"...".parse::<Rule>()` supports the following notation, see [Rulestring](https://conwaylife.com/wiki/Rulestring).
///
/// - The birth/survival notation (ex. "B3/S23")
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    birth: [bool; 9],
    survival: [bool; 9],
}

// Inherent methods

impl Rule {
    /// Returns whether a new cell will be born from the specified number of alive neighbors.
    ///
    /// # Panics
    ///
    /// Panics if the number is greater than 8.
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
    #[inline]
    pub fn is_born(&self, count: usize) -> bool {
        self.birth[count]
    }

    /// Returns whether a cell surrounded by the specified number of alive neighbors will survive.
    ///
    /// # Panics
    ///
    /// Panics if the number is greater than 8.
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
    #[inline]
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

// Trait implementations

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn convert_slice_to_string(slice: &[bool]) -> String {
            slice
                .iter()
                .enumerate()
                .filter_map(|(i, &x)| if x { Some(i) } else { None })
                .map(|n| n.to_string())
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
        fn convert_numbers_to_slice(numbers: &str) -> Option<[bool; 9]> {
            let mut buf = [false; 9];
            for c in numbers.chars() {
                let n = c.to_digit(9)?;
                buf[n as usize] = true;
            }
            Some(buf)
        }
        let fields_splitted: Vec<_> = s.split('/').collect();
        if fields_splitted.len() != 2 {
            return Err(ParseRuleError);
        }
        let (labels, numbers): (Vec<_>, Vec<_>) = fields_splitted
            .iter()
            .map(|s| s.split_at(s.find(|c: char| c.is_ascii_digit()).unwrap_or(s.len())))
            .unzip();
        let numbers = if labels.iter().eq(["B", "S"].iter()) {
            numbers
        } else {
            return Err(ParseRuleError);
        };
        let Some(slices) = numbers
            .into_iter()
            .map(convert_numbers_to_slice)
            .collect::<Option<Vec<_>>>() else {
            return Err(ParseRuleError);
        };
        Ok(Self {
            birth: slices[0],
            survival: slices[1],
        })
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
    fn test_from_str_birth_survival_notation() -> Result<()> {
        let target: Rule = "B3/S23".parse()?;
        check_value(&target, &[3], &[2, 3]);
        Ok(())
    }
    #[test]
    fn test_from_str_birth_survival_notation_without_birth_number() -> Result<()> {
        let target: Rule = "B/S23".parse()?;
        check_value(&target, &[], &[2, 3]);
        Ok(())
    }
    #[test]
    fn test_from_str_birth_survival_notation_without_survival_number() -> Result<()> {
        let target: Rule = "B3/S".parse()?;
        check_value(&target, &[3], &[]);
        Ok(())
    }
    #[test]
    fn test_from_str_no_separator() {
        let target = "B0S0".parse::<Rule>();
        assert!(target.is_err());
    }
    #[test]
    fn test_from_str_too_many_separators() {
        let target = "B0/S0/C0".parse::<Rule>();
        assert!(target.is_err());
    }
    #[test]
    fn test_from_str_no_label_birth() {
        let target = "0/S0".parse::<Rule>();
        assert!(target.is_err());
    }
    #[test]
    fn test_from_str_no_label_survival() {
        let target = "B0/0".parse::<Rule>();
        assert!(target.is_err());
    }
    #[test]
    fn test_from_str_birth_survival_notation_too_large_number() {
        let target = "B9/S0".parse::<Rule>();
        assert!(target.is_err());
    }
}
