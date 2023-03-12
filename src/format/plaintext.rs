use anyhow::{bail, ensure, Result};
use num_iter::{range, range_from};
use num_traits::bounds::UpperBounded;
use num_traits::{One, ToPrimitive, Zero};
use std::fmt;
use std::io::{BufRead, BufReader, Read};

/// The default index type for Plaintext.
pub type DefaultIndexType = i16;

/// A representation for Plaintext file format, described in <https://conwaylife.com/wiki/Plaintext>.
#[derive(Debug, Clone)]
pub struct Plaintext<IndexType = DefaultIndexType> {
    name: String,
    comments: Vec<String>,
    contents: Vec<(IndexType, Vec<IndexType>)>,
}

// An internal struct, used during constructing of Plaintext
struct PlaintextPartial<IndexType> {
    name: Option<String>,
    comments: Vec<String>,
    lines: IndexType,
    contents: Vec<(IndexType, Vec<IndexType>)>,
}

// Inherent methods of PlaintextPartial

impl<IndexType> PlaintextPartial<IndexType> {
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
    fn parse_name_line(line: &str) -> Result<&str> {
        let Some(name) = Self::parse_prefixed_line("!Name: ", line) else {
            bail!("The header line is in wrong format");
        };
        Ok(name)
    }
    #[inline]
    fn parse_comment_line(line: &str) -> Option<&str> {
        Self::parse_prefixed_line("!", line)
    }
    fn parse_content_line(line: &str) -> Result<Vec<IndexType>>
    where
        IndexType: Copy + PartialOrd + Zero + One + UpperBounded,
    {
        let mut buf = Vec::new();
        for (i, char) in range_from(IndexType::zero()).zip(line.chars()) {
            match char {
                '.' => (),
                'O' => buf.push(i),
                _ => bail!("Invalid character found in the pattern"),
            };
            ensure!(i < IndexType::max_value(), "The pattern contains too wide line");
        }
        Ok(buf)
    }
    fn new() -> Self
    where
        IndexType: Zero,
    {
        Self {
            name: None,
            comments: Vec::new(),
            lines: IndexType::zero(),
            contents: Vec::new(),
        }
    }
    fn push(&mut self, line: &str) -> Result<()>
    where
        IndexType: Copy + PartialOrd + Zero + One + UpperBounded,
    {
        if self.name.is_none() {
            let name = Self::parse_name_line(line)?;
            self.name = Some(name.to_string());
        } else {
            if self.lines.is_zero() {
                if let Some(comment) = Self::parse_comment_line(line) {
                    self.comments.push(comment.to_string());
                    return Ok(());
                }
            }
            let content = Self::parse_content_line(line)?;
            ensure!(self.lines < IndexType::max_value(), "The pattern contains too many lines");
            if !content.is_empty() {
                self.contents.push((self.lines, content));
            }
            self.lines = self.lines + IndexType::one();
        }
        Ok(())
    }
}

// Inherent methods

impl<IndexType> Plaintext<IndexType> {
    /// Creates from the specified implementor of Read, such as File or `&[u8]`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Plaintext;
    /// let pattern = "\
    ///     !Name: Glider\n\
    ///     .O\n\
    ///     ..O\n\
    ///     OOO\n\
    /// ";
    /// let parser = Plaintext::<i16>::new(pattern.as_bytes()).unwrap();
    /// ```
    ///
    pub fn new<R: Read>(read: R) -> Result<Self>
    where
        IndexType: Copy + PartialOrd + Zero + One + UpperBounded,
    {
        let partial = {
            let mut buf = PlaintextPartial::new();
            for line in BufReader::new(read).lines() {
                let line = line?;
                buf.push(&line)?;
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Plaintext;
    /// let pattern = "\
    ///     !Name: Glider\n\
    ///     .O\n\
    ///     ..O\n\
    ///     OOO\n\
    /// ";
    /// let parser = Plaintext::<i16>::new(pattern.as_bytes()).unwrap();
    /// assert_eq!(parser.name(), "Glider");
    /// ```
    ///
    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns comments of the pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Plaintext;
    /// let pattern = "\
    ///     !Name: Glider\n\
    ///     !comment0\n\
    ///     !comment1\n\
    ///     .O\n\
    ///     ..O\n\
    ///     OOO\n\
    /// ";
    /// let parser = Plaintext::<i16>::new(pattern.as_bytes()).unwrap();
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
    ///     !Name: Glider\n\
    ///     .O\n\
    ///     ..O\n\
    ///     OOO\n\
    /// ";
    /// let parser = Plaintext::<i16>::new(pattern.as_bytes()).unwrap();
    /// let mut iter = parser.iter();
    /// assert_eq!(iter.next(), Some((1, 0)));
    /// assert_eq!(iter.next(), Some((2, 1)));
    /// assert_eq!(iter.next(), Some((0, 2)));
    /// assert_eq!(iter.next(), Some((1, 2)));
    /// assert_eq!(iter.next(), Some((2, 2)));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    pub fn iter(&self) -> impl Iterator<Item = (IndexType, IndexType)> + '_
    where
        IndexType: Copy,
    {
        self.contents.iter().flat_map(|(y, xs)| xs.iter().map(|x| (*x, *y)))
    }
}

// Trait implementations

impl<IndexType> fmt::Display for Plaintext<IndexType>
where
    IndexType: Copy + PartialOrd + Zero + One + ToPrimitive,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "!Name: {}", self.name())?;
        for line in self.comments() {
            writeln!(f, "!{}", line)?;
        }
        if !self.contents.is_empty() {
            let mut prev_y = IndexType::zero();
            for (curr_y, xs) in &self.contents {
                let curr_y = *curr_y;
                for _ in range(prev_y, curr_y) {
                    writeln!(f)?;
                }
                let line = {
                    let mut buf = String::new();
                    let mut prev_x = IndexType::zero();
                    for &curr_x in xs {
                        for _ in range(prev_x, curr_x) {
                            buf.push('.');
                        }
                        buf.push('O');
                        prev_x = curr_x + IndexType::one();
                    }
                    buf
                };
                writeln!(f, "{line}")?;
                prev_y = curr_y + IndexType::one();
            }
        }
        Ok(())
    }
}

