use anyhow::{Context as _, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use life_backend::format::{Rle, RleBuilder};
use life_backend::{Position, Rule};

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

fn do_build_test(pattern: &[(usize, usize)], name: Option<String>, created: Option<String>, comment: Option<String>, rule: Option<Rule>) -> Result<()> {
    // Create the target with the pattern, the name, the created, the comment and the rule
    let target = {
        let builder = pattern.iter().map(|&(x, y)| Position(x, y)).collect::<RleBuilder>();
        match (name, created, comment, rule) {
            (None, None, None, None) => builder.build()?,
            (Some(name), None, None, None) => builder.name(&name).build()?,
            (None, Some(created), None, None) => builder.created(&created).build()?,
            (Some(name), Some(created), None, None) => builder.name(&name).created(&created).build()?,
            (None, None, Some(comment), None) => builder.comment(&comment).build()?,
            (Some(name), None, Some(comment), None) => builder.name(&name).comment(&comment).build()?,
            (None, Some(created), Some(comment), None) => builder.created(&created).comment(&comment).build()?,
            (Some(name), Some(created), Some(comment), None) => builder.name(&name).created(&created).comment(&comment).build()?,
            (None, None, None, Some(rule)) => builder.rule(rule).build()?,
            (Some(name), None, None, Some(rule)) => builder.name(&name).rule(rule).build()?,
            (None, Some(created), None, Some(rule)) => builder.created(&created).rule(rule).build()?,
            (Some(name), Some(created), None, Some(rule)) => builder.name(&name).created(&created).rule(rule).build()?,
            (None, None, Some(comment), Some(rule)) => builder.comment(&comment).rule(rule).build()?,
            (Some(name), None, Some(comment), Some(rule)) => builder.name(&name).comment(&comment).rule(rule).build()?,
            (None, Some(created), Some(comment), Some(rule)) => builder.created(&created).comment(&comment).rule(rule).build()?,
            (Some(name), Some(created), Some(comment), Some(rule)) => builder.name(&name).created(&created).comment(&comment).rule(rule).build()?,
        }
    };
    println!("Target:");
    println!("{target}");

    // Show the pattern
    println!("Pattern:");
    println!("{:?}", pattern);

    // Check
    assert!(target.live_cells().eq(pattern.iter().map(|&(x, y)| Position(x, y))));
    Ok(())
}

#[test]
fn new_with_string() -> Result<()> {
    let input_pattern = "\
        #N Glider\n\
        x = 3, y = 3\n\
        bo$2bo$3o!\n\
    ";
    let expected_positions = vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
    do_new_test_with_string(input_pattern, &expected_positions)
}

#[test]
fn new_with_file() -> Result<()> {
    let input_path = "patterns/rpentomino.rle";
    let expected_positions = vec![(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)];
    do_new_test_with_path(input_path, &expected_positions)
}

#[test]
fn build() -> Result<()> {
    let pattern = vec![(0, 0), (1, 0), (2, 0), (1, 1)];
    let name = Some("T-tetromino".to_string());
    let comment = Some("----".to_string());
    let rule = Some(Rule::conways_life());
    do_build_test(&pattern, name, None, comment, rule)
}
