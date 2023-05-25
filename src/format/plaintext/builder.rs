use anyhow::{ensure, Result};
use std::collections::{HashMap, HashSet};

use super::{Plaintext, PlaintextLine};
use crate::Position;

/// A builder of [`Plaintext`].
///
/// [`Plaintext`]: Plaintext
///
/// # Examples
///
/// Creates a builder via [`collect()`] with live cell positions, set a name via [`name()`], then builds [`Plaintext`] via [`build()`]:
///
/// [`collect()`]: std::iter::Iterator::collect
/// [`name()`]: #method.name
/// [`build()`]: #method.build
///
/// ```
/// use life_backend::format::PlaintextBuilder;
/// use life_backend::Position;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pattern = [Position(1, 0), Position(2, 0), Position(0, 1), Position(1, 1), Position(1, 2)];
/// let target = pattern.iter().collect::<PlaintextBuilder>().name("R-pentomino").build()?;
/// let expected = "\
///     !Name: R-pentomino\n\
///     .OO\n\
///     OO.\n\
///     .O.\n\
/// ";
/// assert_eq!(format!("{target}"), expected);
/// # Ok(())
/// # }
/// ```
///
/// Creates an empty builder via [`new()`], set a name via [`name()`], injects live cell positions via [`extend()`], then builds [`Plaintext`] via [`build()`]:
///
/// [`new()`]: #method.new
/// [`extend()`]: #method.extend
///
/// ```
/// use life_backend::format::PlaintextBuilder;
/// use life_backend::Position;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pattern = [Position(1, 0), Position(2, 0), Position(0, 1), Position(1, 1), Position(1, 2)];
/// let mut builder = PlaintextBuilder::new().name("R-pentomino");
/// builder.extend(pattern.iter());
/// let target = builder.build()?;
/// let expected = "\
///     !Name: R-pentomino\n\
///     .OO\n\
///     OO.\n\
///     .O.\n\
/// ";
/// assert_eq!(format!("{target}"), expected);
/// # Ok(())
/// # }
/// ```
///
#[derive(Debug, Clone)]
pub struct PlaintextBuilder<Name = PlaintextBuilderNoName, Comment = PlaintextBuilderNoComment>
where
    Name: PlaintextBuilderName,
    Comment: PlaintextBuilderComment,
{
    name: Name,
    comment: Comment,
    contents: HashSet<Position<usize>>,
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

impl PlaintextBuilder<PlaintextBuilderNoName, PlaintextBuilderNoComment> {
    /// Creates a builder that contains no live cells.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::PlaintextBuilder;
    /// let builder = PlaintextBuilder::new();
    /// ```
    ///
    #[inline]
    pub fn new() -> Self {
        Self {
            name: PlaintextBuilderNoName,
            comment: PlaintextBuilderNoComment,
            contents: HashSet::new(),
        }
    }
}

impl<Name, Comment> PlaintextBuilder<Name, Comment>
where
    Name: PlaintextBuilderName,
    Comment: PlaintextBuilderComment,
{
    /// Builds the [`Plaintext`] value.
    ///
    /// [`Plaintext`]: Plaintext
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::PlaintextBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let builder: PlaintextBuilder = pattern.iter().collect();
    /// let target = builder.build()?;
    /// # Ok(())
    /// # }
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
        let contents_group_by_y = self.contents.into_iter().fold(HashMap::new(), |mut acc, Position(x, y)| {
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
    /// use life_backend::format::PlaintextBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<PlaintextBuilder>()
    ///     .name("foo")
    ///     .build()?;
    /// assert_eq!(target.name(), Some("foo".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls [`name()`] twice or more will fail at compile time.  For example:
    ///
    /// [`name()`]: #method.name
    ///
    /// ```compile_fail
    /// use life_backend::format::PlaintextBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<PlaintextBuilder>()
    ///     .name("foo")
    ///     .name("bar") // Compile error
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`build()`] returns an error if the string passed by [`name()`] includes multiple lines.  For example:
    ///
    /// [`build()`]: #method.build
    /// [`name()`]: #method.name
    ///
    /// ```should_panic
    /// use life_backend::format::PlaintextBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<PlaintextBuilder>()
    ///     .name("foo\nbar")
    ///     .build()?; // Should fail
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn name(self, str: &str) -> PlaintextBuilder<PlaintextBuilderWithName, Comment> {
        let name = PlaintextBuilderWithName(str.to_owned());
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
    /// Set the comment.
    /// If the argument includes newlines, the instance of [`Plaintext`] built by [`build()`] includes multiple comment lines.
    ///
    /// [`Plaintext`]: Plaintext
    /// [`build()`]: #method.build
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::PlaintextBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<PlaintextBuilder>()
    ///     .comment("comment0\ncomment1")
    ///     .build()?;
    /// assert_eq!(target.comments().len(), 2);
    /// assert_eq!(target.comments()[0], "comment0");
    /// assert_eq!(target.comments()[1], "comment1");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls [`comment()`] twice or more will fail at compile time.  For example:
    ///
    /// [`comment()`]: #method.comment
    ///
    /// ```compile_fail
    /// use life_backend::format::PlaintextBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<PlaintextBuilder>()
    ///     .comment("comment0")
    ///     .comment("comment1") // Compile error
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn comment(self, str: &str) -> PlaintextBuilder<Name, PlaintextBuilderWithComment> {
        let comment = PlaintextBuilderWithComment(str.to_owned());
        PlaintextBuilder {
            name: self.name,
            comment,
            contents: self.contents,
        }
    }
}

// Trait implementations

impl Default for PlaintextBuilder<PlaintextBuilderNoName, PlaintextBuilderNoComment> {
    /// Returns the default value of the type, same as the return value of [`new()`].
    ///
    /// [`new()`]: #method.new
    ///
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<Name, Comment> PlaintextBuilder<Name, Comment>
where
    Name: PlaintextBuilderName,
    Comment: PlaintextBuilderComment,
{
    // Implementation of public extend()
    #[inline]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Position<usize>>,
    {
        self.contents.extend(iter);
    }
}

impl PlaintextBuilder<PlaintextBuilderNoName, PlaintextBuilderNoComment> {
    // Implementation of public from_iter()
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Position<usize>>,
    {
        let mut v = Self::new();
        v.extend(iter);
        v
    }
}

impl<'a> FromIterator<&'a Position<usize>> for PlaintextBuilder<PlaintextBuilderNoName, PlaintextBuilderNoComment> {
    /// Creates a value from a non-owning iterator over a series of [`&Position<usize>`].
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// [`&Position<usize>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::PlaintextBuilder;
    /// use life_backend::Position;
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let iter = pattern.iter();
    /// let builder: PlaintextBuilder = iter.collect();
    /// ```
    ///
    #[inline]
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a Position<usize>>,
    {
        Self::from_iter(iter.into_iter().copied())
    }
}

impl FromIterator<Position<usize>> for PlaintextBuilder<PlaintextBuilderNoName, PlaintextBuilderNoComment> {
    /// Creates a value from an owning iterator over a series of [`Position<usize>`].
    /// Each item in the series represents a moved live cell position.
    ///
    /// [`Position<usize>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::PlaintextBuilder;
    /// use life_backend::Position;
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let iter = pattern.into_iter();
    /// let builder: PlaintextBuilder = iter.collect();
    /// ```
    ///
    #[inline]
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Position<usize>>,
    {
        Self::from_iter(iter)
    }
}

impl<'a, Name, Comment> Extend<&'a Position<usize>> for PlaintextBuilder<Name, Comment>
where
    Name: PlaintextBuilderName,
    Comment: PlaintextBuilderComment,
{
    /// Extends the builder with the contents of the specified non-owning iterator over the series of [`&Position<usize>`].
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// [`&Position<usize>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::PlaintextBuilder;
    /// use life_backend::Position;
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let iter = pattern.iter();
    /// let mut builder = PlaintextBuilder::new();
    /// builder.extend(iter);
    /// ```
    ///
    #[inline]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = &'a Position<usize>>,
    {
        self.extend(iter.into_iter().copied());
    }
}

impl<Name, Comment> Extend<Position<usize>> for PlaintextBuilder<Name, Comment>
where
    Name: PlaintextBuilderName,
    Comment: PlaintextBuilderComment,
{
    /// Extends the builder with the contents of the specified owning iterator over the series of [`Position<usize>`].
    /// Each item in the series represents a moved live cell position.
    ///
    /// [`Position<usize>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::PlaintextBuilder;
    /// use life_backend::Position;
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let iter = pattern.into_iter();
    /// let mut builder = PlaintextBuilder::new();
    /// builder.extend(iter);
    /// ```
    ///
    #[inline]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Position<usize>>,
    {
        self.extend(iter);
    }
}

// Unit tests

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default() {
        let target = PlaintextBuilder::default();
        assert!(target.contents.is_empty());
    }
}
