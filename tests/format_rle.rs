use anyhow::{Context as _, Result};
use life_backend::format::{Rle, RleBuilder};
use std::fs::File;
use std::io::Read;
use std::path::Path;

// Execute the test with the Read implementor and the expected positions.
fn do_new_test<R>(read: R, expected_positions: &[(usize, usize)]) -> Result<()>
where
    R: Read,
{
    // Create the target with the Read implementor
    let target = Rle::new(read)?;
    println!("Target:");
    println!("{target}");

    // Show the expected positions
    println!("Expected positions:");
    println!("{:?}", expected_positions);

    assert!(target.iter().eq(expected_positions.iter().copied()));
    Ok(())
}

fn do_new_test_with_string(input_string: &str, expected_positions: &[(usize, usize)]) -> Result<()> {
    do_new_test(input_string.as_bytes(), expected_positions)
}

fn do_new_test_with_path(input_path_string: &str, expected_positions: &[(usize, usize)]) -> Result<()> {
    let path = Path::new(input_path_string);
    let file = File::open(path).with_context(|| format!("Failed to open \"{}\"", path.display()))?;
    do_new_test(file, expected_positions)
}

fn do_build_test(pattern: &[(usize, usize)], name: Option<String>, created: Option<String>, comment: Option<String>) -> Result<()> {
    // Create the target with the pattern, the name, the created and the comment
    let target = {
        let builder = pattern.iter().collect::<RleBuilder>();
        match (&name, &created, &comment) {
            (None, None, None) => builder.build()?,
            (Some(name), None, None) => builder.name(name).build()?,
            (None, Some(created), None) => builder.created(created).build()?,
            (Some(name), Some(created), None) => builder.name(name).created(created).build()?,
            (None, None, Some(comment)) => builder.comment(comment).build()?,
            (Some(name), None, Some(comment)) => builder.name(name).comment(comment).build()?,
            (None, Some(created), Some(comment)) => builder.created(created).comment(comment).build()?,
            (Some(name), Some(created), Some(comment)) => builder.name(name).created(created).comment(comment).build()?,
        }
    };
    println!("Target:");
    println!("{target}");

    // Show the pattern
    println!("Pattern:");
    println!("{:?}", pattern);

    // Check
    assert!(target.iter().eq(pattern.iter().copied()));
    Ok(())
}

#[test]
fn format_rle_new_with_string_test() -> Result<()> {
    let input_pattern = "\
        #N Glider\n\
        x = 3, y = 3\n\
        bo$2bo$3o!\n\
    ";
    let expected_positions = vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
    do_new_test_with_string(input_pattern, &expected_positions)
}

#[test]
fn format_rle_new_with_file_test() -> Result<()> {
    let input_path_string = concat!(env!("CARGO_MANIFEST_DIR"), "/patterns/rpentomino.rle");
    let expected_positions = vec![(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)];
    do_new_test_with_path(input_path_string, &expected_positions)
}

#[test]
fn format_rle_build_test() -> Result<()> {
    let pattern = vec![(0, 0), (1, 0), (2, 0), (1, 1)];
    let name = Some(String::from("T-tetromino"));
    let comment = Some(String::from("----"));
    do_build_test(&pattern, name, None, comment)
}
