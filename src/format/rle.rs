use anyhow::{bail, ensure, Result};
use std::fmt;
use std::io::{BufRead, BufReader, Read};

/// A representation for RLE file format, described in <https://conwaylife.com/wiki/Run_Length_Encoded>.
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
    finished: bool,
}

// Inherent methods of RleParser

impl RleParser {
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
    fn parse_comment_line(line: &str) -> Option<&str> {
        Self::parse_prefixed_line("#", line)
    }
    fn parse_header_line(line: &str) -> Result<RleHeader> {
        let mut width: Option<usize> = None;
        let mut height: Option<usize> = None;
        for s in line.split(',') {
            let Some((name, val_str)) = s.find('=').map(|pos| (s[..pos].trim(), s[(pos + 1)..].trim())) else {
                bail!("Parse error in the header line");
            };
            match name {
                "x" => {
                    let Ok(n) = val_str.parse::<usize>() else {
                        bail!("Invalid x value");
                    };
                    width = Some(n);
                }
                "y" => {
                    let Ok(n) = val_str.parse::<usize>() else {
                        bail!("Invalid y value");
                    };
                    height = Some(n);
                }
                "rule" => (),
                _ => bail!(format!("The header line includes unknown variable {}", name)),
            }
        }
        let Some(width) = width else {
            bail!("Variable x not found in the header line");
        };
        let Some(height) = height else {
            bail!("Variable y not found in the header line");
        };
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
                Some('o') => RleTag::AliveCell,
                Some('$') => RleTag::EndOfLine,
                Some('!') => {
                    if run_count.is_none() {
                        finished = true;
                        break;
                    } else {
                        bail!("The pattern is in wrong format");
                    }
                }
                None => {
                    if run_count.is_none() {
                        break;
                    } else {
                        bail!("The pattern is in wrong format");
                    }
                }
                _ => {
                    bail!("The pattern is in wrong format");
                }
            };
            let run_count = run_count.unwrap_or(1);
            buf.push((run_count, tag));
            line = &line[1..];
        }
        Ok((buf, finished))
    }
    fn new() -> Self {
        Self {
            comments: Vec::new(),
            header: None,
            contents: Vec::new(),
            finished: false,
        }
    }
    fn push(&mut self, line: &str) -> Result<()> {
        if !self.finished {
            if self.header.is_none() {
                if let Some(comment) = Self::parse_comment_line(line) {
                    self.comments.push(comment.to_string());
                    return Ok(());
                }
                let header = Self::parse_header_line(line)?;
                self.header = Some(header);
            } else {
                let (mut content, finished) = Self::parse_content_line(line)?;
                self.contents.append(&mut content);
                self.finished = finished;
            }
        }
        Ok(())
    }
}

