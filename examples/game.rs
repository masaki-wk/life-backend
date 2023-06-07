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
    let board = handler.live_cells().map(Position::try_from).collect::<Result<Board<_>, _>>()?;
    let game = Game::new(rule, board);
    simulate(game, config.generation, config.step_size);
    Ok(())
}

fn print_game(game: &Game<I>, generation: usize) {
    let bbox = game.board().bounding_box();
    let population = game.board().iter().count();
    println!("Generation {generation}: bounding-box = {bbox}, population = {population}");
    println!("{game}");
}

fn simulate(mut game: Game<I>, generation: usize, step_size: usize) {
    for i in 0..generation {
        if i % step_size == 0 {
            print_game(&game, i);
        }
        game.update();
    }
    print_game(&game, generation);
}

fn main() -> Result<()> {
    let config = Config::new(env::args())?;
    run(config)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::process::Command;
    #[test]
    fn glider() -> Result<()> {
        let status = Command::new("cargo")
            .args(["run", "--example", "game", "patterns/glider.rle", "4", "4"])
            .status()?;
        assert!(status.success());
        Ok(())
    }
}
