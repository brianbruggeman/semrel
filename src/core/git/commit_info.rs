use std::path::{Path, PathBuf};

use crate::{match_rule, BumpRule, CommitType, ConventionalCommit};

#[derive(Debug, Default, Clone, serde::Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CommitInfo {
    // The commit id
    pub id: String,
    // The files that were changed in the commit
    pub files: Vec<PathBuf>,
    // The commit message
    pub commit: ConventionalCommit,
    // The timestamp of the commit
    pub timestamp: u64,
}

impl CommitInfo {
    pub fn new<I: IntoIterator<Item = impl Into<PathBuf> + Clone>>(id: impl Into<String>, files: I, commit: impl Into<ConventionalCommit>, timestamp: u64) -> Self {
        Self {
            id: id.into(),
            files: files.into_iter().map(|file| file.into()).collect(),
            commit: commit.into(),
            timestamp,
        }
    }

    /// Creates a string representation of the commit
    pub fn message(&self) -> String {
        self.commit.message()
    }

    pub fn commit_type(&self) -> &CommitType {
        &self.commit.commit_type
    }

    pub fn rule(&self, rules: &[(CommitType, BumpRule)]) -> BumpRule {
        if self.commit.is_breaking() {
            return BumpRule::Major;
        }
        let rules = rules.iter().map(|(ct, br)| (ct.into(), *br)).collect::<Vec<_>>();
        let rules = match rules.is_empty() {
            true => crate::build_default_rules().collect::<Vec<_>>(),
            false => rules,
        };
        match_rule(rules, self.commit.commit_type.clone())
    }

    pub fn contains(&self, file: impl AsRef<Path>) -> bool {
        self.files.iter().any(|f| f == file.as_ref())
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_commit(mut self, commit: impl Into<ConventionalCommit>) -> Self {
        self.commit = commit.into();
        self
    }

    pub fn add_file(mut self, file: impl AsRef<Path>) -> Self {
        self.files.push(file.as_ref().to_path_buf());
        self
    }

    pub fn extend_files(mut self, files: &[impl Into<PathBuf> + Clone]) -> Self {
        self.files.extend(files.iter().map(|file| file.clone().into()));
        self
    }
}
