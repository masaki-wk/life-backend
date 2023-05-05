use anyhow::{ensure, Result};
use std::collections::{HashMap, HashSet};

use super::{Plaintext, PlaintextLine};

/// The builder of Plaintext.
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

// Inherent methods

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
            let mut buf: Vec<_> = contents_group_by_y.into_iter().map(|(y, xs)| PlaintextLine(y, xs)).collect();
            buf.sort_by(|PlaintextLine(y0, _), PlaintextLine(y1, _)| y0.partial_cmp(y1).unwrap()); // this unwrap never panic because <usize>.partial_cmp(<usize>) always returns Some(_)
            for PlaintextLine(_, xs) in &mut buf {
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
    /// let target = pattern.iter().collect::<PlaintextBuilder>().name("foo\nbar").build().unwrap(); // this unwrap will panic
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

// Trait implementations

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