use anyhow::{bail, Context, Result};
use life_backend::format::Plaintext;
use life_backend::{Board, Game};
use std::env;
use std::fs::File;
use std::path::Path;

use i16 as I;

struct Config {
    path_str: String,
    steps: usize,
}

impl Config {
    fn new(mut args: env::Args) -> Result<Self> {
        args.next();
        let Some(path_str) = args.next() else {
            bail!("Not enough arguments");
        };
        let steps = match args.next() {
            Some(s) => match s.parse() {
                Ok(n) => n,
                Err(_) => bail!("2nd argument is not a number"),
            },
            None => 0,
        };
        Ok(Self { path_str, steps })
    }
}

fn run(config: Config) -> Result<()> {
    let path = Path::new(&config.path_str);
    let file = File::open(path).with_context(|| format!("Failed to open \"{}\"", path.display()))?;
    let parser = Plaintext::<I>::new(file)?;
    let board: Board<_> = parser.iter().collect();
    let game = Game::<_>::new(board);
    simulate(game, config.steps);
    Ok(())
}

fn simulate(mut game: Game<I>, steps: usize) {
    for i in 0..=steps {
        if i != 0 {
            game.update();
        }
        println!("Generation {i}:");
        println!("{game}");
    }
}

fn main() -> Result<()> {
    let config = Config::new(env::args())?;
    run(config)
}
