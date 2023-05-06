use anyhow::Result;
use criterion::{criterion_group, criterion_main, Criterion};
use num_traits::{Bounded, FromPrimitive, One, ToPrimitive, Zero};
use std::hash::Hash;
use std::ops::{Add, Sub};

use life_backend::format;
use life_backend::{Board, Game, Position};

fn workload<CoordinateType>(game: &Game<CoordinateType>, steps: usize)
where
    CoordinateType: Eq + Hash + Copy + PartialOrd + Add<Output = CoordinateType> + Sub<Output = CoordinateType> + Zero + One + Bounded + ToPrimitive,
{
    let mut game = game.clone();
    for _ in 0..steps {
        game.update();
    }
}

fn do_benchmark<CoordinateType>(c: &mut Criterion, id: &str, path_str: &str, steps: usize) -> Result<()>
where
    CoordinateType:
        Eq + Hash + Copy + PartialOrd + Add<Output = CoordinateType> + Sub<Output = CoordinateType> + Zero + One + Bounded + ToPrimitive + FromPrimitive,
{
    let from_usize_unwrap = |x| CoordinateType::from_usize(x).unwrap();
    let handler = format::open(path_str)?;
    let rule = handler.rule();
    let board: Board<_> = handler
        .live_cells()
        .map(|pos| Position(from_usize_unwrap(pos.0), from_usize_unwrap(pos.1)))
        .collect();
    let game = Game::new(rule, board);
    c.bench_function(id, |b| b.iter(|| workload(&game, steps)));
    Ok(())
}

macro_rules! create_benchmark_function {
    ($function_name:ident, $id:literal, $relative_path_string:literal, $steps:expr) => {
        fn $function_name(c: &mut Criterion) {
            let id = $id;
            let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/", $relative_path_string);
            let steps = $steps;
            do_benchmark::<i8>(c, id, path_str, steps).unwrap();
        }
    };
}

create_benchmark_function!(blinker_1k_benchmark, "blinker-1k", "patterns/blinker.rle", 1000);
create_benchmark_function!(pentadecathlon_1k_benchmark, "pentadecathlon-1k", "patterns/pentadecathlon.rle", 1000);
create_benchmark_function!(queenbeeshuttle_1k_benchmark, "queenbeeshuttle-1k", "patterns/transqueenbeeshuttle.rle", 1000);
create_benchmark_function!(p60glidershuttle_1k_benchmark, "p60glidershuttle-1k", "patterns/p60glidershuttle.rle", 1000);
create_benchmark_function!(moldon30p25_1k_benchmark, "moldon30p25-1k", "patterns/moldon30p25.rle", 1000);
create_benchmark_function!(centinal_1k_benchmark, "centinal-1k", "patterns/centinal.rle", 1000);

criterion_group!(
    benches,
    blinker_1k_benchmark,
    pentadecathlon_1k_benchmark,
    queenbeeshuttle_1k_benchmark,
    p60glidershuttle_1k_benchmark,
    moldon30p25_1k_benchmark,
    centinal_1k_benchmark
);
criterion_main!(benches);
