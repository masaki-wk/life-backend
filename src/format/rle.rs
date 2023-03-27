use anyhow::{bail, ensure, Result};
use std::fmt;
use std::io::{BufRead, BufReader, Read};

/// A representation for RLE file format, described in <https://conwaylife.com/wiki/Run_Length_Encoded>.
#[derive(Debug, Clone)]
pub struct Rle {
    #[allow(dead_code)]
    comments: Vec<String>,
    #[allow(dead_code)]
    width: usize,
    #[allow(dead_code)]
    height: usize,
    #[allow(dead_code)]
    contents: Vec<RleLiveCellRun>,
}

// Internal struct, used in Rle
#[derive(Debug, Clone)]
struct RleLiveCellRun {
    pad_lines: usize,
    pad_dead_cells: usize,
    live_cells: usize,
}

// Internal structs, used during constructing of Rle
struct RleHeader {
    width: usize,
    height: usize,
}
#[derive(Debug, Clone)]
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
                        bail!("Invalid width value");
                    };
                    width = Some(n);
                }
                "y" => {
                    let Ok(n) = val_str.parse::<usize>() else {
                        bail!("Invalid width value");
                    };
                    height = Some(n);
                }
                "rule" => (),
                _ => bail!(format!("The header line includes unknown variable {}", name)),
            }
        }
        match (width, height) {
            (Some(w), Some(h)) => Ok(RleHeader { width: w, height: h }),
            _ => bail!("The header line is invalid"),
        }
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

    /// Creates from the specified implementor of Read, such as File or `&[u8]`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Rle;
    /// let pattern = "\
    ///     #N Glider\n\
    ///     x = 3, y = 3\n\
    ///     bo$2bo$3o!\n\
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
            width: header.width,
            height: header.height,
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
    ///     #N Glider\n\
    ///     x = 3, y = 3\n\
    ///     bo$2bo$3o!\n\
    /// ";
    /// let parser = Rle::new(pattern.as_bytes()).unwrap();
    /// assert_eq!(parser.comments().len(), 1);
    /// assert_eq!(parser.comments()[0], "N Glider");
    /// ```
    ///
    #[inline]
    pub fn comments(&self) -> &Vec<String> {
        &self.comments
    }

    /// Creates a non-owning iterator over the series of immutable live cell positions in ascending order.
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        todo!();
        let v: Vec<(usize, usize)> = Vec::new();
        v.into_iter()
    }
}

// Trait implementations

impl fmt::Display for Rle {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }
}
