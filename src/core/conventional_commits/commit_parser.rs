use pest_derive::Parser as DeriveParser;

#[derive(DeriveParser)]
#[grammar = "src/core/conventional_commits/commit_message.pest"]
pub struct CommitMessageParser;

#[cfg(test)]
mod tests {
    use super::CommitMessageParser;
    use crate::{CommitType, Rule};

    use pest::Parser;
    use rstest::rstest;

    #[rstest]
    #[case::fix("fix: a fix", "fix")]
    #[case::non_compliant("not-compliant: a fix", "not-compliant")]
    fn test_parsing_commit_message(#[case] commit_message: impl AsRef<str>, #[case] expected_type: impl AsRef<str>) {
        let commit_message = commit_message.as_ref();
        let expected_type = CommitType::from(expected_type.as_ref());

        match CommitMessageParser::parse(Rule::commit_message, commit_message) {
            Ok(parsed) => {
                let mut found_match = false;
                for pair in parsed {
                    if matches!(pair.as_rule(), Rule::commit_type) && pair.as_str() == expected_type.as_str() {
                        found_match = true;
                        break;
                    } else {
                        println!("Found: {}", pair.as_str());
                    }
                }
                assert!(found_match, "Parsed commit type '{}' did not match the expected type '{}'", commit_message, expected_type.as_str());
            }
            Err(err) => {
                panic!("Failed to parse commit message: '{}'. Error: {}", commit_message, err);
            }
        }
    }

    #[rstest]
    #[case::fix("fix: a fix", "")]
    #[case::non_compliant("not-compliant: a fix", "")]
    #[case::fix_with_scope("fix(component): a fix", "component")]
    #[case::breaking_change_fix_with_scope_prefix("!fix(component): a fix", "component")]
    #[case::breaking_change_fix_with_scope_suffix("fix(component)!: a fix", "component")]
    #[case::invalid_scope("fix(fix): fix", "fix")]
    fn test_parsing_scope(#[case] commit_message: impl AsRef<str>, #[case] expected_type: impl AsRef<str>) {
        let commit_message = commit_message.as_ref();

        match CommitMessageParser::parse(Rule::commit_message, commit_message) {
            Ok(parsed) => {
                let found_match = match expected_type.as_ref() == "" {
                    true => !parsed
                        .flatten()
                        .inspect(|pair| {
                            println!("pair[{:?}]: {}", pair.as_rule(), pair.as_str());
                        })
                        .any(|pair| matches!(pair.as_rule(), Rule::scope)),
                    false => parsed
                        .flatten()
                        .inspect(|pair| {
                            println!("pair[{:?}]: {}", pair.as_rule(), pair.as_str());
                        })
                        .any(|pair| matches!(pair.as_rule(), Rule::scope)),
                };
                assert!(found_match, "Parsed commit type '{}' did not match the expected type '{}'", commit_message, expected_type.as_ref());
            }
            Err(err) => {
                panic!("Failed to parse commit message: '{}'. Error: {}", commit_message, err);
            }
        }
    }

    #[rstest]
    #[case::fix("fix: a fix", "a fix")]
    #[case::non_compliant("not-compliant: a fix", "a fix")]
    #[case::fix_with_scope("fix(component): a fix", "a fix")]
    #[case::breaking_change_fix_with_scope_prefix("!fix(component): a fix", "a fix")]
    #[case::breaking_change_fix_with_scope_suffix("fix(component)!: a fix", "a fix")]
    #[case::invalid_scope("fix(fix): a fix", "a fix")]
    fn test_parsing_subject(#[case] commit_message: impl AsRef<str>, #[case] expected_type: impl AsRef<str>) {
        let commit_message = commit_message.as_ref();
        match CommitMessageParser::parse(Rule::commit_message, commit_message) {
            Ok(parsed) => {
                let found_match = match expected_type.as_ref() == "" {
                    true => !parsed
                        .flatten()
                        .inspect(|pair| {
                            println!("pair[{:?}]: {}", pair.as_rule(), pair.as_str());
                        })
                        .any(|pair| matches!(pair.as_rule(), Rule::subject)),
                    false => parsed
                        .flatten()
                        .inspect(|pair| {
                            println!("pair[{:?}]: {}", pair.as_rule(), pair.as_str());
                        })
                        .any(|pair| matches!(pair.as_rule(), Rule::subject)),
                };
                assert!(found_match, "Parsed commit type '{}' did not match the expected type '{}'", commit_message, expected_type.as_ref());
            }
            Err(err) => {
                panic!("Failed to parse commit message: '{}'. Error: {}", commit_message, err);
            }
        }
    }

