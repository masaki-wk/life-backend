use anyhow::Result;

use super::{Plaintext, PlaintextBuilder, PlaintextLine};
use crate::Position;

fn do_check(target: &Plaintext, expected_name: &Option<&str>, expected_comments: &[&str], expected_contents: &[PlaintextLine]) {
    let expected_name = expected_name.map(String::from);
    assert_eq!(target.name(), expected_name);
    assert_eq!(target.comments().len(), expected_comments.len());
    for (result, expected) in target.comments().iter().zip(expected_comments.iter()) {
        assert_eq!(result, expected);
    }
    assert_eq!(target.contents.len(), expected_contents.len());
    for (result, expected) in target.contents.iter().zip(expected_contents.iter()) {
        assert_eq!(result, expected);
    }
}

fn do_new_test_to_be_passed(pattern: &str, expected_name: &Option<&str>, expected_comments: &[&str], expected_contents: &[PlaintextLine]) -> Result<()> {
    let target = Plaintext::new(pattern.as_bytes())?;
    do_check(&target, expected_name, expected_comments, expected_contents);
    assert_eq!(target.to_string(), pattern);
    Ok(())
}

fn do_new_test_to_be_failed(pattern: &str) {
    let target = Plaintext::new(pattern.as_bytes());
    assert!(target.is_err());
}

fn do_from_str_test_to_be_passed(pattern: &str, expected_name: &Option<&str>, expected_comments: &[&str], expected_contents: &[PlaintextLine]) -> Result<()> {
    let target: Plaintext = pattern.parse()?;
    do_check(&target, expected_name, expected_comments, expected_contents);
    assert_eq!(target.to_string(), pattern);
    Ok(())
}

#[test]
fn test_new_empty() -> Result<()> {
    let pattern = "";
    do_new_test_to_be_passed(pattern, &None, &Vec::new(), &Vec::new())
}

#[test]
fn test_new_header() -> Result<()> {
    let pattern = "!Name: test\n";
    do_new_test_to_be_passed(pattern, &Some("test"), &Vec::new(), &Vec::new())
}

#[test]
fn test_new_no_header_but_comment() -> Result<()> {
    let pattern = "!comment\n";
    do_new_test_to_be_passed(pattern, &None, &["comment"], &Vec::new())
}

#[test]
fn test_new_header_comment() -> Result<()> {
    let pattern = concat!("!Name: test\n", "!comment\n");
    do_new_test_to_be_passed(pattern, &Some("test"), &["comment"], &Vec::new())
}

#[test]
fn test_new_header_comments() -> Result<()> {
    let pattern = concat!("!Name: test\n", "!comment0\n", "!comment1\n");
    do_new_test_to_be_passed(pattern, &Some("test"), &["comment0", "comment1"], &Vec::new())
}

#[test]
fn test_new_header_content() -> Result<()> {
    let pattern = concat!("!Name: test\n", ".O\n");
    do_new_test_to_be_passed(pattern, &Some("test"), &Vec::new(), &[PlaintextLine(0, vec![1])])
}

#[test]
fn test_new_header_contents() -> Result<()> {
    let pattern = concat!("!Name: test\n", ".O\n", "O.\n");
    do_new_test_to_be_passed(pattern, &Some("test"), &Vec::new(), &[PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])])
}

#[test]
fn test_new_header_comments_contents() -> Result<()> {
    let pattern = concat!("!Name: test\n", "!comment0\n", "!comment1\n", ".O\n", "O.\n");
    do_new_test_to_be_passed(
        pattern,
        &Some("test"),
        &["comment0", "comment1"],
        &[PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])],
    )
}

#[test]
fn test_new_wrong_header() {
    let pattern = "_\n";
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_duplicate_header() -> Result<()> {
    let pattern = concat!("!Name: name0\n", "!Name: name1\n");
    do_new_test_to_be_passed(pattern, &Some("name0"), &["Name: name1"], &Vec::new())
}

#[test]
fn test_new_wrong_content_without_comment() {
    let pattern = concat!("!Name: test\n", "_\n");
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_wrong_content_with_comment() {
    let pattern = concat!("!Name: test\n", "!\n", "_\n");
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_build() -> Result<()> {
    let pattern = [Position(1, 0), Position(0, 1)];
    let target = pattern.iter().collect::<PlaintextBuilder>().build()?;
    do_check(&target, &None, &Vec::new(), &[PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])]);
    Ok(())
}

#[test]
fn test_build_singleline_name() -> Result<()> {
    let pattern = [Position(1, 0), Position(0, 1)];
    let target = pattern.iter().collect::<PlaintextBuilder>().name("test").build()?;
    do_check(&target, &Some("test"), &Vec::new(), &[PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])]);
    Ok(())
}

#[test]
fn test_build_blank_name() -> Result<()> {
    let pattern = [Position(1, 0), Position(0, 1)];
    let target = pattern.iter().collect::<PlaintextBuilder>().name("").build()?;
    do_check(&target, &Some(""), &Vec::new(), &[PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])]);
    Ok(())
}

#[test]
fn test_build_multiline_name() {
    let pattern = [Position(1, 0), Position(0, 1)];
    let target = pattern.iter().collect::<PlaintextBuilder>().name("name\nname").build();
    assert!(target.is_err());
}

#[test]
fn test_build_comment() -> Result<()> {
    let pattern = [Position(1, 0), Position(0, 1)];
    let target = pattern.iter().collect::<PlaintextBuilder>().comment("comment").build()?;
    do_check(&target, &None, &["comment"], &[PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])]);
    Ok(())
}

#[test]
fn test_build_blank_comment() -> Result<()> {
    let pattern = [Position(1, 0), Position(0, 1)];
    let target = pattern.iter().collect::<PlaintextBuilder>().comment("").build()?;
    do_check(&target, &None, &[""], &[PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])]);
    Ok(())
}

#[test]
fn test_build_comments() -> Result<()> {
    let pattern = [Position(1, 0), Position(0, 1)];
    let target = pattern.iter().collect::<PlaintextBuilder>().comment("comment0\ncomment1").build()?;
    do_check(
        &target,
        &None,
        &["comment0", "comment1"],
        &[PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])],
    );
    Ok(())
}

#[test]
fn test_build_name_comment() -> Result<()> {
    let pattern = [Position(1, 0), Position(0, 1)];
    let target = pattern.iter().collect::<PlaintextBuilder>().name("test").comment("comment").build()?;
    do_check(&target, &Some("test"), &["comment"], &[PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])]);
    Ok(())
}

#[test]
fn test_from_str() -> Result<()> {
    let pattern = concat!("!Name: test\n", "!comment0\n", "!comment1\n", ".O\n", "O.\n");
    do_from_str_test_to_be_passed(
        pattern,
        &Some("test"),
        &["comment0", "comment1"],
        &[PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])],
    )
}
