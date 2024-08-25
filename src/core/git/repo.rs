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

pub fn find_top_of_repo(path: impl AsRef<Path>) -> Result<PathBuf, RepositoryError> {
    tracing::debug!("Searching for repository under: {}", path.as_ref().display());
    if let Ok(mut path) = path.as_ref().canonicalize().map_err(|_| RepositoryError::InvalidRepositoryPath(path.as_ref().into())) {
        loop {
            if is_repo(&path) {
                tracing::debug!("Found repository at: {path:?}");
                return path
                    .canonicalize()
                    .map_err(|_| RepositoryError::InvalidRepositoryPath(path.to_owned()));
            }
            tracing::trace!("Repository not found: {path:?}.  Looking for parent");
            path = path
                .parent()
                .ok_or_else(|| RepositoryError::InvalidRepositoryPath(path.to_owned()))?.to_path_buf();
        }
    } else {
        Err(RepositoryError::InvalidRepositoryPath(path.as_ref().into()))
    }
}

pub fn get_repo(path: impl AsRef<Path>) -> Result<Repository, RepositoryError> {
    // Open the repository
    let path = find_top_of_repo(path.as_ref())?;
    let repo = match Repository::open(path) {
        Ok(repo) => repo,
        Err(why) => {
            tracing::error!("Could not open repository: {}", why);
            return Err(RepositoryError::CouldNotOpenRepository(why.to_string()));
        }
    };
    Ok(repo)
}
