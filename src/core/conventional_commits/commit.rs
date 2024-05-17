use std::path::Path;

use pest::Parser;

use crate::{get_recent_commit, ConventionalCommitError};

use super::{CommitMessageParser, CommitType, Rule};

#[derive(Debug, Default, serde::Deserialize)]
pub struct Commit {
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub subject: String,
    pub footer: Option<String>,
    pub body: Option<String>,
}

impl Commit {
    pub fn new(commit_message: impl AsRef<str>) -> Result<Self, ConventionalCommitError> {
        let parsed = CommitMessageParser::parse(Rule::commit_message, commit_message.as_ref()).map_err(|err| ConventionalCommitError::InvalidCommitMessage(err.to_string()))?;
        let mut commit = Commit::default();

        // Assuming `commit_message` is the root rule that captures a full commit message
        for inner in parsed.into_iter() {
            match inner.as_rule() {
                Rule::commit_type => {
                    let commit_type = CommitType::from(inner.as_str());
                    if commit_type == CommitType::Unknown {
                        return Err(ConventionalCommitError::InvalidCommitType(inner.as_str().to_string()));
                    }
                    commit.commit_type = commit_type;
                }
                Rule::scope => {
                    let scope_as_commit_type = CommitType::from(inner.as_str());
                    match scope_as_commit_type {
                        CommitType::Custom(value) => {
                            commit.scope = Some(value);
                        }
                        CommitType::Unknown => {
                            commit.scope = Some(inner.as_str().to_string());
                        }
                        _ => {
                            return Err(ConventionalCommitError::ScopeIsCommitType(inner.as_str().to_string()));
                        }
                    }
                }
                Rule::subject => {
                    commit.subject = inner.as_str().to_string();
                }
                Rule::footer => {
                    commit.footer = Some(inner.as_str().to_string());
                }
                Rule::body => {
                    if !inner.as_str().is_empty() {
                        commit.body = Some(inner.as_str().to_string());
                    }
                }
                _ => {
                    println!("Ignoring rule: {:?}", inner.as_rule());
                }
            }
        }

        if commit.commit_type == CommitType::Unknown && commit.scope.is_none() && !commit.subject.is_empty() {
            commit.commit_type = CommitType::NonCompliant;
        }

        Ok(commit)
    }

    pub fn is_breaking(&self, breaking_message: impl AsRef<str>) -> bool {
        let breaking_message = breaking_message.as_ref();
        self.footer
            .as_ref()
            .map_or(false, |footer| footer.starts_with(breaking_message))
            || self.body.as_ref().map_or(false, |footer| footer.starts_with(breaking_message))
            || self.subject.contains(breaking_message)
    }
}

impl From<&Commit> for Commit {
    fn from(commit: &Commit) -> Self {
        Commit {
            commit_type: commit.commit_type.clone(),
            scope: commit.scope.clone(),
            subject: commit.subject.clone(),
            footer: commit.footer.clone(),
            body: commit.body.clone(),
        }
    }
}

impl TryFrom<&Path> for Commit {
    type Error = ConventionalCommitError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        get_recent_commit(path).map_err(ConventionalCommitError::RepositoryError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn breaking_message() -> &'static str {
        "BREAKING CHANGE"
    }

    #[fixture]
    fn squashed_feature_branch_commit_message() -> &'static str {
        "chore(package): upgrade ruff (#4031)"
    }

    #[fixture]
    fn squashed_feature_branch_commit_body() -> String {
        textwrap::dedent(
            r#"
            * chore(package): upgrade ruff

            - chore(deps): removes black and isort
            - chore(style): run ruff
            - chore(lint): fix linting

            * chore(ci): update ci to use ruff format
        "#,
        )
    }

    #[fixture]
    fn squashed_feature_branch_commit() -> String {
        format!("{}\n\n{}", squashed_feature_branch_commit_message(), squashed_feature_branch_commit_body())
    }

    #[rstest]
    #[case::ci_scoped("ci(core): add commit message parser", "ci", "core", "add commit message parser", "", "")]
    #[case::ci_unscoped("ci: add commit message parser", "ci", "", "add commit message parser", "", "")]
    #[case::feat_unscoped("feat: add commit message parser", "feat", "", "add commit message parser", "", "")]
    #[case::feat_unscoped("build: add commit message parser", "build", "", "add commit message parser", "", "")]
    #[case::natural_commit("add commit message parser", CommitType::NonCompliant, "", "add commit message parser", "", "")]
    #[case::natural_multi_line_commit(
        "add commit message parser\n\nThis is a multi-line commit message",
        CommitType::NonCompliant,
        "",
        "add commit message parser",
        "\nThis is a multi-line commit message",
        ""
    )]
    #[case::merged_pr_commit("Ignore changes from Black -> Ruff (#4032)", CommitType::NonCompliant, "", "Ignore changes from Black -> Ruff (#4032)", "", "")]
    #[case::squashed_feature_branch_commit(squashed_feature_branch_commit(), "chore", "package", "upgrade ruff (#4031)", format!("\n{}", squashed_feature_branch_commit_body()), "")]
    #[case::footer_included(
        "feat: add commit message parser\n\nBREAKING CHANGE: this is a breaking change",
        "feat",
        "",
        "add commit message parser",
        "",
        "this is a breaking change"
    )]
    fn test_commit_message_parser(
        #[case] commit_message: impl AsRef<str>,
        #[case] commit_type: impl Into<CommitType>,
        #[case] scope: impl AsRef<str>,
        #[case] subject: impl AsRef<str>,
        #[case] body: impl AsRef<str>,
        #[case] footer: impl AsRef<str>,
    ) {
        let scope = match scope.as_ref().is_empty() {
            true => None,
            false => Some(scope.as_ref().to_string()),
        };
        let commit = Commit::new(commit_message).unwrap();
        assert_eq!(commit.commit_type, commit_type.into());
        assert_eq!(commit.scope, scope);
        assert_eq!(commit.subject, subject.as_ref());
        assert_eq!(
            commit.body,
            match body.as_ref().is_empty() {
                true => None,
                false => Some(body.as_ref().to_string()),
            }
        );
        assert_eq!(
            commit.footer,
            match footer.as_ref().is_empty() {
                true => None,
                false => Some(footer.as_ref().to_string()),
            }
        );
    }

    #[rstest]
    #[case::empty("", ConventionalCommitError::InvalidCommitMessage(" --> 1:1\n  |\n1 | \n  | ^---\n  |\n  = expected commit_type or subject".to_string()))]
    fn test_commit_parser_unhappy_paths(#[case] commit_message: impl AsRef<str>, #[case] expected: ConventionalCommitError) {
        let result = Commit::new(commit_message);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), expected);
    }
}
