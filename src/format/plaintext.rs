// An internal struct, used in Plaintext
#[derive(Debug, Clone, PartialEq)]
struct PlaintextLine(usize, Vec<usize>);

// A representation for Plaintext file format
mod core;
pub use self::core::Plaintext;

// The parser of Plaintext, used during constructing of Plaintext
mod parser;
use parser::PlaintextParser;

// The builder of Plaintext
mod builder;
pub use builder::PlaintextBuilder;

// Unit tests
#[cfg(test)]
mod tests;
