//! A backend implementation of Conway's Game of Life.
//!
//! This library provides several functionalities for Life-like cellular automata, including Conway's Game of Life.
//! It does not provide frontend functionality for viewing or editing patterns through a user interface.
//!
//! The following operations are supported:
//!
//! - Parsing or writing patterns of Life-like cellular automata (supported formats: Plaintext and RLE)
//! - Parsing or writing a rule in the birth/survival notation (e.g., `"B3/S23"`)
//! - Managing a board, a two-dimensional orthogonal grid map of live and dead cells
//!   (The type of the x- and y-coordinate values of positions is set by the type parameter)
//! - Creating a game from a rule and a board, advancing its generation, and querying the state
//!
//! # Examples
//!
//! ```
//! use std::fs::File;
//! use life_backend::format::{Rle, RleBuilder};
//! use life_backend::{Board, Game, Position};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Read a pattern file
//! let file = File::open("patterns/glider.rle")?;
//! let handler = Rle::new(file)?;
//!
//! // Create a game
//! let rule = handler.rule().to_owned();
//! let board = handler
//!   .live_cells()
//!   .map(Position::<i16>::try_from)
//!   .collect::<Result<Board<_>, _>>()?;
//! let mut game = Game::new(rule, board);
//!
//! // Advance the generation
//! for _ in 0..4 {
//!   game.update();
//! }
//!
//! // Output the result in RLE format
//! let handler = game
//!   .board()
//!   .iter()
//!   .copied()
//!   .map(Position::try_from)
//!   .collect::<Result<RleBuilder, _>>()?
//!   .build()?;
//! println!("{handler}");
//! # Ok(())
//! # }
//! ```

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
