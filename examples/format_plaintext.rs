use anyhow::Result;
use life_backend::format::Plaintext;

fn main() -> Result<()> {
    let parser = Plaintext::new(
        "\
        !Name: Glider\n\
        .O\n\
        ..O\n\
        OOO\n\
    ",
    )?;
    println!("Print as Debug: {:?}", parser);
    println!("Print as Display:\n{}", parser);
    Ok(())
}