    #[rstest]
    #[case::fix_no_body("fix: a fix", "")]
    #[case::fix_with_body("fix: a fix\n\nThis a fix body", "This a fix body")]
    #[case::fix_with_body_and_footer("fix: a fix\n\nThis a fix body\n\nThis is a footer", "This a fix body\n\nThis is a footer")]
    #[case::fix_with_body_and_footer("fix: a fix\n\nThis a fix body\n\nWith another entry\n\nThis is a footer", "This a fix body\n\nWith another entry\n\nThis is a footer")]
    #[case::breaking_change("feat: add commit message parser\n\nBREAKING CHANGE: this is a breaking change", "this is a breaking change")]
    #[case::natural_multi_line_commit("add commit message parser\n\nThis is a multi-line commit message", "This is a multi-line commit message")]
    #[case::squash_and_merge("chore(package): upgrade ruff (#4031)\n\n\n* chore(package): upgrade ruff\n\n- chore(deps): removes black and isort\n- chore(style): run ruff\n- chore(lint): fix linting\n\n* chore(ci): update ci to use ruff format\n", "* chore(package): upgrade ruff\n\n- chore(deps): removes black and isort\n- chore(style): run ruff\n- chore(lint): fix linting\n\n* chore(ci): update ci to use ruff format")]
    fn test_parsing_section(#[case] commit_message: impl AsRef<str>, #[case] expected: impl AsRef<str>) {
        let commit_message = commit_message.as_ref();
        match CommitMessageParser::parse(Rule::commit_message, commit_message) {
            Ok(parsed) => {
                let found_match = match expected.as_ref() == "" {
                    true => !parsed
                        .flatten()
                        .inspect(|pair| {
                            println!("pair[{:?}]: {}", pair.as_rule(), pair.as_str());
                        })
                        .any(|pair| matches!(pair.as_rule(), Rule::section)),
                    false => parsed
                        .flatten()
                        .inspect(|pair| {
                            println!("pair[{:?}]: {}", pair.as_rule(), pair.as_str());
                        })
                        .any(|pair| matches!(pair.as_rule(), Rule::section) && pair.as_str() == expected.as_ref()),
                };
                assert!(found_match, "Parsed commit type did not match the expected type:\n'{}\n'{}'", commit_message, expected.as_ref());
            }
            Err(err) => {
                panic!("Failed to parse commit message: '{}'. Error: {}", commit_message, err);
            }
        }
    }

    #[rstest]
    #[case::no_breaking_change("fix: a fix", false)]
    #[case::breaking_change_shorthand("fix!: a fix", true)]
    #[case::excessive_breaking_change_shorthand("fix!!!!: a fix", true)]
    #[case::breaking_change_shorthand_prefix("!fix: a fix", true)]
    #[case::scoped_breaking_change_shorthand_prefix("!fix(component): a fix", true)]
    #[case::scoped_breaking_change_shorthand_suffix("fix(component)!: a fix", true)]
    #[case::scoped_breaking_change_shorthand_prefix_and_suffix("!fix(component)!: a fix", true)]
    // #[case::breaking_change_footer("fix: a fix\n\nBREAKING CHANGE: This introduces a breaking change.", true)]
    fn test_parsing_breaking_change_shorthand(#[case] commit_message: impl AsRef<str>, #[case] break_change_found: bool) {
        let commit_message = commit_message.as_ref();

        match CommitMessageParser::parse(Rule::commit_message, commit_message) {
            Ok(parsed) => {
                // Check for breaking change shorthand '!'
                let found_breaking_change = parsed
                    .flatten()
                    .inspect(|pair| {
                        println!("pair: {}", pair.as_str());
                    })
                    .any(|pair| matches!(pair.as_rule(), Rule::breaking_change_shorthand) || matches!(pair.as_rule(), Rule::breaking_change_phrase));

                assert_eq!(
                    found_breaking_change, break_change_found,
                    "Parsed commit message '{}' did not match the expected breaking change flag (expected: {}, found: {})",
                    commit_message, break_change_found, found_breaking_change
                );
            }
            Err(err) => {
                panic!("Failed to parse commit message: '{}'. Error: {}", commit_message, err);
            }
        }
    }
}
