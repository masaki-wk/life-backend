use life_backend::Board;

fn main() {
    let board: Board = [(0, 0), (1, 0), (2, 0), (1, 1)].iter().collect();
    println!("Print as Debug: {:?}", board);
    println!("Print as Display:\n{board}");
}
