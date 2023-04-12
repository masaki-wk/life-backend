use anyhow::{bail, ensure, Result};
use std::fmt;
use std::io::{BufRead as _, BufReader, Read};

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
    position: (usize, usize),
    finished: bool,
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
                    let num: usize = num_str.parse().unwrap(); // this unwrap never panic because num_str only includes ascii digits
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
            if count > 1 {
                let mut buf = count.to_string();
                buf.push(char);
                buf
            } else {
                char.to_string()
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
            buf.push_str(s);
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
    fn do_new_test_to_be_passed(pattern: &str, expected_comments: &[&str], expected_contents: &[(usize, usize, usize)], check_tostring: bool) -> Result<()> {
        let target = Rle::new(pattern.as_bytes())?;
        assert_eq!(target.comments().len(), expected_comments.len());
        for (result, expected) in target.comments().iter().zip(expected_comments.iter()) {
            assert_eq!(result, expected);
        }
        assert_eq!(target.contents.len(), expected_contents.len());
        for (result, &expected) in target.contents.iter().zip(expected_contents.iter()) {
            assert_eq!((result.pad_lines, result.pad_dead_cells, result.live_cells), expected);
        }
        if check_tostring {
            assert_eq!(target.to_string(), pattern);
        }
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
