use std::path::{Path, PathBuf};

use git2::{Commit, DiffOptions};

use super::CommitInfo;
use crate::RepositoryError;

pub fn get_commit_log(path: impl AsRef<Path>) -> Result<Vec<CommitInfo>, RepositoryError> {
    let repo = git2::Repository::open(path.as_ref()).map_err(|why| {
        tracing::error!("Failed to open repository at `{}`: {why}", path.as_ref().display());
        RepositoryError::InvalidRepository(path.as_ref().to_path_buf())
    })?;
    let mut revwalk = repo.revwalk().map_err(|why| {
        tracing::error!("Failed to create revwalk: {why}");
        RepositoryError::InvalidRepository(path.as_ref().to_path_buf())
    })?;
    // Push the head of the repository to the revwalk, otherwise it has no where to start
    revwalk.push_head().map_err(|why| {
        tracing::error!("Failed to push head: {why}");
        RepositoryError::InvalidRepository(path.as_ref().to_path_buf())
    })?;
    let mut commits = Vec::with_capacity(1024);
    let mut parent: Option<Commit> = None;
    for oid_result in revwalk {
        let oid = oid_result.map_err(|why| {
            tracing::error!("Failed to get oid: {why}");
            RepositoryError::InvalidRepository(path.as_ref().to_path_buf())
        })?;
        let commit = repo.find_commit(oid).map_err(|why| {
            tracing::error!("Failed to find commit: {why}");
            RepositoryError::InvalidRepository(path.as_ref().to_path_buf())
        })?;

        let mut files: Vec<PathBuf> = Vec::new();
        if let Some(parent_commit) = parent {
            let diff = repo
                .diff_tree_to_tree(
                    Some(&parent_commit.tree().map_err(|why| {
                        tracing::error!("Failed to old get tree: {why}");
                        RepositoryError::InvalidRepository(path.as_ref().to_path_buf())
                    })?),
                    Some(&commit.tree().map_err(|why| {
                        tracing::error!("Failed to new get tree: {why}");
                        RepositoryError::InvalidRepository(path.as_ref().to_path_buf())
                    })?),
                    Some(&mut DiffOptions::new()),
                )
                .map_err(|why| {
                    tracing::error!("Failed to get diff: {why}");
                    RepositoryError::InvalidRepository(path.as_ref().to_path_buf())
                })?;
            for i in 0..diff.deltas().len() {
                if let Some(delta) = diff.get_delta(i) {
                    if let Some(file_path) = delta.new_file().path() {
                        files.push(file_path.into());
                    }
                }
            }
        }
        let message = commit.message().unwrap_or("").to_string();
        parent = Some(commit);
        let info = CommitInfo::new(oid.to_string(), &files, message);
        commits.push(info);
    }
    if commits.is_empty() {
        return Err(RepositoryError::NoCommits(path.as_ref().display().to_string()));
    }
    Ok(commits)
}
