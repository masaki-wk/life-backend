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
    let expected_name = None;
    let expected_comments = Vec::new();
    let expected_contents = Vec::new();
    do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
}

#[test]
fn test_new_header() -> Result<()> {
    let pattern = "!Name: test\n";
    let expected_name = Some("test");
    let expected_comments = Vec::new();
    let expected_contents = Vec::new();
    do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
}

#[test]
fn test_new_no_header_but_comment() -> Result<()> {
    let pattern = "!comment\n";
    let expected_name = None;
    let expected_comments = vec!["comment"];
    let expected_contents = Vec::new();
    do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
}

#[test]
fn test_new_header_comment() -> Result<()> {
    let pattern = concat!("!Name: test\n", "!comment\n");
    let expected_name = Some("test");
    let expected_comments = vec!["comment"];
    let expected_contents = Vec::new();
    do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
}

#[test]
fn test_new_header_comments() -> Result<()> {
    let pattern = concat!("!Name: test\n", "!comment0\n", "!comment1\n");
    let expected_name = Some("test");
    let expected_comments = vec!["comment0", "comment1"];
    let expected_contents = Vec::new();
    do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
}

#[test]
fn test_new_header_content() -> Result<()> {
    let pattern = concat!("!Name: test\n", ".O\n");
    let expected_name = Some("test");
    let expected_comments = Vec::new();
    let expected_contents = vec![PlaintextLine(0, vec![1])];
    do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
}

#[test]
fn test_new_header_contents() -> Result<()> {
    let pattern = concat!("!Name: test\n", ".O\n", "O.\n");
    let expected_name = Some("test");
    let expected_comments = Vec::new();
    let expected_contents = vec![PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])];
    do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
}

#[test]
fn test_new_header_comments_contents() -> Result<()> {
    let pattern = concat!("!Name: test\n", "!comment0\n", "!comment1\n", ".O\n", "O.\n");
    let expected_name = Some("test");
    let expected_comments = vec!["comment0", "comment1"];
    let expected_contents = vec![PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])];
    do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
}

#[test]
fn test_new_wrong_header() {
    let pattern = "_\n";
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_duplicate_header() -> Result<()> {
    let pattern = concat!("!Name: name0\n", "!Name: name1\n");
    let expected_name = Some("name0");
    let expected_comments = vec!["Name: name1"];
    let expected_contents = Vec::new();
    do_new_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
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
    let expected_name = None;
    let expected_comments = Vec::new();
    let expected_contents = vec![PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])];
    let target = pattern.iter().collect::<PlaintextBuilder>().build()?;
    do_check(&target, &expected_name, &expected_comments, &expected_contents);
    Ok(())
}

#[test]
fn test_build_singleline_name() -> Result<()> {
    let pattern = [Position(1, 0), Position(0, 1)];
    let expected_name = Some("test");
    let expected_comments = Vec::new();
    let expected_contents = vec![PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])];
    let target = pattern.iter().collect::<PlaintextBuilder>().name("test").build()?;
    do_check(&target, &expected_name, &expected_comments, &expected_contents);
    Ok(())
}

#[test]
fn test_build_blank_name() -> Result<()> {
    let pattern = [Position(1, 0), Position(0, 1)];
    let expected_name = Some("");
    let expected_comments = Vec::new();
    let expected_contents = vec![PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])];
    let target = pattern.iter().collect::<PlaintextBuilder>().name("").build()?;
    do_check(&target, &expected_name, &expected_comments, &expected_contents);
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
    let expected_name = None;
    let expected_comments = vec!["comment"];
    let expected_contents = vec![PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])];
    let target = pattern.iter().collect::<PlaintextBuilder>().comment("comment").build()?;
    do_check(&target, &expected_name, &expected_comments, &expected_contents);
    Ok(())
}

#[test]
fn test_build_blank_comment() -> Result<()> {
    let pattern = [Position(1, 0), Position(0, 1)];
    let expected_name = None;
    let expected_comments = vec![""];
    let expected_contents = vec![PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])];
    let target = pattern.iter().collect::<PlaintextBuilder>().comment("").build()?;
    do_check(&target, &expected_name, &expected_comments, &expected_contents);
    Ok(())
}

#[test]
fn test_build_comments() -> Result<()> {
    let pattern = [Position(1, 0), Position(0, 1)];
    let expected_name = None;
    let expected_comments = vec!["comment0", "comment1"];
    let expected_contents = vec![PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])];
    let target = pattern.iter().collect::<PlaintextBuilder>().comment("comment0\ncomment1").build()?;
    do_check(&target, &expected_name, &expected_comments, &expected_contents);
    Ok(())
}

#[test]
fn test_build_name_comment() -> Result<()> {
    let pattern = [Position(1, 0), Position(0, 1)];
    let expected_name = Some("test");
    let expected_comments = vec!["comment"];
    let expected_contents = vec![PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])];
    let target = pattern.iter().collect::<PlaintextBuilder>().name("test").comment("comment").build()?;
    do_check(&target, &expected_name, &expected_comments, &expected_contents);
    Ok(())
}

#[test]
fn test_from_str() -> Result<()> {
    let pattern = concat!("!Name: test\n", "!comment0\n", "!comment1\n", ".O\n", "O.\n");
    let expected_name = Some("test");
    let expected_comments = vec!["comment0", "comment1"];
    let expected_contents = vec![PlaintextLine(0, vec![1]), PlaintextLine(1, vec![0])];
    do_from_str_test_to_be_passed(pattern, &expected_name, &expected_comments, &expected_contents)
}
