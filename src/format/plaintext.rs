// An internal struct, used in Plaintext
#[derive(Clone, PartialEq, Debug)]
struct PlaintextLine(usize, Vec<usize>);

mod core;
pub use self::core::Plaintext;

mod parser;
use parser::PlaintextParser;

mod builder;
pub use builder::PlaintextBuilder;

#[cfg(test)]
mod tests;
