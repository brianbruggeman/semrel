use std::fmt;
use std::path::Path;

use pest::Parser;

use crate::{get_recent_commit, prune_message, ConventionalCommitError};

use super::{CommitMessageParser, CommitType, Rule};

#[derive(Debug, Default, serde::Deserialize, Clone)]
pub struct ConventionalCommit {
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub subject: String,
    pub footer: Option<String>,
    pub body: Option<String>,
    pub prefix: Option<String>,
}

impl fmt::Display for ConventionalCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scope = match &self.scope {
            Some(scope) => format!("({})", scope),
            None => "".to_string(),
        };
        let title = match &self.commit_type {
            CommitType::NonCompliant => "".to_string(),
            CommitType::Unknown => "".to_string(),
            CommitType::Custom(value) => value.clone(),
            _ => self.commit_type.to_string(),
        };
        let mut string = match title.is_empty() {
            true => self.subject.to_string(),
            false => format!("{}{}: {}", title, scope, self.subject),
        };

        if let Some(body) = &self.body {
            string = format!("{string}\n\n{body}");
        }
        if let Some(footer) = &self.footer {
            string = format!("{string}\n\n{footer}");
        }
        write!(f, "{string}")
    }
}
impl ConventionalCommit {
    pub fn new(commit_message: impl AsRef<str>) -> Result<Self, ConventionalCommitError> {
        let pruned_message = prune_message(commit_message.as_ref());
        let parsed = CommitMessageParser::parse(Rule::commit_message, &pruned_message).map_err(|err| ConventionalCommitError::InvalidCommitMessage(err.to_string()))?;
        let mut commit = ConventionalCommit::default();

        for inner in parsed.into_iter() {
            match inner.as_rule() {
                Rule::commit_type => commit.commit_type = ConventionalCommit::parse_commit_type(inner)?,
                Rule::scope => commit.scope = ConventionalCommit::parse_scope(inner)?,
                Rule::subject => commit.subject = ConventionalCommit::parse_subject(inner)?,
                Rule::footer => commit.footer = ConventionalCommit::parse_footer(inner)?,
                Rule::body => commit.body = ConventionalCommit::parse_body(inner)?,
                _ => tracing::debug!("Ignoring rule: {:?}", inner.as_rule()),
            }
        }

        ConventionalCommit::finalize_commit(&mut commit);

        Ok(commit)
    }

    fn parse_commit_type(pair: pest::iterators::Pair<Rule>) -> Result<CommitType, ConventionalCommitError> {
        match pair.as_rule() == Rule::commit_type {
            false => Err(ConventionalCommitError::InvalidParse("commit_type".to_string())),
            true => {
                let commit_type = CommitType::from(pair.as_str());
                if commit_type == CommitType::Unknown {
                    return Err(ConventionalCommitError::InvalidCommitType(pair.as_str().to_string()));
                }
                Ok(commit_type)
            }
        }
    }

    fn parse_scope(pair: pest::iterators::Pair<Rule>) -> Result<Option<String>, ConventionalCommitError> {
        let scope_as_commit_type = CommitType::from(pair.as_str());
        match scope_as_commit_type {
            CommitType::Custom(value) => Ok(Some(value)),
            CommitType::Unknown => Ok(Some(pair.as_str().to_string())),
            _ => Err(ConventionalCommitError::ScopeIsCommitType(pair.as_str().to_string())),
        }
    }

    fn parse_subject(pair: pest::iterators::Pair<Rule>) -> Result<String, ConventionalCommitError> {
        Ok(pair.as_str().to_string())
    }

    fn parse_footer(pair: pest::iterators::Pair<Rule>) -> Result<Option<String>, ConventionalCommitError> {
        Ok(Some(pair.as_str().to_string()))
    }

    fn parse_body(pair: pest::iterators::Pair<Rule>) -> Result<Option<String>, ConventionalCommitError> {
        match pair.as_str().is_empty() {
            true => Ok(None),
            false => Ok(Some(pair.as_str().to_string())),
        }
    }

    fn finalize_commit(commit: &mut ConventionalCommit) {
        if commit.commit_type == CommitType::Unknown && commit.scope.is_none() && !commit.subject.is_empty() {
            commit.commit_type = CommitType::NonCompliant;
            tracing::debug!("Setting commit type to {:?} because it was not recognized", commit.commit_type);
        }
    }

    pub fn is_breaking(&self, breaking_message: impl AsRef<str>) -> bool {
        let breaking_message = breaking_message.as_ref();
        let result = self
            .footer
            .as_ref()
            .map_or(false, |footer| footer.starts_with(breaking_message))
            || self.body.as_ref().map_or(false, |body| body.starts_with(breaking_message))
            || self.subject.contains(breaking_message);
        if result {
            tracing::debug!("This commit is a breaking change");
        }
        result
    }
}

impl From<&ConventionalCommit> for ConventionalCommit {
    fn from(commit: &ConventionalCommit) -> Self {
        ConventionalCommit {
            commit_type: commit.commit_type.clone(),
            scope: commit.scope.clone(),
            subject: commit.subject.clone(),
            footer: commit.footer.clone(),
            body: commit.body.clone(),
            ..Default::default()
        }
    }
}

impl From<String> for ConventionalCommit {
    fn from(commit_message: String) -> Self {
        ConventionalCommit::new(commit_message).unwrap_or_default()
    }
}

impl From<&str> for ConventionalCommit {
    fn from(commit_message: &str) -> Self {
        ConventionalCommit::new(commit_message).unwrap_or_default()
    }
}

impl TryFrom<&Path> for ConventionalCommit {
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
    #[case::squashed_feature_branch_commit(squashed_feature_branch_commit(), "chore", "package", "upgrade ruff (#4031)", format!("\n\n{}", squashed_feature_branch_commit_body().trim()), "")]
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
        let commit = ConventionalCommit::new(commit_message.as_ref()).unwrap();
        assert_eq!(commit.commit_type, commit_type.into());
        assert_eq!(commit.scope, scope);
        assert_eq!(commit.subject, subject.as_ref());
        assert_eq!(
            commit.body,
            match body.as_ref().is_empty() {
                true => None,
                false => Some(body.as_ref().to_string()),
            },
            "Commit input was: {:?}.  Got: {commit:?}",
            commit_message.as_ref()
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
        let result = ConventionalCommit::new(commit_message);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), expected);
    }

    #[rstest]
    #[case::empty("", "")]
    #[case::ci("ci(core): add commit message parser", "ci(core): add commit message parser")]
    #[case::feat("feat: add commit message parser", "feat: add commit message parser")]
    #[case::non_compliant("add commit message parser", "add commit message parser")]
    fn test_display(#[case] commit: impl Into<ConventionalCommit>, #[case] expected: String) {
        assert_eq!(commit.into().to_string(), expected);
    }
}
