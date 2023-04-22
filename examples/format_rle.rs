use anyhow::{Context as _, Result};
use life_backend::format::Rle;
use std::env;
use std::fs::File;
use std::path::Path;

struct Config {
    path_str: String,
}

impl Config {
    fn new(mut args: env::Args) -> Result<Self> {
        args.next();
        let path_str = args.next().context("Not enough arguments")?;
        Ok(Self { path_str })
    }
}

fn run(config: Config) -> Result<()> {
    let path = Path::new(&config.path_str);
    let file = File::open(path).with_context(|| format!("Failed to open \"{}\"", path.display()))?;
    let parser = Rle::new(file)?;
    println!("{parser}");
    Ok(())
}

fn main() -> Result<()> {
    let config = Config::new(env::args())?;
    run(config)
}
