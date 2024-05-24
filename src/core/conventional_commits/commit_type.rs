use std::fmt;
use std::str::FromStr;

use crate::ConventionalCommitError;

/// A commit message that follows the conventional commit standard
#[derive(Debug, Default, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub enum CommitType {
    /// A commit that is used to add a new feature
    /// example: `feat: add support for dark mode`
    Feat,
    /// A commit that is used to fix a bug
    /// example: `fix: resolve issue with login form`
    Fix,
    /// A commit that is used to improve performance
    /// example: `perf: optimize database queries`
    Perf,
    /// A commit that is used to refactor code
    /// example: `refactor: extract function to helper module`
    Refactor,
    /// A commit that is used to revert a previous commit
    /// example: `revert: revert changes from commit 123456`
    Revert,
    /// A commit that is used to update code style
    /// example: `style: format python code with ruff`
    Style,
    /// A commit that is used to add or update tests
    /// example: `test: add unit tests for feature X`
    Test,
    /// A commit that is used to build the project
    /// example: `build: update dependencies`
    Build,
    /// A commit that is used to perform a task that is not user-facing
    /// example: `chore: upgrade dependencies`
    Chore,
    /// A commit that is used to perform a task related to the CI pipeline
    /// example: `ci: run tests on Windows`
    Ci,
    /// A commit that is used to perform a task related to the CD pipeline
    /// example: `cd: deploy to production`
    Cd,
    /// A commit that is used to update documentation
    /// example: `docs: add usage instructions to README.md`
    Docs,
    /// A custom commit type that is not part of the standard list
    /// example: `ENG-2345: updated dependencies`
    Custom(String),
    /// A commit that does not follow the conventional commit standard
    /// example: `updated dependencies`
    NonCompliant,
    /// An unknown commit type
    /// This is the default commit type, and probably should only be
    ///  used for testing purposes
    #[default]
    Unknown,
}

impl CommitType {
    pub fn as_release_note(&self) -> &str {
        match self {
            CommitType::Feat => "Features",
            CommitType::Fix => "Fixes",
            CommitType::Perf => "Performance",
            CommitType::Refactor => "Refactor",
            CommitType::Revert => "Revert",
            CommitType::Style => "Style",
            CommitType::Test => "Test",
            CommitType::Build => "Build",
            CommitType::Chore => "Chore",
            CommitType::Ci => "Continuous Integration",
            CommitType::Cd => "Deployment",
            CommitType::Docs => "Documentation",
            CommitType::Custom(value) => value.as_str(),
            CommitType::NonCompliant => "Non Compliant",
            CommitType::Unknown => "Unknown",
        }
    }

    /// Returns the commit type as a string
    pub fn as_str(&self) -> &str {
        match self {
            CommitType::Feat => "feat",
            CommitType::Fix => "fix",
            CommitType::Perf => "perf",
            CommitType::Refactor => "refactor",
            CommitType::Revert => "revert",
            CommitType::Style => "style",
            CommitType::Test => "test",
            CommitType::Build => "build",
            CommitType::Chore => "chore",
            CommitType::Ci => "ci",
            CommitType::Cd => "cd",
            CommitType::Docs => "docs",
            CommitType::Custom(value) => value.as_str(),
            CommitType::NonCompliant => "NonCompliant",
            CommitType::Unknown => "Unknown",
        }
    }
}

impl serde::Serialize for CommitType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for CommitType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CommitTypeVisitor;

        impl<'de> serde::de::Visitor<'de> for CommitTypeVisitor {
            type Value = CommitType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing a commit type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CommitType::from_str(value).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(CommitTypeVisitor)
    }
}

impl From<&str> for CommitType {
    fn from(s: &str) -> Self {
        CommitType::from_str(s).unwrap_or(CommitType::Unknown)
    }
}

impl FromStr for CommitType {
    type Err = ConventionalCommitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "build" => Ok(CommitType::Build),
            "chore" => Ok(CommitType::Chore),
            "ci" | "continuous integration" => Ok(CommitType::Ci),
            "cd" | "deploy" => Ok(CommitType::Cd),
            "doc" | "docs" | "documentation" => Ok(CommitType::Docs),
            "feat" | "feature" | "features" => Ok(CommitType::Feat),
            "fix" | "fixes" => Ok(CommitType::Fix),
            "perf" | "performance" => Ok(CommitType::Perf),
            "refactor" => Ok(CommitType::Refactor),
            "revert" => Ok(CommitType::Revert),
            "style" => Ok(CommitType::Style),
            "test" | "tests" => Ok(CommitType::Test),
            _ => Ok(CommitType::Custom(s.to_string())),
        }
    }
}

impl From<&CommitType> for CommitType {
    fn from(commit_type: &CommitType) -> Self {
        commit_type.clone()
    }
}

impl fmt::Display for CommitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::empty("", CommitType::Custom("".to_string()))]
    #[case::build("build", CommitType::Build)]
    #[case::chore("chore", CommitType::Chore)]
    #[case::ci("ci", CommitType::Ci)]
    #[case::cd("cd", CommitType::Cd)]
    #[case::docs("docs", CommitType::Docs)]
    #[case::feat_ignore_case_upper("Feat", CommitType::Feat)]
    #[case::feat_ignore_case_lower("feat", CommitType::Feat)]
    #[case::feat_ignore_case_wierd("fEAT", CommitType::Feat)]
    #[case::fix("fix", CommitType::Fix)]
    #[case::perf("perf", CommitType::Perf)]
    #[case::refactor("refactor", CommitType::Refactor)]
    #[case::style("style", CommitType::Style)]
    #[case::test("test", CommitType::Test)]
    #[case::custom("ENG-2345", CommitType::Custom("ENG-2345".to_string()))]
    fn test_convert_commit_type(#[case] input: impl AsRef<str>, #[case] expected: CommitType) {
        let input = input.as_ref().to_string();
        let actual: CommitType = serde_json::from_str(&format!("\"{}\"", input)).unwrap_or(CommitType::Unknown);
        assert_eq!(actual, expected);
    }
}
