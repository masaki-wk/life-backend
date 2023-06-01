use std::error::Error;
use std::fmt;
use std::str::FromStr;

const TRUTH_TABLE_SIZE: usize = 9;

/// A representation of a rule of [Life-like cellular automaton](https://conwaylife.com/wiki/Life-like_cellular_automaton).
///
/// The following operations are supported:
///
/// - Constructing from a pair of truth tables
/// - Parsing a string into a value of this type, ex. `"B3/S23"`.
///   The following notations are supported, see [Rulestring](https://conwaylife.com/wiki/Rulestring):
///   - The birth/survival notation (ex. `"B3/S23"`). Lowercase `'b'` or `'s'` are also allowed in the notation instead of `'B'` or `'S'`
///   - S/B notation (ex. `"23/3"`)
/// - Determining whether a new cell will be born from the specified number of alive neighbors
/// - Determining whether a cell surrounded by the specified number of alive neighbors will survive
/// - Converting into a [`String`] value, ex. `"B3/S23"`.
///   This operation only supports the birth/survival notation
///
/// [`String`]: std::string::String
///
/// # Examples
///
/// ```
/// use life_backend::Rule;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let rule = "B3/S23".parse::<Rule>()?;
/// for i in 0..=8 {
///     assert_eq!(rule.is_born(i), [3].iter().any(|&x| x == i));
///     assert_eq!(rule.is_survive(i), [2, 3].iter().any(|&x| x == i));
/// }
/// assert_eq!(format!("{rule}"), "B3/S23");
/// # Ok(())
/// # }
/// ```
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Rule {
    birth: [bool; TRUTH_TABLE_SIZE],
    survival: [bool; TRUTH_TABLE_SIZE],
}

// Inherent methods

impl Rule {
    /// Creates a new rule based on the specified pair of truth tables.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::Rule;
    /// let rule = Rule::new(
    ///     &[false, false, false, true, false, false, false, false, false],
    ///     &[false, false, true, true, false, false, false, false, false],
    /// );
    /// let b = [3];
    /// let s = [2, 3];
    /// for i in 0..=8 {
    ///     assert_eq!(rule.is_born(i), b.iter().any(|&x| x == i));
    ///     assert_eq!(rule.is_survive(i), s.iter().any(|&x| x == i));
    /// }
    /// ```
    ///
    pub const fn new(birth: &[bool; 9], survival: &[bool; 9]) -> Self {
        Self {
            birth: *birth,
            survival: *survival,
        }
    }

    /// Returns whether a new cell will be born from the specified number of alive neighbors.
    ///
    /// # Panics
    ///
    /// Panics if the argument `count` is greater than 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::Rule;
    /// let rule = Rule::conways_life();
    /// let b = [3];
    /// for i in 0..=8 {
    ///     assert_eq!(rule.is_born(i), b.iter().any(|&x| x == i));
    /// }
    /// ```
    ///
    #[inline]
    pub const fn is_born(&self, count: usize) -> bool {
        self.birth[count]
    }

    /// Returns whether a cell surrounded by the specified number of alive neighbors will survive.
    ///
    /// # Panics
    ///
    /// Panics if the argument `count` is greater than 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::Rule;
    /// let rule = Rule::conways_life();
    /// let s = [2, 3];
    /// for i in 0..=8 {
    ///     assert_eq!(rule.is_survive(i), s.iter().any(|&x| x == i));
    /// }
    /// ```
    ///
    #[inline]
    pub const fn is_survive(&self, count: usize) -> bool {
        self.survival[count]
    }

    /// Returns the rule of [Conway's Game of Life](https://conwaylife.com/wiki/Conway%27s_Game_of_Life).
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::Rule;
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
        Self::new(
            &[false, false, false, true, false, false, false, false, false],
            &[false, false, true, true, false, false, false, false, false],
        )
    }
}

