use anyhow::{bail, Result};
use std::fmt;
use std::io::{BufRead as _, BufReader, Read};

/// A representation for Plaintext file format, described in <https://conwaylife.com/wiki/Plaintext>.
#[derive(Debug, Clone)]
pub struct Plaintext {
    name: Option<String>,
    comments: Vec<String>,
    contents: Vec<(usize, Vec<usize>)>,
}

// An internal struct, used during constructing of Plaintext
struct PlaintextParser {
    name: Option<String>,
    comments: Vec<String>,
    lines: usize,
    contents: Vec<(usize, Vec<usize>)>,
}

// Inherent methods of PlaintextParser

impl PlaintextParser {
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
    fn parse_content_line(line: &str) -> Result<Vec<usize>> {
        let mut buf = Vec::new();
        for (i, char) in line.chars().enumerate() {
            match char {
                '.' => (),
                'O' => buf.push(i),
                _ => bail!("Invalid character found in the pattern"),
            }
        }
        Ok(buf)
    }
    fn new() -> Self {
        Self {
            name: None,
            comments: Vec::new(),
            lines: 0,
            contents: Vec::new(),
        }
    }
    fn push(&mut self, line: &str) -> Result<()> {
        if self.comments.is_empty() && self.lines == 0 {
            if let Some(name) = Self::parse_name_line(line) {
                self.name = Some(name.to_string());
                return Ok(());
            }
        }
        if self.lines == 0 {
            if let Some(comment) = Self::parse_comment_line(line) {
                self.comments.push(comment.to_string());
                return Ok(());
            }
        }
        let content = Self::parse_content_line(line)?;
        if !content.is_empty() {
            self.contents.push((self.lines, content));
        }
        self.lines += 1;
        Ok(())
    }
}

// Inherent methods

impl Plaintext {
    /// Creates from the specified implementor of Read, such as File or `&[u8]`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Plaintext;
    /// let pattern = "\
    ///     !Name: T-tetromino\n\
    ///     OOO\n\
    ///     .O.\n\
    /// ";
    /// let parser = Plaintext::new(pattern.as_bytes()).unwrap();
    /// ```
    ///
    pub fn new<R>(read: R) -> Result<Self>
    where
        R: Read,
    {
        let parser = {
            let mut buf = PlaintextParser::new();
            for line in BufReader::new(read).lines() {
                let line = line?;
                buf.push(&line)?;
            }
            buf
        };
        Ok(Self {
            name: parser.name,
            comments: parser.comments,
            contents: parser.contents,
        })
    }

    /// Returns the name of the pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Plaintext;
    /// let pattern = "\
    ///     !Name: T-tetromino\n\
    ///     OOO\n\
    ///     .O.\n\
    /// ";
    /// let parser = Plaintext::new(pattern.as_bytes()).unwrap();
    /// assert_eq!(parser.name(), Some("T-tetromino".to_string()));
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
    /// # use life_backend::format::Plaintext;
    /// let pattern = "\
    ///     !Name: T-tetromino\n\
    ///     !comment0\n\
    ///     !comment1\n\
    ///     OOO\n\
    ///     .O.\n\
    /// ";
    /// let parser = Plaintext::new(pattern.as_bytes()).unwrap();
    /// assert_eq!(parser.comments().len(), 2);
    /// assert_eq!(parser.comments()[0], "comment0");
    /// assert_eq!(parser.comments()[1], "comment1");
    /// ```
    ///
    #[inline]
    pub fn comments(&self) -> &Vec<String> {
        &self.comments
    }

    /// Creates a non-owning iterator over the series of immutable live cell positions in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Plaintext;
    /// let pattern = "\
    ///     !Name: T-tetromino\n\
    ///     OOO\n\
    ///     .O.\n\
    /// ";
    /// let parser = Plaintext::new(pattern.as_bytes()).unwrap();
    /// let mut iter = parser.iter();
    /// assert_eq!(iter.next(), Some((0, 0)));
    /// assert_eq!(iter.next(), Some((1, 0)));
    /// assert_eq!(iter.next(), Some((2, 0)));
    /// assert_eq!(iter.next(), Some((1, 1)));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.contents.iter().flat_map(|(y, xs)| xs.iter().map(|x| (*x, *y)))
    }
}

// Trait implementations

