use anyhow::{bail, ensure, Result};
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
        if !contents_to_be_append.is_empty() {
            ensure!(header.height > 0, "The pattern exceeds specified height"); // this check is required for the header with "y = 0"
        }
        let (mut x, mut y) = current_position;
        for RleRun(count, tag) in contents_to_be_append {
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

// Inherent methods

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
            *buf += s;
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
