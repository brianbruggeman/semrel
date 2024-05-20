use std::path::Path;

use crate::{Commit, RepositoryError};

pub fn get_recent_commit(path: impl AsRef<Path>) -> Result<Commit, RepositoryError> {
    let repo_path = match path.as_ref().is_file() {
        true => path
            .as_ref()
            .parent()
            .ok_or_else(|| RepositoryError::InvalidRepository(path.as_ref().to_path_buf()))?
            .to_path_buf(),
        false => path.as_ref().to_path_buf(),
    };
    tracing::debug!("Getting commit from: {}", repo_path.display());

    // Open the repository
    let repo = gix::open(&repo_path).map_err(|_| RepositoryError::InvalidRepository(repo_path.clone()))?;
    tracing::debug!("Found repo: {:?}", repo);

    // Get the reference to the HEAD
    let head = repo.head().map_err(|_| RepositoryError::NoHead(repo_path.clone()))?;

    // Peel to the most recent commit
    let commit_object = head
        .into_peeled_object()
        .map_err(|_| RepositoryError::NoCommit(repo_path.clone()))?;
    tracing::debug!("Found commit object: {:?}", commit_object);

    // Get the commit details
    let message = String::from_utf8_lossy(&commit_object.data);
    let message = prune_message(message);
    tracing::debug!("Found commit message: {:?}", message);
    let commit = Commit::new(&message).map_err(|_| RepositoryError::NoCommitMessage(repo_path.clone(), message.to_string()))?;
    tracing::debug!("commit={commit}");
    Ok(commit)
}

pub fn prune_message(message: impl AsRef<str>) -> String {
    message
        .as_ref()
        .lines()
        .filter(|line| {
            !line.starts_with("Co-authored-by:")
                && !line.starts_with("Signed-off-by:")
                && !line.starts_with("Change-Id:")
                && !line.starts_with("tree")
                && !line.starts_with("author")
                && !line.starts_with("committer")
                && !line.is_empty()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
