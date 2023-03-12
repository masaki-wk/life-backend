use life_backend::{Board, Game};

fn main() {
    let pattern = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
    let steps = 4;
    let board: Board = pattern.iter().collect();
    let mut game = Game::new(board);
    for i in 0..=steps {
        if i != 0 {
            game.update();
        }
        println!("Generation {i}:");
        println!("{game}");
    }
}
