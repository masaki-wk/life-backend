use anyhow::{bail, ensure, Result};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::{BufRead as _, BufReader, Read};

/// A representation for RLE file format.
///
/// The detail of this format is described in:
///
/// - [Run Length Encoded - LifeWiki](https://conwaylife.com/wiki/Run_Length_Encoded)
/// - [Golly Help: File Formats > Extended RLE format](https://golly.sourceforge.net/Help/formats.html#rle)
///
#[derive(Debug, Clone)]
pub struct Rle {
    comments: Vec<String>,
    header: RleHeader,
    contents: Vec<RleRunsTriple>,
}

// Internal structs, used in Rle
#[derive(Debug, Clone)]
struct RleHeader {
    width: usize,
    height: usize,
}
#[derive(Debug, Clone)]
struct RleRunsTriple {
    pad_lines: usize,
    pad_dead_cells: usize,
    live_cells: usize,
}

// Internal structs, used during constructing of Rle
enum RleTag {
    DeadCell,
    AliveCell,
    EndOfLine,
}
struct RleRun(usize, RleTag);
struct RleParser {
    comments: Vec<String>,
    header: Option<RleHeader>,
    contents: Vec<RleRun>,
    position: (usize, usize),
    finished: bool,
}

/// A builder of Rle.
#[derive(Debug, Clone)]
pub struct RleBuilder<Name = RleBuilderNoName, Created = RleBuilderNoCreated, Comment = RleBuilderNoComment>
where
    Name: RleBuilderName,
    Created: RleBuilderCreated,
    Comment: RleBuilderComment,
{
    name: Name,
    created: Created,
    comment: Comment,
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

// Inherent methods of RleParser

impl RleParser {
    fn is_comment_line(line: &str) -> bool {
        matches!(line.chars().next(), Some('#') | None)
    }
    fn parse_header_line(line: &str) -> Result<RleHeader> {
        let check_variable_name = |expected_name, name, label| {
            ensure!(name == expected_name, format!("{label} variable in the header line is not \"{expected_name}\""));
            Ok(())
        };
        let parse_number = |(name, val_str): (&str, &str)| {
            let Ok(n) = val_str.parse() else {
                bail!(format!("Invalid {name} value"));
            };
            Ok(n)
        };
        let fields = line
            .split(',')
            .enumerate()
            .map(|(index, str)| {
                ensure!(index <= 2, "Too many fields in the header line");
                let Some((name, val_str)) = str.find('=').map(|pos| (str[..pos].trim(), str[(pos + 1)..].trim())) else {
                    bail!("Parse error in the header line");
                };
                Ok((name, val_str))
            })
            .collect::<Result<Vec<_>>>()?;
        ensure!(fields.len() >= 2, "Too few fields in the header line");
        check_variable_name("x", fields[0].0, "1st")?;
        let width = parse_number(fields[0])?;
        check_variable_name("y", fields[1].0, "2nd")?;
        let height = parse_number(fields[1])?;
        if fields.len() > 2 {
            check_variable_name("rule", fields[2].0, "3rd")?;
            // TODO: rule parser is not implemented yet
        }
        Ok(RleHeader { width, height })
    }
    fn parse_content_line(mut line: &str) -> Result<(Vec<RleRun>, bool)> {
        let mut buf = Vec::new();
        let terminated = loop {
            let (run_count_str, tag_char, line_remain) = {
                let line_remain = line.trim_start();
                let (run_count_str, line_remain) = line_remain.split_at(line_remain.find(|c: char| !c.is_ascii_digit()).unwrap_or(line_remain.len()));
                let Some(tag_char) = line_remain.chars().next() else {
                    ensure!(run_count_str.is_empty(), "The pattern is in wrong format");
                    break false;
                };
                (run_count_str, tag_char, &line_remain[1..])
            };
            let run_count = if !run_count_str.is_empty() {
                Some(run_count_str.parse().unwrap()) // this unwrap never panic because num_str only includes ascii digits
            } else {
                None
            };
            let tag = match tag_char {
                '!' => {
                    ensure!(run_count.is_none(), "The pattern is in wrong format");
                    break true;
                }
                'o' => RleTag::AliveCell,
                'b' => RleTag::DeadCell,
                '$' => RleTag::EndOfLine,
                c => {
                    ensure!(!c.is_whitespace(), "The pattern is in wrong format");
                    RleTag::AliveCell
                }
            };
            buf.push(RleRun(run_count.unwrap_or(1), tag));
            line = line_remain;
        };
        Ok((buf, terminated))
    }
    fn advanced_position(header: &RleHeader, current_position: (usize, usize), contents_to_be_append: &[RleRun]) -> Result<(usize, usize)> {
        ensure!(contents_to_be_append.is_empty() || header.height > 0, "The pattern exceeds specified height"); // this check is required for the header with "y = 0"
        contents_to_be_append.iter().try_fold(current_position, |(curr_x, curr_y), RleRun(count, tag)| {
            if matches!(tag, RleTag::EndOfLine) {
                let next_y = curr_y + count;
                ensure!(next_y < header.height, "The pattern exceeds specified height");
                Ok((0, next_y))
            } else {
                let next_x = curr_x + count;
                ensure!(next_x <= header.width, "The pattern exceeds specified width");
                Ok((next_x, curr_y))
            }
        })
    }
    fn new() -> Self {
        Self {
            comments: Vec::new(),
            header: None,
            contents: Vec::new(),
            position: (0, 0),
            finished: false,
        }
    }
    fn push(&mut self, line: &str) -> Result<()> {
        if let Some(header) = &self.header {
            if !self.finished {
                let (contents, terminated) = Self::parse_content_line(line)?;
                let advanced_position = Self::advanced_position(header, self.position, &contents)?;
                self.contents.extend(contents.into_iter());
                self.position = advanced_position;
                self.finished = terminated;
            }
        } else if Self::is_comment_line(line) {
            self.comments.push(line.to_string());
        } else {
            let header = Self::parse_header_line(line)?;
            self.header = Some(header);
        }
        Ok(())
    }
}

// Inherent methods of RleBuilder

impl<Name, Created, Comment> RleBuilder<Name, Created, Comment>
where
    Name: RleBuilderName,
    Created: RleBuilderCreated,
    Comment: RleBuilderComment,
{
    /// Builds the Rle.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::RleBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let rle = pattern.iter().collect::<RleBuilder>().build().unwrap();
    /// ```
    ///
    pub fn build(self) -> Result<Rle> {
        let comments = {
            let parse_to_comments = |str: Option<String>, prefix: &str| {
                let prefixed_str = |str: &str, prefix: &str| {
                    let mut buf = prefix.to_string();
                    if !str.is_empty() {
                        buf.push(' ');
                        buf.push_str(str);
                    }
                    buf
                };
                match str {
                    Some(str) => {
                        if str.is_empty() {
                            vec![prefix.to_string()]
                        } else {
                            str.lines().map(|s| prefixed_str(s, prefix)).collect::<Vec<_>>()
                        }
                    }
                    None => Vec::new(),
                }
            };
            let name = self.name.drain();
            if let Some(str) = &name {
                ensure!(str.lines().count() <= 1, "the string passed by name(str) includes multiple lines");
            }
            [(name, "#N"), (self.created.drain(), "#O"), (self.comment.drain(), "#C")]
                .into_iter()
                .flat_map(|(str, prefix)| parse_to_comments(str, prefix).into_iter())
                .collect::<Vec<_>>()
        };
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
            RleHeader { width, height }
        };
        let contents = {
            let flush_to_buf = |buf: &mut Vec<RleRunsTriple>, (prev_x, prev_y), (curr_x, curr_y), live_cells| {
                if live_cells > 0 {
                    let pad_lines = curr_y - prev_y;
                    let pad_dead_cells = if pad_lines > 0 { curr_x } else { curr_x - prev_x };
                    buf.push(RleRunsTriple {
                        pad_lines,
                        pad_dead_cells,
                        live_cells,
                    })
                };
            };
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
        Ok(Rle { comments, header, contents })
    }
}

impl<Created, Comment> RleBuilder<RleBuilderNoName, Created, Comment>
where
    Created: RleBuilderCreated,
    Comment: RleBuilderComment,
{
    /// Set the name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::RleBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let rle = pattern.iter().collect::<RleBuilder>().name("foo").build().unwrap();
    /// assert_eq!(rle.comments().len(), 1);
    /// assert_eq!(rle.comments()[0], "#N foo".to_string());
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls name() twice or more will fail at compile time.  For example:
    ///
    /// ```compile_fail
    /// # use life_backend::format::RleBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let rle = pattern.iter().collect::<RleBuilder>().name("foo").name("bar").build().unwrap(); // Compile error
    /// ```
    ///
    /// build() returns an error if the string passed by name(str) includes multiple lines.  For example:
    ///
    /// ```should_panic
    /// # use life_backend::format::RleBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let rle = pattern.iter().collect::<RleBuilder>().name("foo\nbar").build().unwrap();
    /// ```
    ///
    pub fn name(self, str: &str) -> RleBuilder<RleBuilderWithName, Created, Comment> {
        let name = RleBuilderWithName(str.to_string());
        RleBuilder::<RleBuilderWithName, Created, Comment> {
            name,
            created: self.created,
            comment: self.comment,
            contents: self.contents,
        }
    }
}

impl<Name, Comment> RleBuilder<Name, RleBuilderNoCreated, Comment>
where
    Name: RleBuilderName,
    Comment: RleBuilderComment,
{
    /// Set the information when and by whom the pattern was created. If the argument includes newlines, the instance of Rle built by build() includes multiple comment lines.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::RleBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let rle = pattern.iter().collect::<RleBuilder>().created("foo").build().unwrap();
    /// assert_eq!(rle.comments().len(), 1);
    /// assert_eq!(rle.comments()[0], "#O foo".to_string());
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls created() twice or more will fail at compile time.  For example:
    ///
    /// ```compile_fail
    /// # use life_backend::format::RleBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let rle = pattern.iter().collect::<RleBuilder>().created("foo").created("bar").build().unwrap(); // Compile error
    /// ```
    ///
    pub fn created(self, str: &str) -> RleBuilder<Name, RleBuilderWithCreated, Comment> {
        let created = RleBuilderWithCreated(str.to_string());
        RleBuilder::<Name, RleBuilderWithCreated, Comment> {
            name: self.name,
            created,
            comment: self.comment,
            contents: self.contents,
        }
    }
}

impl<Name, Created> RleBuilder<Name, Created, RleBuilderNoComment>
where
    Name: RleBuilderName,
    Created: RleBuilderCreated,
{
    /// Set the comment. If the argument includes newlines, the instance of Rle built by build() includes multiple comment lines.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::RleBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let rle = pattern.iter().collect::<RleBuilder>().comment("comment0\ncomment1").build().unwrap();
    /// assert_eq!(rle.comments().len(), 2);
    /// assert_eq!(rle.comments()[0], "#C comment0".to_string());
    /// assert_eq!(rle.comments()[1], "#C comment1".to_string());
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls comment() twice or more will fail at compile time.  For example:
    ///
    /// ```compile_fail
    /// # use life_backend::format::RleBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let rle = pattern.iter().collect::<RleBuilder>().comment("comment0").comment("comment1").build().unwrap(); // Compile error
    /// ```
    ///
    pub fn comment(self, str: &str) -> RleBuilder<Name, Created, RleBuilderWithComment> {
        let comment = RleBuilderWithComment(str.to_string());
        RleBuilder::<Name, Created, RleBuilderWithComment> {
            name: self.name,
            created: self.created,
            comment,
            contents: self.contents,
        }
    }
}

// Trait implementations of RleBuilder

impl<'a> FromIterator<&'a (usize, usize)> for RleBuilder<RleBuilderNoName, RleBuilderNoCreated, RleBuilderNoComment> {
    /// Conversion from a non-owning iterator over a series of &(usize, usize).
    /// Each item in the series represents an immutable reference of a live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::RleBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let builder = pattern.iter().collect::<RleBuilder>();
    /// let rle = builder.build().unwrap();
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
            contents,
        }
    }
}

