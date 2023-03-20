use anyhow::Result;
use life_backend::format::Plaintext;
use life_backend::{Board, Game};
use std::fs::File;
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

fn do_oscillator_test(path_str: &str, period: usize) -> Result<()> {
    let path = Path::new(path_str);
    let file = File::open(path)?;
    let loader = Plaintext::new(file)?;
    let board: Board<_> = loader.iter().map(|(x, y)| (x as I, y as I)).collect();
    let game = do_game(board.clone(), period);
    assert_eq!(*game.board(), board);
    Ok(())
}

fn do_stilllife_test(path_str: &str) -> Result<()> {
    do_oscillator_test(path_str, 1)
}

fn do_spaceship_test(path_str: &str, period: usize, relative_position: (I, I)) -> Result<()> {
    let path = Path::new(path_str);
    let file = File::open(path)?;
    let loader = Plaintext::new(file)?;
    let init: Board<_> = loader.iter().map(|(x, y)| (x as I, y as I)).collect();
    let expected: Board<_> = init.iter().map(|&(x, y)| (x + relative_position.0, y + relative_position.1)).collect();
    let game = do_game(init, period);
    assert_eq!(*game.board(), expected);
    Ok(())
}

macro_rules! create_stilllife_test_function {
    ($function_name:ident, $relative_path_string:literal) => {
        #[test]
        fn $function_name() -> Result<()> {
            let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/", $relative_path_string);
            do_stilllife_test(path_str)
        }
    };
}

macro_rules! create_oscillator_test_function {
    ($function_name:ident, $relative_path_string:literal, $period:expr) => {
        #[test]
        fn $function_name() -> Result<()> {
            let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/", $relative_path_string);
            do_oscillator_test(path_str, $period)
        }
    };
}

macro_rules! create_spaceship_test_function {
    ($function_name:ident, $relative_path_string:literal, $period:expr, $relative_position:expr) => {
        #[test]
        fn $function_name() -> Result<()> {
            let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/", $relative_path_string);
            do_spaceship_test(path_str, $period, $relative_position)
        }
    };
}

// Still life tests

create_stilllife_test_function!(game_block_test, "patterns/block.cells");
create_stilllife_test_function!(game_boat_test, "patterns/boat.cells");
create_stilllife_test_function!(game_spiral_test, "patterns/spiral.cells");

// Oscillator tests

create_oscillator_test_function!(game_blinker_test, "patterns/blinker.cells", 2);
create_oscillator_test_function!(game_toad_test, "patterns/toad.cells", 2);
create_oscillator_test_function!(game_koksgalaxy_test, "patterns/koksgalaxy.cells", 8);
create_oscillator_test_function!(game_pentadecathlon_test, "patterns/pentadecathlon.cells", 15);
create_oscillator_test_function!(game_queenbeeshuttle_test, "patterns/transqueenbeeshuttle.cells", 30);
create_oscillator_test_function!(game_twinbeesshuttle_test, "patterns/3blocktwinbeesshuttle.cells", 46);
create_oscillator_test_function!(game_p60glidershuttle_test, "patterns/p60glidershuttle.cells", 60);
create_oscillator_test_function!(game_centinal_test, "patterns/centinal.cells", 100);

// Spaceship tests

create_spaceship_test_function!(game_glider_test, "patterns/glider.cells", 4, (1, 1));
create_spaceship_test_function!(game_lwss_test, "patterns/lwss.cells", 4, (-2, 0));
create_spaceship_test_function!(game_loafer_test, "patterns/loafer.cells", 7, (-1, 0));
create_spaceship_test_function!(game_copperhead_test, "patterns/copperhead.cells", 10, (0, -1));
