use anyhow::{Context as _, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use life_backend::format::{Plaintext, PlaintextBuilder};
use life_backend::Position;

fn do_new_test<R>(read: R, expected_positions: &[(usize, usize)]) -> Result<()>
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

    // Check
    assert!(target.live_cells().eq(expected_positions.iter().map(|&(x, y)| Position(x, y))));
    Ok(())
}

fn do_new_test_with_string(input_string: &str, expected_positions: &[(usize, usize)]) -> Result<()> {
    do_new_test(input_string.as_bytes(), expected_positions)
}

fn do_new_test_with_path<P>(input_path: P, expected_positions: &[(usize, usize)]) -> Result<()>
where
    P: AsRef<Path>,
{
    let input_path_for_display = input_path.as_ref().to_owned();
    let file = File::open(input_path).with_context(|| format!("Failed to open \"{}\"", input_path_for_display.display()))?;
    do_new_test(file, expected_positions)
}

fn do_build_test(pattern: &[(usize, usize)], name: Option<String>, comment: Option<String>) -> Result<()> {
    // Create the target with the pattern, the name and the comment
    let target = {
        let builder = pattern.iter().map(|&(x, y)| Position(x, y)).collect::<PlaintextBuilder>();
        match (&name, &comment) {
            (None, None) => builder.build()?,
            (Some(name), None) => builder.name(name).build()?,
            (None, Some(comment)) => builder.comment(comment).build()?,
            (Some(name), Some(comment)) => builder.name(name).comment(comment).build()?,
        }
    };
    println!("Target:");
    println!("{target}");

    // Show the pattern
    println!("Pattern:");
    println!("{:?}", pattern);

    // Check
    assert!(target.live_cells().eq(pattern.iter().map(|&(x, y)| Position(x, y))));
    assert_eq!(target.name(), name);
    if let Some(comment) = &comment {
        let comments: Vec<_> = comment.lines().map(|s| s.to_string()).collect();
        assert_eq!(target.comments(), &comments);
    }
    Ok(())
}

#[test]
fn new_with_string() -> Result<()> {
    let input_pattern = "\
        !Name: Glider\n\
        .O\n\
        ..O\n\
        OOO\n\
    ";
    let expected_positions = vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
    do_new_test_with_string(input_pattern, &expected_positions)
}

#[test]
fn new_with_file() -> Result<()> {
    let input_path = "patterns/rpentomino.cells";
    let expected_positions = vec![(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)];
    do_new_test_with_path(input_path, &expected_positions)
}

#[test]
fn build() -> Result<()> {
    let pattern = vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
    let name = Some("Glider".to_string());
    let comment = Some("----".to_string());
    do_build_test(&pattern, name, comment)
}