// Unit tests

#[cfg(test)]
mod tests {
    use super::*;
    type TargetIndexType = i16;
    fn test_new(pattern: &str, expected_name: &str, expected_comments: &[&str], expected_contents: &[(TargetIndexType, Vec<TargetIndexType>)]) -> Result<()> {
        let target = Plaintext::<TargetIndexType>::new(pattern.as_bytes())?;
        assert_eq!(target.name(), expected_name);
        assert_eq!(target.comments().len(), expected_comments.len());
        for (result, expected) in target.comments().iter().zip(expected_comments.iter()) {
            assert_eq!(result, expected);
        }
        assert_eq!(target.contents.len(), expected_contents.len());
        for (result, expected) in target.contents.iter().zip(expected_contents.iter()) {
            assert_eq!(result, expected);
        }
        Ok(())
    }
    #[test]
    fn test_new_header() -> Result<()> {
        let pattern = "!Name: test\n";
        let expected_name = "test";
        let expected_comments = Vec::new();
        let expected_contents: Vec<(TargetIndexType, _)> = Vec::new();
        test_new(pattern, expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_comment() -> Result<()> {
        let pattern = concat!("!Name: test\n", "!comment\n");
        let expected_name = "test";
        let expected_comments = vec!["comment"];
        let expected_contents: Vec<(TargetIndexType, _)> = Vec::new();
        test_new(pattern, expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_comments() -> Result<()> {
        let pattern = concat!("!Name: test\n", "!comment0\n", "!comment1\n");
        let expected_name = "test";
        let expected_comments = vec!["comment0", "comment1"];
        let expected_contents: Vec<(TargetIndexType, _)> = Vec::new();
        test_new(pattern, expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_content() -> Result<()> {
        let pattern = concat!("!Name: test\n", ".O\n");
        let expected_name = "test";
        let expected_comments = Vec::new();
        let expected_contents = vec![(0 as TargetIndexType, vec![1])];
        test_new(pattern, expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_contents() -> Result<()> {
        let pattern = concat!("!Name: test\n", ".O\n", "O\n");
        let expected_name = "test";
        let expected_comments = Vec::new();
        let expected_contents = vec![(0 as TargetIndexType, vec![1]), (1 as TargetIndexType, vec![0])];
        test_new(pattern, expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_comments_contents() -> Result<()> {
        let pattern = concat!("!Name: test\n", "!comment0\n", "!comment1\n", ".O\n", "O\n");
        let expected_name = "test";
        let expected_comments = vec!["comment0", "comment1"];
        let expected_contents = vec![(0 as TargetIndexType, vec![1]), (1 as TargetIndexType, vec![0])];
        test_new(pattern, expected_name, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_empty() {
        let pattern = "";
        let target = Plaintext::<TargetIndexType>::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_wrong_header() {
        let pattern = "_\n";
        let target = Plaintext::<TargetIndexType>::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_wrong_content_without_comment() {
        let pattern = concat!("!Name: test\n", "_\n");
        let target = Plaintext::<TargetIndexType>::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_wrong_content_with_comment() {
        let pattern = concat!("!Name: test\n", "!\n", "_\n");
        let target = Plaintext::<TargetIndexType>::new(pattern.as_bytes());
        assert!(target.is_err());
    }
}
