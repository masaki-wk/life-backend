//! A backend implementation of Conway's Game of Life.

mod rule;
pub use rule::Rule;

mod position;
pub use position::Position;

mod boardrange;
pub use boardrange::BoardRange;

mod board;
pub use board::Board;

mod game;
pub use game::Game;

pub mod format;
pub use format::Format;
