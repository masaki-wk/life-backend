use anyhow::{ensure, Context as _, Result};
use std::io::{BufRead as _, BufReader, Read};

use super::{Rle, RleHeader, RleRunsTriple};
use crate::Rule;

// The parser of RLE format, used during constructing of Rle
pub(super) struct RleParser {
    comments: Vec<String>,
    header: Option<RleHeader>,
    contents: Vec<RleRun>,
    position: (usize, usize),
    finished: bool,
}

// Internal structs
enum RleTag {
    DeadCell,
    AliveCell,
    EndOfLine,
}
struct RleRun(usize, RleTag);

// Inherent methods

impl RleParser {
    // Parses the specified implementor of Read (e.g., File, `&[u8]`) into Rle
    pub(super) fn parse<R>(read: R) -> Result<Rle>
    where
        R: Read,
    {
        let parser = BufReader::new(read).lines().try_fold(Self::new(), |mut buf, line| {
            let line = line?;
            buf.push(&line)?;
            Ok::<_, anyhow::Error>(buf)
        })?;
        ensure!(parser.finished, "The terminal symbol not found");
        let header = parser.header.context("Header line not found in the pattern")?;
        let comments = parser.comments;
        let contents = Self::convert_runs_to_triples(&parser.contents);
        Ok(Rle { header, comments, contents })
    }

    // Creates an empty parser
    fn new() -> Self {
        Self {
            comments: Vec::new(),
            header: None,
            contents: Vec::new(),
            position: (0, 0),
            finished: false,
        }
    }

    // Adds a line into the parser
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
            self.comments.push(line.to_owned());
        } else {
            let header = Self::parse_header_line(line)?;
            self.header = Some(header);
        }
        Ok(())
    }

    // Determines whether the line is a comment line or not
    fn is_comment_line(line: &str) -> bool {
        matches!(line.chars().next(), Some('#') | None)
    }

    // Parses the line as a header line
    fn parse_header_line(line: &str) -> Result<RleHeader> {
        fn check_variable_name(expected_name: &str, label: &str, name: &str) -> Result<()> {
            ensure!(name == expected_name, format!("{label} variable in the header line is not \"{expected_name}\""));
            Ok(())
        }
        fn parse_as_number((name, val_str): (&str, &str)) -> Result<usize> {
            val_str.parse().with_context(|| format!("Invalid {name} value"))
        }
        fn parse_as_rule((name, val_str): (&str, &str)) -> Result<Rule> {
            val_str.parse().with_context(|| format!("Invalid {name} value"))
        }
        let fields = line
            .split(',')
            .enumerate()
            .map(|(index, str)| {
                ensure!(index <= 2, "Too many fields in the header line");
                str.find('=')
                    .map(|pos| (str[..pos].trim(), str[(pos + 1)..].trim()))
                    .context("Parse error in the header line")
            })
            .collect::<Result<Vec<_>>>()?;
        ensure!(fields.len() >= 2, "Too few fields in the header line");
        check_variable_name("x", "1st", fields[0].0)?;
        let width = parse_as_number(fields[0])?;
        check_variable_name("y", "2nd", fields[1].0)?;
        let height = parse_as_number(fields[1])?;
        let rule = if fields.len() > 2 {
            check_variable_name("rule", "3rd", fields[2].0)?;
            parse_as_rule(fields[2])?
        } else {
            Rule::conways_life()
        };
        Ok(RleHeader { width, height, rule })
    }

    // Parses the line as a content line
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

    // Calculates the advanced position
    fn advanced_position(header: &RleHeader, current_position: (usize, usize), contents_to_be_append: &[RleRun]) -> Result<(usize, usize)> {
        ensure!(contents_to_be_append.is_empty() || header.height > 0, "The pattern exceeds specified height"); // this check is required for the header with "y = 0"
        contents_to_be_append
            .iter()
            .try_fold(current_position, |(curr_x, curr_y), RleRun(count, tag)| match tag {
                RleTag::EndOfLine => {
                    let next_y = curr_y + count;
                    ensure!(next_y < header.height, "The pattern exceeds specified height");
                    Ok((0, next_y))
                }
                _ => {
                    let next_x = curr_x + count;
                    ensure!(next_x <= header.width, "The pattern exceeds specified width");
                    Ok((next_x, curr_y))
                }
            })
    }

    // Convert the series of (usize, RleTag) into the series of RleRunsTriple
    fn convert_runs_to_triples(runs: &[RleRun]) -> Vec<RleRunsTriple> {
        const TRIPLE_ZERO: RleRunsTriple = RleRunsTriple {
            pad_lines: 0,
            pad_dead_cells: 0,
            live_cells: 0,
        };
        let (mut buf, curr_triple) = runs.iter().fold((Vec::new(), TRIPLE_ZERO), |(mut buf, curr_triple), run| {
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
        if curr_triple.live_cells > 0 {
            buf.push(curr_triple);
        }
        buf
    }
}
