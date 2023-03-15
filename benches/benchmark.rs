use criterion::{criterion_group, criterion_main, Criterion};
use life_backend::format::Plaintext;
use life_backend::{Board, Game};
use num_traits::{Bounded, FromPrimitive, One, ToPrimitive, Zero};
use std::hash::Hash;
use std::ops::{Add, Sub};

fn workload<IndexType>(board: &Board<IndexType>, steps: usize)
where
    IndexType: Eq + Hash + Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + Zero + One + Bounded + ToPrimitive,
{
    let mut game = Game::<_>::new(board.clone());
    for _ in 0..steps {
        game.update();
    }
}

fn do_benchmark<IndexType>(c: &mut Criterion, id: &str, pattern: &str, steps: usize)
where
    IndexType: Eq + Hash + Copy + PartialOrd + Add<Output = IndexType> + Sub<Output = IndexType> + Zero + One + Bounded + ToPrimitive + FromPrimitive,
{
    let from_usize_unwrap = |x| IndexType::from_usize(x).unwrap();
    let loader = Plaintext::new(pattern.as_bytes()).unwrap();
    let board: Board<_> = loader.iter().map(|(x, y)| (from_usize_unwrap(x), from_usize_unwrap(y))).collect();
    c.bench_function(id, |b| b.iter(|| workload(&board, steps)));
}

fn blinker_1k_benchmark(c: &mut Criterion) {
    // See: https://conwaylife.com/wiki/Blinker
    let id = "blinker-1k";
    let pattern = "\
        !Name: Blinker\n\
        OOO\n\
    ";
    let steps = 1000;
    do_benchmark::<i8>(c, id, pattern, steps);
}

criterion_group!(benches, blinker_1k_benchmark);
criterion_main!(benches);