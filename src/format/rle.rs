use crate::Rule;

// Internal structs, used in Rle
#[derive(Clone, Debug)]
struct RleHeader {
    width: usize,
    height: usize,
    rule: Rule,
}
#[derive(Clone, Debug)]
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
