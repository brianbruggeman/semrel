use std::path::PathBuf;

use super::VersionError;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ConfigError {
    #[error("{0}")]
    InvalidRepositoryError(#[from] RepositoryError),
    #[error("{0}")]
    InvalidManifestError(#[from] ManifestError),
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
    #[error("Invalid config path: {0}")]
    ConfigNotFound(PathBuf),
    #[error("Empty config: {0}")]
    EmptyConfig(PathBuf),
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ManifestError {
    #[error("Invalid manifest path: {0}")]
    InvalidManifestPath(PathBuf),
    #[error("Invalid manifest version: {0}")]
    InvalidManifestVersion(VersionError),
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
    #[error("Invalid repository: {0}")]
    InvalidRepository(String),
    #[error("Invalid repository path: {0}")]
    WriteError(String),
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum RepositoryError {
    #[error("Failed to find repository: {0}. {1}")]
    BlobNotFound(String, String),
    #[error("Failed to find blob in repository: {0}. {1}")]
    BlobToTextError(String, String),
    #[error("Failed to retrieve commit diff: {0}")]
    CommitDiffError(String),
    #[error("Failed to retrieve commit tree: {0}")]
    CommitTreeError(String),
    #[error("Could not open repository: {0}")]
    CouldNotOpenRepository(String),
    #[error("Failed to find commit in repository: {0}")]
    CommitNotFound(String),
    #[error("Failed to find file in repository: {0}. {1}")]
    FileNotFound(String, String),
    #[error("Invalid repository path: {0}")]
    InvalidRepositoryPath(PathBuf),
    #[error("Invalid repository: {0}")]
    InvalidRepository(String),
    #[error("No HEAD found in repository: {0}")]
    NoHead(PathBuf),
    #[error("Failed to peel to commit in repository: {0}")]
    NoCommit(PathBuf),
    #[error("No commit message found in repository: {0} with id {1}")]
    NoCommitMessage(PathBuf, String),
    #[error("No parent commit found for commit: {0}")]
    NoParentCommit(String),
    #[error("No tags found for commit: {0}")]
    NoTags(String),
    #[error("Failed to retrieve commits: {0}")]
    NoCommits(String),
    #[error("Invalid manifest path: {0}")]
    InvalidManifestPath(PathBuf),
    #[error("{0}")]
    InvalidManifest(#[from] ManifestError),
    #[error("{0}")]
    InvalidConventionalCommit(#[from] ConventionalCommitError),
    #[error("Could not read file: {0}")]
    CouldNotReadFile(PathBuf),
    #[error("Invalid commit: {0}")]
    InvalidCommit(String),
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ConventionalCommitError {
    #[error("Invalid commit type: {0}")]
    InvalidCommitType(String),
    #[error("Invalid commit message: {0}")]
    InvalidCommitMessage(String),
    #[error("Invalid scope: {0}.  Do not use a standard conventional commit type as a scope.")]
    ScopeIsCommitType(String),
    #[error("{0}")]
    InvalidRepositoryError(String),
    #[error("Invalid Parse: could not parse as {0}.")]
    InvalidParse(String),
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum BumpRuleParse {
    #[error("Error parsing bump rule: {0}.  {1}")]
    ParseError(String, String),
}
