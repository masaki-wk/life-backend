use life_backend::format::Plaintext;

fn main() {
    let parser = Plaintext::new(
        "\
        !Name: Glider\n\
        .O\n\
        ..O\n\
        OOO\n\
    ",
    );
    println!("Print as Debug: {:?}", parser);
}
