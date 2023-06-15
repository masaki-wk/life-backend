# life-backend

[![GitHub](https://img.shields.io/badge/github-masaki--wk/life--backend-blue?logo=github)](https://github.com/masaki-wk/life-backend)
[![CI Status](https://img.shields.io/github/actions/workflow/status/masaki-wk/life-backend/ci.yml?label=CI&logo=github)](https://github.com/masaki-wk/life-backend/actions/workflows/ci.yml)

A backend implementation of Conway's Game of Life.

This library provides several functionalities for Life-like cellular automata,
including Conway's Game of Life.

The following operations are supported:

- Parsing or writing patterns of Life-like cellular automata
  (supported formats: Plaintext and RLE)
- Parsing or writing a rule in the birth/survival notation (e.g., `"B3/S23"`)
- Managing a board, a two-dimensional orthogonal grid map of live and dead cells
  (The type of the x- and y-coordinates of positions is generalized)
- Creating a new game from the given rule and board, advancing the generation
  and querying the state

It does not provide frontend functionality for viewing or editing patterns
through a user interface.

## Example

Creating a new game from the pattern file, advancing it and show the last state:

```rust
use life_backend::format;
use life_backend::{Board, Game, Position};

// Read a pattern file
let handler = format::open("patterns/glider.rle")?;

// Create a new game (the type parameter is `i16`)
let rule = handler.rule();
let board = handler
  .live_cells()
  .map(Position::try_from)
  .collect::<Result<Board<i16>, _>>()?;
let mut game = Game::new(rule, board);

// Advance the generation
let generation = 4;
for _ in 0..generation {
  game.update();
}

// Print the last state
let bbox = game.board().bounding_box();
let population = game.board().iter().count();
println!("Generation {generation}: bounding-box = {bbox}, population = {population}");
println!("{game}");
```

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/license/mit/>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
