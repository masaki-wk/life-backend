//! A backend implementation of Conway's Game of Life.

mod rule;
pub use rule::Rule;

mod board;
pub use board::Board;

mod game;
pub use game::Game;

pub mod format;
pub use format::Format;
