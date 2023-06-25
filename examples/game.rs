use anyhow::Result;
use clap::Parser;

use life_backend::format;
use life_backend::{Board, Game, Position};

use i16 as I;

#[derive(Parser, Debug)]
struct Args {
    #[arg(help = "Pattern file path")]
    path: String,

    #[arg(short, long, default_value_t = 0, help = "Target generation")]
    generation: usize,

    #[arg(short, long, default_value_t = 1, help = "Step size")]
    step_size: usize,
}

fn run(args: Args) -> Result<()> {
    let handler = format::open(&args.path)?;
    let rule = handler.rule();
    let board = handler.live_cells().map(Position::try_from).collect::<Result<Board<_>, _>>()?;
    let game = Game::new(rule, board);
    simulate(game, args.generation, args.step_size);
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
        game.advance();
    }
    print_game(&game, generation);
}

fn main() -> Result<()> {
    let args = Args::parse();
    run(args)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::process::Command;
    #[test]
    fn glider() -> Result<()> {
        let status = Command::new("cargo")
            .args(["run", "--example", "game", "--", "--generation=4", "--step-size=4", "patterns/glider.rle"])
            .status()?;
        assert!(status.success());
        Ok(())
    }
}
