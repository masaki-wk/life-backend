use crate::Rule;

// Internal structs, used in Rle
#[derive(Debug, Clone)]
struct RleHeader {
    width: usize,
    height: usize,
    rule: Rule,
}
#[derive(Debug, Clone)]
struct RleRunsTriple {
    pad_lines: usize,
    pad_dead_cells: usize,
    live_cells: usize,
}

// A representation for RLE file format
mod core;
pub use self::core::Rle;

// The parser of RLE format, used during constructing of Rle
mod parser;
use parser::RleParser;

// The builder of Rle
mod builder;
pub use builder::RleBuilder;

// Unit tests
#[cfg(test)]
mod tests;