impl FromIterator<(usize, usize)> for RleBuilder<RleBuilderNoName, RleBuilderNoCreated, RleBuilderNoComment> {
    /// Conversion from an owning iterator over a series of (usize, usize).
    /// Each item in the series represents a moved live cell position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::RleBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let builder = pattern.into_iter().collect::<RleBuilder>();
    /// let rle = builder.build().unwrap();
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
            contents,
        }
    }
}

// Inherent methods of Rle

impl Rle {
    // Convert the series of (usize, RleTag) into the series of RleRunsTriple.
    fn convert_runs_to_triples(runs: &[RleRun]) -> Vec<RleRunsTriple> {
        const TRIPLE_ZERO: RleRunsTriple = RleRunsTriple {
            pad_lines: 0,
            pad_dead_cells: 0,
            live_cells: 0,
        };
        let (mut buf, triple) = runs.iter().fold((Vec::new(), TRIPLE_ZERO), |(mut buf, curr_triple), run| {
            let mut next_triple = if curr_triple.live_cells > 0 && !matches!(run, RleRun(_, RleTag::AliveCell)) {
                buf.push(curr_triple);
                TRIPLE_ZERO
            } else {
                curr_triple
            };
            match run {
                RleRun(n, RleTag::AliveCell) => next_triple.live_cells += n,
                RleRun(n, RleTag::DeadCell) => {
                    next_triple.pad_dead_cells += n;
                }
                RleRun(n, RleTag::EndOfLine) => {
                    next_triple.pad_lines += n;
                    next_triple.pad_dead_cells = 0;
                }
            }
            (buf, next_triple)
        });
        if triple.live_cells > 0 {
            buf.push(triple);
        }
        buf
    }

