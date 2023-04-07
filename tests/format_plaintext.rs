use anyhow::{Context as _, Result};
use life_backend::format::Plaintext;
use std::fs::File;
use std::io::Read;
use std::path::Path;

// Execute the test with the Read implementor and the expected positions.
fn do_test<R>(read: R, expected_positions: &[(usize, usize)]) -> Result<()>
where
    R: Read,
{
    // Create the target with the Read implementor
    let target = Plaintext::new(read)?;
    println!("Target:");
    println!("{target}");

    // Show the expected positions
    println!("Expected positions:");
    println!("{:?}", expected_positions);

    assert!(target.iter().eq(expected_positions.iter().copied()));
    Ok(())
}

fn do_test_with_string(input_string: &str, expected_positions: &[(usize, usize)]) -> Result<()> {
    do_test(input_string.as_bytes(), expected_positions)
}

fn do_test_with_path(input_path_string: &str, expected_positions: &[(usize, usize)]) -> Result<()> {
    let path = Path::new(input_path_string);
    let file = File::open(path).with_context(|| format!("Failed to open \"{}\"", path.display()))?;
    do_test(file, expected_positions)
}

#[test]
fn format_plaintext_new_with_string_test() -> Result<()> {
    let input_pattern = "\
        !Name: Glider\n\
        .O\n\
        ..O\n\
        OOO\n\
    ";
    let expected_positions = vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
    do_test_with_string(input_pattern, &expected_positions)
}

#[test]
fn format_plaintext_new_with_file_test() -> Result<()> {
    let input_path_string = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/rpentomino.cells");
    let expected_positions = vec![(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)];
    do_test_with_path(input_path_string, &expected_positions)
}
