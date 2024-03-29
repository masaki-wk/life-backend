# life-backend

[![GitHub](https://img.shields.io/badge/GitHub-masaki--wk/life--backend-informational?logo=github)](https://github.com/masaki-wk/life-backend)
[![crates.io](https://img.shields.io/crates/v/life-backend.svg?logo=rust)](https://crates.io/crates/life-backend)
[![Docs.rs](https://img.shields.io/badge/Docs.rs-life--backend-informational?logo=docs.rs)](https://docs.rs/life-backend)
[![CI Status](https://img.shields.io/github/actions/workflow/status/masaki-wk/life-backend/ci.yml?label=CI&logo=github)](https://github.com/masaki-wk/life-backend/actions/workflows/ci.yml)
[![Docs Status](https://img.shields.io/docsrs/life-backend?logo=docs.rs)](https://docs.rs/crate/life-backend)

A backend implementation of Conway's Game of Life.

This library provides several functionalities for simulating Life-like cellular
automata, including Conway's Game of Life.

The following operations are supported:

- Parsing or writing patterns of Life-like cellular automata
  (Plaintext and RLE formats are supported)
- Parsing or writing rules in the birth/survival notation (e.g., `"B3/S23"`)
- Managing a board, a two-dimensional orthogonal grid map of live and dead cells
  (The type of the x- and y-coordinates of positions is generalized)
- Creating a new game based on a given rule and board, advancing the generation
  and querying the state

It does not provide frontend functionality for viewing or editing patterns
through a user interface.

## Examples

The following code example demonstrates how to create a new game from a pattern
file, advance the game and print its final state:

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
  game.advance();
}

// Print the last state
let bbox = game.board().bounding_box();
let population = game.board().iter().count();
println!("Generation {generation}: bounding-box = {bbox}, population = {population}");
println!("{game}");
```

`examples/game.rs` is a simple Game of Life program.  It creates a new game from
a pattern file, advances the game and prints its state to the standard output.
You can run this program like as:

```shell
$ cargo run --example game -- --generation=1 patterns/glider.rle
...
Generation 0: bounding-box = (x:[0, 2], y:[0, 2]), population = 5
.O.
..O
OOO

Generation 1: bounding-box = (x:[0, 2], y:[1, 3]), population = 5
O.O
.OO
.O.
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
