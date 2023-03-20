use anyhow::Result;
use life_backend::format::Plaintext;
use life_backend::{Board, Game};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use i16 as I;

// Creates a new game from the specific board and advances it to the specific generation.
fn do_game(board: Board<I>, steps: usize) -> Game<I> {
    // Utility closure
    let print_game_with_header = |header: &str, game: &Game<_>| {
        println!("{header}");
        println!("(boundary: {:?})", game.board().bounding_box());
        println!("{game}");
    };

    // Create the game with the board
    let mut game = Game::new(board);
    print_game_with_header("Generation 0:", &game);

    // Advance the game to the target generation
    for _ in 0..steps {
        game.update();
    }
    print_game_with_header(&format!("Generation {}:", steps), &game);

    // Return the game
    game
}

fn do_oscillator_test<R>(read: R, period: usize) -> Result<()>
where
    R: Read,
{
    let loader = Plaintext::new(read)?;
    let board: Board<_> = loader.iter().map(|(x, y)| (x as I, y as I)).collect();
    let game = do_game(board.clone(), period);
    assert_eq!(*game.board(), board);
    Ok(())
}

fn do_oscillator_test_with_string(pattern: &str, steps: usize) -> Result<()> {
    do_oscillator_test(pattern.as_bytes(), steps)
}

fn do_oscillator_test_with_path(path_str: &str, steps: usize) -> Result<()> {
    let path = Path::new(path_str);
    let file = File::open(path)?;
    do_oscillator_test(file, steps)
}

fn do_spaceship_test<R>(read: R, steps: usize, relative_position: (I, I)) -> Result<()>
where
    R: Read,
{
    let loader = Plaintext::new(read)?;
    let init: Board<_> = loader.iter().map(|(x, y)| (x as I, y as I)).collect();
    let expected: Board<_> = init.iter().map(|&(x, y)| (x + relative_position.0, y + relative_position.1)).collect();
    let game = do_game(init, steps);
    assert_eq!(*game.board(), expected);
    Ok(())
}

fn do_spaceship_test_with_string(pattern: &str, steps: usize, relative_position: (I, I)) -> Result<()> {
    do_spaceship_test(pattern.as_bytes(), steps, relative_position)
}

fn do_spaceship_test_with_path(path_str: &str, steps: usize, relative_position: (I, I)) -> Result<()> {
    let path = Path::new(path_str);
    let file = File::open(path)?;
    do_spaceship_test(file, steps, relative_position)
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
fn game_toad_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/toad.cells");
    let period = 2;
    do_oscillator_test_with_path(path_str, period)
}

#[test]
fn game_koksgalaxy_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/koksgalaxy.cells");
    let period = 8;
    do_oscillator_test_with_path(path_str, period)
}

#[test]
fn game_pentadecathlon_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/pentadecathlon.cells");
    let period = 15;
    do_oscillator_test_with_path(path_str, period)
}

#[test]
fn game_queenbeeshuttle_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/transqueenbeeshuttle.cells");
    let period = 30;
    do_oscillator_test_with_path(path_str, period)
}

#[test]
fn game_twinbeesshuttle_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/3blocktwinbeesshuttle.cells");
    let period = 46;
    do_oscillator_test_with_path(path_str, period)
}

#[test]
fn game_p60glidershuttle_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/p60glidershuttle.cells");
    let period = 60;
    do_oscillator_test_with_path(path_str, period)
}

#[test]
fn game_centinal_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/centinal.cells");
    let period = 100;
    do_oscillator_test_with_path(path_str, period)
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

#[test]
fn game_lwss_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/lwss.cells");
    let steps = 4;
    let relative_position = (-2, 0);
    do_spaceship_test_with_path(path_str, steps, relative_position)
}
