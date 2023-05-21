use anyhow::{ensure, Result};
use std::collections::{HashMap, HashSet};

use super::{Rle, RleHeader, RleRunsTriple};
use crate::{Position, Rule};

/// A builder of [`Rle`].
///
/// [`Rle`]: Rle
///
/// # Examples
///
/// Creates a builder via [`collect()`] with live cell positions, set a name via [`name()`], then builds [`Rle`] via [`build()`]:
///
/// [`collect()`]: std::iter::Iterator::collect
/// [`name()`]: #method.name
/// [`build()`]: #method.build
///
/// ```
/// use life_backend::format::RleBuilder;
/// use life_backend::Position;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pattern = [Position(1, 0), Position(2, 0), Position(0, 1), Position(1, 1), Position(1, 2)];
/// let target = pattern.iter().collect::<RleBuilder>().name("R-pentomino").build()?;
/// let expected = "\
///     #N R-pentomino\n\
///     x = 3, y = 3, rule = B3/S23\n\
///     b2o$2o$bo!\n\
/// ";
/// assert_eq!(format!("{target}"), expected);
/// # Ok(())
/// # }
/// ```
///
/// Creates an empty builder via [`new()`], set a name via [`name()`], injects live cell positions via [`extend()`], then builds [`Rle`] via [`build()`]:
///
/// [`new()`]: #method.new
/// [`extend()`]: #method.extend
///
/// ```
/// use life_backend::format::RleBuilder;
/// use life_backend::Position;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pattern = [Position(1, 0), Position(2, 0), Position(0, 1), Position(1, 1), Position(1, 2)];
/// let mut builder = RleBuilder::new().name("R-pentomino");
/// builder.extend(pattern.iter());
/// let target = builder.build()?;
/// let expected = "\
///     #N R-pentomino\n\
///     x = 3, y = 3, rule = B3/S23\n\
///     b2o$2o$bo!\n\
/// ";
/// assert_eq!(format!("{target}"), expected);
/// # Ok(())
/// # }
/// ```
///
#[derive(Debug, Clone)]
pub struct RleBuilder<Name = RleBuilderNoName, Created = RleBuilderNoCreated, Comment = RleBuilderNoComment, Rule = RleBuilderNoRule>
where
    Name: RleBuilderName,
    Created: RleBuilderCreated,
    Comment: RleBuilderComment,
    Rule: RleBuilderRule,
{
    name: Name,
    created: Created,
    comment: Comment,
    rule: Rule,
    contents: HashSet<Position<usize>>,
}

// Traits and types for RleBuilder's typestate
pub trait RleBuilderName {
    fn drain(self) -> Option<String>;
}
pub trait RleBuilderCreated {
    fn drain(self) -> Option<String>;
}
pub trait RleBuilderComment {
    fn drain(self) -> Option<String>;
}
pub trait RleBuilderRule {
    fn drain(self) -> Option<Rule>;
}
pub struct RleBuilderNoName;
impl RleBuilderName for RleBuilderNoName {
    fn drain(self) -> Option<String> {
        None
    }
}
pub struct RleBuilderWithName(String);
impl RleBuilderName for RleBuilderWithName {
    fn drain(self) -> Option<String> {
        Some(self.0)
    }
}
pub struct RleBuilderNoCreated;
pub struct RleBuilderWithCreated(String);
impl RleBuilderCreated for RleBuilderNoCreated {
    fn drain(self) -> Option<String> {
        None
    }
}
impl RleBuilderCreated for RleBuilderWithCreated {
    fn drain(self) -> Option<String> {
        Some(self.0)
    }
}
pub struct RleBuilderNoComment;
pub struct RleBuilderWithComment(String);
impl RleBuilderComment for RleBuilderNoComment {
    fn drain(self) -> Option<String> {
        None
    }
}
impl RleBuilderComment for RleBuilderWithComment {
    fn drain(self) -> Option<String> {
        Some(self.0)
    }
}
pub struct RleBuilderNoRule;
pub struct RleBuilderWithRule(Rule);
impl RleBuilderRule for RleBuilderNoRule {
    fn drain(self) -> Option<Rule> {
        None
    }
}
impl RleBuilderRule for RleBuilderWithRule {
    fn drain(self) -> Option<Rule> {
        Some(self.0)
    }
}

