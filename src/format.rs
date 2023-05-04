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

/// Attempts to open a file with the file format hander specified by the file extension.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use life_backend::format;
/// # use life_backend::Rule;
/// let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/rpentomino.cells");
/// let handler = format::open(path_str).unwrap();
/// assert_eq!(handler.rule(), Rule::conways_life());
/// assert_eq!(handler.live_cells().count(), 5);
/// ```
///
/// ```
/// # use std::path::Path;
/// # use life_backend::format;
/// # use life_backend::Rule;
/// let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/bheptomino.rle");
/// let handler = format::open(path_str).unwrap();
/// assert_eq!(handler.rule(), Rule::conways_life());
/// assert_eq!(handler.live_cells().count(), 7);
/// ```
///
pub fn open<P>(path: P) -> Result<Box<dyn Format>>
where
    P: AsRef<Path>,
{
    let path_to_display = path.as_ref().display().to_string();
    let ext = path
        .as_ref()
        .extension()
        .map(|s| s.to_str().unwrap_or_default())
        .with_context(|| format!("\"{}\" has no extension", path_to_display))?
        .to_string();
    let file = File::open(path).with_context(|| format!("Failed to open \"{}\"", path_to_display))?;
    let result: Box<dyn Format> = match &ext[..] {
        "cells" => Box::new(Plaintext::new(file)?),
        "rle" => Box::new(Rle::new(file)?),
        _ => bail!("\"{}\" has unknown extension", path_to_display),
    };
    Ok(result)
}
