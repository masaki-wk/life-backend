# life-backend

[![GitHub](https://img.shields.io/badge/github-masaki--wk/life--backend-blue?logo=github)](https://github.com/masaki-wk/life-backend)
[![CI Status](https://img.shields.io/github/actions/workflow/status/masaki-wk/life-backend/ci.yml?label=CI&logo=github)](https://github.com/masaki-wk/life-backend/actions/workflows/ci.yml)

A backend implementation of Conway's Game of Life.

## Introduction

This library provides several functionalities for Life-like cellular automata,
including Conway's Game of Life. It does not provide frontend functionality for
viewing or editing patterns through a user interface.

The following operations are supported:

- Parsing or writing patterns of Life-like cellular automata
  (supported formats: Plaintext and RLE)
- Parsing or writing a rule in the birth/survival notation (e.g., `"B3/S23"`)
- Managing a board, a two-dimensional orthogonal grid map of live and dead cells
  (The type of the x- and y-coordinates of positions is generalized)
- Creating a new game from the given rule and board, advancing the generation
  and querying the state

## Examples

```rust
use std::fs::File;
use life_backend::format::{Rle, RleBuilder};
use life_backend::{Board, Game, Position};

// Read a pattern file
let file = File::open("patterns/glider.rle")?;
let handler = Rle::new(file)?;

// Create a game
let rule = handler.rule().to_owned();
let board = handler
  .live_cells()
  .map(Position::<i16>::try_from)
  .collect::<Result<Board<_>, _>>()?;
let mut game = Game::new(rule, board);

// Advance the generation
for _ in 0..4 {
  game.update();
}

// Output the result in RLE format
let handler = game
  .board()
  .iter()
  .copied()
  .map(Position::try_from)
  .collect::<Result<RleBuilder, _>>()?
  .build()?;
println!("{handler}");
```

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
