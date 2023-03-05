use life_backend::Board;

fn main() {
    let pattern = [(0, 0), (1, 0), (2, 0), (1, 1)]; // T-tetromino
    let board: Board = pattern.iter().collect();
    println!("Print as Debug: {:?}", board);
    println!("Print as Display:\n{board}");
}
