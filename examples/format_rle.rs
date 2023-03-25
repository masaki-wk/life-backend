use anyhow::Result;
use life_backend::format::Rle;

fn main() -> Result<()> {
    let pattern = "\
        #N Glider\n\
        x = 3, y = 3\n\
        bo$2bo$3o!\n\
    ";
    let parser = Rle::new(pattern.as_bytes())?;
    println!("{:?}", parser);
    Ok(())
}
