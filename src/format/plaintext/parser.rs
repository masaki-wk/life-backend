use anyhow::{anyhow, Result};

use super::PlaintextLine;

// The parser of Plaintext format, used during constructing of Plaintext
pub(super) struct PlaintextParser {
    pub(super) name: Option<String>,
    pub(super) comments: Vec<String>,
    pub(super) lines: usize,
    pub(super) contents: Vec<PlaintextLine>,
}

// Inherent methods

impl PlaintextParser {
    // Parses the line with the specified prefix
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

    // Parses the line as a name line
    #[inline]
    fn parse_name_line(line: &str) -> Option<&str> {
        Self::parse_prefixed_line("!Name: ", line)
    }

    // Parses the line as a comment line
    #[inline]
    fn parse_comment_line(line: &str) -> Option<&str> {
        Self::parse_prefixed_line("!", line)
    }

    // Parses the line as a content line
    fn parse_content_line(line: &str) -> Result<Vec<usize>> {
        line.chars()
            .enumerate()
            .filter_map(|(i, c)| match c {
                '.' => None,
                'O' => Some(Ok(i)),
                _ => Some(Err(anyhow!("Invalid character found in the pattern"))),
            })
            .collect()
    }

    // Creates an empty parser
    pub(super) fn new() -> Self {
        Self {
            name: None,
            comments: Vec::new(),
            lines: 0,
            contents: Vec::new(),
        }
    }

    // Adds a line into the parser
    pub(super) fn push(&mut self, line: &str) -> Result<()> {
        if self.name.is_none() && self.comments.is_empty() && self.lines == 0 {
            if let Some(name) = Self::parse_name_line(line) {
                self.name = Some(name.to_string());
                return Ok(());
            }
        }
        if self.lines == 0 {
            if let Some(comment) = Self::parse_comment_line(line) {
                self.comments.push(comment.to_string());
                return Ok(());
            }
        }
        let content = Self::parse_content_line(line)?;
        if !content.is_empty() {
            self.contents.push(PlaintextLine(self.lines, content));
        }
        self.lines += 1;
        Ok(())
    }
}
