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

mod core;
pub use self::core::Rle;

mod parser;
use parser::RleParser;

mod builder;
pub use builder::RleBuilder;

#[cfg(test)]
mod tests;
