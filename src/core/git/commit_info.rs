use std::path::{Path, PathBuf};

use crate::{get_rule, BumpRule, CommitType, ConventionalCommit};

#[derive(Debug, Default, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub files: Vec<PathBuf>,
    pub commit: ConventionalCommit,
}

impl CommitInfo {
    pub fn new(id: impl Into<String>, files: &[impl Into<PathBuf> + Clone], commit: impl Into<ConventionalCommit>) -> Self {
        Self {
            id: id.into(),
            files: files.iter().map(|file| file.clone().into()).collect(),
            commit: commit.into(),
        }
    }

    pub fn rule(&self, rules: &[(impl Into<CommitType> + Clone, impl Into<BumpRule> + Clone)]) -> BumpRule {
        let rules = rules
            .iter()
            .map(|(ct, br)| (ct.clone().into(), br.clone().into()))
            .collect::<Vec<_>>();
        let rules = match rules.is_empty() {
            true => crate::build_default_rules().collect::<Vec<_>>(),
            false => rules,
        };
        get_rule(rules.into_iter(), self.commit.commit_type.clone())
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
