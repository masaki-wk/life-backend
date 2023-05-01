use anyhow::Result;
use std::fmt;
use std::io::Read;

use super::{RleHeader, RleParser, RleRunsTriple};
use crate::Rule;

/// A representation for RLE file format.
///
/// The detail of this format is described in:
///
/// - [Run Length Encoded - LifeWiki](https://conwaylife.com/wiki/Run_Length_Encoded)
/// - [Golly Help: File Formats > Extended RLE format](https://golly.sourceforge.net/Help/formats.html#rle)
///
#[derive(Debug, Clone)]
pub struct Rle {
    pub(super) header: RleHeader,
    pub(super) comments: Vec<String>,
    pub(super) contents: Vec<RleRunsTriple>,
}

// Inherent methods

impl Rle {
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
    #[inline]
    pub fn new<R>(read: R) -> Result<Self>
    where
        R: Read,
    {
        RleParser::parse(read)
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

    /// Returns the rule.
    ///
    /// # Examples
    ///
    /// ```
    /// # use life_backend::format::Rle;
    /// # use life_backend::Rule;
    /// let pattern = "\
    ///     #N T-tetromino\n\
    ///     x = 3, y = 2, rule = B3/S23\n\
    ///     3o$bo!\n\
    /// ";
    /// let parser = Rle::new(pattern.as_bytes()).unwrap();
    /// assert_eq!(*parser.rule(), Rule::conways_life());
    /// ```
    ///
    #[inline]
    pub fn rule(&self) -> &Rule {
        &self.header.rule
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
        fn convert_run_to_string(run_count: usize, tag_char: char) -> String {
            if run_count > 1 {
                let mut buf = run_count.to_string();
                buf.push(tag_char);
                buf
            } else {
                tag_char.to_string()
            }
        }
        fn flush_buf(f: &mut fmt::Formatter, buf: &mut String) -> Result<(), fmt::Error> {
            writeln!(f, "{buf}")?;
            Ok(())
        }
        fn write_with_buf(f: &mut fmt::Formatter, buf: &mut String, s: &str) -> Result<(), fmt::Error> {
            if buf.len() + s.len() > MAX_LINE_WIDTH {
                flush_buf(f, buf)?;
                buf.clear();
            }
            *buf += s;
            Ok(())
        }
        for line in self.comments() {
            writeln!(f, "{line}")?;
        }
        writeln!(f, "x = {}, y = {}, rule = {}", self.width(), self.height(), self.rule())?;
        let mut buf = String::new();
        for x in &self.contents {
            for (run_count, tag_char) in [(x.pad_lines, '$'), (x.pad_dead_cells, 'b'), (x.live_cells, 'o')] {
                if run_count > 0 {
                    let s = convert_run_to_string(run_count, tag_char);
                    write_with_buf(f, &mut buf, &s)?;
                }
            }
        }
        write_with_buf(f, &mut buf, "!")?;
        flush_buf(f, &mut buf)?;
        Ok(())
    }
}
