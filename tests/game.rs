use anyhow::Result;
use std::path::Path;

use life_backend::format;
use life_backend::{Board, Game, Position};

use i16 as I;

fn load_game<P>(path: P) -> Result<Game<I>>
where
    P: AsRef<Path>,
{
    let handler = format::open(path)?;
    let rule = handler.rule();
    let board: Board<_> = handler.live_cells().map(|pos| Position(pos.0 as I, pos.1 as I)).collect();
    let game = Game::new(rule, board);
    Ok(game)
}

fn print_game(game: &Game<I>, generation: usize) {
    let bbox = game.board().bounding_box();
    let population = game.board().iter().count();
    println!("Generation {generation}: bounding-box = {bbox}, population = {population}");
    println!("{game}");
}

fn advance_game(mut game: Game<I>, steps: usize) -> Game<I> {
    for _ in 0..steps {
        game.update();
    }
    game
}

fn advance_game_with_check(mut game: Game<I>, steps: usize, init: &Board<I>) -> Game<I> {
    for i in 0..steps {
        if i > 0 {
            assert_ne!(game.board(), init);
        }
        game.update();
    }
    game
}

fn shift_board(board: &Board<I>, relative_position: (I, I)) -> Board<I> {
    let board: Board<_> = board
        .iter()
        .map(|pos| Position(pos.0 + relative_position.0, pos.1 + relative_position.1))
        .collect();
    board
}

fn do_oscillator_test<P>(path: P, period: usize) -> Result<()>
where
    P: AsRef<Path>,
{
    // Load the given file and create a game
    let game = load_game(path)?;
    print_game(&game, 0);

    // Set the initial pattern to the variable
    let init = game.board().to_owned();

    // Advance the game to the target generation
    let game = advance_game_with_check(game, period, &init);
    print_game(&game, period);

    // Check the result
    let result = game.board();
    assert_eq!(result, &init);
    Ok(())
}

fn do_stilllife_test<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    do_oscillator_test(path, 1)
}

fn do_spaceship_test<P>(path: P, period: usize, relative_position: (I, I)) -> Result<()>
where
    P: AsRef<Path>,
{
    // Load the given file and create a game
    let game = load_game(path)?;
    print_game(&game, 0);

    // Set the expected pattern to the variable
    let expected = shift_board(game.board(), relative_position);

    // Advance the game to the target generation
    let game = advance_game(game, period);
    print_game(&game, period);

    // Check the result
    let result = game.board();
    assert_eq!(result, &expected);
    Ok(())
}

fn do_methuselah_test<P>(path: P, steps: usize, expected_final_population: usize) -> Result<()>
where
    P: AsRef<Path>,
{
    // Load the given file and create a game
    let game = load_game(path)?;
    print_game(&game, 0);

    // Advance the game to the target generation
    let game = advance_game(game, steps);
    print_game(&game, steps);

    // Check the result
    let result = game.board().iter().count();
    assert_eq!(result, expected_final_population);
    Ok(())
}

fn do_diehard_test<P>(path: P, steps: usize) -> Result<()>
where
    P: AsRef<Path>,
{
    do_methuselah_test(path, steps, 0)
}

macro_rules! create_stilllife_test_function {
    ($function_name:ident, $relative_path_string:literal) => {
        #[test]
        fn $function_name() -> Result<()> {
            let path = $relative_path_string;
            do_stilllife_test(path)
        }
    };
}

macro_rules! create_oscillator_test_function {
    ($function_name:ident, $relative_path_string:literal, $period:expr) => {
        #[test]
        fn $function_name() -> Result<()> {
            let path = $relative_path_string;
            do_oscillator_test(path, $period)
        }
    };
}

macro_rules! create_spaceship_test_function {
    ($function_name:ident, $relative_path_string:literal, $period:expr, $relative_position:expr) => {
        #[test]
        fn $function_name() -> Result<()> {
            let path = $relative_path_string;
            do_spaceship_test(path, $period, $relative_position)
        }
    };
}

macro_rules! create_methuselah_test_function {
    ($function_name:ident, $relative_path_string:literal, $steps:expr, $expected_final_population:expr) => {
        #[test]
        fn $function_name() -> Result<()> {
            let path = $relative_path_string;
            do_methuselah_test(path, $steps, $expected_final_population)
        }
    };
    ($function_name:ident, $relative_path_string:literal, $steps:expr, $expected_final_population:expr, ignore = $reason:literal) => {
        #[test]
        #[ignore = $reason]
        fn $function_name() -> Result<()> {
            let path = $relative_path_string;
            do_methuselah_test(path, $steps, $expected_final_population)
        }
    };
}

macro_rules! create_diehard_test_function {
    ($function_name:ident, $relative_path_string:literal, $steps:expr) => {
        #[test]
        fn $function_name() -> Result<()> {
            let path = $relative_path_string;
            do_diehard_test(path, $steps)
        }
    };
}

#[rustfmt::skip]
mod game {
    use super::*;