// Trait implementations

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn count_slice_numbers(slice: &[bool]) -> usize {
            slice.iter().filter(|x| **x).count()
        }
        fn convert_slice_to_string(slice: &[bool]) -> String {
            slice
                .iter()
                .enumerate()
                .filter_map(|(i, &x)| if x { Some(i) } else { None })
                .map(|n| char::from_digit(n as u32, TRUTH_TABLE_SIZE as u32).unwrap()) // this unwrap never panic because `n < TRUTH_TABLE_SIZE` is always guaranteed
                .collect()
        }
        let mut buf = String::with_capacity(count_slice_numbers(&self.birth) + count_slice_numbers(&self.survival));
        buf += "B";
        buf += &convert_slice_to_string(&self.birth);
        buf += "/S";
        buf += &convert_slice_to_string(&self.survival);
        f.write_str(&buf)?;
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
        fn convert_numbers_to_slice(numbers: &str) -> Option<[bool; TRUTH_TABLE_SIZE]> {
            numbers.chars().try_fold([false; TRUTH_TABLE_SIZE], |mut buf, c| {
                let n = c.to_digit(TRUTH_TABLE_SIZE as u32)? as usize;
                buf[n] = true;
                Some(buf)
            })
        }
        let fields_splitted: Vec<_> = s.split('/').collect();
        if fields_splitted.len() != 2 {
            return Err(ParseRuleError);
        }
        let (labels, numbers): (Vec<_>, Vec<_>) = fields_splitted
            .iter()
            .map(|s| s.split_at(s.find(|c: char| c.is_ascii_digit()).unwrap_or(s.len())))
            .unzip();
        let numbers = if labels.iter().zip(["B", "S"]).all(|(lhs, rhs)| lhs.eq_ignore_ascii_case(rhs)) {
            // the birth/survival notation, ex. "B3/S23"
            numbers
        } else if labels.iter().all(|s| s.is_empty()) {
            // S/B notation, ex. "23/3"
            vec![numbers[1], numbers[0]]
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
    const RULE_HIGHLIFE: Rule = Rule::new(
        &[false, false, false, true, false, false, true, false, false],
        &[false, false, true, true, false, false, false, false, false],
    );
    fn check_value(target: &Rule, expected_birth: &[usize], expected_survival: &[usize]) {
        for i in 0..=8 {
            assert_eq!(target.is_born(i), expected_birth.iter().any(|&x| x == i));
            assert_eq!(target.is_survive(i), expected_survival.iter().any(|&x| x == i));
        }
    }
    #[test]
    fn new_conways_life() {
        let target = Rule::new(
            &[false, false, false, true, false, false, false, false, false],
            &[false, false, true, true, false, false, false, false, false],
        );
        check_value(&target, &[3], &[2, 3]);
    }
    #[test]
    fn new_highlife() {
        let target = Rule::new(
            &[false, false, false, true, false, false, true, false, false],
            &[false, false, true, true, false, false, false, false, false],
        );
        check_value(&target, &[3, 6], &[2, 3]);
    }
    #[test]
    fn conways_life() {
        let target = Rule::conways_life();
        check_value(&target, &[3], &[2, 3]);
    }
    #[test]
    fn display_conways_life() {
        let target = Rule::conways_life();
        assert_eq!(target.to_string(), "B3/S23");
    }
    #[test]
    fn display_highlife() {
        let target = RULE_HIGHLIFE;
        assert_eq!(target.to_string(), "B36/S23");
    }
    #[test]
    fn from_str_birth_survival_notation() -> Result<()> {
        let target: Rule = "B3/S23".parse()?;
        check_value(&target, &[3], &[2, 3]);
        Ok(())
    }
    #[test]
    fn from_str_s_b_notation() -> Result<()> {
        let target: Rule = "23/3".parse()?;
        check_value(&target, &[3], &[2, 3]);
        Ok(())
    }
    #[test]
    fn from_str_birth_survival_notation_without_birth_number() -> Result<()> {
        let target: Rule = "B/S23".parse()?;
        check_value(&target, &[], &[2, 3]);
        Ok(())
    }
    #[test]
    fn from_str_birth_survival_notation_without_survival_number() -> Result<()> {
        let target: Rule = "B3/S".parse()?;
        check_value(&target, &[3], &[]);
        Ok(())
    }
    #[test]
    fn from_str_birth_survival_notation_lowercase_b() -> Result<()> {
        let target: Rule = "b3/S23".parse()?;
        check_value(&target, &[3], &[2, 3]);
        Ok(())
    }
    #[test]
    fn from_str_birth_survival_notation_lowercase_s() -> Result<()> {
        let target: Rule = "B3/s23".parse()?;
        check_value(&target, &[3], &[2, 3]);
        Ok(())
    }
    #[test]
    fn from_str_no_separator() {
        let target = "B0S0".parse::<Rule>();
        assert!(target.is_err());
    }
    #[test]
    fn from_str_too_many_separators() {
        let target = "B0/S0/C0".parse::<Rule>();
        assert!(target.is_err());
    }
    #[test]
    fn from_str_no_label_birth() {
        let target = "0/S0".parse::<Rule>();
        assert!(target.is_err());
    }
    #[test]
    fn from_str_no_label_survival() {
        let target = "B0/0".parse::<Rule>();
        assert!(target.is_err());
    }
    #[test]
    fn from_str_birth_survival_notation_too_large_number() {
        let target = "B9/S0".parse::<Rule>();
        assert!(target.is_err());
    }
}
