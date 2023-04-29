use anyhow::{Context as _, Result};
use criterion::{criterion_group, criterion_main, Criterion};
use num_traits::{Bounded, FromPrimitive, One, ToPrimitive, Zero};
use std::fs::File;
use std::hash::Hash;
use std::ops::{Add, Sub};
use std::path::Path;

use life_backend::format::Rle;
use life_backend::{Board, Game, Rule};

fn workload<IndexType>(rule: &Rule, board: &Board<IndexType>, steps: usize)
where
    IndexType: Eq + Hash + Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + Zero + One + Bounded + ToPrimitive,
{
    let mut game = Game::<_>::new(rule.clone(), board.clone());
    for _ in 0..steps {
        game.update();
    }
}

fn do_benchmark<IndexType>(c: &mut Criterion, id: &str, path_str: &str, steps: usize) -> Result<()>
where
    IndexType: Eq + Hash + Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + Zero + One + Bounded + ToPrimitive + FromPrimitive,
{
    let from_usize_unwrap = |x| IndexType::from_usize(x).unwrap();
    let path = Path::new(path_str);
    let file = File::open(path).with_context(|| format!("Failed to open \"{}\"", path.display()))?;
    let parser = Rle::new(file)?;
    let rule = parser.rule();
    let board: Board<_> = parser.iter().map(|(x, y)| (from_usize_unwrap(x), from_usize_unwrap(y))).collect();
    c.bench_function(id, |b| b.iter(|| workload(rule, &board, steps)));
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
