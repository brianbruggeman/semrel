use std::path::{Path, PathBuf};

use crate::RepositoryError;

pub fn is_repo(path: impl AsRef<Path>) -> bool {
    gix::open(path.as_ref()).is_ok()
}

pub fn top_of_repo(path: impl AsRef<Path>) -> Result<PathBuf, RepositoryError> {
    let repo_path = path.as_ref().to_path_buf();

    // Open the repository
    let repo = gix::open(&repo_path).map_err(|_| RepositoryError::InvalidRepository(repo_path.clone()))?;

    // Get the top-level directory of the repository
    let repo_top_path = repo
        .path()
        .parent()
        .ok_or_else(|| RepositoryError::InvalidRepository(repo_path.clone()))?
        .to_path_buf();

    Ok(repo_top_path)
}
