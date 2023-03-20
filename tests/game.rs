use anyhow::Result;
use life_backend::format::Plaintext;
use life_backend::{Board, Game};
use std::io::Read;

use i16 as I;

// Start from the specific pattern and advance to the specific generation, and check if the final state is the same as the expected pattern.
fn do_test(init: &Board<I>, steps: usize, expected: &Board<I>) -> Result<()> {
    // Utility closure
    let print_game_with_header = |header: &str, game: &Game<_>| {
        println!("{header}");
        println!("(boundary: {:?})", game.board().bounding_box());
        println!("{game}");
    };

    // Create the game with the initial pattern
    let mut game = Game::new(init.clone());
    print_game_with_header("Generation 0:", &game);

    // Advance the game to the target generation
    for _ in 0..steps {
        game.update();
    }
    print_game_with_header(&format!("Generation {}:", steps), &game);

    // Check the current state of the game
    print_game_with_header("Expected:", &game);
    assert_eq!(*game.board(), *expected);
    Ok(())
}

fn do_oscillator_test<R>(read: R, period: usize) -> Result<()>
where
    R: Read,
{
    let loader = Plaintext::new(read)?;
    let board: Board<_> = loader.iter().map(|(x, y)| (x as I, y as I)).collect();
    do_test(&board, period, &board)
}

fn do_oscillator_test_with_string(pattern: &str, steps: usize) -> Result<()> {
    do_oscillator_test(pattern.as_bytes(), steps)
}

fn do_spaceship_test<R>(read: R, steps: usize, relative_position: (I, I)) -> Result<()>
where
    R: Read,
{
    let loader = Plaintext::new(read)?;
    let init: Board<_> = loader.iter().map(|(x, y)| (x as I, y as I)).collect();
    let expected: Board<_> = init.iter().map(|&(x, y)| (x + relative_position.0, y + relative_position.1)).collect();
    do_test(&init, steps, &expected)
}

fn do_spaceship_test_with_string(pattern: &str, steps: usize, relative_position: (I, I)) -> Result<()> {
    do_spaceship_test(pattern.as_bytes(), steps, relative_position)
}

#[test]
fn game_blinker_test() -> Result<()> {
    let pattern = "\
        !Name: Blinker\n\
        OOO\n\
    ";
    let period = 2;
    do_oscillator_test_with_string(pattern, period)
}

#[test]
fn game_glider_test() -> Result<()> {
    let pattern = "\
        !Name: Glider\n\
        .O.\n\
        ..O\n\
        OOO\n\
    ";
    let steps = 4;
    let relative_position = (1, 1);
    do_spaceship_test_with_string(pattern, steps, relative_position)
}
