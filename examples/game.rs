use life_backend::{Board, Game};

fn main() {
    let board: Board = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)].iter().collect();
    let game = Game::new(board);
    println!("Print as Debug: {:?}", game);
    println!("Print as Display:\n{}", game);
}
