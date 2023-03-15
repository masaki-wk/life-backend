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

fn moldon30p25_1k_benchmark(c: &mut Criterion) {
    // See: https://conwaylife.com/wiki/30P25#LCM_oscillators, p100 with mold
    let id = "moldon30p25-1k";
    let pattern = "\
        !Name: moldon30p25\n\
        ..................OO\n\
        ..................O.\n\
        ................O.O.\n\
        ..........O..O..OO..\n\
        ..........O..O......\n\
        .........O....O.....\n\
        .............O......\n\
        ..........OO.O......\n\
        ....................\n\
        ....................\n\
        ....................\n\
        ....................\n\
        ......O.OO..........\n\
        ......O.............\n\
        .....O....O.........\n\
        ......O..O......OOO.\n\
        ..OO..O..O....O.OOO.\n\
        .O.O.........O.O.O..\n\
        .O...........O..O...\n\
        OO............OO....\n\
    ";
    let steps = 1000;
    do_benchmark::<i8>(c, id, pattern, steps);
}

criterion_group!(benches, blinker_1k_benchmark, moldon30p25_1k_benchmark);
criterion_main!(benches);