    // Still life tests
    create_stilllife_test_function!(stilllife_block_test, "patterns/block.rle");
    create_stilllife_test_function!(stilllife_boat_test, "patterns/boat.rle");
    create_stilllife_test_function!(stilllife_spiral_test, "patterns/spiral.rle");
    create_stilllife_test_function!(stilllife_34life_block_test, "patterns/34life_block.rle");
    create_stilllife_test_function!(stilllife_34life_36bitfortress_test, "patterns/34life_36bitfortress.rle");

    // Oscillator tests
    create_oscillator_test_function!(oscillator_blinker_test, "patterns/blinker.rle", 2);
    create_oscillator_test_function!(oscillator_toad_test, "patterns/toad.rle", 2);
    create_oscillator_test_function!(oscillator_koksgalaxy_test, "patterns/koksgalaxy.rle", 8);
    create_oscillator_test_function!(oscillator_pentadecathlon_test, "patterns/pentadecathlon.rle", 15);
    create_oscillator_test_function!(oscillator_queenbeeshuttle_test, "patterns/transqueenbeeshuttle.rle", 30);
    create_oscillator_test_function!(oscillator_twinbeesshuttle_test, "patterns/3blocktwinbeesshuttle.rle", 46);
    create_oscillator_test_function!(oscillator_p60glidershuttle_test, "patterns/p60glidershuttle.rle", 60);
    create_oscillator_test_function!(oscillator_centinal_test, "patterns/centinal.rle", 100);
    create_oscillator_test_function!(oscillator_highlife_p7_test, "patterns/highlife_p7.rle", 7);
    create_oscillator_test_function!(oscillator_highlife_p10_test, "patterns/highlife_p10.rle", 10);
    create_oscillator_test_function!(oscillator_seeds_duoplet_test, "patterns/seeds_duoplet.rle", 2);
    create_oscillator_test_function!(oscillator_seeds_anchor_test, "patterns/seeds_anchor.rle", 4);
    create_oscillator_test_function!(oscillator_34life_z_test, "patterns/34life_z.rle", 2);
    create_oscillator_test_function!(oscillator_34life_loaf_test, "patterns/34life_loaf.rle", 12);
    create_oscillator_test_function!(oscillator_2x2_largedomino_test, "patterns/2x2_largedomino.rle", 2);
    create_oscillator_test_function!(oscillator_2x2_largetetromino_test, "patterns/2x2_largetetromino.rle", 6);

    // Spaceship tests
    create_spaceship_test_function!(spaceship_glider_test, "patterns/glider.rle", 4, (1, 1));
    create_spaceship_test_function!(spaceship_lwss_test, "patterns/lwss.rle", 4, (-2, 0));
    create_spaceship_test_function!(spaceship_loafer_test, "patterns/loafer.rle", 7, (-1, 0));
    create_spaceship_test_function!(spaceship_copperhead_test, "patterns/copperhead.rle", 10, (0, -1));
    create_spaceship_test_function!(spaceship_highlife_bomber_test, "patterns/highlife_bomber.rle", 48, (8, 8));
    create_spaceship_test_function!(spaceship_daynight_rocket_test, "patterns/daynight_rocket.rle", 40, (-20, 0));
    create_spaceship_test_function!(spaceship_seeds_moon_test, "patterns/seeds_moon.rle", 1, (-1, 0));
    create_spaceship_test_function!(spaceship_34life_glider_test, "patterns/34life_glider.rle", 3, (0, -1));
    create_spaceship_test_function!(spaceship_2x2_crawler_test, "patterns/2x2_crawler.rle", 8, (1, -1));

    // Methuselah tests
    create_methuselah_test_function!(methuselah_rpentomino_test, "patterns/rpentomino.rle", 1103, 116);
    create_methuselah_test_function!(methuselah_bheptomino_test, "patterns/bheptomino.rle", 148, 28);
    create_methuselah_test_function!(methuselah_eheptomino_test, "patterns/eheptomino.rle", 343, 52);
    create_methuselah_test_function!(methuselah_fheptomino_test, "patterns/fheptomino.rle", 437, 61);
    create_methuselah_test_function!(methuselah_herschel_test, "patterns/herschel.rle", 128, 24);
    create_methuselah_test_function!(methuselah_piheptomino_test, "patterns/piheptomino.rle", 173, 55);
    create_methuselah_test_function!(methuselah_century_test, "patterns/century.rle", 103, 15);
    create_methuselah_test_function!(methuselah_queenbee_test, "patterns/queenbee.rle", 191, 30);
    create_methuselah_test_function!(methuselah_thunderbird_test, "patterns/thunderbird.rle", 243, 46);
    create_methuselah_test_function!(methuselah_switchengine_test, "patterns/switchengine.rle", 3911, 842, ignore = "too long for testing");
    create_methuselah_test_function!(methuselah_acorn_test, "patterns/acorn.rle", 5206, 633, ignore = "too long for testing");
    create_methuselah_test_function!(methuselah_bunnies_test, "patterns/bunnies.rle", 17332, 1744, ignore = "too long for testing");

    // Diehard tests
    create_diehard_test_function!(diehard_diehard_test, "patterns/diehard.rle", 130);
}
