use anyhow::Result;
use std::fmt;
use std::io::Read;
use std::str::FromStr;

use super::{PlaintextLine, PlaintextParser};
use crate::{Format, Rule};

/// A representation for Plaintext file format.
///
/// The detail of this format is described in:
///
/// - [Plaintext - LifeWiki](https://conwaylife.com/wiki/Plaintext)
///
/// # Examples
///
/// Parses the given Plaintext file, and checks live cells included in it:
///
/// ```
/// use std::fs::File;
/// use life_backend::format::Plaintext;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let file = File::open("patterns/rpentomino.cells")?;
/// let parser = Plaintext::new(file)?;
/// assert!(parser.live_cells().eq([(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)]));
/// # Ok(())
/// # }
/// ```
///
/// Parses the given string in Plaintext format:
///
/// ```
/// use life_backend::format::Plaintext;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pattern = "\
///     !Name: R-pentomino\n\
///     .OO\n\
///     OO.\n\
///     .O.\n\
/// ";
/// let parser = pattern.parse::<Plaintext>()?;
/// assert!(parser.live_cells().eq([(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)]));
/// # Ok(())
/// # }
/// ```
///
#[derive(Clone, Debug)]
pub struct Plaintext {
    pub(super) name: Option<String>,
    pub(super) comments: Vec<String>,
    pub(super) contents: Vec<PlaintextLine>,
}

// Inherent methods

impl Plaintext {
    /// Creates from the specified implementor of [`Read`], such as [`File`] or `&[u8]`.
    ///
    /// [`Read`]: std::io::Read
    /// [`File`]: std::fs::File
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::Plaintext;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = "\
    ///     !Name: T-tetromino\n\
    ///     OOO\n\
    ///     .O.\n\
    /// ";
    /// let parser = Plaintext::new(pattern.as_bytes())?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    #[inline]
    pub fn new<R>(read: R) -> Result<Self>
    where
        R: Read,
    {
        PlaintextParser::parse(read)
    }

    /// Returns the name of the pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::Plaintext;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = "\
    ///     !Name: T-tetromino\n\
    ///     OOO\n\
    ///     .O.\n\
    /// ";
    /// let parser = Plaintext::new(pattern.as_bytes())?;
    /// assert_eq!(parser.name(), Some("T-tetromino".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    /// Returns comments of the pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::Plaintext;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = "\
    ///     !Name: T-tetromino\n\
    ///     !comment0\n\
    ///     !comment1\n\
    ///     OOO\n\
    ///     .O.\n\
    /// ";
    /// let parser = Plaintext::new(pattern.as_bytes())?;
    /// assert_eq!(parser.comments().len(), 2);
    /// assert_eq!(parser.comments()[0], "comment0");
    /// assert_eq!(parser.comments()[1], "comment1");
    /// # Ok(())
    /// # }
    /// ```
    ///
    #[inline]
    pub const fn comments(&self) -> &Vec<String> {
        &self.comments
    }

    /// Creates an owning iterator over the series of live cell positions in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::Plaintext;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = "\
    ///     !Name: T-tetromino\n\
    ///     OOO\n\
    ///     .O.\n\
    /// ";
    /// let parser = Plaintext::new(pattern.as_bytes())?;
    /// assert!(parser.live_cells().eq([(0, 0), (1, 0), (2, 0), (1, 1)]));
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn live_cells(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.contents.iter().flat_map(|PlaintextLine(y, xs)| xs.iter().map(|x| (*x, *y)))
    }
}

// Trait implementations

impl Format for Plaintext {
    fn rule(&self) -> Rule {
        Rule::conways_life()
    }
    fn live_cells(&self) -> Box<dyn Iterator<Item = (usize, usize)> + '_> {
        Box::new(self.live_cells())
    }
}

impl fmt::Display for Plaintext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = self.name() {
            writeln!(f, "!Name: {name}")?;
        }
        for line in self.comments() {
            writeln!(f, "!{line}")?;
        }
        if !self.contents.is_empty() {
            let max_x = self.contents.iter().flat_map(|PlaintextLine(_, xs)| xs.iter()).copied().max().unwrap(); // this unwrap() never panic because flat_map() always returns at least one value under !self.contents.is_empty()
            let dead_cell_chars = ".".repeat(max_x) + "."; // this code avoids `".".repeat(max_x + 1)` because `max_x + 1` overflows if max_x == usize::MAX
            let mut prev_y = 0;
            for PlaintextLine(curr_y, xs) in &self.contents {
                for _ in prev_y..(*curr_y) {
                    writeln!(f, "{dead_cell_chars}")?;
                }
                let line = {
                    let capacity = if max_x < usize::MAX { max_x + 1 } else { max_x };
                    let (mut buf, prev_x) = xs.iter().fold((String::with_capacity(capacity), 0), |(mut buf, prev_x), &curr_x| {
                        buf += &dead_cell_chars[0..(curr_x - prev_x)];
                        buf += "O";
                        (buf, curr_x + 1)
                    });
                    if prev_x <= max_x {
                        buf += &dead_cell_chars[0..(max_x - prev_x + 1)]; // `!xs.is_empty()` is guaranteed by the structure of Plaintext, so `prev_x > 0` is also guaranteed. Thus `max_x - prev_x + 1` never overflow
                    }
                    buf
                };
                writeln!(f, "{line}")?;
                prev_y = curr_y + 1;
            }
        }
        Ok(())
    }
}

impl FromStr for Plaintext {
    type Err = anyhow::Error;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.as_bytes())
    }
}
