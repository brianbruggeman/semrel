use std::path::Path;

use super::prune_message;
use crate::{ConventionalCommit, RepositoryError};

pub fn get_recent_commit(path: impl AsRef<Path>) -> Result<ConventionalCommit, RepositoryError> {
    let repo_path = match path.as_ref().is_file() {
        true => path
            .as_ref()
            .parent()
            .ok_or_else(|| RepositoryError::InvalidRepositoryPath(path.as_ref().to_path_buf()))?
            .to_path_buf(),
        false => path.as_ref().to_path_buf(),
    };
    tracing::debug!("Getting commit from: {}", repo_path.display());

    // Open the repository
    let repo = git2::Repository::open(&repo_path).map_err(|_| RepositoryError::InvalidRepositoryPath(repo_path.clone()))?;
    tracing::debug!("Found repo under: {}", path.as_ref().display());

    // Get the reference to the HEAD
    let head = repo.head().map_err(|_| RepositoryError::NoHead(repo_path.clone()))?;

    // Peel to the most recent commit
    let commit_object = head
        .peel(git2::ObjectType::Commit)
        .map_err(|_| RepositoryError::NoCommit(repo_path.clone()))?;
    tracing::debug!("Found commit object: {:?}", commit_object);

    // Get the commit details
    let commit = commit_object
        .into_commit()
        .map_err(|_| RepositoryError::NoCommit(repo_path.clone()))?;
    let message = commit.message().unwrap_or_default();
    tracing::debug!("Full commit message: \n{message}");
    let message = prune_message(message);
    tracing::debug!("Commit::new({message:?})");
    let commit = ConventionalCommit::new(&message).map_err(|_| RepositoryError::NoCommitMessage(repo_path.clone(), message.to_string()))?;
    tracing::debug!("{commit:?}");
    Ok(commit)
}