// Inherent methods

impl RleBuilder<RleBuilderNoName, RleBuilderNoCreated, RleBuilderNoComment, RleBuilderNoRule> {
    /// Creates a builder that contains no live cells.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// let builder = RleBuilder::new();
    /// ```
    ///
    #[inline]
    pub fn new() -> Self {
        Self {
            name: RleBuilderNoName,
            created: RleBuilderNoCreated,
            comment: RleBuilderNoComment,
            rule: RleBuilderNoRule,
            contents: HashSet::new(),
        }
    }
}

impl<Name, Created, Comment, RuleSpec> RleBuilder<Name, Created, Comment, RuleSpec>
where
    Name: RleBuilderName,
    Created: RleBuilderCreated,
    Comment: RleBuilderComment,
    RuleSpec: RleBuilderRule,
{
    /// Builds the [`Rle`] value.
    ///
    /// [`Rle`]: Rle
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let builder: RleBuilder = pattern.iter().collect();
    /// let target = builder.build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn build(self) -> Result<Rle> {
        let comments: Vec<_> = {
            fn parse_to_comments(str: Option<String>, prefix: &str) -> Vec<String> {
                fn append_prefix(str: &str, prefix: &str) -> String {
                    let mut buf = prefix.to_owned();
                    if !str.is_empty() {
                        buf.push(' ');
                        buf.push_str(str);
                    }
                    buf
                }
                match str {
                    Some(str) => {
                        if str.is_empty() {
                            vec![prefix.to_owned()]
                        } else {
                            str.lines().map(|s| append_prefix(s, prefix)).collect::<Vec<_>>()
                        }
                    }
                    None => Vec::new(),
                }
            }
            let name = self.name.drain();
            if let Some(str) = &name {
                ensure!(str.lines().count() <= 1, "the string passed by name(str) includes multiple lines");
            }
            [(name, "#N"), (self.created.drain(), "#O"), (self.comment.drain(), "#C")]
                .into_iter()
                .flat_map(|(str, prefix)| parse_to_comments(str, prefix).into_iter())
                .collect()
        };
        let rule = self.rule.drain().unwrap_or(Rule::conways_life());
        let contents_group_by_y = self.contents.into_iter().fold(HashMap::new(), |mut acc, Position(x, y)| {
            acc.entry(y).or_insert_with(Vec::new).push(x);
            acc
        });
        let contents_sorted = {
            let mut contents_sorted: Vec<_> = contents_group_by_y.into_iter().collect();
            contents_sorted.sort_by(|(y0, _), (y1, _)| y0.partial_cmp(y1).unwrap()); // this unwrap never panic because <usize>.partial_cmp(<usize>) always returns Some(_)
            for (_, xs) in &mut contents_sorted {
                xs.sort();
            }
            contents_sorted
        };
        let header = {
            let width = contents_sorted.iter().flat_map(|(_, xs)| xs.iter()).copied().max().map(|x| x + 1).unwrap_or(0);
            let height = contents_sorted.iter().last().map(|&(y, _)| y + 1).unwrap_or(0);
            RleHeader { width, height, rule }
        };
        let contents = {
            fn flush_to_buf(buf: &mut Vec<RleRunsTriple>, (prev_x, prev_y): (usize, usize), (curr_x, curr_y): (usize, usize), live_cells: usize) {
                if live_cells > 0 {
                    let pad_lines = curr_y - prev_y;
                    let pad_dead_cells = if pad_lines > 0 { curr_x } else { curr_x - prev_x };
                    buf.push(RleRunsTriple {
                        pad_lines,
                        pad_dead_cells,
                        live_cells,
                    })
                }
            }
            let (mut buf, (prev_x, prev_y), (curr_x, curr_y), live_cells) =
                contents_sorted.into_iter().flat_map(|(y, xs)| xs.into_iter().map(move |x| (x, y))).fold(
                    (Vec::new(), (0, 0), (0, 0), 0),
                    |(mut buf, (prev_x, prev_y), (curr_x, curr_y), live_cells), (next_x, next_y)| {
                        if next_y == curr_y && next_x == curr_x + live_cells {
                            (buf, (prev_x, prev_y), (curr_x, curr_y), live_cells + 1)
                        } else {
                            flush_to_buf(&mut buf, (prev_x, prev_y), (curr_x, curr_y), live_cells);
                            (buf, (curr_x + live_cells, curr_y), (next_x, next_y), 1)
                        }
                    },
                );
            flush_to_buf(&mut buf, (prev_x, prev_y), (curr_x, curr_y), live_cells);
            buf
        };
        Ok(Rle { header, comments, contents })
    }
}

