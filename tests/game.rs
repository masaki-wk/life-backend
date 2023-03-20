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

fn do_oscillator_test_with_path(path_str: &str, steps: usize) -> Result<()> {
    let path = Path::new(path_str);
    let file = File::open(path)?;
    do_oscillator_test(file, steps)
}

fn do_stilllife_test_with_path(path_str: &str) -> Result<()> {
    let path = Path::new(path_str);
    let file = File::open(path)?;
    do_oscillator_test(file, 1)
}

fn do_spaceship_test<R>(read: R, period: usize, relative_position: (I, I)) -> Result<()>
where
    R: Read,
{
    let loader = Plaintext::new(read)?;
    let init: Board<_> = loader.iter().map(|(x, y)| (x as I, y as I)).collect();
    let expected: Board<_> = init.iter().map(|&(x, y)| (x + relative_position.0, y + relative_position.1)).collect();
    let game = do_game(init, period);
    assert_eq!(*game.board(), expected);
    Ok(())
}

fn do_spaceship_test_with_path(path_str: &str, period: usize, relative_position: (I, I)) -> Result<()> {
    let path = Path::new(path_str);
    let file = File::open(path)?;
    do_spaceship_test(file, period, relative_position)
}

// Still life tests

#[test]
fn game_block_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/block.cells");
    do_stilllife_test_with_path(path_str)
}

#[test]
fn game_boat_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/boat.cells");
    do_stilllife_test_with_path(path_str)
}

#[test]
fn game_spiral_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/spiral.cells");
    do_stilllife_test_with_path(path_str)
}

// Oscillator tests

#[test]
fn game_blinker_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/blinker.cells");
    let period = 2;
    do_oscillator_test_with_path(path_str, period)
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

// Spaceship tests

#[test]
fn game_glider_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/glider.cells");
    let period = 4;
    let relative_position = (1, 1);
    do_spaceship_test_with_path(path_str, period, relative_position)
}

#[test]
fn game_lwss_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/lwss.cells");
    let period = 4;
    let relative_position = (-2, 0);
    do_spaceship_test_with_path(path_str, period, relative_position)
}

#[test]
fn game_loafer_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/loafer.cells");
    let period = 7;
    let relative_position = (-1, 0);
    do_spaceship_test_with_path(path_str, period, relative_position)
}

#[test]
fn game_copperhead_test() -> Result<()> {
    let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/copperhead.cells");
    let period = 10;
    let relative_position = (0, -1);
    do_spaceship_test_with_path(path_str, period, relative_position)
}
