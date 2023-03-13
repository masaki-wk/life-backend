use anyhow::Result;
use life_backend::{Board, Game};

use i16 as I;

// Execute the test with the initial pattern, steps to be proceed, and the expected final pattern.
fn do_test(init_pattern: &[(I, I)], steps: usize, expected_final_pattern: &[(I, I)]) -> Result<()> {
    // Create the game with the initial pattern
    let board: Board = init_pattern.iter().collect();
    let mut game = Game::new(board);
    println!("Generation 0:");
    println!("(boundary: {:?})", game.board().bounding_box());
    println!("{game}");

    // Advance the game to the target generation
    for _ in 0..steps {
        game.update();
    }
    println!("Generation {}:", steps - 1);
    println!("(boundary: {:?})", game.board().bounding_box());
    println!("{game}");

    // Check the current state of the game
    let expected_board: Board = expected_final_pattern.iter().collect();
    println!("Expected:");
    println!("(boundary: {:?})", expected_board.bounding_box());
    println!("{expected_board}");
    assert_eq!(*game.board(), expected_board);
    Ok(())
}

#[test]
fn game_glider_test() -> Result<()> {
    let init_pattern = vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]; // Glider pattern
    let steps = 4;
    let expected_final_pattern: Vec<_> = init_pattern.iter().map(|&(x, y)| (x + 1, y + 1)).collect();
    do_test(&init_pattern, steps, &expected_final_pattern)
}
