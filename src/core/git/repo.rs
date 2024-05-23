use git2::Repository;
use std::path::{Path, PathBuf};

use crate::RepositoryError;

pub fn is_repo(path: impl AsRef<Path>) -> bool {
    Repository::open(path.as_ref()).is_ok()
}

pub fn top_of_repo(path: impl AsRef<Path>) -> Result<PathBuf, RepositoryError> {
    let repo_path = path.as_ref().to_path_buf();

    // Open the repository
    let repo = get_repo(path.as_ref())?;

    // Get the top-level directory of the repository
    let repo_top_path = repo
        .workdir()
        .ok_or_else(|| RepositoryError::InvalidRepositoryPath(repo_path.clone()))?
        .to_path_buf();

    Ok(repo_top_path)
}

pub fn get_repo(path: impl AsRef<Path>) -> Result<Repository, RepositoryError> {
    // Open the repository
    tracing::debug!("Searching for repository under: {}", path.as_ref().display());
    let repo = Repository::open(path.as_ref()).map_err(|why| RepositoryError::CouldNotOpenRepository(why.to_string()))?;
    tracing::debug!("Found repository at: {}", repo.path().parent().unwrap().display());
    Ok(repo)
}
