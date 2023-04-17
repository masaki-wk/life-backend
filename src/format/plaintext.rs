use anyhow::{anyhow, ensure, Result};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::{BufRead as _, BufReader, Read};

/// A representation for Plaintext file format.
///
/// The detail of this format is described in:
///
/// - [Plaintext - LifeWiki](https://conwaylife.com/wiki/Plaintext)
///
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

/// A builder of Plaintext.
#[derive(Debug, Clone)]
pub struct PlaintextBuilder<Name = PlaintextBuilderNoName, Comment = PlaintextBuilderNoComment>
where
    Name: PlaintextBuilderName,
    Comment: PlaintextBuilderComment,
{
    name: Name,
    comment: Comment,
    contents: HashSet<(usize, usize)>,
}

// Traits and types for PlaintextBuilder's typestate
pub trait PlaintextBuilderName {
    fn drain(self) -> Option<String>;
}
pub trait PlaintextBuilderComment {
    fn drain(self) -> Option<String>;
}
pub struct PlaintextBuilderNoName;
impl PlaintextBuilderName for PlaintextBuilderNoName {
    fn drain(self) -> Option<String> {
        None
    }
}
pub struct PlaintextBuilderWithName(String);
impl PlaintextBuilderName for PlaintextBuilderWithName {
    fn drain(self) -> Option<String> {
        Some(self.0)
    }
}
pub struct PlaintextBuilderNoComment;
pub struct PlaintextBuilderWithComment(String);
impl PlaintextBuilderComment for PlaintextBuilderNoComment {
    fn drain(self) -> Option<String> {
        None
    }
}
impl PlaintextBuilderComment for PlaintextBuilderWithComment {
    fn drain(self) -> Option<String> {
        Some(self.0)
    }
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
        line.chars()
            .enumerate()
            .filter_map(|(i, char)| match char {
                '.' => None,
                'O' => Some(Ok(i)),
                _ => Some(Err(anyhow!("Invalid character found in the pattern"))),
            })
            .collect()
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
        if self.name.is_none() && self.comments.is_empty() && self.lines == 0 {
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

// Inherent methods of PlaintextBuilder

impl<Name, Comment> PlaintextBuilder<Name, Comment>
where
    Name: PlaintextBuilderName,
    Comment: PlaintextBuilderComment,
{
    /// Builds the Plaintext.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::PlaintextBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<PlaintextBuilder>().build().unwrap();
    /// ```
    ///
    pub fn build(self) -> Result<Plaintext> {
        let name = self.name.drain();
        if let Some(str) = &name {
            ensure!(str.lines().count() <= 1, "the string passed by name(str) includes multiple lines");
        };
        let comments = match self.comment.drain() {
            Some(str) => {
                let buf: Vec<_> = str.lines().map(String::from).collect();
                if buf.is_empty() {
                    // buf is empty only if str == "" || str == "\n"
                    vec![String::new()]
                } else {
                    buf
                }
            }
            None => Vec::new(),
        };
        let contents_group_by_y = self.contents.into_iter().fold(HashMap::new(), |mut acc, (x, y)| {
            acc.entry(y).or_insert_with(Vec::new).push(x);
            acc
        });
        let contents_sorted = {
            let mut buf: Vec<_> = contents_group_by_y.into_iter().collect();
            buf.sort_by(|(y0, _), (y1, _)| y0.partial_cmp(y1).unwrap()); // this unwrap never panic because <usize>.partial_cmp(<usize>) always returns Some(_)
            for (_, xs) in &mut buf {
                xs.sort();
            }
            buf
        };
        Ok(Plaintext {
            name,
            comments,
            contents: contents_sorted,
        })
    }
}

impl<Comment> PlaintextBuilder<PlaintextBuilderNoName, Comment>
where
    Comment: PlaintextBuilderComment,
{
    /// Set the name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::PlaintextBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<PlaintextBuilder>().name("foo").build().unwrap();
    /// assert_eq!(target.name(), Some("foo".to_string()));
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls name() twice or more will fail at compile time.  For example:
    ///
    /// ```compile_fail
    /// # use life_backend::format::PlaintextBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<PlaintextBuilder>().name("foo").name("bar").build().unwrap(); // Compile error
    /// ```
    ///
    /// build() returns an error if the string passed by name(str) includes multiple lines.  For example:
    ///
    /// ```should_panic
    /// # use life_backend::format::PlaintextBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<PlaintextBuilder>().name("foo\nbar").build().unwrap();
    /// ```
    ///
    pub fn name(self, str: &str) -> PlaintextBuilder<PlaintextBuilderWithName, Comment> {
        let name = PlaintextBuilderWithName(str.to_string());
        PlaintextBuilder {
            name,
            comment: self.comment,
            contents: self.contents,
        }
    }
}

impl<Name> PlaintextBuilder<Name, PlaintextBuilderNoComment>
where
    Name: PlaintextBuilderName,
{
    /// Set the comment.  If the argument includes newlines, the instance of Plaintext built by build() includes multiple comment lines.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::PlaintextBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<PlaintextBuilder>().comment("comment0\ncomment1").build().unwrap();
    /// assert_eq!(target.comments().len(), 2);
    /// assert_eq!(target.comments()[0], "comment0");
    /// assert_eq!(target.comments()[1], "comment1");
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls comment() twice or more will fail at compile time.  For example:
    ///
    /// ```compile_fail
    /// # use life_backend::format::PlaintextBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<PlaintextBuilder>().comment("comment0").comment("comment1").build().unwrap(); // Compile error
    /// ```
    ///
    pub fn comment(self, str: &str) -> PlaintextBuilder<Name, PlaintextBuilderWithComment> {
        let comment = PlaintextBuilderWithComment(str.to_string());
        PlaintextBuilder {
            name: self.name,
            comment,
            contents: self.contents,
        }
    }
}

// Trait implementations of PlaintextBuilder

impl<'a> FromIterator<&'a (usize, usize)> for PlaintextBuilder<PlaintextBuilderNoName, PlaintextBuilderNoComment> {
    /// Conversion from a non-owning iterator over a series of &(usize, usize).
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::PlaintextBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let builder = pattern.iter().collect::<PlaintextBuilder>();
    /// let target = builder.build().unwrap();
    /// ```
    ///
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a (usize, usize)>,
    {
        let contents = iter.into_iter().copied().collect();
        Self {
            name: PlaintextBuilderNoName,
            comment: PlaintextBuilderNoComment,
            contents,
        }
    }
}

impl FromIterator<(usize, usize)> for PlaintextBuilder<PlaintextBuilderNoName, PlaintextBuilderNoComment> {
    /// Conversion from an owning iterator over a series of (usize, usize).
    /// Each item in the series represents a moved live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::PlaintextBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let builder = pattern.into_iter().collect::<PlaintextBuilder>();
    /// let target = builder.build().unwrap();
    /// ```
    ///
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (usize, usize)>,
    {
        let contents = iter.into_iter().collect();
        Self {
            name: PlaintextBuilderNoName,
            comment: PlaintextBuilderNoComment,
            contents,
        }
    }
}

// Inherent methods of Plaintext

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
        let parser = BufReader::new(read).lines().try_fold(PlaintextParser::new(), |mut buf, line| {
            let line = line?;
            buf.push(&line)?;
            Ok::<_, anyhow::Error>(buf)
        })?;
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

// Trait implementations of Plaintext

impl fmt::Display for Plaintext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = self.name() {
            writeln!(f, "!Name: {}", name)?;
        }
        for line in self.comments() {
            writeln!(f, "!{}", line)?;
        }
        if !self.contents.is_empty() {
            let max_x = self.contents.iter().flat_map(|(_, xs)| xs.iter()).copied().max().unwrap(); // this unwrap() never panic because flat_map() always returns at least one value under !self.contents.is_empty()
            let dead_cell_chars = ".".repeat(max_x + 1); // max_x + 1 never overflows because max_x < usize::MAX is guaranteed by the format of self.contents
            let mut prev_y = 0;
            for (curr_y, xs) in &self.contents {
                for _ in prev_y..(*curr_y) {
                    writeln!(f, "{dead_cell_chars}")?;
                }
                let line = {
                    let (mut buf, prev_x) = xs.iter().fold((String::with_capacity(max_x + 1), 0), |(mut buf, prev_x), &curr_x| {
                        buf += &dead_cell_chars[0..(curr_x - prev_x)];
                        buf += "O";
                        (buf, curr_x + 1)
                    });
                    if prev_x <= max_x {
                        buf += &dead_cell_chars[0..((max_x + 1) - prev_x)];
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
    fn do_check(target: &Plaintext, expected_name: &Option<&str>, expected_comments: &[&str], expected_contents: &[(usize, Vec<usize>)]) {
        let expected_name = expected_name.map(|s| s.to_string());
        assert_eq!(target.name(), expected_name);
        assert_eq!(target.comments().len(), expected_comments.len());
        for (result, expected) in target.comments().iter().zip(expected_comments.iter()) {
            assert_eq!(result, expected);
        }
        assert_eq!(target.contents.len(), expected_contents.len());
        for (result, expected) in target.contents.iter().zip(expected_contents.iter()) {
            assert_eq!(result, expected);
        }
    }
    fn do_new_test_to_be_passed(
        pattern: &str,
        expected_name: &Option<&str>,
        expected_comments: &[&str],
        expected_contents: &[(usize, Vec<usize>)],
    ) -> Result<()> {
        let target = Plaintext::new(pattern.as_bytes())?;
        do_check(&target, expected_name, expected_comments, expected_contents);
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
    #[test]
    fn test_build() -> Result<()> {
        let pattern = [(1, 0), (0, 1)];
        let expected_name = None;
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, vec![1]), (1, vec![0])];
        let target = pattern.iter().collect::<PlaintextBuilder>().build()?;
        do_check(&target, &expected_name, &expected_comments, &expected_contents);
        Ok(())
    }
    #[test]
    fn test_build_singleline_name() -> Result<()> {
        let pattern = [(1, 0), (0, 1)];
        let expected_name = Some("test");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, vec![1]), (1, vec![0])];
        let target = pattern.iter().collect::<PlaintextBuilder>().name("test").build()?;
        do_check(&target, &expected_name, &expected_comments, &expected_contents);
        Ok(())
    }
    #[test]
    fn test_build_blank_name() -> Result<()> {
        let pattern = [(1, 0), (0, 1)];
        let expected_name = Some("");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, vec![1]), (1, vec![0])];
        let target = pattern.iter().collect::<PlaintextBuilder>().name("").build()?;
        do_check(&target, &expected_name, &expected_comments, &expected_contents);
        Ok(())
    }
    #[test]
    fn test_build_multiline_name() {
        let pattern = [(1, 0), (0, 1)];
        let target = pattern.iter().collect::<PlaintextBuilder>().name("name\nname").build();
        assert!(target.is_err());
    }
    #[test]
    fn test_build_comment() -> Result<()> {
        let pattern = [(1, 0), (0, 1)];
        let expected_name = None;
        let expected_comments = vec!["comment"];
        let expected_contents = vec![(0, vec![1]), (1, vec![0])];
        let target = pattern.iter().collect::<PlaintextBuilder>().comment("comment").build()?;
        do_check(&target, &expected_name, &expected_comments, &expected_contents);
        Ok(())
    }
    #[test]
    fn test_build_blank_comment() -> Result<()> {
        let pattern = [(1, 0), (0, 1)];
        let expected_name = None;
        let expected_comments = vec![""];
        let expected_contents = vec![(0, vec![1]), (1, vec![0])];
        let target = pattern.iter().collect::<PlaintextBuilder>().comment("").build()?;
        do_check(&target, &expected_name, &expected_comments, &expected_contents);
        Ok(())
    }
    #[test]
    fn test_build_comments() -> Result<()> {
        let pattern = [(1, 0), (0, 1)];
        let expected_name = None;
        let expected_comments = vec!["comment0", "comment1"];
        let expected_contents = vec![(0, vec![1]), (1, vec![0])];
        let target = pattern.iter().collect::<PlaintextBuilder>().comment("comment0\ncomment1").build()?;
        do_check(&target, &expected_name, &expected_comments, &expected_contents);
        Ok(())
    }
    #[test]
    fn test_build_name_comment() -> Result<()> {
        let pattern = [(1, 0), (0, 1)];
        let expected_name = Some("test");
        let expected_comments = vec!["comment"];
        let expected_contents = vec![(0, vec![1]), (1, vec![0])];
        let target = pattern.iter().collect::<PlaintextBuilder>().name("test").comment("comment").build()?;
        do_check(&target, &expected_name, &expected_comments, &expected_contents);
        Ok(())
    }
}
