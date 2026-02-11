use std::fmt;
use std::path::Path;

use git2::Commit;
use pest::Parser;

use crate::{ConventionalCommitError, get_recent_commit, prune_message};

use super::{CommitMessageParser, CommitType, Rule};

#[derive(Debug, Default, serde::Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConventionalCommit {
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub subject: String,
    pub footer: Option<String>,
    pub body: Option<String>,
    pub prefix: Option<String>,
    pub breaking_change: bool,
}

impl ConventionalCommit {
    pub fn new(commit_message: impl AsRef<str>) -> Result<Self, ConventionalCommitError> {
        if commit_message.as_ref().trim().is_empty() {
            return Err(ConventionalCommitError::EmptyCommitMessage);
        }
        let pruned_message = {
            let new_message = prune_message(commit_message.as_ref());
            match new_message.is_empty() {
                true => {
                    tracing::trace!("Pruned message empty.  Using original commit message: {:?}", commit_message.as_ref());
                    commit_message.as_ref().to_owned()
                }
                false => new_message,
            }
        };
        let parsed = CommitMessageParser::parse(Rule::commit_message, &pruned_message).map_err(|err| ConventionalCommitError::InvalidCommitMessage(err.to_string()))?;
        let mut commit = ConventionalCommit::default();

        for inner in parsed.into_iter() {
            match inner.as_rule() {
                Rule::breaking_change_shorthand => commit.breaking_change = true,
                Rule::breaking_change_phrase => commit.breaking_change = true,
                Rule::commit_type => commit.commit_type = ConventionalCommit::parse_commit_type(inner)?,
                Rule::scope => commit.scope = ConventionalCommit::parse_scope(inner)?,
                Rule::subject => commit.subject = ConventionalCommit::parse_subject(inner)?,
                Rule::section => {
                    let blocks: Vec<&str> = inner.as_str().split("\n\n").collect();
                    let footer = blocks.last().unwrap_or(&"").to_string();
                    let body = blocks[..blocks.len().saturating_sub(1)].join("\n\n");
                    if !body.is_empty() {
                        commit.body = Some(body);
                    }
                    if !footer.is_empty() {
                        commit.footer = Some(footer);
                    }
                }
                // Rule::body => commit.body = ConventionalCommit::parse_body(inner)?,
                _ => tracing::debug!("Ignoring rule: {:?}", inner.as_rule()),
            }
        }

        ConventionalCommit::finalize_commit(&mut commit);

        Ok(commit)
    }

    /// Creates a string representation of the commit
    pub fn message(&self) -> String {
        prune_message(self.to_string())
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

    fn finalize_commit(commit: &mut ConventionalCommit) {
        if commit.commit_type == CommitType::Unknown && commit.scope.is_none() && !commit.subject.is_empty() {
            commit.commit_type = CommitType::NonCompliant;
        }
        if commit.breaking_change {
            return;
        }
        if let Some(footer) = &commit.footer {
            if let Some(rest) = footer.strip_prefix("BREAKING CHANGE:") {
                commit.footer = Some(rest.trim_start().to_string());
                commit.breaking_change = true;
                return;
            }
            if footer.starts_with("BREAKING CHANGE") {
                commit.breaking_change = true;
                return;
            }
        }
        if let Some(body) = &commit.body {
            commit.breaking_change = body.split("\n\n").any(|p| p.starts_with("BREAKING CHANGE"));
        }
    }

    pub fn is_breaking(&self) -> bool {
        self.breaking_change
    }
}

impl<'a> TryFrom<Commit<'a>> for ConventionalCommit {
    type Error = ConventionalCommitError;

    fn try_from(commit: Commit<'a>) -> Result<Self, Self::Error> {
        let message = commit.message().ok_or(ConventionalCommitError::EmptyCommitMessage)?;
        ConventionalCommit::new(message)
    }
}

impl fmt::Display for ConventionalCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scope = match &self.scope {
            Some(scope) => format!("({scope})"),
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
            string = match self.breaking_change {
                true => format!("{string}\n\nBREAKING CHANGE: {footer}"),
                false => format!("{string}\n\n{footer}"),
            };
        } else if self.breaking_change {
            string = format!("{string}\n\nBREAKING CHANGE");
        }
        write!(f, "{string}")
    }
}

impl TryFrom<String> for ConventionalCommit {
    type Error = ConventionalCommitError;

    fn try_from(commit_message: String) -> Result<Self, Self::Error> {
        ConventionalCommit::new(commit_message)
    }
}

impl TryFrom<&str> for ConventionalCommit {
    type Error = ConventionalCommitError;

    fn try_from(commit_message: &str) -> Result<Self, Self::Error> {
        ConventionalCommit::new(commit_message)
    }
}

