use anyhow::Result;
use life_backend::format::Plaintext;

fn main() -> Result<()> {
    let pattern = "\
        !Name: Glider\n\
        .O\n\
        ..O\n\
        OOO\n\
    ";
    let parser = Plaintext::<i16>::new(pattern.as_bytes())?;
    println!("Print as Debug: {:?}", parser);
    println!("Print as Display:");
    println!("{}", parser);
    let poses: Vec<_> = parser.iter().collect();
    println!("{:?}", poses);
    Ok(())
}
