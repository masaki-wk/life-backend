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

// Internal structs, used during constructing of Rle
enum RleTag {
    DeadCell,
    AliveCell,
    EndOfLine,
}
struct RleRun(usize, RleTag);

// The parser of the RLE format, used during constructing of Rle
mod parser;
use parser::RleParser;

// A builder of Rle
mod builder;
pub use builder::RleBuilder;

// A representation for RLE file format
mod core;
pub use self::core::Rle;

// Unit tests
#[cfg(test)]
mod tests;
