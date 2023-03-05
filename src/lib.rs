//! A backend implementation of Conway's Game of Life.

mod board;
pub mod format;
mod game;

pub use board::Board;
pub use board::IndexType as BoardIndexType;
pub use game::Game;