    /// Creates from the specified implementor of Read, such as File or `&[u8]`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Rle;
    /// let pattern = "\
    ///     #N T-tetromino\n\
    ///     x = 3, y = 2\n\
    ///     3o$bo!\n\
    /// ";
    /// let parser = Rle::new(pattern.as_bytes()).unwrap();
    /// ```
    ///
    pub fn new<R>(read: R) -> Result<Self>
    where
        R: Read,
    {
        let parser = {
            let mut buf = RleParser::new();
            for line in BufReader::new(read).lines() {
                let line = line?;
                buf.push(&line)?;
            }
            buf
        };
        let Some(header) = parser.header else {
            bail!("Header line not found in the pattern");
        };
        ensure!(parser.finished, "The terminal symbol not found");
        let contents = Self::convert_runs_to_triples(&parser.contents);
        Ok(Self {
            comments: parser.comments,
            header,
            contents,
        })
    }

    /// Returns comments of the pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Rle;
    /// let pattern = "\
    ///     #N T-tetromino\n\
    ///     x = 3, y = 2\n\
    ///     3o$bo!\n\
    /// ";
    /// let parser = Rle::new(pattern.as_bytes()).unwrap();
    /// assert_eq!(parser.comments().len(), 1);
    /// assert_eq!(parser.comments()[0], "#N T-tetromino");
    /// ```
    ///
    #[inline]
    pub fn comments(&self) -> &Vec<String> {
        &self.comments
    }

