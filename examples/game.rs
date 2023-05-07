use anyhow::{Context as _, Result};
use std::env;

use life_backend::format;
use life_backend::{Board, Game, Position};

use i16 as I;

struct Config {
    path_str: String,
    generation: usize,
    step_size: usize,
}

impl Config {
    fn new(mut args: env::Args) -> Result<Self> {
        args.next();
        let path_str = args.next().context("Not enough arguments")?;
        let generation = args.next().map_or(Ok(0), |s| s.parse().context("2nd argument is not a number"))?;
        let step_size = args.next().map_or(Ok(1), |s| s.parse().context("3rd argument is not a number"))?;
        Ok(Self {
            path_str,
            generation,
            step_size,
        })
    }
}

fn run(config: Config) -> Result<()> {
    let handler = format::open(&config.path_str)?;
    let rule = handler.rule();
    let board: Board<_> = handler.live_cells().map(|pos| Position(pos.0 as I, pos.1 as I)).collect();
    let game = Game::new(rule, board);
    simulate(game, config.generation, config.step_size);
    Ok(())
}

fn simulate(mut game: Game<I>, generation: usize, step_size: usize) {
    for i in 0..generation {
        if i % step_size == 0 {
            println!("Generation {i}:");
            println!("{game}");
        }
        game.update();
    }
    println!("Generation {generation}:");
    println!("{game}");
}

fn main() -> Result<()> {
    let config = Config::new(env::args())?;
    run(config)
}
