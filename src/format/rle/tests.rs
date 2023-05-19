use anyhow::Result;

use super::{Rle, RleBuilder};
use crate::{Position, Rule};

const RULE_HIGHLIFE: Rule = Rule::new(
    &[false, false, false, true, false, false, true, false, false],
    &[false, false, true, true, false, false, false, false, false],
);

fn do_check(
    target: &Rle,
    expected_width: usize,
    expected_height: usize,
    expected_rule: &Rule,
    expected_comments: &[&str],
    expected_contents: &[(usize, usize, usize)],
    expected_pattern: Option<&str>,
) {
    assert_eq!(target.width(), expected_width);
    assert_eq!(target.height(), expected_height);
    assert_eq!(target.rule(), expected_rule);
    assert_eq!(target.comments().len(), expected_comments.len());
    for (result, expected) in target.comments().iter().zip(expected_comments.iter()) {
        assert_eq!(result, expected);
    }
    assert_eq!(target.contents.len(), expected_contents.len());
    for (result, &expected) in target.contents.iter().zip(expected_contents.iter()) {
        assert_eq!((result.pad_lines, result.pad_dead_cells, result.live_cells), expected);
    }
    if let Some(expected_pattern) = expected_pattern {
        assert_eq!(target.to_string(), expected_pattern);
    }
}

fn do_new_test_to_be_passed(
    pattern: &str,
    expected_width: usize,
    expected_height: usize,
    expected_rule: &Rule,
    expected_comments: &[&str],
    expected_contents: &[(usize, usize, usize)],
    check_tostring: bool,
) -> Result<()> {
    let target = Rle::new(pattern.as_bytes())?;
    do_check(
        &target,
        expected_width,
        expected_height,
        expected_rule,
        expected_comments,
        expected_contents,
        if check_tostring { Some(pattern) } else { None },
    );
    Ok(())
}

fn do_new_test_to_be_failed(pattern: &str) {
    let target = Rle::new(pattern.as_bytes());
    assert!(target.is_err());
}

fn do_from_str_test_to_be_passed(
    pattern: &str,
    expected_width: usize,
    expected_height: usize,
    expected_rule: &Rule,
    expected_comments: &[&str],
    expected_contents: &[(usize, usize, usize)],
    check_tostring: bool,
) -> Result<()> {
    let target: Rle = pattern.parse()?;
    do_check(
        &target,
        expected_width,
        expected_height,
        expected_rule,
        expected_comments,
        expected_contents,
        if check_tostring { Some(pattern) } else { None },
    );
    Ok(())
}

#[test]
fn test_new_header_with_conways_rule() -> Result<()> {
    let pattern = concat!("x = 0, y = 0, rule = B3/S23\n", "!\n");
    do_new_test_to_be_passed(pattern, 0, 0, &Rule::conways_life(), &Vec::new(), &Vec::new(), true)
}

#[test]
fn test_new_header_with_highlife_rule() -> Result<()> {
    let pattern = concat!("x = 0, y = 0, rule = B36/S23\n", "!\n");
    do_new_test_to_be_passed(pattern, 0, 0, &RULE_HIGHLIFE, &Vec::new(), &Vec::new(), true)
}

#[test]
fn test_new_header_without_rule() -> Result<()> {
    let pattern = concat!("x = 0, y = 0\n", "!\n");
    do_new_test_to_be_passed(pattern, 0, 0, &Rule::conways_life(), &Vec::new(), &Vec::new(), false)
}

#[test]
fn test_new_comment_header() -> Result<()> {
    let pattern = concat!("#comment\n", "x = 0, y = 0, rule = B3/S23\n", "!\n");
    do_new_test_to_be_passed(pattern, 0, 0, &Rule::conways_life(), &["#comment"], &Vec::new(), true)
}

#[test]
fn test_new_comments_header() -> Result<()> {
    let pattern = concat!("#comment0\n", "#comment1\n", "x = 0, y = 0, rule = B3/S23\n", "!\n");
    do_new_test_to_be_passed(pattern, 0, 0, &Rule::conways_life(), &["#comment0", "#comment1"], &Vec::new(), true)
}