    /// Returns the width written in the pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Rle;
    /// let pattern = "\
    ///     #N T-tetromino\n\
    ///     x = 3, y = 2\n\
    ///     3o$bo!\n\
    /// ";
    /// let parser = Rle::new(pattern.as_bytes()).unwrap();
    /// assert_eq!(parser.width(), 3);
    /// ```
    ///
    #[inline]
    pub fn width(&self) -> usize {
        self.header.width
    }

    /// Returns the height written in the pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Rle;
    /// let pattern = "\
    ///     #N T-tetromino\n\
    ///     x = 3, y = 2\n\
    ///     3o$bo!\n\
    /// ";
    /// let parser = Rle::new(pattern.as_bytes()).unwrap();
    /// assert_eq!(parser.height(), 2);
    /// ```
    ///
    #[inline]
    pub fn height(&self) -> usize {
        self.header.height
    }

    /// Creates a non-owning iterator over the series of immutable live cell positions in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Rle;
    /// let pattern = "\
    ///     #N T-tetromino\n\
    ///     x = 3, y = 2\n\
    ///     3o$bo!\n\
    /// ";
    /// let parser = Rle::new(pattern.as_bytes()).unwrap();
    /// let mut iter = parser.iter();
    /// assert_eq!(iter.next(), Some((0, 0)));
    /// assert_eq!(iter.next(), Some((1, 0)));
    /// assert_eq!(iter.next(), Some((2, 0)));
    /// assert_eq!(iter.next(), Some((1, 1)));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.contents
            .iter()
            .scan((0, 0), |(state_x, state_y), item| {
                if item.pad_lines > 0 {
                    *state_y += item.pad_lines;
                    *state_x = 0;
                }
                if item.pad_dead_cells > 0 {
                    *state_x += item.pad_dead_cells;
                }
                let output = (*state_y, *state_x, item.live_cells);
                *state_x += item.live_cells;
                Some(output)
            })
            .flat_map(|(y, x, num)| (x..(x + num)).map(move |x| (x, y)))
    }
}

