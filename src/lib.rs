//! A backend implementation of Conway's Game of Life.

mod board;
pub mod format;
mod game;
mod rule;

pub use board::Board;
pub use game::Game;
pub use rule::Rule;
