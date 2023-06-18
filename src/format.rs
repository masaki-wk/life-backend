use anyhow::{bail, Context as _, Result};
use std::fmt;
use std::fs::File;
use std::path::Path;

use crate::{Position, Rule};

mod plaintext;
pub use plaintext::{Plaintext, PlaintextBuilder};

mod rle;
pub use rle::{Rle, RleBuilder};

/// Provides several methods for Conway's Game of Life pattern file formats.
///
/// # Examples
///
/// ```
/// use std::fs::File;
/// use life_backend::{Format, Rule};
/// use life_backend::format::Plaintext;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let file = File::open("patterns/rpentomino.cells")?;
/// let handler: Box<dyn Format> = Box::new(Plaintext::new(file)?);
/// assert_eq!(handler.rule(), Rule::conways_life());
/// assert_eq!(handler.live_cells().count(), 5);
/// # Ok(())
/// # }
/// ```
///
pub trait Format: fmt::Display {
    /// Returns the rule.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Format, Rule};
    /// use life_backend::format::Rle;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = "\
    ///     #N T-tetromino\n\
    ///     x = 3, y = 2, rule = B3/S23\n\
    ///     3o$bo!\n\
    /// ";
    /// let handler: Box<dyn Format> = Box::new(pattern.parse::<Rle>()?);
    /// assert_eq!(handler.rule(), Rule::conways_life());
    /// # Ok(())
    /// # }
    /// ```
    ///
    fn rule(&self) -> Rule;

    /// Creates an owning iterator over the series of live cell positions in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use life_backend::{Format, Position, Rule};
    /// use life_backend::format::Rle;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let pattern = "\
    ///     #N T-tetromino\n\
    ///     x = 3, y = 2, rule = B3/S23\n\
    ///     3o$bo!\n\
    /// ";
    /// let handler: Box<dyn Format> = Box::new(pattern.parse::<Rle>()?);
    /// assert!(handler.live_cells().eq([Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)]));
    /// # Ok(())
    /// # }
    /// ```
    ///
    fn live_cells(&self) -> Box<dyn Iterator<Item = Position<usize>> + '_>;
}

/// Attempts to open a file with the file format handler specified by the file extension.
///
/// # Examples
///
/// ```
/// use life_backend::format;
/// use life_backend::Rule;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path = "patterns/rpentomino.cells";
/// let handler = format::open(path)?;
/// assert_eq!(handler.rule(), Rule::conways_life());
/// assert_eq!(handler.live_cells().count(), 5);
/// # Ok(())
/// # }
/// ```
///
/// ```
/// use life_backend::format;
/// use life_backend::Rule;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path = "patterns/bheptomino.rle";
/// let handler = format::open(path)?;
/// assert_eq!(handler.rule(), Rule::conways_life());
/// assert_eq!(handler.live_cells().count(), 7);
/// # Ok(())
/// # }
/// ```
///
pub fn open<P>(path: P) -> Result<Box<dyn Format>>
where
    P: AsRef<Path>,
{
    let path_for_display = path.as_ref().to_owned();
    let ext = path
        .as_ref()
        .extension()
        .with_context(|| format!("\"{}\" has no extension", path_for_display.display()))?
        .to_owned();
    let file = File::open(path).with_context(|| format!("Failed to open \"{}\"", path_for_display.display()))?;
    let result: Box<dyn Format> = if ext.as_os_str() == "cells" {
        Box::new(Plaintext::new(file)?)
    } else if ext.as_os_str() == "rle" {
        Box::new(Rle::new(file)?)
    } else {
        bail!("\"{}\" has unknown extension", path_for_display.display());
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn open_no_extension() {
        let path = "patterns/rpentomino";
        let result = open(path);
        assert!(result.is_err());
    }
    #[test]
    fn open_unknown_extension() {
        let path = "patterns/rpentomino.unknown";
        let result = open(path);
        assert!(result.is_err());
    }
}
