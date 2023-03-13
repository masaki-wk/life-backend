use anyhow::Result;
use life_backend::format::Plaintext;
use std::io::Read;

use i16 as I;

// Execute the test with the Read implementor and the expected positions.
fn do_test<R>(read: R, expected_positions: &[(I, I)]) -> Result<()>
where
    R: Read,
{
    // Create the target with the Read implementor
    let target = Plaintext::<I>::new(read)?;
    println!("Target:");
    println!("{target}");

    // Show the expected positions
    println!("Expected positions:");
    println!("{:?}", expected_positions);

    assert!(target.iter().eq(expected_positions.iter().copied()));
    Ok(())
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
    do_test(input_pattern.as_bytes(), &expected_positions)
}
