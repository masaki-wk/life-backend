use anyhow::{ensure, Result};
use std::collections::{HashMap, HashSet};

use super::{Rle, RleHeader, RleRunsTriple};
use crate::Rule;

/// A builder of Rle.
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
    contents: HashSet<(usize, usize)>,
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

impl<Name, Created, Comment, RuleSpec> RleBuilder<Name, Created, Comment, RuleSpec>
where
    Name: RleBuilderName,
    Created: RleBuilderCreated,
    Comment: RleBuilderComment,
    RuleSpec: RleBuilderRule,
{
    /// Builds the specified Rle value.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<RleBuilder>().build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn build(self) -> Result<Rle> {
        let comments: Vec<_> = {
            fn parse_to_comments(str: Option<String>, prefix: &str) -> Vec<String> {
                fn append_prefix(str: &str, prefix: &str) -> String {
                    let mut buf = prefix.to_string();
                    if !str.is_empty() {
                        buf.push(' ');
                        buf.push_str(str);
                    }
                    buf
                }
                match str {
                    Some(str) => {
                        if str.is_empty() {
                            vec![prefix.to_string()]
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
        let contents_group_by_y = self.contents.into_iter().fold(HashMap::new(), |mut acc, (x, y)| {
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<RleBuilder>().name("foo").build()?;
    /// assert_eq!(target.comments().len(), 1);
    /// assert_eq!(target.comments()[0], "#N foo".to_string());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls name() twice or more will fail at compile time.  For example:
    ///
    /// ```compile_fail
    /// use life_backend::format::RleBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<RleBuilder>().name("foo").name("bar").build()?; // Compile error
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// build() returns an error if the string passed by name(str) includes multiple lines.  For example:
    ///
    /// ```should_panic
    /// use life_backend::format::RleBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<RleBuilder>().name("foo\nbar").build()?; // Should fail
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn name(self, str: &str) -> RleBuilder<RleBuilderWithName, Created, Comment, Rule> {
        let name = RleBuilderWithName(str.to_string());
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
    /// Set the information when and by whom the pattern was created. If the argument includes newlines, the instance of Rle built by build() includes multiple comment lines.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<RleBuilder>().created("foo").build()?;
    /// assert_eq!(target.comments().len(), 1);
    /// assert_eq!(target.comments()[0], "#O foo".to_string());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls created() twice or more will fail at compile time.  For example:
    ///
    /// ```compile_fail
    /// use life_backend::format::RleBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<RleBuilder>().created("foo").created("bar").build()?; // Compile error
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn created(self, str: &str) -> RleBuilder<Name, RleBuilderWithCreated, Comment, Rule> {
        let created = RleBuilderWithCreated(str.to_string());
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
    /// Set the comment. If the argument includes newlines, the instance of Rle built by build() includes multiple comment lines.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<RleBuilder>().comment("comment0\ncomment1").build()?;
    /// assert_eq!(target.comments().len(), 2);
    /// assert_eq!(target.comments()[0], "#C comment0".to_string());
    /// assert_eq!(target.comments()[1], "#C comment1".to_string());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls comment() twice or more will fail at compile time.  For example:
    ///
    /// ```compile_fail
    /// use life_backend::format::RleBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<RleBuilder>().comment("comment0").comment("comment1").build()?; // Compile error
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn comment(self, str: &str) -> RleBuilder<Name, Created, RleBuilderWithComment, Rule> {
        let comment = RleBuilderWithComment(str.to_string());
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
    /// use life_backend::Rule;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<RleBuilder>().rule(Rule::conways_life()).build()?;
    /// assert_eq!(*target.rule(), Rule::conways_life());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls rule() twice or more will fail at compile time.  For example:
    ///
    /// ```compile_fail
    /// use life_backend::format::RleBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [(1, 0), (0, 1)];
    /// let target = pattern.iter().collect::<RleBuilder>().rule(Rule::conways_life()).rule(Rule::highlife()).build()?; // Compile error
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

impl<'a> FromIterator<&'a (usize, usize)> for RleBuilder<RleBuilderNoName, RleBuilderNoCreated, RleBuilderNoComment, RleBuilderNoRule> {
    /// Conversion from a non-owning iterator over a series of &(usize, usize).
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [(1, 0), (0, 1)];
    /// let builder = pattern.iter().collect::<RleBuilder>();
    /// let target = builder.build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a (usize, usize)>,
    {
        let contents = iter.into_iter().copied().collect();
        Self {
            name: RleBuilderNoName,
            created: RleBuilderNoCreated,
            comment: RleBuilderNoComment,
            rule: RleBuilderNoRule,
            contents,
        }
    }
}

impl FromIterator<(usize, usize)> for RleBuilder<RleBuilderNoName, RleBuilderNoCreated, RleBuilderNoComment, RleBuilderNoRule> {
    /// Conversion from an owning iterator over a series of (usize, usize).
    /// Each item in the series represents a moved live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::format::RleBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = [(1, 0), (0, 1)];
    /// let builder = pattern.into_iter().collect::<RleBuilder>();
    /// let target = builder.build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (usize, usize)>,
    {
        let contents = iter.into_iter().collect();
        Self {
            name: RleBuilderNoName,
            created: RleBuilderNoCreated,
            comment: RleBuilderNoComment,
            rule: RleBuilderNoRule,
            contents,
        }
    }
}