impl<Created, Comment, Rule> RleBuilder<RleBuilderNoName, Created, Comment, Rule>
where
    Created: RleBuilderCreated,
    Comment: RleBuilderComment,
    Rule: RleBuilderRule,
{
    /// Set the name.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<RleBuilder>()
    ///     .name("foo")
    ///     .build()?;
    /// assert_eq!(target.comments().len(), 1);
    /// assert_eq!(target.comments()[0], "#N foo".to_string());
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
    /// use life_backend::format::RleBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<RleBuilder>()
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
    /// use life_backend::format::RleBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<RleBuilder>()
    ///     .name("foo\nbar")
    ///     .build()?; // Should fail
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn name(self, str: &str) -> RleBuilder<RleBuilderWithName, Created, Comment, Rule> {
        let name = RleBuilderWithName(str.to_owned());
        RleBuilder {
            name,
            created: self.created,
            comment: self.comment,
            rule: self.rule,
            contents: self.contents,
        }
    }
}

impl<Name, Comment, Rule> RleBuilder<Name, RleBuilderNoCreated, Comment, Rule>
where
    Name: RleBuilderName,
    Comment: RleBuilderComment,
    Rule: RleBuilderRule,
{
    /// Set the information when and by whom the pattern was created.
    /// If the argument includes newlines, the instance of [`Rle`] built by [`build()`] includes multiple comment lines.
    ///
    /// [`Rle`]: Rle
    /// [`build()`]: #method.build
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<RleBuilder>()
    ///     .created("foo")
    ///     .build()?;
    /// assert_eq!(target.comments().len(), 1);
    /// assert_eq!(target.comments()[0], "#O foo".to_string());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls [`created()`] twice or more will fail at compile time.  For example:
    ///
    /// [`created()`]: #method.created
    ///
    /// ```compile_fail
    /// use life_backend::format::RleBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<RleBuilder>()
    ///     .created("foo")
    ///     .created("bar") // Compile error
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn created(self, str: &str) -> RleBuilder<Name, RleBuilderWithCreated, Comment, Rule> {
        let created = RleBuilderWithCreated(str.to_owned());
        RleBuilder {
            name: self.name,
            created,
            comment: self.comment,
            rule: self.rule,
            contents: self.contents,
        }
    }
}

impl<Name, Created, Rule> RleBuilder<Name, Created, RleBuilderNoComment, Rule>
where
    Name: RleBuilderName,
    Created: RleBuilderCreated,
    Rule: RleBuilderRule,
{
    /// Set the comment.
    /// If the argument includes newlines, the instance of [`Rle`] built by [`build()`] includes multiple comment lines.
    ///
    /// [`Rle`]: Rle
    /// [`build()`]: #method.build
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<RleBuilder>()
    ///     .comment("comment0\ncomment1")
    ///     .build()?;
    /// assert_eq!(target.comments().len(), 2);
    /// assert_eq!(target.comments()[0], "#C comment0".to_string());
    /// assert_eq!(target.comments()[1], "#C comment1".to_string());
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
    /// use life_backend::format::RleBuilder;
    /// use life_backend::Position;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<RleBuilder>()
    ///     .comment("comment0")
    ///     .comment("comment1")
    ///     .build()?; // Compile error
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn comment(self, str: &str) -> RleBuilder<Name, Created, RleBuilderWithComment, Rule> {
        let comment = RleBuilderWithComment(str.to_owned());
        RleBuilder {
            name: self.name,
            created: self.created,
            comment,
            rule: self.rule,
            contents: self.contents,
        }
    }
}

