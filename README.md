# life-backend

A backend implementation of [Conway's Game of Life](https://conwaylife.com/wiki/Conway%27s_Game_of_Life).

This library provides several functionalities for [Life-like cellular automata](https://conwaylife.com/wiki/Life-like_cellular_automaton), including Conway's Game of Life.
It does not provide frontend functionality for viewing or editing patterns through a user interface.

This library includes the following structs:

- [`Plaintext`], [`Rle`]: File format handlers for Life-like cellular automata.
- [`Rule`]: Represents a rule and supports the birth/survival notation (e.g., `"B3/S23"`).
- [`Position<T>`]: A position of a cell. The type parameter `T` is used as the type of the x- and y-coordinate values of positions.
- [`Board<T>`]: A two-dimensional orthogonal grid map of live and dead cells.
- [`Game<T>`]: Represents a game, including methods for creating a new game, advancing its generation, and querying the current state.

[`Plaintext`]: format::Plaintext
[`Rle`]: format::Rle
[`Rule`]: Rule
[`Position<T>`]: Position
[`Board<T>`]: Board
[`Game<T>`]: Game

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