impl TryFrom<&Path> for ConventionalCommit {
    type Error = ConventionalCommitError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        get_recent_commit(path).map_err(|err| ConventionalCommitError::InvalidRepositoryError(err.to_string()))
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
    #[case::ci_scoped("ci(core): add commit message parser", "ci", "core", "add commit message parser", "", "", false)]
    #[case::ci_unscoped("ci: add commit message parser", "ci", "", "add commit message parser", "", "", false)]
    #[case::feat_unscoped("feat: add commit message parser", "feat", "", "add commit message parser", "", "", false)]
    #[case::build_unscoped("build: add commit message parser", "build", "", "add commit message parser", "", "", false)]
    #[case::natural_commit("add commit message parser", CommitType::NonCompliant, "", "add commit message parser", "", "", false)]
    #[case::natural_multi_line_commit(
        "add commit message parser\n\nThis is a multi-line commit message",
        CommitType::NonCompliant,
        "",
        "add commit message parser",
        "",
        "This is a multi-line commit message",
        false
    )]
    #[case::merged_pr_commit("Ignore changes from Black -> Ruff (#4032)", CommitType::NonCompliant, "", "Ignore changes from Black -> Ruff (#4032)", "", "", false)]
    #[case::squashed_feature_branch_commit(
        squashed_feature_branch_commit(),
        "chore",
        "package",
        "upgrade ruff (#4031)",
        "* chore(package): upgrade ruff\n\n- chore(deps): removes black and isort\n- chore(style): run ruff\n- chore(lint): fix linting",
        "* chore(ci): update ci to use ruff format",
        false
    )]
    #[case::breaking_change_footer(
        "feat: add commit message parser\n\nBREAKING CHANGE: this is a breaking change",
        "feat",
        "",
        "add commit message parser",
        "",
        "this is a breaking change",
        true
    )]
    #[case::breaking_change_footer_with_body(
        "feat: add API endpoint\n\nSome implementation details\n\nBREAKING CHANGE: removed old endpoint",
        "feat",
        "",
        "add API endpoint",
        "Some implementation details",
        "removed old endpoint",
        true
    )]
    #[case::breaking_change_footer_with_multipart_body(
        "feat: redesign\n\nFirst paragraph\n\nSecond paragraph\n\nBREAKING CHANGE: old API removed",
        "feat",
        "",
        "redesign",
        "First paragraph\n\nSecond paragraph",
        "old API removed",
        true
    )]
    #[case::breaking_change_shorthand_with_body(
        "feat!: add API endpoint\n\nSome body text\n\nSome footer",
        "feat",
        "",
        "add API endpoint",
        "Some body text",
        "Some footer",
        true
    )]
    fn test_commit_message_parser(
        #[case] commit_message: impl AsRef<str>,
        #[case] commit_type: impl Into<CommitType>,
        #[case] scope: impl AsRef<str>,
        #[case] subject: impl AsRef<str>,
        #[case] body: impl AsRef<str>,
        #[case] footer: impl AsRef<str>,
        #[case] breaking_change: bool,
    ) {
        let scope = match scope.as_ref().is_empty() {
            true => None,
            false => Some(scope.as_ref().to_string()),
        };
        let commit = ConventionalCommit::new(commit_message.as_ref()).unwrap();
        assert_eq!(
            commit.commit_type,
            commit_type.into(),
            "Commit Type failed. Commit input was: {:#?}.  Got: {commit:#?}.",
            commit_message.as_ref()
        );
        assert_eq!(commit.scope, scope, "Scope failed. Commit input was: {:#?}.  Got: {commit:#?}.", commit_message.as_ref());
        assert_eq!(commit.subject, subject.as_ref(), "Subject failed.  Commit input was: {:#?}.  Got: {commit:#?}", commit_message.as_ref());
        assert_eq!(
            commit.body.clone().unwrap_or_default(),
            body.as_ref(),
            "Body failed. Commit input was: {:#?}.  Got: {commit:#?}",
            commit_message.as_ref()
        );
        assert_eq!(
            commit.footer.clone().unwrap_or_default(),
            footer.as_ref(),
            "Footer failed. Commit input was: {:#?}.  Got: {commit:#?}",
            commit_message.as_ref()
        );
        assert_eq!(
            commit.breaking_change, breaking_change,
            "Breaking change failed. Commit input was: {:#?}.  Got: {commit:#?}",
            commit_message.as_ref()
        );
    }

    #[rstest]
    #[case::empty("", ConventionalCommitError::EmptyCommitMessage)]
    fn test_commit_parser_unhappy_paths(#[case] commit_message: impl AsRef<str>, #[case] expected: ConventionalCommitError) {
        let result = ConventionalCommit::new(commit_message);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), expected);
    }
}
