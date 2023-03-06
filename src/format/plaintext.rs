use anyhow::{bail, Result};
use std::fmt;

/// A representation for Plaintext file format, described in <https://conwaylife.com/wiki/Plaintext>.
#[derive(Debug)]
pub struct Plaintext {
    name: String,
    comments: Vec<String>,
    contents: Vec<Vec<bool>>,
}

// An internal struct, used during constructing of Plaintext
struct PlaintextPartial {
    name: Option<String>,
    comments: Vec<String>,
    contents: Vec<Vec<bool>>,
}

// Inherent methods of PlaintextPartial

impl PlaintextPartial {
    fn parse_prefixed_line<'a>(prefix: &str, line: &'a str) -> Option<&'a str> {
        if line.len() < prefix.len() {
            None
        } else {
            let (first, last) = line.split_at(prefix.len());
            if first == prefix {
                Some(last)
            } else {
                None
            }
        }
    }
    #[inline]
    fn parse_name_line(line: &str) -> Option<&str> {
        Self::parse_prefixed_line("!Name: ", line)
    }
    #[inline]
    fn parse_comment_line(line: &str) -> Option<&str> {
        Self::parse_prefixed_line("!", line)
    }
    fn parse_content_line(line: &str) -> Option<Vec<bool>> {
        let mut buf = Vec::new();
        for char in line.chars() {
            let value = match char {
                '.' => false,
                'O' => true,
                _ => return None,
            };
            buf.push(value);
        }
        Some(buf)
    }
    fn new() -> Self {
        Self {
            name: None,
            comments: Vec::new(),
            contents: Vec::new(),
        }
    }
    fn push(&mut self, line: &str) -> Result<()> {
        if self.name.is_none() {
            let Some(name) = Self::parse_name_line(line) else {
                bail!("The header line is in wrong format");
            };
            self.name = Some(name.to_string());
        } else {
            if self.contents.is_empty() {
                if let Some(comment) = Self::parse_comment_line(line) {
                    self.comments.push(comment.to_string());
                    return Ok(());
                }
            }
            let Some(content) = Self::parse_content_line(line) else {
                bail!("Invalid character found in the pattern");
            };
            self.contents.push(content);
        }
        Ok(())
    }
}

// Inherent methods of Plaintext

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
    /// assert_eq!(parser.comments().len(), 0);
    /// assert_eq!(parser.contents()[0], vec![false, true]);
    /// assert_eq!(parser.contents()[1], vec![false, false, true]);
    /// assert_eq!(parser.contents()[2], vec![true, true, true]);
    /// ```
    ///
    pub fn new(str: &str) -> Result<Self> {
        let partial = {
            let mut buf = PlaintextPartial::new();
            for line in str.lines() {
                buf.push(line)?;
            }
            buf
        };
        let Some(name) = partial.name else {
            bail!("No header line in the pattern");
        };
        Ok(Self {
            name,
            comments: partial.comments,
            contents: partial.contents,
        })
    }

    /// Returns the name of the pattern.
    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns comments of the pattern.
    #[inline]
    pub fn comments(&self) -> &Vec<String> {
        &self.comments
    }

    /// Returns contents of the pattern.
    #[inline]
    pub fn contents(&self) -> &Vec<Vec<bool>> {
        &self.contents
    }
}

impl fmt::Display for Plaintext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "!Name: {}", self.name())?;
        for line in self.comments() {
            writeln!(f, "!{}", line)?;
        }
        for line in self.contents() {
            let str: String = line.iter().map(|&x| if x { 'O' } else { '.' }).collect();
            writeln!(f, "{str}")?;
        }
        Ok(())
    }
}
