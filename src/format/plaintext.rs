use anyhow::{bail, ensure, Result};
use std::fmt;

/// A representation for Plaintext file format, described in <https://conwaylife.com/wiki/Plaintext>.
#[derive(Debug)]
pub struct Plaintext {
    name: String,
    comment: Vec<String>,
    pattern: Vec<Vec<bool>>,
}

// Inherent methods

impl Plaintext {
    /// Creates from the specified string.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::Plaintext;
    /// let parser = Plaintext::new("\
    /// !Name: Glider\n\
    /// .O\n\
    /// ..O\n\
    /// OOO\n\
    /// ").unwrap();
    /// assert_eq!(parser.name(), "Glider");
    /// assert_eq!(parser.comment().len(), 0);
    /// assert_eq!(parser.pattern()[0], vec![false, true]);
    /// assert_eq!(parser.pattern()[1], vec![false, false, true]);
    /// assert_eq!(parser.pattern()[2], vec![true, true, true]);
    /// ```
    ///
    pub fn new(str: &str) -> Result<Self> {
        let mut lines = str.lines();
        let name: String = {
            let Some(line) = lines.next() else {
                bail!("No header line in the pattern");
            };
            let expected_prefix = "!Name: ";
            if line.len() < expected_prefix.len() {
                bail!("The header line is in wrong format");
            }
            let (prefix, body) = line.split_at(expected_prefix.len());
            if prefix != expected_prefix {
                bail!("The header line is in wrong format");
            }
            body.to_string()
        };
        let mut comment = Vec::new();
        let mut pattern = Vec::new();
        let mut comment_done = false;
        for line in lines {
            if !comment_done {
                if !line.is_empty() {
                    let (first, rest) = line.split_at(1);
                    if first == "!" {
                        comment.push(rest.to_string());
                        continue;
                    }
                } else {
                    ensure!(
                        comment.is_empty(),
                        "Invalid empty line found in the optional header"
                    );
                }
                comment_done = true;
            }
            let mut buf = Vec::new();
            for char in line.chars() {
                let value = match char {
                    '.' => false,
                    'O' => true,
                    _ => bail!("Invalid character found in the pattern"),
                };
                buf.push(value);
            }
            pattern.push(buf);
        }
        Ok(Self {
            name,
            comment,
            pattern,
        })
    }

    /// Returns the name of the pattern.
    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns the name of the pattern.
    #[inline]
    pub fn comment(&self) -> &Vec<String> {
        &self.comment
    }

    /// Returns the name of the pattern.
    #[inline]
    pub fn pattern(&self) -> &Vec<Vec<bool>> {
        &self.pattern
    }
}

impl fmt::Display for Plaintext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "!Name: {}", self.name())?;
        for line in self.comment() {
            writeln!(f, "!{}", line)?;
        }
        for line in self.pattern() {
            let str: String = line.iter().map(|&x| if x { 'O' } else { '.' }).collect();
            writeln!(f, "{str}")?;
        }
        Ok(())
    }
}
