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
    contents: Vec<RleLiveCellRun>,
}

// Internal structs, used in Rle
#[derive(Debug, Clone)]
struct RleHeader {
    width: usize,
    height: usize,
}
#[derive(Debug, Clone)]
struct RleLiveCellRun {
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
struct RleParser {
    comments: Vec<String>,
    header: Option<RleHeader>,
    contents: Vec<(usize, RleTag)>,
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

// Traits and Types for RleBuilder's typestate
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
        let fields = {
            let mut buf = Vec::new();
            for (index, str) in line.split(',').enumerate() {
                ensure!(index <= 2, "Too many fields in the header line");
                let Some((name, val_str)) = str.find('=').map(|pos| (str[..pos].trim(), str[(pos + 1)..].trim())) else {
                    bail!("Parse error in the header line");
                };
                buf.push((name, val_str));
            }
            buf
        };
        ensure!(fields.len() >= 2, "Too few fields in the header line");
        let width = {
            const EXPECTED_NAME: &str = "x";
            let (name, val_str) = fields[0];
            ensure!(name == EXPECTED_NAME, format!("1st variable in the header line is not \"{EXPECTED_NAME}\""));
            let Ok(n) = val_str.parse::<usize>() else {
                bail!(format!("Invalid {EXPECTED_NAME} value"));
            };
            n
        };
        let height = {
            const EXPECTED_NAME: &str = "y";
            let (name, val_str) = fields[1];
            ensure!(name == EXPECTED_NAME, format!("2nd variable in the header line is not \"{EXPECTED_NAME}\""));
            let Ok(n) = val_str.parse::<usize>() else {
                bail!(format!("Invalid {EXPECTED_NAME} value"));
            };
            n
        };
        if fields.len() > 2 {
            const EXPECTED_NAME: &str = "rule";
            let (name, _) = fields[2];
            ensure!(name == EXPECTED_NAME, format!("3rd variable in the header line is not \"{EXPECTED_NAME}\""));
            // TODO: rule parser is not implemented yet
        }
        Ok(RleHeader { width, height })
    }
    fn parse_content_line(mut line: &str) -> Result<(Vec<(usize, RleTag)>, bool)> {
        let mut buf = Vec::new();
        let mut finished = false;
        loop {
            line = line.trim_start();
            let run_count = match line.chars().next() {
                Some(c) if c.is_ascii_digit() => {
                    let (num_str, rest) = line.split_at(line.find(|c: char| !c.is_ascii_digit()).unwrap_or(line.len()));
                    ensure!(!rest.is_empty(), "The pattern is in wrong format");
                    let num: usize = num_str.parse().unwrap_or_default();
                    line = rest;
                    Some(num)
                }
                _ => None,
            };
            let tag = match line.chars().next() {
                Some('b') => RleTag::DeadCell,
                Some('$') => RleTag::EndOfLine,
                Some('!') => {
                    if run_count.is_none() {
                        finished = true;
                        break;
                    } else {
                        bail!("The pattern is in wrong format");
                    }
                }
                Some('o') | Some(_) => RleTag::AliveCell,
                None => {
                    if run_count.is_none() {
                        break;
                    } else {
                        bail!("The pattern is in wrong format");
                    }
                }
            };
            let run_count = run_count.unwrap_or(1);
            buf.push((run_count, tag));
            line = &line[1..];
        }
        Ok((buf, finished))
    }
    fn advanced_position(header: &RleHeader, current_position: (usize, usize), contents_to_be_append: &[(usize, RleTag)]) -> Result<(usize, usize)> {
        if !contents_to_be_append.is_empty() {
            ensure!(header.height > 0, "The pattern exceeds specified height"); // this check is required for the header with "y = 0"
        }
        let (mut x, mut y) = current_position;
        for (count, tag) in contents_to_be_append {
            if matches!(tag, RleTag::EndOfLine) {
                y += count;
                ensure!(y < header.height, "The pattern exceeds specified height");
                x = 0;
            } else {
                x += count;
                ensure!(x <= header.width, "The pattern exceeds specified width");
            }
        }
        Ok((x, y))
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
        if !self.finished {
            if let Some(header) = &self.header {
                let (mut contents, finished) = Self::parse_content_line(line)?;
                let advanced_position = Self::advanced_position(header, self.position, &contents)?;
                self.contents.append(&mut contents);
                self.position = advanced_position;
                self.finished = finished;
            } else {
                if Self::is_comment_line(line) {
                    self.comments.push(line.to_string());
                    return Ok(());
                }
                let header = Self::parse_header_line(line)?;
                self.header = Some(header);
            }
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
                let prefixed_str = |str: &str, prefix| {
                    let mut buf = String::from(prefix);
                    if !str.is_empty() {
                        buf.push(' ');
                        buf.push_str(str);
                    }
                    buf
                };
                match str {
                    Some(str) => {
                        if str.is_empty() {
                            vec![String::from(prefix)]
                        } else {
                            str.lines().map(|s| prefixed_str(s, prefix)).collect::<Vec<_>>()
                        }
                    }
                    None => Vec::new(),
                }
            };
            let mut buf = Vec::new();
            {
                let name = self.name.drain();
                if let Some(str) = &name {
                    ensure!(str.lines().count() <= 1, "the string passed by name() includes multiple lines");
                }
                buf.append(&mut parse_to_comments(name, "#N"));
            }
            buf.append(&mut parse_to_comments(self.created.drain(), "#O"));
            buf.append(&mut parse_to_comments(self.comment.drain(), "#C"));
            buf
        };
        let contents_group_by_y = self.contents.into_iter().fold(HashMap::new(), |mut acc, (x, y)| {
            acc.entry(y).or_insert_with(Vec::new).push(x);
            acc
        });
        let contents_sorted = {
            let mut contents_sorted: Vec<_> = contents_group_by_y.into_iter().collect();
            contents_sorted.sort_by(|(y0, _), (y1, _)| y0.partial_cmp(y1).unwrap()); // note: this unwrap never panic because <usize>.partial_cmp(<usize>) always returns Some(_)
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
            let flush_to_buf = |buf: &mut Vec<RleLiveCellRun>, (prev_x, prev_y), (curr_x, curr_y), live_cells| {
                if live_cells > 0 {
                    let pad_lines = curr_y - prev_y;
                    let pad_dead_cells = if pad_lines > 0 { curr_x } else { curr_x - prev_x };
                    buf.push(RleLiveCellRun {
                        pad_lines,
                        pad_dead_cells,
                        live_cells,
                    })
                };
            };
            let mut buf = Vec::new();
            let (mut prev_x, mut prev_y) = (0, 0);
            let (mut curr_x, mut curr_y) = (0, 0);
            let mut live_cells = 0;
            for (next_x, next_y) in contents_sorted.into_iter().flat_map(|(y, xs)| xs.into_iter().map(move |x| (x, y))) {
                if next_y > curr_y || next_x > curr_x + 2 {
                    flush_to_buf(&mut buf, (prev_x, prev_y), (curr_x, curr_y), live_cells);
                    (prev_x, prev_y) = (curr_x + live_cells, curr_y);
                    (curr_x, curr_y) = (next_x, next_y);
                    live_cells = 1;
                } else {
                    live_cells += 1;
                }
            }
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
    /// assert_eq!(rle.comments()[0], String::from("#N foo"));
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
    /// Set the information when and whom the pattern was created. If the argument includes newlines, the instance of Rle built by build() includes multiple comment lines.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::RleBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let rle = pattern.iter().collect::<RleBuilder>().created("foo").build().unwrap();
    /// assert_eq!(rle.comments().len(), 1);
    /// assert_eq!(rle.comments()[0], String::from("#O foo"));
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
    /// let rle = pattern.iter().collect::<RleBuilder>().comment("foo").build().unwrap();
    /// assert_eq!(rle.comments().len(), 1);
    /// assert_eq!(rle.comments()[0], String::from("#C foo"));
    /// ```
    ///
    /// # Errors
    ///
    /// Code that calls comment() twice or more will fail at compile time.  For example:
    ///
    /// ```compile_fail
    /// # use life_backend::format::RleBuilder;
    /// let pattern = [(1, 0), (0, 1)];
    /// let rle = pattern.iter().collect::<RleBuilder>().comment("foo").comment("bar").build().unwrap(); // Compile error
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
    // Convert the series of (usize, RleTag) into the series of RleLiveCellRun.
    fn convert_tags_to_livecellruns(tags: &[(usize, RleTag)]) -> Vec<RleLiveCellRun> {
        let mut buf = Vec::new();
        let mut item = RleLiveCellRun {
            pad_lines: 0,
            pad_dead_cells: 0,
            live_cells: 0,
        };
        for tag in tags {
            match *tag {
                (n, RleTag::AliveCell) => item.live_cells += n,
                (n, RleTag::DeadCell) => {
                    if item.live_cells > 0 {
                        buf.push(item);
                        item = RleLiveCellRun {
                            pad_lines: 0,
                            pad_dead_cells: n,
                            live_cells: 0,
                        };
                    } else {
                        item.pad_dead_cells += n;
                    }
                }
                (n, RleTag::EndOfLine) => {
                    if item.live_cells > 0 {
                        buf.push(item);
                        item = RleLiveCellRun {
                            pad_lines: n,
                            pad_dead_cells: 0,
                            live_cells: 0,
                        };
                    } else {
                        item.pad_lines += n;
                        item.pad_dead_cells = 0;
                    }
                }
            }
        }
        if item.live_cells > 0 {
            buf.push(item);
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
        let contents = Self::convert_tags_to_livecellruns(&parser.contents);
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
        let count_tag_to_string = |count: usize, char| {
            let mut buf = String::new();
            if count > 1 {
                buf.push_str(&format!("{}", count));
            }
            buf.push(char);
            buf
        };
        let flush_buf = |f: &mut fmt::Formatter, buf: &mut String| {
            writeln!(f, "{buf}")?;
            Ok(())
        };
        let write_with_buf = |f: &mut fmt::Formatter, buf: &mut String, s: String| {
            if buf.len() + s.len() > MAX_LINE_WIDTH {
                flush_buf(f, buf)?;
                buf.clear();
            }
            buf.push_str(&s);
            Ok(())
        };
        for line in self.comments() {
            writeln!(f, "{line}")?;
        }
        writeln!(f, "x = {}, y = {}", self.header.width, self.header.height)?;
        let mut buf = String::new();
        for x in &self.contents {
            for (count, char) in [(x.pad_lines, '$'), (x.pad_dead_cells, 'b'), (x.live_cells, 'o')] {
                if count > 0 {
                    let s = count_tag_to_string(count, char);
                    write_with_buf(f, &mut buf, s)?;
                }
            }
        }
        write_with_buf(f, &mut buf, String::from("!"))?;
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
        let pattern = concat!("x = 1, y = 1\n", "2\n", "!\n");
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
    fn test_display_max_width() -> Result<()> {
        let pattern = ["x = 72, y = 1", &"bo".repeat(35), "bo!"]
            .iter()
            .map(|&s| String::from(s) + "\n")
            .collect::<String>();
        let target = Rle::new(pattern.as_bytes())?;
        assert_eq!(target.to_string(), pattern);
        Ok(())
    }
}