// Trait implementations

impl fmt::Display for Rle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const MAX_LINE_WIDTH: usize = 70;
        let convert_count_tag_to_string = |run_count: usize, tag_char| {
            if run_count > 1 {
                let mut buf = run_count.to_string();
                buf.push(tag_char);
                buf
            } else {
                tag_char.to_string()
            }
        };
        let flush_buf = |f: &mut fmt::Formatter, buf: &mut String| {
            writeln!(f, "{buf}")?;
            Ok(())
        };
        let write_with_buf = |f: &mut fmt::Formatter, buf: &mut String, s: &str| {
            if buf.len() + s.len() > MAX_LINE_WIDTH {
                flush_buf(f, buf)?;
                buf.clear();
            }
            *buf += s;
            Ok(())
        };
        for line in self.comments() {
            writeln!(f, "{line}")?;
        }
        writeln!(f, "x = {}, y = {}", self.width(), self.height())?;
        let mut buf = String::new();
        for x in &self.contents {
            for (run_count, tag_char) in [(x.pad_lines, '$'), (x.pad_dead_cells, 'b'), (x.live_cells, 'o')] {
                if run_count > 0 {
                    let s = convert_count_tag_to_string(run_count, tag_char);
                    write_with_buf(f, &mut buf, &s)?;
                }
            }
        }
        write_with_buf(f, &mut buf, "!")?;
        flush_buf(f, &mut buf)?;
        Ok(())
    }
}

// Unit tests