impl<Name, Created, Comment> RleBuilder<Name, Created, Comment, RleBuilderNoRule>
where
    Name: RleBuilderName,
    Created: RleBuilderCreated,
    Comment: RleBuilderComment,
{
    /// Set the rule.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// use life_backend::{Position, Rule};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<RleBuilder>()
    ///     .rule(Rule::conways_life())
    ///     .build()?;
    /// assert_eq!(*target.rule(), Rule::conways_life());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls [`rule()`] twice or more will fail at compile time.  For example:
    ///
    /// [`rule()`]: #method.rule
    ///
    /// ```compile_fail
    /// use life_backend::format::RleBuilder;
    /// use life_backend::{Position, Rule};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let target = pattern
    ///     .iter()
    ///     .collect::<RleBuilder>()
    ///     .rule(Rule::conways_life())
    ///     .rule(Rule::conways_life()) // Compile error
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn rule(self, rule: Rule) -> RleBuilder<Name, Created, Comment, RleBuilderWithRule> {
        let rule = RleBuilderWithRule(rule);
        RleBuilder {
            name: self.name,
            created: self.created,
            comment: self.comment,
            rule,
            contents: self.contents,
        }
    }
}

// Trait implementations

impl Default for RleBuilder<RleBuilderNoName, RleBuilderNoCreated, RleBuilderNoComment, RleBuilderNoRule> {
    /// Returns the default value of the type, same as the return value of [`new()`].
    ///
    /// [`new()`]: #method.new
    ///
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<Name, Created, Comment, RuleSpec> RleBuilder<Name, Created, Comment, RuleSpec>
where
    Name: RleBuilderName,
    Created: RleBuilderCreated,
    Comment: RleBuilderComment,
    RuleSpec: RleBuilderRule,
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

impl RleBuilder<RleBuilderNoName, RleBuilderNoCreated, RleBuilderNoComment, RleBuilderNoRule> {
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

impl<'a> FromIterator<&'a Position<usize>> for RleBuilder<RleBuilderNoName, RleBuilderNoCreated, RleBuilderNoComment, RleBuilderNoRule> {
    /// Creates a value from a non-owning iterator over a series of [`&Position<usize>`].
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// [`&Position<usize>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// use life_backend::Position;
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let iter = pattern.iter();
    /// let builder: RleBuilder = iter.collect();
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

impl FromIterator<Position<usize>> for RleBuilder<RleBuilderNoName, RleBuilderNoCreated, RleBuilderNoComment, RleBuilderNoRule> {
    /// Creates a value from an owning iterator over a series of [`Position<usize>`].
    /// Each item in the series represents a moved live cell position.
    ///
    /// [`Position<usize>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// use life_backend::Position;
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let iter = pattern.into_iter();
    /// let builder: RleBuilder = iter.collect();
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

impl<'a, Name, Created, Comment, RuleSpec> Extend<&'a Position<usize>> for RleBuilder<Name, Created, Comment, RuleSpec>
where
    Name: RleBuilderName,
    Created: RleBuilderCreated,
    Comment: RleBuilderComment,
    RuleSpec: RleBuilderRule,
{
    /// Extends the builder with the contents of the specified non-owning iterator over the series of [`&Position<usize>`].
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// [`&Position<usize>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// use life_backend::Position;
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let iter = pattern.iter();
    /// let mut builder = RleBuilder::new();
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

impl<Name, Created, Comment, RuleSpec> Extend<Position<usize>> for RleBuilder<Name, Created, Comment, RuleSpec>
where
    Name: RleBuilderName,
    Created: RleBuilderCreated,
    Comment: RleBuilderComment,
    RuleSpec: RleBuilderRule,
{
    /// Extends the builder with the contents of the specified owning iterator over the series of [`Position<usize>`].
    /// Each item in the series represents a moved live cell position.
    ///
    /// [`Position<usize>`]: Position
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// use life_backend::Position;
    /// let pattern = [Position(1, 0), Position(0, 1)];
    /// let iter = pattern.into_iter();
    /// let mut builder = RleBuilder::new();
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
