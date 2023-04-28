use anyhow::Result;
use std::fs::File;
use std::path::Path;

use life_backend::format::Rle;
use life_backend::{Board, Game};

use i16 as I;

fn load_board(path_str: &str) -> Result<Board<I>> {
    let path = Path::new(path_str);
    let file = File::open(path)?;
    let parser = Rle::new(file)?;
    let board: Board<_> = parser.iter().map(|(x, y)| (x as I, y as I)).collect();
    Ok(board)
}

fn print_game_with_header(header: &str, game: &Game<I>) {
    println!("{header}");
    println!("(boundary: {:?})", game.board().bounding_box());
    println!("{game}");
}

fn do_oscillator_test(path_str: &str, period: usize) -> Result<()> {
    // Load the board
    let init = load_board(path_str)?;

    // Create the game
    let mut game = Game::new(init.clone());
    print_game_with_header("Generation 0:", &game);

    // Advance the game to the target generation
    for i in 0..period {
        if i > 0 {
            assert_ne!(game.board(), &init);
        }
        game.update();
    }
    print_game_with_header(&format!("Generation {}:", period), &game);

    // Check the result
    let result = game.board();
    assert_eq!(result, &init);
    Ok(())
}

fn do_stilllife_test(path_str: &str) -> Result<()> {
    do_oscillator_test(path_str, 1)
}

fn do_spaceship_test(path_str: &str, period: usize, relative_position: (I, I)) -> Result<()> {
    // Load the board
    let init = load_board(path_str)?;

    // Setup the expected board
    let expected: Board<_> = init.iter().map(|&(x, y)| (x + relative_position.0, y + relative_position.1)).collect();

    // Create the game
    let mut game = Game::new(init);
    print_game_with_header("Generation 0:", &game);

    // Advance the game to the target generation
    for _ in 0..period {
        game.update();
    }
    print_game_with_header(&format!("Generation {}:", period), &game);

    // Check the result
    let result = game.board();
    assert_eq!(result, &expected);
    Ok(())
}

fn do_methuselah_test(path_str: &str, steps: usize, expected_final_population: usize) -> Result<()> {
    // Load the board
    let init = load_board(path_str)?;

    // Create the game
    let mut game = Game::new(init);
    print_game_with_header("Generation 0:", &game);

    // Advance the game to the target generation
    for _ in 0..steps {
        game.update();
    }
    print_game_with_header(&format!("Generation {}:", steps), &game);

    // Check the result
    let result = game.board().iter().count();
    assert_eq!(result, expected_final_population);
    Ok(())
}

fn do_diehard_test(path_str: &str, steps: usize) -> Result<()> {
    do_methuselah_test(path_str, steps, 0)
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

macro_rules! create_methuselah_test_function {
    ($function_name:ident, $relative_path_string:literal, $steps:expr, $expected_final_population:expr) => {
        #[test]
        fn $function_name() -> Result<()> {
            let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/", $relative_path_string);
            do_methuselah_test(path_str, $steps, $expected_final_population)
        }
    };
    ($function_name:ident, $relative_path_string:literal, $steps:expr, $expected_final_population:expr, ignore = $reason:literal) => {
        #[test]
        #[ignore = $reason]
        fn $function_name() -> Result<()> {
            let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/", $relative_path_string);
            do_methuselah_test(path_str, $steps, $expected_final_population)
        }
    };
}

macro_rules! create_diehard_test_function {
    ($function_name:ident, $relative_path_string:literal, $steps:expr) => {
        #[test]
        fn $function_name() -> Result<()> {
            let path_str = concat!(env!("CARGO_MANIFEST_DIR"), "/", $relative_path_string);
            do_diehard_test(path_str, $steps)
        }
    };
}

// Still life tests

create_stilllife_test_function!(game_block_test, "patterns/block.rle");
create_stilllife_test_function!(game_boat_test, "patterns/boat.rle");
create_stilllife_test_function!(game_spiral_test, "patterns/spiral.rle");

// Oscillator tests

create_oscillator_test_function!(game_blinker_test, "patterns/blinker.rle", 2);
create_oscillator_test_function!(game_toad_test, "patterns/toad.rle", 2);
create_oscillator_test_function!(game_koksgalaxy_test, "patterns/koksgalaxy.rle", 8);
create_oscillator_test_function!(game_pentadecathlon_test, "patterns/pentadecathlon.rle", 15);
create_oscillator_test_function!(game_queenbeeshuttle_test, "patterns/transqueenbeeshuttle.rle", 30);
create_oscillator_test_function!(game_twinbeesshuttle_test, "patterns/3blocktwinbeesshuttle.rle", 46);
create_oscillator_test_function!(game_p60glidershuttle_test, "patterns/p60glidershuttle.rle", 60);
create_oscillator_test_function!(game_centinal_test, "patterns/centinal.rle", 100);

// Spaceship tests

create_spaceship_test_function!(game_glider_test, "patterns/glider.rle", 4, (1, 1));
create_spaceship_test_function!(game_lwss_test, "patterns/lwss.rle", 4, (-2, 0));
create_spaceship_test_function!(game_loafer_test, "patterns/loafer.rle", 7, (-1, 0));
create_spaceship_test_function!(game_copperhead_test, "patterns/copperhead.rle", 10, (0, -1));

// Methuselah tests

create_methuselah_test_function!(game_rpentomino_test, "patterns/rpentomino.rle", 1103, 116);
create_methuselah_test_function!(game_bheptomino_test, "patterns/bheptomino.rle", 148, 28);
create_methuselah_test_function!(game_eheptomino_test, "patterns/eheptomino.rle", 343, 52);
create_methuselah_test_function!(game_fheptomino_test, "patterns/fheptomino.rle", 437, 61);
create_methuselah_test_function!(game_herschel_test, "patterns/herschel.rle", 128, 24);
create_methuselah_test_function!(game_piheptomino_test, "patterns/piheptomino.rle", 173, 55);
create_methuselah_test_function!(game_century_test, "patterns/century.rle", 103, 15);
create_methuselah_test_function!(game_queenbee_test, "patterns/queenbee.rle", 191, 30);
create_methuselah_test_function!(game_thunderbird_test, "patterns/thunderbird.rle", 243, 46);
create_methuselah_test_function!(game_switchengine_test, "patterns/switchengine.rle", 3911, 842, ignore = "too long for testing");
create_methuselah_test_function!(game_acorn_test, "patterns/acorn.rle", 5206, 633, ignore = "too long for testing");
create_methuselah_test_function!(game_bunnies_test, "patterns/bunnies.rle", 17332, 1744, ignore = "too long for testing");

// Diehard tests

create_diehard_test_function!(game_diehard_test, "patterns/diehard.rle", 130);