#[test]
fn test_new_comments_with_blank_header() -> Result<()> {
    let pattern = concat!("#comment\n", "\n", "x = 0, y = 0, rule = B3/S23\n", "!\n");
    do_new_test_to_be_passed(pattern, 0, 0, &Rule::conways_life(), &["#comment", ""], &Vec::new(), true)
}

#[test]
fn test_new_header_content() -> Result<()> {
    let pattern = concat!("x = 1, y = 1, rule = B3/S23\n", "o!\n");
    do_new_test_to_be_passed(pattern, 1, 1, &Rule::conways_life(), &Vec::new(), &[(0, 0, 1)], true)
}

#[test]
fn test_new_header_contents() -> Result<()> {
    let pattern = concat!("x = 2, y = 2, rule = B3/S23\n", "o$bo!\n");
    do_new_test_to_be_passed(pattern, 2, 2, &Rule::conways_life(), &Vec::new(), &[(0, 0, 1), (1, 1, 1)], true)
}

#[test]
fn test_new_comments_header_contents() -> Result<()> {
    let pattern = concat!("#comment0\n", "#comment1\n", "x = 2, y = 2, rule = B3/S23\n", "o$bo!\n");
    do_new_test_to_be_passed(pattern, 2, 2, &Rule::conways_life(), &["#comment0", "#comment1"], &[(0, 0, 1), (1, 1, 1)], true)
}

