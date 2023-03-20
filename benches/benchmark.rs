use anyhow::{Context, Result};
use criterion::{criterion_group, criterion_main, Criterion};
use life_backend::format::Plaintext;
use life_backend::{Board, Game};
use num_traits::{Bounded, FromPrimitive, One, ToPrimitive, Zero};
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::ops::{Add, Sub};
use std::path::Path;

fn workload<IndexType>(board: &Board<IndexType>, steps: usize)
where
    IndexType: Eq + Hash + Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + Zero + One + Bounded + ToPrimitive,
{
    let mut game = Game::<_>::new(board.clone());
    for _ in 0..steps {
        game.update();
    }
}

fn do_benchmark<IndexType, R>(c: &mut Criterion, id: &str, read: R, steps: usize) -> Result<()>
where
    IndexType: Eq + Hash + Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + Zero + One + Bounded + ToPrimitive + FromPrimitive,
    R: Read,
{
    let from_usize_unwrap = |x| IndexType::from_usize(x).unwrap();
    let loader = Plaintext::new(read)?;
    let board: Board<_> = loader.iter().map(|(x, y)| (from_usize_unwrap(x), from_usize_unwrap(y))).collect();
    c.bench_function(id, |b| b.iter(|| workload(&board, steps)));
    Ok(())
}

fn do_benchmark_with_string<IndexType>(c: &mut Criterion, id: &str, pattern: &str, steps: usize) -> Result<()>
where
    IndexType: Eq + Hash + Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + Zero + One + Bounded + ToPrimitive + FromPrimitive,
{
    do_benchmark::<i8, _>(c, id, pattern.as_bytes(), steps)?;
    Ok(())
}

fn do_benchmark_with_path<IndexType>(c: &mut Criterion, id: &str, path_str: &str, steps: usize) -> Result<()>
where
    IndexType: Eq + Hash + Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + Zero + One + Bounded + ToPrimitive + FromPrimitive,
{
    let path = Path::new(path_str);
    let file = File::open(path).with_context(|| format!("Failed to open \"{}\"", path.display()))?;
    do_benchmark::<i8, _>(c, id, file, steps)?;
    Ok(())
}

fn blinker_1k_benchmark(c: &mut Criterion) {
    let id = "blinker-1k";
    let pattern = "\
        !Name: Blinker\n\
        OOO\n\
    ";
    let steps = 1000;
    do_benchmark_with_string::<i8>(c, id, pattern, steps).unwrap();
}

fn pentadecathlon_1k_benchmark(c: &mut Criterion) {
    let id = "pentadecathlon-1k";
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/pentadecathlon.cells");
    let steps = 1000;
    do_benchmark_with_path::<i8>(c, id, path_str, steps).unwrap();
}

fn queenbeeshuttle_1k_benchmark(c: &mut Criterion) {
    let id = "queenbeeshuttle-1k";
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/transqueenbeeshuttle.cells");
    let steps = 1000;
    do_benchmark_with_path::<i8>(c, id, path_str, steps).unwrap();
}

fn p60glidershuttle_1k_benchmark(c: &mut Criterion) {
    let id = "p60glidershuttle-1k";
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/p60glidershuttle.cells");
    let steps = 1000;
    do_benchmark_with_path::<i8>(c, id, path_str, steps).unwrap();
}

fn moldon30p25_1k_benchmark(c: &mut Criterion) {
    let id = "moldon30p25-1k";
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/moldon30p25.cells");
    let steps = 1000;
    do_benchmark_with_path::<i8>(c, id, path_str, steps).unwrap();
}

fn centinal_1k_benchmark(c: &mut Criterion) {
    let id = "centinal-1k";
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/centinal.cells");
    let steps = 1000;
    do_benchmark_with_path::<i8>(c, id, path_str, steps).unwrap();
}

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
