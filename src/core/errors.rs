use std::path::PathBuf;

use super::VersionError;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ManifestError {
    #[error("Invalid manifest path: {0}")]
    InvalidManifestPath(PathBuf),
    #[error("Invalid manifest version: {0}")]
    InvalidManifestVersion(VersionError),
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
    #[error("{0}")]
    RepositoryError(#[from] RepositoryError),
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum RepositoryError {
    #[error("Invalid repository: {0}")]
    InvalidRepository(PathBuf),
    #[error("No HEAD found in repository: {0}")]
    NoHead(PathBuf),
    #[error("Failed to peel to commit in repository: {0}")]
    NoCommit(PathBuf),
    #[error("No commit message found in repository: {0} with id {1}")]
    NoCommitMessage(PathBuf, String),
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
    RepositoryError(#[from] RepositoryError),
    #[error("Invalid Parse: could not parse as {0}.")]
    InvalidParse(String),
}