#[test]
fn test_new_empty() {
    let pattern = "";
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_header_invalid_format() {
    let pattern = "_\n";
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_header_unknown_variable() {
    let pattern = "z = 0\n";
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_header_invalid_width() {
    let pattern = "x = _, y = 0\n";
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_header_invalid_height() {
    let pattern = "x = 0, y = _\n";
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_header_without_width() {
    let pattern = "y = 0\n";
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_header_without_height() {
    let pattern = "x = 0\n";
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_header_exceed_width() {
    let pattern = concat!("x = 0, y = 1\n", "o!\n");
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_header_exceed_height() {
    let pattern = concat!("x = 1, y = 0\n", "o!\n");
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_header_larger_width() -> Result<()> {
    let pattern = concat!("x = 2, y = 1, rule = B3/S23\n", "o!\n");
    do_new_test_to_be_passed(pattern, 2, 1, &Rule::conways_life(), &Vec::new(), &[(0, 0, 1)], true)
}

#[test]
fn test_new_header_larger_height() -> Result<()> {
    let pattern = concat!("x = 1, y = 2, rule = B3/S23\n", "o!\n");
    do_new_test_to_be_passed(pattern, 1, 2, &Rule::conways_life(), &Vec::new(), &[(0, 0, 1)], true)
}

#[test]
fn test_new_content_acceptable_tag_without_count() -> Result<()> {
    let pattern = concat!("x = 1, y = 1\n", "_!\n");
    do_new_test_to_be_passed(pattern, 1, 1, &Rule::conways_life(), &Vec::new(), &[(0, 0, 1)], false)
}

#[test]
fn test_new_content_acceptable_tag_with_count() -> Result<()> {
    let pattern = concat!("x = 2, y = 1\n", "2_!\n");
    do_new_test_to_be_passed(pattern, 2, 1, &Rule::conways_life(), &Vec::new(), &[(0, 0, 2)], false)
}

#[test]
fn test_new_content_alone_count() {
    let pattern = concat!("x = 1, y = 1\n", "1\n", "!\n");
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_content_count_with_whitespace() {
    let pattern = concat!("x = 1, y = 1\n", "1 \n", "!\n");
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_content_without_terminator() {
    let pattern = concat!("x = 1, y = 1\n", "o\n");
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_content_terminator_with_count() {
    let pattern = concat!("x = 1, y = 1\n", "2!\n");
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_content_exceeds_width_with_dead_cell() {
    let pattern = concat!("x = 1, y = 1\n", "ob!\n");
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_content_exceeds_width_with_dead_cells() {
    let pattern = concat!("x = 2, y = 2\n", "2o$o2b!\n");
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_content_exceeds_height_with_end_of_line() {
    let pattern = concat!("x = 1, y = 1\n", "o$!\n");
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_content_exceeds_height_with_end_of_lines() {
    let pattern = concat!("x = 1, y = 2\n", "o2$!\n");
    do_new_test_to_be_failed(pattern)
}

#[test]
fn test_new_nonoptimal_dead_cells() -> Result<()> {
    let pattern = concat!("x = 4, y = 1\n", "bbbo!\n");
    do_new_test_to_be_passed(pattern, 4, 1, &Rule::conways_life(), &Vec::new(), &[(0, 3, 1)], false)
}

#[test]
fn test_new_nonoptimal_live_cells() -> Result<()> {
    let pattern = concat!("x = 3, y = 1\n", "ooo!\n");
    do_new_test_to_be_passed(pattern, 3, 1, &Rule::conways_life(), &Vec::new(), &[(0, 0, 3)], false)
}

#[test]
fn test_new_nonoptimal_end_of_lines() -> Result<()> {
    let pattern = concat!("x = 1, y = 4\n", "$$$o!\n");
    do_new_test_to_be_passed(pattern, 1, 4, &Rule::conways_life(), &Vec::new(), &[(3, 0, 1)], false)
}

#[test]
fn test_new_nonoptimal_line_end_dead_cell() -> Result<()> {
    let pattern = concat!("x = 1, y = 2\n", "b$o!\n");
    do_new_test_to_be_passed(pattern, 1, 2, &Rule::conways_life(), &Vec::new(), &[(1, 0, 1)], false)
}

#[test]
fn test_new_nonoptimal_line_end_dead_cells() -> Result<()> {
    let pattern = concat!("x = 2, y = 2\n", "2b$2o!\n");
    do_new_test_to_be_passed(pattern, 2, 2, &Rule::conways_life(), &Vec::new(), &[(1, 0, 2)], false)
}

#[test]
fn test_new_nonoptimal_trailing_dead_cell() -> Result<()> {
    let pattern = concat!("x = 2, y = 2\n", "2o$ob!\n");
    do_new_test_to_be_passed(pattern, 2, 2, &Rule::conways_life(), &Vec::new(), &[(0, 0, 2), (1, 0, 1)], false)
}

#[test]
fn test_new_nonoptimal_trailing_dead_cells() -> Result<()> {
    let pattern = concat!("x = 3, y = 2\n", "3o$o2b!\n");
    do_new_test_to_be_passed(pattern, 3, 2, &Rule::conways_life(), &Vec::new(), &[(0, 0, 3), (1, 0, 1)], false)
}

#[test]
fn test_new_nonoptimal_trailing_line_end() -> Result<()> {
    let pattern = concat!("x = 1, y = 2\n", "o$!\n");
    do_new_test_to_be_passed(pattern, 1, 2, &Rule::conways_life(), &Vec::new(), &[(0, 0, 1)], false)
}

#[test]
fn test_new_nonoptimal_trailing_line_ends() -> Result<()> {
    let pattern = concat!("x = 1, y = 3\n", "o2$!\n");
    do_new_test_to_be_passed(pattern, 1, 3, &Rule::conways_life(), &Vec::new(), &[(0, 0, 1)], false)
}

#[test]
fn test_new_trailing_ignored_content() -> Result<()> {
    let pattern = concat!("x = 1, y = 1\n", "o!_\n");
    do_new_test_to_be_passed(pattern, 1, 1, &Rule::conways_life(), &Vec::new(), &[(0, 0, 1)], false)
}

#[test]
fn test_new_trailing_ignored_line() -> Result<()> {
    let pattern = concat!("x = 1, y = 1\n", "o!\n", "ignored line\n");
    do_new_test_to_be_passed(pattern, 1, 1, &Rule::conways_life(), &Vec::new(), &[(0, 0, 1)], false)
}

#[test]
fn test_build() -> Result<()> {
    let pattern = [Position(0, 0), Position(1, 0), Position(2, 0), Position(1, 1)];
    let target = pattern.iter().collect::<RleBuilder>().build()?;
    do_check(&target, 3, 2, &Rule::conways_life(), &Vec::new(), &[(0, 0, 3), (1, 1, 1)], None);
    Ok(())
}

#[test]
fn test_build_singleline_name() -> Result<()> {
    let pattern = [Position(0, 0)];
    let target = pattern.iter().collect::<RleBuilder>().name("name").build()?;
    do_check(&target, 1, 1, &Rule::conways_life(), &["#N name"], &[(0, 0, 1)], None);
    Ok(())
}

#[test]
fn test_build_blank_name() -> Result<()> {
    let pattern = [Position(0, 0)];
    let target = pattern.iter().collect::<RleBuilder>().name("").build()?;
    do_check(&target, 1, 1, &Rule::conways_life(), &["#N"], &[(0, 0, 1)], None);
    Ok(())
}

#[test]
fn test_build_multiline_name() {
    let pattern = [Position(0, 0)];
    let target = pattern.iter().collect::<RleBuilder>().name("name\nname").build();
    assert!(target.is_err());
}

#[test]
fn test_build_created() -> Result<()> {
    let pattern = [Position(0, 0)];
    let target = pattern.iter().collect::<RleBuilder>().created("created").build()?;
    do_check(&target, 1, 1, &Rule::conways_life(), &["#O created"], &[(0, 0, 1)], None);
    Ok(())
}

#[test]
fn test_build_blank_created() -> Result<()> {
    let pattern = [Position(0, 0)];
    let target = pattern.iter().collect::<RleBuilder>().created("").build()?;
    do_check(&target, 1, 1, &Rule::conways_life(), &["#O"], &[(0, 0, 1)], None);
    Ok(())
}

#[test]
fn test_build_createds() -> Result<()> {
    let pattern = [Position(0, 0)];
    let target = pattern.iter().collect::<RleBuilder>().created("created0\ncreated1").build()?;
    do_check(&target, 1, 1, &Rule::conways_life(), &["#O created0", "#O created1"], &[(0, 0, 1)], None);
    Ok(())
}

#[test]
fn test_build_comment() -> Result<()> {
    let pattern = [Position(0, 0)];
    let target = pattern.iter().collect::<RleBuilder>().comment("comment").build()?;
    do_check(&target, 1, 1, &Rule::conways_life(), &["#C comment"], &[(0, 0, 1)], None);
    Ok(())
}

#[test]
fn test_build_blank_comment() -> Result<()> {
    let pattern = [Position(0, 0)];
    let target = pattern.iter().collect::<RleBuilder>().comment("").build()?;
    do_check(&target, 1, 1, &Rule::conways_life(), &["#C"], &[(0, 0, 1)], None);
    Ok(())
}

#[test]
fn test_build_comments() -> Result<()> {
    let pattern = [Position(0, 0)];
    let target = pattern.iter().collect::<RleBuilder>().comment("comment0\ncomment1").build()?;
    do_check(&target, 1, 1, &Rule::conways_life(), &["#C comment0", "#C comment1"], &[(0, 0, 1)], None);
    Ok(())
}

#[test]
fn test_build_rule() -> Result<()> {
    let pattern = [Position(0, 0)];
    let target = pattern.iter().collect::<RleBuilder>().rule(RULE_HIGHLIFE.clone()).build()?;
    do_check(&target, 1, 1, &RULE_HIGHLIFE, &Vec::new(), &[(0, 0, 1)], None);
    Ok(())
}

#[test]
fn test_build_name_created_comment() -> Result<()> {
    let pattern = [Position(0, 0)];
    let target = pattern
        .iter()
        .collect::<RleBuilder>()
        .name("name")
        .created("created")
        .comment("comment")
        .build()?;
    do_check(
        &target,
        1,
        1,
        &Rule::conways_life(),
        &["#N name", "#O created", "#C comment"],
        &[(0, 0, 1)],
        None,
    );
    Ok(())
}

#[test]
fn test_display_max_width() -> Result<()> {
    let pattern = ["x = 72, y = 1, rule = B3/S23", &"bo".repeat(35), "bo!"]
        .iter()
        .map(|&s| s.to_string() + "\n")
        .collect::<String>();
    let target = Rle::new(pattern.as_bytes())?;
    assert_eq!(target.to_string(), pattern);
    Ok(())
}

#[test]
fn test_from_str() -> Result<()> {
    let pattern = concat!("#comment0\n", "#comment1\n", "x = 2, y = 2, rule = B3/S23\n", "o$bo!\n");
    do_from_str_test_to_be_passed(pattern, 2, 2, &Rule::conways_life(), &["#comment0", "#comment1"], &[(0, 0, 1), (1, 1, 1)], true)
}
