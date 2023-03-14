use anyhow::{bail, Context, Result};
use life_backend::format::Plaintext;
use life_backend::{Board, Game};
use std::env;
use std::fs::File;
use std::path::Path;

use i16 as I;

struct Config {
    path_str: String,
    generation: usize,
    step_size: usize,
}

impl Config {
    fn new(mut args: env::Args) -> Result<Self> {
        args.next();
        let Some(path_str) = args.next() else {
            bail!("Not enough arguments");
        };
        let generation = match args.next() {
            Some(s) => match s.parse() {
                Ok(n) => n,
                Err(_) => bail!("2nd argument is not a number"),
            },
            None => 0,
        };
        let step_size = match args.next() {
            Some(s) => match s.parse() {
                Ok(n) => n,
                Err(_) => bail!("3rd argument is not a number"),
            },
            None => 1,
        };
        Ok(Self {
            path_str,
            generation,
            step_size,
        })
    }
}

fn run(config: Config) -> Result<()> {
    let path = Path::new(&config.path_str);
    let file = File::open(path).with_context(|| format!("Failed to open \"{}\"", path.display()))?;
    let parser = Plaintext::<I>::new(file)?;
    let board: Board<_> = parser.iter().collect();
    let game = Game::<_>::new(board);
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
