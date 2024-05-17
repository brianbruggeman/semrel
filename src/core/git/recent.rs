use std::path::Path;

use crate::{Commit, RepositoryError};

pub fn get_recent_commit(path: impl AsRef<Path>) -> Result<Commit, RepositoryError> {
    let repo_path = path.as_ref().to_path_buf();

    // Open the repository
    let repo = gix::open(&repo_path).map_err(|_| RepositoryError::InvalidRepository(repo_path.clone()))?;

    // Get the reference to the HEAD
    let head = repo.head().map_err(|_| RepositoryError::NoHead(repo_path.clone()))?;

    // Peel to the most recent commit
    let commit_object = head
        .into_peeled_object()
        .map_err(|_| RepositoryError::NoCommit(repo_path.clone()))?;

    // Get the commit details
    let message = String::from_utf8_lossy(&commit_object.data);
    let commit = Commit::new(&message).map_err(|_| RepositoryError::NoCommitMessage(repo_path.clone(), message.to_string()))?;

    Ok(commit)
}
