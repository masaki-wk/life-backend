use anyhow::Result;
use criterion::{criterion_group, criterion_main, Criterion};
use num_traits::{Bounded, One, ToPrimitive, Zero};
use std::convert::TryFrom;
use std::hash::Hash;
use std::ops::{Add, Sub};
use std::path::Path;

use life_backend::format;
use life_backend::{Board, Game, Position};

fn workload<T>(game: &Game<T>, steps: usize)
where
    T: Eq + Hash + Copy + PartialOrd + Add<Output = T> + Sub<Output = T> + Zero + One + Bounded + ToPrimitive,
{
    let mut game = game.clone();
    for _ in 0..steps {
        game.advance();
    }
}

fn do_benchmark<T, P>(c: &mut Criterion, id: &str, path: P, steps: usize) -> Result<()>
where
    T: Eq + Hash + Copy + PartialOrd + Add<Output = T> + Sub<Output = T> + Zero + One + Bounded + ToPrimitive + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: std::error::Error + Send + Sync + 'static,
    P: AsRef<Path>,
{
    let handler = format::open(path)?;
    let rule = handler.rule();
    let board = handler.live_cells().map(Position::try_from).collect::<Result<Board<T>, _>>()?;
    let game = Game::new(rule, board);
    c.bench_function(id, |b| b.iter(|| workload(&game, steps)));
    Ok(())
}

macro_rules! create_benchmark_function {
    ($function_name:ident, $id:literal, $relative_path_string:literal, $steps:expr) => {
        pub fn $function_name(c: &mut Criterion) {
            let id = $id;
            let path = $relative_path_string;
            let steps = $steps;
            do_benchmark::<i8, _>(c, id, path, steps).unwrap();
        }
    };
}

#[rustfmt::skip]
mod benchmarks {
    use super::*;
    create_benchmark_function!(oscillator_blinker_benchmark, "oscillator-blinker", "patterns/blinker.rle", 2);
    create_benchmark_function!(oscillator_pentadecathlon_benchmark, "oscillator-pentadecathlon", "patterns/pentadecathlon.rle", 15);
    create_benchmark_function!(oscillator_queenbeeshuttle_benchmark, "oscillator-queenbeeshuttle", "patterns/transqueenbeeshuttle.rle", 30);
    create_benchmark_function!(oscillator_p60glidershuttle_benchmark, "oscillator-p60glidershuttle", "patterns/p60glidershuttle.rle", 60);
    create_benchmark_function!(oscillator_centinal_benchmark, "oscillator-centinal", "patterns/centinal.rle", 100);
    create_benchmark_function!(methuselah_bheptomino_benchmark, "methuselah-bheptomino", "patterns/bheptomino.rle", 148);
    create_benchmark_function!(methuselah_rpentomino_benchmark, "methuselah-rpentomino", "patterns/rpentomino.rle", 1103);
}

criterion_group!(
    benches,
    benchmarks::oscillator_blinker_benchmark,
    benchmarks::oscillator_pentadecathlon_benchmark,
    benchmarks::oscillator_queenbeeshuttle_benchmark,
    benchmarks::oscillator_p60glidershuttle_benchmark,
    benchmarks::oscillator_centinal_benchmark,
    benchmarks::methuselah_bheptomino_benchmark,
    benchmarks::methuselah_rpentomino_benchmark,
);
criterion_main!(benches);