// Inherent methods

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
            match tag {
                (n, RleTag::AliveCell) => item.live_cells += *n,
                (n, RleTag::DeadCell) => {
                    if item.live_cells > 0 {
                        buf.push(item);
                        item = RleLiveCellRun {
                            pad_lines: 0,
                            pad_dead_cells: *n,
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
                            pad_lines: *n,
                            pad_dead_cells: 0,
                            live_cells: 0,
                        };
                    } else {
                        item.pad_lines += *n;
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

    // Returns (width, height) of the series of RleLiveCellRun.
    fn livecellruns_size(runs: &[RleLiveCellRun]) -> (usize, usize) {
        let (width, height, x) = runs.iter().fold(
            (0, 0, 0),
            |(mut width, mut height, mut x),
             &RleLiveCellRun {
                 pad_lines,
                 pad_dead_cells,
                 live_cells,
             }| {
                let cells = pad_dead_cells + live_cells;
                if pad_lines > 0 {
                    height += pad_lines;
                    x = cells;
                } else {
                    x += cells;
                }
                width = width.max(x);
                (width, height, x)
            },
        );
        (width, height + if x > 0 { 1 } else { 0 })
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
        let (actual_width, actual_height) = Self::livecellruns_size(&contents);
        ensure!(actual_width <= header.width, "The pattern exceeds specified width");
        ensure!(actual_height <= header.height, "The pattern exceeds specified height");
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
    /// assert_eq!(parser.comments()[0], "N T-tetromino");
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
            writeln!(f, "#{}", line)?;
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
    fn do_test(pattern: &str, expected_comments: &[&str], expected_contents: &[(usize, usize, usize)]) -> Result<()> {
        let target = Rle::new(pattern.as_bytes())?;
        assert_eq!(target.comments().len(), expected_comments.len());
        for (result, expected) in target.comments().iter().zip(expected_comments.iter()) {
            assert_eq!(result, expected);
        }
        assert_eq!(target.contents.len(), expected_contents.len());
        for (result, &expected) in target.contents.iter().zip(expected_contents.iter()) {
            assert_eq!((result.pad_lines, result.pad_dead_cells, result.live_cells), expected);
        }
        Ok(())
    }
    #[test]
    fn test_new_header() -> Result<()> {
        let pattern = concat!("x = 0, y = 0\n", "!\n");
        let expected_comments = Vec::new();
        let expected_contents = Vec::new();
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_comment_header() -> Result<()> {
        let pattern = concat!("#comment\n", "x = 0, y = 0\n", "!\n");
        let expected_comments = vec!["comment"];
        let expected_contents = Vec::new();
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_comments_header() -> Result<()> {
        let pattern = concat!("#comment0\n", "#comment1\n", "x = 0, y = 0\n", "!\n");
        let expected_comments = vec!["comment0", "comment1"];
        let expected_contents = Vec::new();
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_content() -> Result<()> {
        let pattern = concat!("x = 1, y = 1\n", "o!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_contents() -> Result<()> {
        let pattern = concat!("x = 2, y = 2\n", "o$bo!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1), (1, 1, 1)];
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_comments_header_contents() -> Result<()> {
        let pattern = concat!("#comment0\n", "#comment1\n", "x = 2, y = 2\n", "o$bo!\n");
        let expected_comments = vec!["comment0", "comment1"];
        let expected_contents = vec![(0, 0, 1), (1, 1, 1)];
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_empty() {
        let pattern = "";
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_header_invalid_format() {
        let pattern = "_\n";
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_header_unknown_variable() {
        let pattern = "z = 0\n";
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_header_invalid_width() {
        let pattern = "x = _, y = 0\n";
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_header_invalid_height() {
        let pattern = "x = 0, y = _\n";
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_header_without_width() {
        let pattern = "y = 0\n";
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_header_without_height() {
        let pattern = "x = 0\n";
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_header_exceed_width() {
        let pattern = concat!("x = 0, y = 1\n", "o!\n");
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_header_exceed_height() {
        let pattern = concat!("x = 1, y = 0\n", "o!\n");
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_header_larger_width() -> Result<()> {
        let pattern = concat!("x = 2, y = 1\n", "o!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_header_larger_height() -> Result<()> {
        let pattern = concat!("x = 1, y = 2\n", "o!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_content_invalid_tag_without_count() {
        let pattern = concat!("x = 1, y = 1\n", "_\n");
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_content_invalid_tag_with_count() {
        let pattern = concat!("x = 1, y = 1\n", "2_\n");
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_content_alone_count() {
        let pattern = concat!("x = 1, y = 1\n", "2\n", "!\n");
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_content_terminator_with_count() {
        let pattern = concat!("x = 1, y = 1\n", "2!\n");
        let target = Rle::new(pattern.as_bytes());
        assert!(target.is_err());
    }
    #[test]
    fn test_new_nonoptimal_dead_cells() -> Result<()> {
        let pattern = concat!("x = 4, y = 1\n", "bbbo!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 3, 1)];
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_nonoptimal_live_cells() -> Result<()> {
        let pattern = concat!("x = 3, y = 1\n", "ooo!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 3)];
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_nonoptimal_end_of_lines() -> Result<()> {
        let pattern = concat!("x = 1, y = 4\n", "$$$o!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(3, 0, 1)];
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_nonoptimal_line_end_dead_cells() -> Result<()> {
        let pattern = concat!("x = 1, y = 2\n", "2b$o!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(1, 0, 1)];
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_nonoptimal_trailing_dead_cells() -> Result<()> {
        let pattern = concat!("x = 1, y = 1\n", "o2b!\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_trailing_ignored_content() -> Result<()> {
        let pattern = concat!("x = 1, y = 1\n", "o!_\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_test(pattern, &expected_comments, &expected_contents)
    }
    #[test]
    fn test_new_trailing_ignored_line() -> Result<()> {
        let pattern = concat!("x = 1, y = 1\n", "o!\n", "ignored line\n");
        let expected_comments = Vec::new();
        let expected_contents = vec![(0, 0, 1)];
        do_test(pattern, &expected_comments, &expected_contents)
    }
}