impl fmt::Display for Plaintext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = self.name() {
            writeln!(f, "!Name: {}", name)?;
        }
        for line in self.comments() {
            writeln!(f, "!{}", line)?;
        }
        if !self.contents.is_empty() {
            let max_x = self.contents.iter().flat_map(|(_, xs)| xs.iter()).copied().max().unwrap(); // note: this unwrap() never panic because flat_map() always returns at least one value under !self.contents.is_empty()
            let pad_line = ".".repeat(max_x + 1); // max_x + 1 never overflows because max_x < usize::MAX is guaranteed by the format
            let mut prev_y = 0;
            for (curr_y, xs) in &self.contents {
                let curr_y = *curr_y;
                for _ in prev_y..curr_y {
                    writeln!(f, "{pad_line}")?;
                }
                let line = {
                    let mut buf = String::new();
                    let mut prev_x = 0;
                    for &curr_x in xs {
                        buf.push_str(&pad_line[0..(curr_x - prev_x)]);
                        buf.push('O');
                        prev_x = curr_x + 1;
                    }
                    if prev_x <= max_x {
                        buf.push_str(&pad_line[0..(max_x - prev_x + 1)]);
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

// Unit tests

#[cfg(test)]
mod tests {
    use super::*;
    fn do_new_test_to_be_passed(
        pattern: &str,
        expected_name: &Option<&str>,
        expected_comments: &[&str],
        expected_contents: &[(usize, Vec<usize>)],
    ) -> Result<()> {
        let expected_name = expected_name.map(|s| s.to_string());
        let target = Plaintext::new(pattern.as_bytes())?;
        assert_eq!(target.name(), expected_name);
        assert_eq!(target.comments().len(), expected_comments.len());
        for (result, expected) in target.comments().iter().zip(expected_comments.iter()) {
            assert_eq!(result, expected);
        }
        assert_eq!(target.contents.len(), expected_contents.len());
        for (result, expected) in target.contents.iter().zip(expected_contents.iter()) {
            assert_eq!(result, expected);
        }
        assert_eq!(target.to_string(), pattern);
        Ok(())
    }
    fn do_new_test_to_be_failed(pattern: &str) {
        let target = Plaintext::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_empty() -> Result<()> {
        let pattern = "";
        let expected_name = None;
        let expected_comments = Vec::new();
        let expected_contents = Vec::new();
        do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header() -> Result<()> {
        let pattern = "!Name: test\n";
        let expected_name = Some("test");
        let expected_comments = Vec::new();
        let expected_contents = Vec::new();
        do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_no_header_but_comment() -> Result<()> {
        let pattern = "!comment\n";
        let expected_name = None;
        let expected_comments = vec!["comment"];
        let expected_contents = Vec::new();
        do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_comment() -> Result<()> {
        let pattern = concat!("!Name: test\n", "!comment\n");
        let expected_name = Some("test");
        let expected_comments = vec!["comment"];
        let expected_contents = Vec::new();
        do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_comments() -> Result<()> {
        let pattern = concat!("!Name: test\n", "!comment0\n", "!comment1\n");
        let expected_name = Some("test");
        let expected_comments = vec!["comment0", "comment1"];
        let expected_contents = Vec::new();
        do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_content() -> Result<()> {
        let pattern = concat!("!Name: test\n", ".O\n");
        let expected_name = Some("test");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, vec![1])];
        do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_contents() -> Result<()> {
        let pattern = concat!("!Name: test\n", ".O\n", "O.\n");
        let expected_name = Some("test");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, vec![1]), (1, vec![0])];
        do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_comments_contents() -> Result<()> {
        let pattern = concat!("!Name: test\n", "!comment0\n", "!comment1\n", ".O\n", "O.\n");
        let expected_name = Some("test");
        let expected_comments = vec!["comment0", "comment1"];
        let expected_contents = vec![(0, vec![1]), (1, vec![0])];
        do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_wrong_header() {
        let pattern = "_\n";
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_duplicate_header() -> Result<()> {
        let pattern = concat!("!Name: name0\n", "!Name: name1\n");
        let expected_name = Some("name0");
        let expected_comments = vec!["Name: name1"];
        let expected_contents = Vec::new();
        do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_wrong_content_without_comment() {
        let pattern = concat!("!Name: test\n", "_\n");
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_wrong_content_with_comment() {
        let pattern = concat!("!Name: test\n", "!\n", "_\n");
        do_new_test_to_be_failed(pattern)
    }
}