#[cfg(test)]
mod tests {
    use super::*;
    fn do_check(target: &Rle, expected_comments: &[&str], expected_contents: &[(usize, usize, usize)], expected_pattern: Option<&str>) {
        assert_eq!(target.comments().len(), expected_comments.len());
        for (result, expected) in target.comments().iter().zip(expected_comments.iter()) {
            assert_eq!(result, expected);
        }
        assert_eq!(target.contents.len(), expected_contents.len());
        for (result, &expected) in target.contents.iter().zip(expected_contents.iter()) {
            assert_eq!((result.pad_lines, result.pad_dead_cells, result.live_cells), expected);
        }
        if let Some(expected_pattern) = expected_pattern {
            assert_eq!(target.to_string(), expected_pattern);
        }
    }
    fn do_new_test_to_be_passed(pattern: &str, expected_comments: &[&str], expected_contents: &[(usize, usize, usize)], check_tostring: bool) -> Result<()> {
        let target = Rle::new(pattern.as_bytes())?;
        do_check(&target, expected_comments, expected_contents, if check_tostring { Some(pattern) } else { None });
        Ok(())
    }
    fn do_new_test_to_be_failed(pattern: &str) {
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_header() -> Result<()> {
        let pattern = concat!("x = 0, y = 0\n", "!\n");
        let expected_comments = Vec::new();
        let expected_contents = Vec::new();
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, true)
    }
    #[test]
    fn test_new_comment_header() -> Result<()> {
        let pattern = concat!("#comment\n", "x = 0, y = 0\n", "!\n");
        let expected_comments = vec!["#comment"];
        let expected_contents = Vec::new();
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, true)
    }
    #[test]
    fn test_new_comments_header() -> Result<()> {
        let pattern = concat!("#comment0\n", "#comment1\n", "x = 0, y = 0\n", "!\n");
        let expected_comments = vec!["#comment0", "#comment1"];
        let expected_contents = Vec::new();
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, true)
    }
    #[test]
    fn test_new_comments_with_blank_header() -> Result<()> {
        let pattern = concat!("#comment\n", "\n", "x = 0, y = 0\n", "!\n");
        let expected_comments = vec!["#comment", ""];
        let expected_contents = Vec::new();
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, true)
    }
    #[test]
    fn test_new_header_content() -> Result<()> {
        let pattern = concat!("x = 1, y = 1\n", "o!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, true)
    }
    #[test]
    fn test_new_header_contents() -> Result<()> {
        let pattern = concat!("x = 2, y = 2\n", "o$bo!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1), (1, 1, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, true)
    }
    #[test]
    fn test_new_comments_header_contents() -> Result<()> {
        let pattern = concat!("#comment0\n", "#comment1\n", "x = 2, y = 2\n", "o$bo!\n");
        let expected_comments = vec!["#comment0", "#comment1"];
        let expected_contents = vec![(0, 0, 1), (1, 1, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, true)
    }
    #[test]
    fn test_new_empty() {
        let pattern = "";
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_header_invalid_format() {
        let pattern = "_\n";
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_header_unknown_variable() {
        let pattern = "z = 0\n";
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_header_invalid_width() {
        let pattern = "x = _, y = 0\n";
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_header_invalid_height() {
        let pattern = "x = 0, y = _\n";
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_header_without_width() {
        let pattern = "y = 0\n";
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_header_without_height() {
        let pattern = "x = 0\n";
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_header_exceed_width() {
        let pattern = concat!("x = 0, y = 1\n", "o!\n");
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_header_exceed_height() {
        let pattern = concat!("x = 1, y = 0\n", "o!\n");
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_header_larger_width() -> Result<()> {
        let pattern = concat!("x = 2, y = 1\n", "o!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, true)
    }
    #[test]
    fn test_new_header_larger_height() -> Result<()> {
        let pattern = concat!("x = 1, y = 2\n", "o!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, true)
    }
    #[test]
    fn test_new_content_acceptable_tag_without_count() -> Result<()> {
        let pattern = concat!("x = 1, y = 1\n", "_!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_new_content_acceptable_tag_with_count() -> Result<()> {
        let pattern = concat!("x = 2, y = 1\n", "2_!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 2)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_new_content_alone_count() {
        let pattern = concat!("x = 1, y = 1\n", "1\n", "!\n");
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_content_count_with_whitespace() {
        let pattern = concat!("x = 1, y = 1\n", "1 \n", "!\n");
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_content_without_terminator() {
        let pattern = concat!("x = 1, y = 1\n", "o\n");
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_content_terminator_with_count() {
        let pattern = concat!("x = 1, y = 1\n", "2!\n");
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_content_exceeds_width_with_dead_cell() {
        let pattern = concat!("x = 1, y = 1\n", "ob!\n");
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_content_exceeds_width_with_dead_cells() {
        let pattern = concat!("x = 2, y = 2\n", "2o$o2b!\n");
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_content_exceeds_height_with_end_of_line() {
        let pattern = concat!("x = 1, y = 1\n", "o$!\n");
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_content_exceeds_height_with_end_of_lines() {
        let pattern = concat!("x = 1, y = 2\n", "o2$!\n");
        do_new_test_to_be_failed(pattern)
    }
    #[test]
    fn test_new_nonoptimal_dead_cells() -> Result<()> {
        let pattern = concat!("x = 4, y = 1\n", "bbbo!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 3, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_new_nonoptimal_live_cells() -> Result<()> {
        let pattern = concat!("x = 3, y = 1\n", "ooo!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 3)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_new_nonoptimal_end_of_lines() -> Result<()> {
        let pattern = concat!("x = 1, y = 4\n", "$$$o!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(3, 0, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_new_nonoptimal_line_end_dead_cell() -> Result<()> {
        let pattern = concat!("x = 1, y = 2\n", "b$o!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(1, 0, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_new_nonoptimal_line_end_dead_cells() -> Result<()> {
        let pattern = concat!("x = 2, y = 2\n", "2b$2o!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(1, 0, 2)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_new_nonoptimal_trailing_dead_cell() -> Result<()> {
        let pattern = concat!("x = 2, y = 2\n", "2o$ob!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 2), (1, 0, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_new_nonoptimal_trailing_dead_cells() -> Result<()> {
        let pattern = concat!("x = 3, y = 2\n", "3o$o2b!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 3), (1, 0, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_new_nonoptimal_trailing_line_end() -> Result<()> {
        let pattern = concat!("x = 1, y = 2\n", "o$!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_new_nonoptimal_trailing_line_ends() -> Result<()> {
        let pattern = concat!("x = 1, y = 3\n", "o2$!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_new_trailing_ignored_content() -> Result<()> {
        let pattern = concat!("x = 1, y = 1\n", "o!_\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_new_trailing_ignored_line() -> Result<()> {
        let pattern = concat!("x = 1, y = 1\n", "o!\n", "ignored line\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_new_test_to_be_passed(pattern, &expected_comments, &expected_contents, false)
    }
    #[test]
    fn test_build() -> Result<()> {
        let pattern = [(0, 0), (1, 0), (2, 0), (1, 1)];
        let target = pattern.iter().collect::<RleBuilder>().build()?;
        assert_eq!(target.width(), 3);
        assert_eq!(target.height(), 2);
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 3), (1, 1, 1)];
        do_check(&target, &expected_comments, &expected_contents, None);
        Ok(())
    }
    #[test]
    fn test_build_singleline_name() -> Result<()> {
        let pattern = [(0, 0)];
        let target = pattern.iter().collect::<RleBuilder>().name("name").build()?;
        let expected_comments = vec!["#N name"];
        let expected_contents = vec![(0, 0, 1)];
        do_check(&target, &expected_comments, &expected_contents, None);
        Ok(())
    }
    #[test]
    fn test_build_blank_name() -> Result<()> {
        let pattern = [(0, 0)];
        let target = pattern.iter().collect::<RleBuilder>().name("").build()?;
        let expected_comments = vec!["#N"];
        let expected_contents = vec![(0, 0, 1)];
        do_check(&target, &expected_comments, &expected_contents, None);
        Ok(())
    }
    #[test]
    fn test_build_multiline_name() {
        let pattern = [(0, 0)];
        let target = pattern.iter().collect::<RleBuilder>().name("name\nname").build();
        assert!(target.is_err());
    }
    #[test]
    fn test_build_created() -> Result<()> {
        let pattern = [(0, 0)];
        let target = pattern.iter().collect::<RleBuilder>().created("created").build()?;
        let expected_comments = vec!["#O created"];
        let expected_contents = vec![(0, 0, 1)];
        do_check(&target, &expected_comments, &expected_contents, None);
        Ok(())
    }
    #[test]
    fn test_build_blank_created() -> Result<()> {
        let pattern = [(0, 0)];
        let target = pattern.iter().collect::<RleBuilder>().created("").build()?;
        let expected_comments = vec!["#O"];
        let expected_contents = vec![(0, 0, 1)];
        do_check(&target, &expected_comments, &expected_contents, None);
        Ok(())
    }
    #[test]
    fn test_build_createds() -> Result<()> {
        let pattern = [(0, 0)];
        let target = pattern.iter().collect::<RleBuilder>().created("created0\ncreated1").build()?;
        let expected_comments = vec!["#O created0", "#O created1"];
        let expected_contents = vec![(0, 0, 1)];
        do_check(&target, &expected_comments, &expected_contents, None);
        Ok(())
    }
    #[test]
    fn test_build_comment() -> Result<()> {
        let pattern = [(0, 0)];
        let target = pattern.iter().collect::<RleBuilder>().comment("comment").build()?;
        let expected_comments = vec!["#C comment"];
        let expected_contents = vec![(0, 0, 1)];
        do_check(&target, &expected_comments, &expected_contents, None);
        Ok(())
    }
    #[test]
    fn test_build_blank_comment() -> Result<()> {
        let pattern = [(0, 0)];
        let target = pattern.iter().collect::<RleBuilder>().comment("").build()?;
        let expected_comments = vec!["#C"];
        let expected_contents = vec![(0, 0, 1)];
        do_check(&target, &expected_comments, &expected_contents, None);
        Ok(())
    }
    #[test]
    fn test_build_comments() -> Result<()> {
        let pattern = [(0, 0)];
        let target = pattern.iter().collect::<RleBuilder>().comment("comment0\ncomment1").build()?;
        let expected_comments = vec!["#C comment0", "#C comment1"];
        let expected_contents = vec![(0, 0, 1)];
        do_check(&target, &expected_comments, &expected_contents, None);
        Ok(())
    }
    #[test]
    fn test_build_name_created_comment() -> Result<()> {
        let pattern = [(0, 0)];
        let target = pattern
            .iter()
            .collect::<RleBuilder>()
            .name("name")
            .created("created")
            .comment("comment")
            .build()?;
        let expected_comments = vec!["#N name", "#O created", "#C comment"];
        let expected_contents = vec![(0, 0, 1)];
        do_check(&target, &expected_comments, &expected_contents, None);
        Ok(())
    }
    #[test]
    fn test_display_max_width() -> Result<()> {
        let pattern = ["x = 72, y = 1", &"bo".repeat(35), "bo!"]
            .iter()
            .map(|&s| s.to_string() + "\n")
            .collect::<String>();
        let target = Rle::new(pattern.as_bytes())?;
        assert_eq!(target.to_string(), pattern);
        Ok(())
    }
}
