use anyhow::{bail, Context as _, Result};
use std::fmt;
use std::fs::File;
use std::path::Path;

use crate::Rule;

mod plaintext;
pub use plaintext::{Plaintext, PlaintextBuilder};

mod rle;
pub use rle::{Rle, RleBuilder};

/// Provides several methods for Conway's Game of Life pattern file formats.
pub trait Format: fmt::Display {
    fn rule(&self) -> Rule;
    fn live_cells(&self) -> Box<dyn Iterator<Item = (usize, usize)> + '_>;
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
