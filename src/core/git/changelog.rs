use std::collections::HashMap;
use std::path::{Path, PathBuf};

use git2::{Oid, TreeWalkMode};

use super::CommitInfo;
use crate::{find_top_of_repo, BumpRule, CommitType, ConventionalCommit, RepositoryError, SimpleVersion, SupportedManifest};

#[derive(Debug, Clone)]
pub struct CommitWithVersion {
    pub commit_info: CommitInfo,
    pub version_at_commit: Option<SimpleVersion>, // None if no manifest change
}

#[derive(Debug, Clone)]
pub struct StoppingContext {
    pub current_version: SimpleVersion,
    pub max_bump_so_far: BumpRule,
    pub commits_collected: Vec<CommitInfo>,
}

/// Determines if we should stop collecting commits based on the version boundary and max bump rule
pub fn should_stop_collecting(context: &StoppingContext, commit_with_version: &CommitWithVersion, _rules: &[(CommitType, BumpRule)]) -> bool {
    // If this commit doesn't have a version change, continue collecting
    let Some(version_at_commit) = &commit_with_version.version_at_commit else {
        return false;
    };

    // If the version at this commit is not less than current, continue
    if *version_at_commit >= context.current_version {
        return false;
    }

    // Now we have a version boundary. Check if it's the right boundary for our max bump rule
    match (context.max_bump_so_far, version_at_commit.minor(), version_at_commit.patch()) {
        (BumpRule::Major, 0, 0) => {
            tracing::debug!("Stopped at major boundary: version {}", version_at_commit);
            true
        }
        (BumpRule::Minor, _, 0) => {
            tracing::debug!("Stopped at minor boundary: version {}", version_at_commit);
            true
        }
        (BumpRule::Patch, _, _) => {
            tracing::debug!("Stopped at patch boundary: version {}", version_at_commit);
            true
        }
        _ => {
            tracing::debug!("Not the right boundary for {:?}, continuing past version {}", context.max_bump_so_far, version_at_commit);
            false
        }
    }
}

/// Transforms commits into CommitWithVersion by checking for manifest changes
pub fn transform_commits_to_versioned(
    repo: &git2::Repository,
    _manifest_path: &Path,
    relative_manifest_path: &Path,
    commits: impl IntoIterator<Item = CommitInfo>,
) -> Result<Vec<CommitWithVersion>, RepositoryError> {
    let mut result = Vec::new();

    for commit_info in commits {
        let oid = Oid::from_str(&commit_info.id).map_err(|_| RepositoryError::InvalidRepositoryPath(PathBuf::from(&commit_info.id)))?;
        let commit = repo
            .find_commit(oid)
            .map_err(|_| RepositoryError::CommitNotFound(oid.to_string()))?;

        // Check if this commit changed the manifest file
        let version_at_commit = if commit_info.files.iter().any(|f| f == relative_manifest_path) {
            tracing::debug!("Manifest file found in commit: {}", commit_info.id);
            let data = load_file_data(repo, &commit, relative_manifest_path)?;
            let version = SupportedManifest::parse(relative_manifest_path, &data)?.version()?;
            Some(version)
        } else {
            None
        };

        result.push(CommitWithVersion { commit_info, version_at_commit });
    }

    Ok(result)
}

/// Optimized streaming commit collection with early stopping
/// This walks commits one at a time and stops as soon as we find the appropriate version boundary
pub fn collect_changelog_commits_streaming(
    repo: &git2::Repository,
    manifest_path: &Path,
    relative_manifest_path: &Path,
    current_version: SimpleVersion,
    rules: &[(CommitType, BumpRule)],
) -> Result<Vec<CommitInfo>, RepositoryError> {
    let mut collected_commits = Vec::new();
    let mut max_bump_so_far = BumpRule::default();

    // Create revwalk iterator
    let walker = revwalk(repo, manifest_path)?;

    for oid in walker {
        let commit = repo
            .find_commit(oid)
            .map_err(|_| RepositoryError::CommitNotFound(oid.to_string()))?;

        // Parse conventional commit
        let conventional_commit = ConventionalCommit::try_from(commit.message().unwrap_or_default())?;
        let files_changed = get_files_changed(repo, oid)?;
        let timestamp = commit.time().seconds();
        let timestamp = num_traits::cast::<i64, u64>(timestamp).unwrap();
        let commit_info = CommitInfo::new(oid.to_string(), files_changed, conventional_commit, timestamp);

        // Check if this commit changed the manifest file (has version boundary)
        let version_at_commit = if commit_info.files.iter().any(|f| f == relative_manifest_path) {
            tracing::debug!("Manifest file found in commit: {}", commit_info.id);
            let data = load_file_data(repo, &commit, relative_manifest_path)?;
            let version = SupportedManifest::parse(relative_manifest_path, &data)?.version()?;
            Some(version)
        } else {
            None
        };

        // Apply stopping logic immediately
        if let Some(version_at_commit) = &version_at_commit {
            // If the version at this commit is not less than current, collect and continue
            if *version_at_commit >= current_version {
                let commit_bump = commit_info.rule(rules);
                max_bump_so_far = max_bump_so_far.max(commit_bump);
                collected_commits.push(commit_info);
                tracing::debug!("Including newer version commit: {} {}", oid, commit.message().unwrap_or_default());
                continue;
            }

            // We found a version boundary that's less than current version
            // Check if this boundary is appropriate for our max bump level
            let boundary_matches = match (max_bump_so_far, version_at_commit.minor(), version_at_commit.patch()) {
                (BumpRule::Major, 0, 0) => true, // Major bump needs major boundary (x.0.0)
                (BumpRule::Minor, _, 0) => true, // Minor bump needs minor boundary (x.y.0)
                (BumpRule::Patch, _, _) => true, // Patch bump can stop at any boundary
                _ => false,
            };

            if boundary_matches {
                tracing::debug!("Found appropriate boundary for {:?} at version {} - stopping collection", max_bump_so_far, version_at_commit);
                break;
            } else {
                tracing::debug!("Boundary at {} not appropriate for {:?}, continuing past it", version_at_commit, max_bump_so_far);
                // Continue past this boundary - collect this commit too since it's part of our changelog
                let commit_bump = commit_info.rule(rules);
                max_bump_so_far = max_bump_so_far.max(commit_bump);
                collected_commits.push(commit_info);
                tracing::debug!("Including boundary commit: {} {}", oid, commit.message().unwrap_or_default());
            }
        } else {
            // No version boundary - collect this commit and update max bump
            let commit_bump = commit_info.rule(rules);
            max_bump_so_far = max_bump_so_far.max(commit_bump);
            collected_commits.push(commit_info);
            tracing::debug!("Including commit: {} {}", oid, commit.message().unwrap_or_default());
        }
    }

    Ok(collected_commits)
}

/// Legacy function for backwards compatibility and testing
/// This loads all commits into memory first (inefficient for large repos)
pub fn collect_changelog_commits(commits_with_versions: Vec<CommitWithVersion>, current_version: SimpleVersion, rules: &[(CommitType, BumpRule)]) -> Vec<CommitInfo> {
    let mut collected_commits = Vec::new();
    let mut max_bump_so_far = BumpRule::default();

    // Phase 1: First pass - collect all commits without version boundaries and calculate max bump
    for commit_with_version in &commits_with_versions {
        if commit_with_version.version_at_commit.is_none() {
            // No version boundary - collect this commit and update max bump
            let commit_bump = commit_with_version.commit_info.rule(rules);
            max_bump_so_far = max_bump_so_far.max(commit_bump);
            collected_commits.push(commit_with_version.commit_info.clone());
            tracing::debug!("Including commit: {} {}", commit_with_version.commit_info.id, commit_with_version.commit_info.message());
        }
    }

    // Phase 2: Now look for the appropriate stopping boundary based on max_bump_so_far
    for commit_with_version in &commits_with_versions {
        if let Some(version_at_commit) = &commit_with_version.version_at_commit {
            // If the version at this commit is not less than current, skip it (it's newer than current)
            if *version_at_commit >= current_version {
                continue;
            }

            // We found a version boundary that's less than current version
            // Check if this boundary is appropriate for our max bump level
            let boundary_matches = match (max_bump_so_far, version_at_commit.minor(), version_at_commit.patch()) {
                (BumpRule::Major, 0, 0) => true, // Major bump needs major boundary (x.0.0)
                (BumpRule::Minor, _, 0) => true, // Minor bump needs minor boundary (x.y.0)
                (BumpRule::Patch, _, _) => true, // Patch bump can stop at any boundary
                _ => false,
            };

            if boundary_matches {
                tracing::debug!("Found appropriate boundary for {:?} at version {}", max_bump_so_far, version_at_commit);
                break;
            } else {
                tracing::debug!("Boundary at {} not appropriate for {:?}, continuing past it", version_at_commit, max_bump_so_far);
                // Continue past this boundary - collect this commit too since it's part of our changelog
                let commit_bump = commit_with_version.commit_info.rule(rules);
                max_bump_so_far = max_bump_so_far.max(commit_bump);
                collected_commits.push(commit_with_version.commit_info.clone());
                tracing::debug!("Including boundary commit: {} {}", commit_with_version.commit_info.id, commit_with_version.commit_info.message());
            }
        }
    }

    collected_commits
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CommitGroup {
    commit_type: CommitType,
    scopes: Vec<(String, Vec<CommitInfo>)>,
}

impl CommitGroup {
    pub fn new(commit_type: CommitType, scopes: Vec<(String, Vec<CommitInfo>)>) -> Self {
        Self { commit_type, scopes }
    }
}

pub struct ChangeLog {
    pub current_version: SimpleVersion,
    pub changes: Vec<CommitInfo>,
}

impl ChangeLog {
    pub fn new(current_version: impl Into<SimpleVersion>, changes: impl AsRef<[CommitInfo]>) -> Self {
        Self {
            current_version: current_version.into(),
            changes: changes.as_ref().to_owned(),
        }
    }

    pub fn next_version(&self, rules: &[(CommitType, BumpRule)]) -> SimpleVersion {
        let rules = rules.to_vec();
        let version = self.current_version;
        let max_bump = self
            .changes
            .iter()
            .fold(BumpRule::default(), |max_bump, commit| max_bump.max(commit.rule(&rules)));
        version.bump(max_bump)
    }

    /// Generates a release notes for the changelog
    ///
    /// The release notes are generated by organizing the commits into sections based on the commit type
    /// and scope.  Within each scoped section, the commit messages are listed in reverse chronological order.
    ///
    /// # Example
    /// ```markdown
    /// # Release notes: (v1.0.0) - 2021-01-01
    /// ## Breaking Changes
    /// ## Features
    /// ### <scope>
    /// - <commit message>
    /// - <commit message>
    /// - <commit message>
    /// ## Fixes
    /// - Others
    /// ```
    pub fn release_notes(&self, rules: &[(CommitType, BumpRule)]) -> String {
        let aggregated_commits = self.aggregated_commits();
        let today = chrono::Local::now();
        let mut notes = format!("# Release notes: {} ({})\n", self.next_version(rules), today.format("%Y-%m-%d"));
        for commit_group in aggregated_commits {
            notes.push_str(&format!("\n\n## {}\n", commit_group.commit_type.as_release_note()));
            for (scope, commits) in commit_group.scopes {
                if !scope.is_empty() {
                    notes.push_str(&format!("\n### {scope}\n"));
                }
                for commit in commits {
                    if commit.commit_type().as_str().starts_with("semrel") {
                        continue;
                    }
                    notes.push_str(&format!("- {}\n", commit.commit.subject));
                }
            }
        }
        notes
    }

    pub fn aggregated_commits(&self) -> Vec<CommitGroup> {
        let mut map: HashMap<CommitType, HashMap<String, Vec<CommitInfo>>> = HashMap::new();
        for commit_info in &self.changes {
            let commit_type = commit_info.commit.commit_type.clone();
            let scope = commit_info.commit.scope.clone().unwrap_or_default();
            let entry = map.entry(commit_type).or_default();
            let scope = entry.entry(scope).or_default();
            scope.push(commit_info.clone());
        }
        let ignored = ["semrel"];
        let mut vec: Vec<CommitGroup> = map
            .into_iter()
            .filter(|(commit_type, _)| !ignored.iter().any(|s| commit_type.as_str().starts_with(s)))
            .map(|(commit_type, scopes)| CommitGroup::new(commit_type, scopes.into_iter().collect()))
            .collect();
        vec.sort();
        vec
    }
}

/// Generates a changelog for the commit
///
/// This requires going back to the previous bump level and collecting all commits up since that point.
///   - For patches, this is just going back to the previous patch bump
///   - For minors, this is going back to the previous minor bump which includes all patches since then
///   - For majors, this is going back to the previous major bump which includes all minors and patches since then
///
///
pub fn get_changelog(repo: &git2::Repository, manifest_path: impl Into<PathBuf>, rules: &[(CommitType, BumpRule)]) -> Result<ChangeLog, RepositoryError> {
    let manifest_path = manifest_path.into();
    tracing::trace!("Getting changelog for manifest path: {}", manifest_path.display());
    let project_path = manifest_path.parent().unwrap();
    tracing::trace!("Getting changelog for project path: {}", project_path.display());
    let repo_path = find_top_of_repo(project_path)?;
    let project_path = Box::leak(Box::new({
        let project_path = project_path.canonicalize().unwrap();
        if project_path.is_dir() {
            project_path.canonicalize().unwrap()
        } else {
            project_path.parent().unwrap().canonicalize().unwrap().to_path_buf()
        }
    }));
    let relative_manifest_path = {
        let new_path = compute_relative_path(&repo_path, &manifest_path);
        match new_path.starts_with("./") {
            true => new_path.strip_prefix("./").unwrap().to_path_buf(),
            false => new_path,
        }
    };
    tracing::trace!("Searching for relative manifest path: {}", relative_manifest_path.display());
    let relative_project_path = compute_relative_path(&repo_path, project_path);
    tracing::debug!("Starting get_changelog for path: {}", relative_project_path.display());
    let manifest = SupportedManifest::try_from(manifest_path.to_owned()).map_err(|err| {
        tracing::error!("Failed to get manifest: {err}");
        err
    })?;
    let current_version = manifest.version()?;
    tracing::debug!("Current version: {}", current_version);

    // Use the optimized streaming approach that stops early
    let captured_commits = collect_changelog_commits_streaming(repo, &manifest_path, &relative_manifest_path, current_version, rules)?;

    let changelog = ChangeLog::new(current_version, captured_commits);
    tracing::debug!("Finished get_changelog. Current version: {}", current_version);
    Ok(changelog)
}

/// Retrieves the data of a file in a specific commit
///
/// # Arguments
///
/// * `repo` - A reference to the repository
/// * `commit` - The commit to retrieve the file data from
/// * `path` - The path of the file in the repository
///
/// # Returns
///
/// * `Result<String, RepositoryError>` - The file data as a string if successful, or an error
fn load_file_data(repo: &git2::Repository, commit: &git2::Commit, path: impl AsRef<Path>) -> Result<String, RepositoryError> {
    let path = path.as_ref();
    let oid = commit.id();
    tracing::trace!("Loading file data for path: {} using commit id: {}", path.display(), oid);
    let tree = commit
        .tree()
        .map_err(|_| RepositoryError::CommitTreeError(commit.id().to_string()))?;
    let entry = tree
        .get_path(path)
        .map_err(|why| RepositoryError::FileNotFound(path.to_str().unwrap().to_string(), why.to_string()))?;
    let blob = repo
        .find_blob(entry.id())
        .map_err(|why| RepositoryError::BlobNotFound(entry.id().to_string(), why.to_string()))?;
    let content = std::str::from_utf8(blob.content()).map_err(|why| RepositoryError::BlobToTextError(entry.id().to_string(), why.to_string()))?;
    tracing::trace!("Successfully loaded file data for path: {}", path.display());
    Ok(content.to_string())
}

/// Generates a changelog as an iterator of commit information
///
/// This has no filter but simply returns all commits in the repository converted from oid to commit information
/// As an iterator, this is a lazy eval.  We do not want to capture the entire commit history in memory
#[allow(clippy::needless_lifetimes)]
pub fn revwalk_commit_log<'a>(repo: &'a git2::Repository, project_path: impl Into<PathBuf>) -> Result<impl IntoIterator<Item = CommitInfo> + 'a, RepositoryError> {
    let walker = revwalk(repo, project_path)?;
    let data = walker.into_iter().flat_map(|oid| {
        let commit = repo
            .find_commit(oid)
            .map_err(|_| RepositoryError::CommitNotFound(oid.to_string()))?;
        let conventional_commit = ConventionalCommit::try_from(commit.message().unwrap_or_default())?;
        let files_changed = get_files_changed(repo, oid)?;
        let timestamp = commit.time().seconds();
        let timestamp = num_traits::cast::<i64, u64>(timestamp).unwrap();
        let info: CommitInfo = CommitInfo::new(oid.to_string(), files_changed, conventional_commit, timestamp);
        Ok::<CommitInfo, RepositoryError>(info)
    });
    Ok(data)
}

/// Retrieves the files changed in a commit
fn get_files_changed(repo: &git2::Repository, oid: impl Into<git2::Oid>) -> Result<Vec<PathBuf>, RepositoryError> {
    let oid = oid.into();
    let commit = repo
        .find_commit(oid)
        .map_err(|_| RepositoryError::CommitNotFound(oid.to_string()))?;
    let tree = commit.tree().map_err(|why| RepositoryError::CommitTreeError(why.to_string()))?;
    let mut files = vec![];
    // If the commit has a parent, compare it to the parent's tree
    if let Some(parent_commit) = commit.parents().next() {
        let parent_tree = parent_commit
            .tree()
            .map_err(|why| RepositoryError::CommitTreeError(why.to_string()))?;
        let diff = repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
            .map_err(|why| RepositoryError::CommitDiffError(why.to_string()))?;

        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    files.push(path.to_path_buf());
                }
                true
            },
            None,
            None,
            None,
        )
        .map_err(|why| RepositoryError::CommitDiffError(why.to_string()))?;
    } else {
        // If there's no parent, this is the initial commit
        // We consider all files in the initial commit as "changed"
        tree.walk(TreeWalkMode::PreOrder, |_, entry| {
            if let Some(name) = entry.name() {
                files.push(PathBuf::from(name));
            }
            0
        })
        .map_err(|why| RepositoryError::CommitTreeError(why.to_string()))?;
    }
    Ok(files)
}

/// Generates an iterator that walks the repository in reverse order
#[allow(clippy::needless_lifetimes)]
pub fn revwalk<'a>(repo: &'a git2::Repository, project_path: impl Into<PathBuf>) -> Result<impl IntoIterator<Item = Oid> + 'a, RepositoryError> {
    let repo = Box::leak(Box::new(repo));
    let project_path = project_path.into();
    let repo_path = find_top_of_repo(&project_path)?;
    let project_path = Box::leak(Box::new({
        let project_path = project_path.canonicalize().unwrap();
        if project_path.is_dir() {
            project_path.canonicalize().unwrap()
        } else {
            project_path.parent().unwrap().canonicalize().unwrap().to_path_buf()
        }
    }));

    let mut revwalk = repo.revwalk().map_err(|why| {
        tracing::error!("Failed to create revwalk: {why}");
        RepositoryError::InvalidRepository(why.to_string())
    })?;
    // Push the head of the repository to the revwalk, otherwise it has no where to start
    revwalk.push_head().map_err(|why| {
        tracing::error!("Failed to push head: {why}");
        RepositoryError::InvalidRepository(why.to_string())
    })?;

    // Use topological sort
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL).map_err(|why| {
        tracing::error!("Failed to sort repo: {why}");
        RepositoryError::InvalidRepository(why.to_string())
    })?;

    // Include merging branches
    revwalk.simplify_first_parent().map_err(|why| {
        tracing::error!("Failed to simplify: {why}");
        RepositoryError::InvalidRepository(why.to_string())
    })?;

    // Return all of the oids, but filter on the project files
    //  This is preliminary support for multi-package/monorepos
    #[allow(clippy::needless_borrows_for_generic_args)]
    let data = revwalk
        .flat_map(|oid| oid.map_err(|why| RepositoryError::InvalidRepository(why.to_string())))
        .flat_map::<Result<(Oid, Vec<PathBuf>), RepositoryError>, _>(|oid| {
            let files_changed = get_files_changed(repo, oid)?;
            Ok((oid, files_changed))
        })
        .filter_map(move |(oid, files)| match files.is_empty() {
            true => None,
            false => match files
                .iter()
                .filter_map(|file| match repo_path.join(file).starts_with(&project_path) {
                    true => Some(repo_path.join(file)),
                    false => None,
                })
                .any(|file| file.starts_with(&project_path))
            {
                true => Some(oid),
                false => None,
            },
        });
    Ok(data.into_iter())
}

fn compute_relative_path(repo_path: impl AsRef<Path>, project_path: impl AsRef<Path>) -> PathBuf {
    let repo_path = repo_path.as_ref();
    let project_path = project_path.as_ref();
    let mut relative_path = PathBuf::new();
    let mut repo_components = repo_path.components();
    let mut project_components = project_path.components();

    // Find the common prefix
    let mut index = 0;
    loop {
        match (repo_components.next(), project_components.next()) {
            (Some(repo_comp), Some(project_comp)) if repo_comp == project_comp => {
                index += 1;
            }
            _ => break,
        }
    }

    // If there is no common prefix, return project_path
    if index == 0 {
        return project_path.to_path_buf();
    }

    // Determine the number of components to pop from repo_path
    let mut repo_components = repo_path.components();
    for _ in 0..index {
        repo_components.next();
    }
    for _ in repo_components {
        relative_path.push("..");
    }

    // Add the remaining components of project_path
    let mut project_components = project_path.components();
    for _ in 0..index {
        project_components.next();
    }
    for component in project_components {
        relative_path.push(component.as_os_str());
    }

    relative_path
}

#[cfg(test)]
mod tests {
    use super::*;

    use git2::{Oid, Repository, Signature};
    use rstest::rstest;
    use tempfile::TempDir;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TestRepoLanguage {
        Rust,
        Python,
        JavaScript,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TestRepoType {
        Empty,
        SingleCommit,
        MultipleCommits,
        BranchedMultipleCommits,
    }

    // Factory for building test projects
    struct TestProject<'a> {
        path: PathBuf,
        test_repo: &'a TestRepo,
        language: TestRepoLanguage,
        type_: TestRepoType,
    }

    impl<'a> TestProject<'a> {
        const FIRST_COMMIT_MESSAGE: &'static str = "First commit";

        pub fn new(path: impl AsRef<Path>, repo: &'a TestRepo, language: TestRepoLanguage, type_: TestRepoType) -> Self {
            Self {
                path: path.as_ref().to_owned(),
                test_repo: repo,
                language,
                type_,
            }
        }

        pub fn build(&self) -> Result<(), RepositoryError> {
            match self.type_ {
                TestRepoType::Empty => {}
                TestRepoType::SingleCommit => self.init()?,
                TestRepoType::MultipleCommits => {
                    self.init()?;
                    self.build_multiple_commits()?;
                }
                TestRepoType::BranchedMultipleCommits => {
                    self.init()?;
                    self.test_repo.branch("add_library")?;
                    self.build_multiple_commits()?
                }
            };
            Ok(())
        }

        fn build_multiple_commits(&self) -> Result<(), RepositoryError> {
            match self.language {
                TestRepoLanguage::Rust => self.build_multiple_commits_rust()?,
                TestRepoLanguage::Python => self.build_multiple_commits_python()?,
                TestRepoLanguage::JavaScript => self.build_multiple_commits_javascript()?,
            }
            Ok(())
        }

        fn build_multiple_commits_rust(&self) -> Result<(), RepositoryError> {
            // insert small sleep delay
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Add lib.rs
            self.test_repo.add_file("lib.rs", "Hello, world!").expect("Failed to add file");
            self.test_repo.commit("fix: create library").unwrap();

            // insert small sleep delay
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Update dependencies
            self.test_repo
                .add_file("foo.rs", "fn add(i: i32, j: i32) -> i32 {\n    i + j\n}")
                .expect("Failed to add file");
            self.test_repo.commit("fix: create add").unwrap();

            // Add foo.rs
            self.test_repo
                .add_file("foo.rs", "fn add(i: i32, j: i32) -> i32 {\n    i + j\n}")
                .expect("Failed to add file");
            self.test_repo.commit("fix: create add").unwrap();

            // insert small sleep delay
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Update lib to include foo
            self.test_repo
                .add_file("lib.rs", "mod foo;\n\npub use foo::*;\n")
                .expect("Failed to add file");
            self.test_repo.commit("fix: use add").unwrap();

            // insert small sleep delay
            std::thread::sleep(std::time::Duration::from_millis(10));
            Ok(())
        }

        fn build_multiple_commits_python(&self) -> Result<(), RepositoryError> {
            // insert small sleep delay
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Add main.py
            self.test_repo
                .add_file("main.py", "print('Hello, world!')")
                .expect("Failed to add file");
            self.test_repo.commit("fix: create library").unwrap();

            // insert small sleep delay
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Add utils.py
            self.test_repo
                .add_file("utils.py", "def add(i, j):\n    return i + j")
                .expect("Failed to add file");
            self.test_repo.commit("fix: create add").unwrap();

            // insert small sleep delay
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Update main.py to use utils
            self.test_repo
                .add_file("main.py", "from utils import add\n\nprint(add(1, 2))")
                .expect("Failed to add file");
            self.test_repo.commit("fix: use add").unwrap();

            // insert small sleep delay
            std::thread::sleep(std::time::Duration::from_millis(10));
            Ok(())
        }

        fn build_multiple_commits_javascript(&self) -> Result<(), RepositoryError> {
            // insert small sleep delay
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Add main.js
            self.test_repo
                .add_file("main.js", "console.log('Hello, world!');")
                .expect("Failed to add file");
            self.test_repo.commit("fix: create library").unwrap();

            // insert small sleep delay
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Add utils.js
            self.test_repo
                .add_file("utils.js", "function add(i, j) { return i + j; }")
                .expect("Failed to add file");
            self.test_repo.commit("fix: create add").unwrap();

            // insert small sleep delay
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Update main.js to use utils
            self.test_repo
                .add_file("main.js", "const { add } = require('./utils');\n\nconsole.log(add(1, 2));")
                .expect("Failed to add file");
            self.test_repo.commit("fix: use add").unwrap();

            // insert small sleep delay
            std::thread::sleep(std::time::Duration::from_millis(10));
            Ok(())
        }

        fn generate_changelog(&self, rules: &[(CommitType, BumpRule)]) -> Result<ChangeLog, RepositoryError> {
            let manifest_path = crate::find_manifest(&self.path)?;
            get_changelog(&self.test_repo.repo, manifest_path, &rules)
        }

        fn generate_next_version(&self, rules: &[(CommitType, BumpRule)]) -> Result<SimpleVersion, RepositoryError> {
            let manifest_path = crate::find_manifest(&self.path)?;
            let changelog = get_changelog(&self.test_repo.repo, manifest_path, &rules)?;
            Ok(changelog.next_version(&rules))
        }

        fn generate_log_messages(&self, rules: &[(CommitType, BumpRule)]) -> Result<Vec<String>, RepositoryError> {
            let manifest_path = crate::find_manifest(&self.path)?;
            let changelog = get_changelog(&self.test_repo.repo, manifest_path, &rules)?;
            let log_messages = changelog.changes.iter().map(|v| v.commit.message()).collect::<Vec<_>>();
            Ok(log_messages)
        }

        fn generate_pretty_log_messages(&self, rules: &[(CommitType, BumpRule)]) -> Result<String, RepositoryError> {
            let messages = self.generate_log_messages(rules)?;
            let mut final_message = "\n".to_string();
            for (index, message) in messages.iter().enumerate() {
                final_message.push_str(&format!("{}: {}\n", index, message));
            }
            Ok(final_message)
        }

        fn init(&self) -> Result<(), RepositoryError> {
            match self.language {
                TestRepoLanguage::Rust => self.init_rust(),
                TestRepoLanguage::Python => self.init_python(),
                TestRepoLanguage::JavaScript => self.init_javascript(),
            }
        }

        fn init_rust(&self) -> Result<(), RepositoryError> {
            let cargo_toml = textwrap::dedent(
                r#"
                [package]
                name = "test"
                version = "0.1.0"
                "#,
            );
            self.test_repo.add_file("Cargo.toml", cargo_toml)?;
            self.test_repo.commit(TestProject::FIRST_COMMIT_MESSAGE)?;
            Ok(())
        }

        fn init_python(&self) -> Result<(), RepositoryError> {
            let pyproject_toml = textwrap::dedent(
                r#"
                [tool.poetry]
                name = "test"
                version = "0.1.0"
                "#,
            );
            self.test_repo.add_file("pyproject.toml", pyproject_toml)?;
            self.test_repo.commit(TestProject::FIRST_COMMIT_MESSAGE)?;
            Ok(())
        }

        fn init_javascript(&self) -> Result<(), RepositoryError> {
            let package_json = textwrap::dedent(
                r#"
                {
                    "name": "test",
                    "version": "0.1.0"
                }
                "#,
            );
            self.test_repo.add_file("package.json", package_json)?;
            self.test_repo.commit(TestProject::FIRST_COMMIT_MESSAGE)?;
            Ok(())
        }
    }

    struct TestRepo {
        temp_dir: TempDir,
        repo: Repository,
    }

    impl TestRepo {
        fn new() -> Self {
            // Default behavior is to setup tracing to error if the RUST_LOG variable is not set
            //  This code will only setup tracing if the RUST_LOG variable is set
            if std::env::var("RUST_LOG").is_ok() {
                tracing_subscriber::fmt()
                    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                    .try_init()
                    .ok();
            }
            let temp_dir = TempDir::new().unwrap();
            let temp_path = temp_dir.path().join("repo");
            let repo = Repository::init(&temp_path).unwrap();
            println!("Created repository here: {}", &temp_path.canonicalize().unwrap().display());
            TestRepo { temp_dir, repo }
        }

        fn path(&self) -> PathBuf {
            self.temp_dir.path().join("repo")
        }

        fn commit(&self, message: &str) -> Result<Oid, RepositoryError> {
            let sig = Signature::now("Test", "test@example.com").unwrap();
            let tree_id = {
                let mut index = self
                    .repo
                    .index()
                    .map_err(|_| RepositoryError::InvalidRepositoryPath(self.path().to_path_buf()))?;
                index
                    .write_tree()
                    .map_err(|_| RepositoryError::InvalidRepositoryPath(self.path().to_path_buf()))?
            };
            let tree = self
                .repo
                .find_tree(tree_id)
                .map_err(|_| RepositoryError::InvalidRepositoryPath(self.path().to_path_buf()))?;
            let parent_commit = self
                .repo
                .head()
                .ok()
                .and_then(|h| h.target())
                .and_then(|t| self.repo.find_commit(t).ok());
            let parents = parent_commit.as_ref().map(|p| vec![p]).unwrap_or_default();
            self.repo
                .commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
                .map_err(|_| RepositoryError::InvalidRepositoryPath(self.path().to_path_buf()))
        }

        fn add_file(&self, path: impl AsRef<Path>, content: impl AsRef<str>) -> Result<(), RepositoryError> {
            use std::fs::File;
            use std::io::Write;
            let file_path = self.path().join(path.as_ref());
            let mut file = File::create(&file_path).unwrap();
            file.write_all(content.as_ref().as_bytes()).unwrap();

            let mut index = self.repo.index().unwrap();
            index.add_path(path.as_ref()).unwrap();
            index.write_tree().unwrap();

            Ok(())
        }

        #[allow(dead_code)]
        fn branch(&self, name: impl AsRef<str>) -> Result<(), RepositoryError> {
            let branch_name = name.as_ref();
            println!("Switching to branch '{}'", branch_name); // Debug message
            match self.repo.find_branch(branch_name, git2::BranchType::Local) {
                Ok(_) => {
                    // Branch exists, so check it out
                    let obj = self.repo.revparse_single(&format!("refs/heads/{}", branch_name)).unwrap();
                    let mut checkout_builder = git2::build::CheckoutBuilder::new();
                    checkout_builder.force();
                    match self.repo.checkout_tree(&obj, None) {
                        Ok(_) => {
                            self.repo.set_head(&format!("refs/heads/{}", branch_name)).unwrap();
                            println!("Switched to existing branch '{}'", branch_name);
                        }
                        Err(why) => {
                            println!("Failed to checkout tree: {why}");
                            checkout_builder.remove_untracked(true);
                            match self.repo.checkout_tree(&obj, Some(&mut checkout_builder)) {
                                Ok(_) => {
                                    self.repo.set_head(&format!("refs/heads/{}", branch_name)).unwrap();
                                    println!("Switched to existing branch '{}'", branch_name);
                                }
                                Err(why) => {
                                    println!("Failed to checkout tree: {why}");
                                    self.repo.set_head(&format!("refs/heads/{}", branch_name)).unwrap();
                                    println!("Switched to existing branch '{}'", branch_name);
                                }
                            }
                        }
                    }
                    self.repo.set_head(&format!("refs/heads/{}", branch_name)).unwrap();
                    println!("Switched to existing branch '{}'", branch_name);
                }
                Err(_) => {
                    // Branch does not exist, so create it
                    let head_ref = self.repo.head().unwrap();
                    let head_commit = head_ref.peel_to_commit().unwrap();
                    let _branch = self.repo.branch(branch_name, &head_commit, false).unwrap();

                    // Checkout the newly created branch
                    let obj = self.repo.revparse_single(&format!("refs/heads/{}", branch_name)).unwrap();
                    let mut checkout_builder = git2::build::CheckoutBuilder::new();
                    checkout_builder.force(); // Force checkout to overcome conflicts
                    match self.repo.checkout_tree(&obj, Some(&mut checkout_builder)) {
                        Ok(_) => println!("Created and switched to new branch '{}'", branch_name),
                        Err(e) => {
                            println!("Failed to checkout tree: {e}");
                            checkout_builder.remove_untracked(true);
                            self.repo.set_head(&format!("refs/heads/{}", branch_name)).unwrap();
                            println!("Switched to existing branch '{}'", branch_name);
                        }
                    }
                    self.repo.set_head(&format!("refs/heads/{}", branch_name)).unwrap();
                }
            }

            Ok(())
        }
    }

    #[test]
    fn case_empty() {
        let test_repo = TestRepo::new();
        let result = revwalk(&test_repo.repo, test_repo.path());
        assert!(result.is_err(), "Expected an error for an empty repository");
    }

    #[test]
    fn case_single_commit() {
        let test_repo = TestRepo::new();
        test_repo.commit("Initial commit").expect("Failed to commit");
        let result: Vec<Oid> = revwalk(&test_repo.repo, test_repo.path())
            .expect("Could not revwalk")
            .into_iter()
            .collect();
        assert_eq!(result.len(), 0, "Expected no commits in the revwalk");
    }

    #[test]
    fn case_multiple_commits() {
        let test_repo = TestRepo::new();
        test_repo.commit("Initial commit").expect("Failed to commit");
        let result: Vec<Oid> = revwalk(&test_repo.repo, test_repo.path())
            .expect("Could not revwalk")
            .into_iter()
            .collect();
        assert_eq!(result.len(), 0, "Expected no commits in the revwalk");
        test_repo.commit("Another commit").expect("Failed to commit");
        let result: Vec<Oid> = revwalk(&test_repo.repo, test_repo.path())
            .expect("Could not revwalk")
            .into_iter()
            .collect();
        assert_eq!(result.len(), 0, "Expected no commits in the revwalk");
    }

    #[test]
    fn case_iter_interface() {
        let test_repo = TestRepo::new();
        test_repo.commit("Initial commit").expect("Failed to commit");
        test_repo.commit("Another commit").expect("Failed to commit");
        let walker = revwalk(&test_repo.repo, test_repo.path())
            .expect("Could not revwalk")
            .into_iter();
        let mut counter = 0;
        for _oid in walker {
            counter += 1;
        }
        assert_eq!(counter, 0, "Expected no commits in the revwalk, because no files");
    }

    #[test]
    fn test_get_files_changed_empty() {
        let test_repo = TestRepo::new();
        let commit_oid = test_repo.commit("Initial commit").expect("Failed to commit");
        let result = get_files_changed(&test_repo.repo, commit_oid).expect("Failed to get files changed");
        assert_eq!(result.len(), 0, "Expected no files changed in the initial commit");
    }

    #[test]
    fn test_get_files_changed_single_file() {
        let test_repo = TestRepo::new();
        test_repo.add_file("test.txt", "Hello, world!").expect("Failed to add file");
        let commit_oid = test_repo.commit("Add test.txt").expect("Failed to commit");
        let result = get_files_changed(&test_repo.repo, commit_oid).expect("Failed to get files changed");
        assert_eq!(result.len(), 1, "Expected one file changed");
        assert_eq!(result[0].to_str().unwrap(), "test.txt", "Expected test.txt to be changed");
    }

    #[test]
    fn test_get_files_changed_multiple_files() {
        let test_repo = TestRepo::new();
        test_repo.add_file("test1.txt", "Hello, world!").expect("Failed to add file");
        test_repo.add_file("test2.txt", "Hello, world!").expect("Failed to add file");
        let commit_oid = test_repo.commit("Add test1.txt and test2.txt").expect("Failed to commit");
        let result = get_files_changed(&test_repo.repo, commit_oid).expect("Failed to get files changed");
        assert_eq!(result.len(), 2, "Expected two files changed");
        let filenames: Vec<_> = result.iter().map(|path| path.to_str().unwrap()).collect();
        assert!(filenames.contains(&"test1.txt"), "Expected test1.txt to be changed");
        assert!(filenames.contains(&"test2.txt"), "Expected test2.txt to be changed");
    }

    #[test]
    fn test_get_commit_log() {
        let test_repo = TestRepo::new();
        test_repo.add_file("file1.txt", "Hello, world!").expect("Failed to add file");
        test_repo.commit("Add file1.txt").unwrap();
        test_repo.add_file("file2.txt", "Hello, again!").expect("Failed to add file");
        test_repo.commit("Add file2.txt").unwrap();
        let path = test_repo.path();

        let commit_log = revwalk_commit_log(&test_repo.repo, path).unwrap();
        let commits: Vec<_> = commit_log.into_iter().collect();

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].commit.message(), "Add file2.txt");
        assert_eq!(commits[1].commit.message(), "Add file1.txt");
    }

    #[rstest]
    #[case::empty_empty("", "", "")]
    #[case::empty_root("", "/root", "/root")]
    #[case::root_empty("/root", "", "")]
    #[case::initial_overlap_extra_project("/root/path", "/root/path/to/file", "to/file")]
    #[case::initial_overlap_extra_root("/root/path/to/file", "/root/path", "../../")]
    #[case::cargo_toml("/root/path", "/root/path/Cargo.toml", "Cargo.toml")]
    fn test_relative_path(#[case] repo_path: &str, #[case] project_path: &str, #[case] expected: &str) {
        let repo_path = Path::new(repo_path);
        let project_path = Path::new(project_path);
        let expected = Path::new(expected);
        let result = compute_relative_path(repo_path, project_path);
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case::empty_rs_patch(TestRepoLanguage::Rust, TestRepoType::Empty, crate::BumpRule::Patch, "")]
    #[case::empty_py_minor(TestRepoLanguage::Python, TestRepoType::Empty, crate::BumpRule::Minor, "")]
    #[case::empty_js_major(TestRepoLanguage::JavaScript, TestRepoType::Empty, crate::BumpRule::Major, "")]
    #[case::single_rs_patch(TestRepoLanguage::Rust, TestRepoType::SingleCommit, crate::BumpRule::Patch, "0.1.0")]
    #[case::single_py_minor(TestRepoLanguage::Python, TestRepoType::SingleCommit, crate::BumpRule::Minor, "0.1.0")]
    #[case::single_js_major(TestRepoLanguage::JavaScript, TestRepoType::SingleCommit, crate::BumpRule::Major, "0.1.0")]
    #[case::multi_rs_patch(TestRepoLanguage::Rust, TestRepoType::MultipleCommits, crate::BumpRule::Patch, "0.1.1")]
    #[case::multi_py_minor(TestRepoLanguage::Python, TestRepoType::MultipleCommits, crate::BumpRule::Minor, "0.2.0")]
    #[case::multi_js_major(TestRepoLanguage::JavaScript, TestRepoType::MultipleCommits, crate::BumpRule::Major, "1.0.0")]
    #[case::branching_rs_patch(TestRepoLanguage::Rust, TestRepoType::BranchedMultipleCommits, crate::BumpRule::Patch, "0.1.1")]
    #[case::branching_py_minor(TestRepoLanguage::Python, TestRepoType::BranchedMultipleCommits, crate::BumpRule::Minor, "0.2.0")]
    #[case::branching_js_major(TestRepoLanguage::JavaScript, TestRepoType::BranchedMultipleCommits, crate::BumpRule::Major, "1.0.0")]

    fn test_version_bumping(#[case] language: TestRepoLanguage, #[case] type_: TestRepoType, #[case] bump_rule: crate::BumpRule, #[case] expected_version: impl AsRef<str>) {
        let test_repo = TestRepo::new();
        let path = test_repo.path().canonicalize().expect("Could not get full path");
        let rules = vec![(crate::CommitType::Chore, crate::BumpRule::Patch), (crate::CommitType::Fix, bump_rule)];
        let project = TestProject::new(path.clone(), &test_repo, language, type_);
        project.build().expect("Failed to init project");
        match type_ {
            TestRepoType::Empty => {
                if let Ok(_changelog) = project.generate_changelog(&rules) {
                    assert!(false, "changelog should fail with empty repo");
                }
            }
            TestRepoType::SingleCommit => {
                let log = project.generate_log_messages(&rules).expect("Could not build repo");
                let log_messages = project.generate_pretty_log_messages(&rules).expect("Could not build repo");
                assert_eq!(log.len(), 1, "{}", "{log_messages}");
                assert_eq!(log.last().expect("Could not find log"), TestProject::FIRST_COMMIT_MESSAGE, "Commit does not match: {log_messages}");
                assert_eq!(project.generate_next_version(&rules).expect("Could not build repo"), expected_version.as_ref(), "{log_messages}");
            }
            TestRepoType::MultipleCommits | TestRepoType::BranchedMultipleCommits => {
                let log = project.generate_log_messages(&rules).expect("Could not build repo");
                let log_messages = project.generate_pretty_log_messages(&rules).expect("Could not build repo");
                assert_eq!(log.len(), 4, "{}", "{log_messages}");
                assert_eq!(log.first().expect("Could not find log"), "fix: use add", "Recent commit does not match: {log_messages}");
                assert_eq!(
                    log.last().expect("Could not find log"),
                    TestProject::FIRST_COMMIT_MESSAGE,
                    "Oldest commit does not match: {log_messages}"
                );
                assert_eq!(project.generate_next_version(&rules).expect("Could not build repo"), expected_version.as_ref(), "{log_messages}");
            }
        }
    }

    // ============================================================================
    // COMPREHENSIVE ALGORITHM TESTS - SYSTEMATIC MATRIX WITH RSTEST
    // ============================================================================

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[allow(dead_code)]
    pub enum VersionHistory {
        NoVersions,
        PatchOnly,
        PatchPlusOneMinor,
        PatchPlusMultiMinors,
        PatchMultiMinorsPlusOneMajor,
        PatchMultiMinorsMultiMajors,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[allow(dead_code)]
    pub enum CommitPattern {
        OneRev,
        OneMin,
        OneMaj,
        MultipleRev,
        MultipleMin,
        MultipleMaj,
        MultiRevOneMin,
        MultiRevMultiMin,
        MultiRevMultiMinOneMaj,
        MultiRevMultiMinMultiMaj,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CommitPosition {
        Front,
        Back,
        Middle,
        Distributed, // For cases where position doesn't matter
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[allow(dead_code)]
    pub enum CurrentVersion {
        NoVersion,    // 0.0.0 - no previous releases
        PatchVersion, // 0.0.1 - at a patch version
        MinorVersion, // 0.1.0 - at a minor version
        MajorVersion, // 1.0.0 - at a major version
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[allow(dead_code)]
    pub enum VersionPosition {
        NoVersions,  // No version boundaries in history
        Front,       // Version boundaries at front of history
        Back,        // Version boundaries at back of history
        Middle,      // Version boundaries in middle of history
        Distributed, // Version boundaries distributed throughout
    }

    /// Helper function to get current version based on CurrentVersion enum
    fn get_current_version(current_version: CurrentVersion) -> SimpleVersion {
        match current_version {
            CurrentVersion::NoVersion => SimpleVersion::new(0, 0, 0),
            CurrentVersion::PatchVersion => SimpleVersion::new(0, 0, 1),
            CurrentVersion::MinorVersion => SimpleVersion::new(0, 1, 0),
            CurrentVersion::MajorVersion => SimpleVersion::new(1, 0, 0),
        }
    }

    /// Helper function to create mock commits with version boundaries
    fn create_version_history(history: VersionHistory, _version_position: VersionPosition) -> Vec<CommitWithVersion> {
        match history {
            VersionHistory::NoVersions => vec![],
            VersionHistory::PatchOnly => vec![
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: patch 1").unwrap(), 100),
                    version_at_commit: Some(SimpleVersion::new(0, 0, 1)),
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: patch 2").unwrap(), 200),
                    version_at_commit: Some(SimpleVersion::new(0, 0, 2)),
                },
            ],
            VersionHistory::PatchPlusOneMinor => vec![
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: patch 1").unwrap(), 100),
                    version_at_commit: Some(SimpleVersion::new(0, 0, 1)),
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: minor 1").unwrap(), 200),
                    version_at_commit: Some(SimpleVersion::new(0, 1, 0)),
                },
            ],
            VersionHistory::PatchPlusMultiMinors => vec![
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: patch 1").unwrap(), 100),
                    version_at_commit: Some(SimpleVersion::new(0, 0, 1)),
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: minor 1").unwrap(), 200),
                    version_at_commit: Some(SimpleVersion::new(0, 1, 0)),
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary3".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: minor 2").unwrap(), 300),
                    version_at_commit: Some(SimpleVersion::new(0, 2, 0)),
                },
            ],
            VersionHistory::PatchMultiMinorsPlusOneMajor => vec![
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: minor 1").unwrap(), 100),
                    version_at_commit: Some(SimpleVersion::new(0, 1, 0)),
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: minor 2").unwrap(), 200),
                    version_at_commit: Some(SimpleVersion::new(0, 2, 0)),
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary3".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat!: major 1").unwrap(), 300),
                    version_at_commit: Some(SimpleVersion::new(1, 0, 0)),
                },
            ],
            VersionHistory::PatchMultiMinorsMultiMajors => vec![
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: minor 1").unwrap(), 100),
                    version_at_commit: Some(SimpleVersion::new(0, 1, 0)),
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat!: major 1").unwrap(), 200),
                    version_at_commit: Some(SimpleVersion::new(1, 0, 0)),
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("boundary3".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat!: major 2").unwrap(), 300),
                    version_at_commit: Some(SimpleVersion::new(2, 0, 0)),
                },
            ],
        }
    }

    /// Helper function to create commit patterns
    fn create_commit_pattern(pattern: CommitPattern, position: CommitPosition) -> Vec<CommitWithVersion> {
        match pattern {
            CommitPattern::OneRev => vec![CommitWithVersion {
                commit_info: CommitInfo::new("commit1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: single fix").unwrap(), 1000),
                version_at_commit: None,
            }],
            CommitPattern::OneMin => vec![CommitWithVersion {
                commit_info: CommitInfo::new("commit1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: single feature").unwrap(), 1000),
                version_at_commit: None,
            }],
            CommitPattern::OneMaj => vec![CommitWithVersion {
                commit_info: CommitInfo::new("commit1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat!: breaking change").unwrap(), 1000),
                version_at_commit: None,
            }],
            CommitPattern::MultipleRev => vec![
                CommitWithVersion {
                    commit_info: CommitInfo::new("commit1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: fix 1").unwrap(), 1000),
                    version_at_commit: None,
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("commit2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: fix 2").unwrap(), 1100),
                    version_at_commit: None,
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("commit3".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: fix 3").unwrap(), 1200),
                    version_at_commit: None,
                },
            ],
            CommitPattern::MultipleMin => vec![
                CommitWithVersion {
                    commit_info: CommitInfo::new("commit1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: feature 1").unwrap(), 1000),
                    version_at_commit: None,
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("commit2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: feature 2").unwrap(), 1100),
                    version_at_commit: None,
                },
            ],
            CommitPattern::MultipleMaj => vec![
                CommitWithVersion {
                    commit_info: CommitInfo::new("commit1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat!: breaking 1").unwrap(), 1000),
                    version_at_commit: None,
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("commit2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat!: breaking 2").unwrap(), 1100),
                    version_at_commit: None,
                },
            ],
            CommitPattern::MultiRevOneMin => {
                let mut commits = vec![
                    CommitWithVersion {
                        commit_info: CommitInfo::new("commit1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: fix 1").unwrap(), 1000),
                        version_at_commit: None,
                    },
                    CommitWithVersion {
                        commit_info: CommitInfo::new("commit2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: fix 2").unwrap(), 1100),
                        version_at_commit: None,
                    },
                ];

                let feat_commit = CommitWithVersion {
                    commit_info: CommitInfo::new("feat_commit".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: new feature").unwrap(), 1200),
                    version_at_commit: None,
                };

                match position {
                    CommitPosition::Front => {
                        commits.insert(0, feat_commit);
                        commits
                    }
                    CommitPosition::Back => {
                        commits.push(feat_commit);
                        commits
                    }
                    CommitPosition::Middle => {
                        commits.insert(1, feat_commit);
                        commits
                    }
                    CommitPosition::Distributed => {
                        commits.push(feat_commit);
                        commits
                    }
                }
            }
            CommitPattern::MultiRevMultiMin => {
                let mut commits = vec![
                    CommitWithVersion {
                        commit_info: CommitInfo::new("commit1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: fix 1").unwrap(), 1000),
                        version_at_commit: None,
                    },
                    CommitWithVersion {
                        commit_info: CommitInfo::new("commit2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: fix 2").unwrap(), 1100),
                        version_at_commit: None,
                    },
                ];

                let feat_commits = vec![
                    CommitWithVersion {
                        commit_info: CommitInfo::new("feat1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: feature 1").unwrap(), 1200),
                        version_at_commit: None,
                    },
                    CommitWithVersion {
                        commit_info: CommitInfo::new("feat2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: feature 2").unwrap(), 1300),
                        version_at_commit: None,
                    },
                ];

                match position {
                    CommitPosition::Front => {
                        let mut result = feat_commits;
                        result.extend(commits);
                        result
                    }
                    CommitPosition::Back => {
                        commits.extend(feat_commits);
                        commits
                    }
                    CommitPosition::Middle => {
                        commits.insert(1, feat_commits[0].clone());
                        commits.push(feat_commits[1].clone());
                        commits
                    }
                    CommitPosition::Distributed => {
                        commits.extend(feat_commits);
                        commits
                    }
                }
            }
            CommitPattern::MultiRevMultiMinOneMaj => {
                let mut commits = vec![
                    CommitWithVersion {
                        commit_info: CommitInfo::new("commit1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: fix 1").unwrap(), 1000),
                        version_at_commit: None,
                    },
                    CommitWithVersion {
                        commit_info: CommitInfo::new("commit2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: fix 2").unwrap(), 1100),
                        version_at_commit: None,
                    },
                    CommitWithVersion {
                        commit_info: CommitInfo::new("feat1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: feature 1").unwrap(), 1200),
                        version_at_commit: None,
                    },
                    CommitWithVersion {
                        commit_info: CommitInfo::new("feat2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: feature 2").unwrap(), 1300),
                        version_at_commit: None,
                    },
                ];

                let major_commit = CommitWithVersion {
                    commit_info: CommitInfo::new("major".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat!: breaking change").unwrap(), 1400),
                    version_at_commit: None,
                };

                match position {
                    CommitPosition::Front => {
                        commits.insert(0, major_commit);
                        commits
                    }
                    CommitPosition::Back => {
                        commits.push(major_commit);
                        commits
                    }
                    CommitPosition::Middle => {
                        commits.insert(2, major_commit);
                        commits
                    }
                    CommitPosition::Distributed => {
                        commits.push(major_commit);
                        commits
                    }
                }
            }
            CommitPattern::MultiRevMultiMinMultiMaj => vec![
                CommitWithVersion {
                    commit_info: CommitInfo::new("commit1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: fix 1").unwrap(), 1000),
                    version_at_commit: None,
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("commit2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("fix: fix 2").unwrap(), 1100),
                    version_at_commit: None,
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("feat1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: feature 1").unwrap(), 1200),
                    version_at_commit: None,
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("feat2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat: feature 2").unwrap(), 1300),
                    version_at_commit: None,
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("major1".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat!: breaking 1").unwrap(), 1400),
                    version_at_commit: None,
                },
                CommitWithVersion {
                    commit_info: CommitInfo::new("major2".to_string(), vec![] as Vec<PathBuf>, ConventionalCommit::new("feat!: breaking 2").unwrap(), 1500),
                    version_at_commit: None,
                },
            ],
        }
    }

    /// Main parameterized test using rstest with all 6 dimensions including expected bump
    #[rstest]
    // ============================================================================
    // NO VERSIONS SCENARIOS - Testing with clean slate (no version boundaries)
    // ============================================================================
    #[case::no_versions_one_rev_expect_patch(
        VersionHistory::NoVersions,
        CommitPattern::OneRev,
        CommitPosition::Distributed,
        CurrentVersion::NoVersion,
        VersionPosition::NoVersions,
        BumpRule::Patch
    )]
    #[case::no_versions_one_min_expect_minor(
        VersionHistory::NoVersions,
        CommitPattern::OneMin,
        CommitPosition::Distributed,
        CurrentVersion::PatchVersion,
        VersionPosition::NoVersions,
        BumpRule::Minor
    )]
    #[case::no_versions_one_maj_expect_major(
        VersionHistory::NoVersions,
        CommitPattern::OneMaj,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::NoVersions,
        BumpRule::Major
    )]
    #[case::no_versions_multiple_rev_expect_patch(
        VersionHistory::NoVersions,
        CommitPattern::MultipleRev,
        CommitPosition::Distributed,
        CurrentVersion::NoVersion,
        VersionPosition::NoVersions,
        BumpRule::Patch
    )]
    #[case::no_versions_multiple_min_expect_minor(
        VersionHistory::NoVersions,
        CommitPattern::MultipleMin,
        CommitPosition::Distributed,
        CurrentVersion::PatchVersion,
        VersionPosition::NoVersions,
        BumpRule::Minor
    )]
    #[case::no_versions_multiple_maj_expect_major(
        VersionHistory::NoVersions,
        CommitPattern::MultipleMaj,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::NoVersions,
        BumpRule::Major
    )]
    // ============================================================================
    // PATCH ONLY SCENARIOS - Testing with patch version boundaries (0.0.1, 0.0.2)
    // ============================================================================
    #[case::patch_only_one_rev_expect_patch(
        VersionHistory::PatchOnly,
        CommitPattern::OneRev,
        CommitPosition::Distributed,
        CurrentVersion::PatchVersion,
        VersionPosition::Distributed,
        BumpRule::Patch
    )]
    #[case::patch_only_one_min_expect_minor(
        VersionHistory::PatchOnly,
        CommitPattern::OneMin,
        CommitPosition::Distributed,
        CurrentVersion::PatchVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_only_one_maj_expect_major(
        VersionHistory::PatchOnly,
        CommitPattern::OneMaj,
        CommitPosition::Distributed,
        CurrentVersion::PatchVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    #[case::patch_only_multiple_rev_expect_patch(
        VersionHistory::PatchOnly,
        CommitPattern::MultipleRev,
        CommitPosition::Distributed,
        CurrentVersion::PatchVersion,
        VersionPosition::Distributed,
        BumpRule::Patch
    )]
    #[case::patch_only_multiple_min_expect_minor(
        VersionHistory::PatchOnly,
        CommitPattern::MultipleMin,
        CommitPosition::Distributed,
        CurrentVersion::PatchVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_only_multiple_maj_expect_major(
        VersionHistory::PatchOnly,
        CommitPattern::MultipleMaj,
        CommitPosition::Distributed,
        CurrentVersion::PatchVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    // Position variations for patch-only scenarios
    #[case::patch_only_multi_rev_one_min_front_expect_minor(
        VersionHistory::PatchOnly,
        CommitPattern::MultiRevOneMin,
        CommitPosition::Front,
        CurrentVersion::MajorVersion,
        VersionPosition::Front,
        BumpRule::Minor
    )]
    #[case::patch_only_multi_rev_one_min_back_expect_minor(
        VersionHistory::PatchOnly,
        CommitPattern::MultiRevOneMin,
        CommitPosition::Back,
        CurrentVersion::PatchVersion,
        VersionPosition::Back,
        BumpRule::Minor
    )]
    #[case::patch_only_multi_rev_one_min_middle_expect_minor(
        VersionHistory::PatchOnly,
        CommitPattern::MultiRevOneMin,
        CommitPosition::Middle,
        CurrentVersion::MinorVersion,
        VersionPosition::Middle,
        BumpRule::Minor
    )]
    // ============================================================================
    // PATCH + ONE MINOR SCENARIOS - Testing with patch + one minor boundary
    // ============================================================================
    #[case::patch_plus_one_minor_one_rev_expect_patch(
        VersionHistory::PatchPlusOneMinor,
        CommitPattern::OneRev,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Patch
    )]
    #[case::patch_plus_one_minor_one_min_expect_minor(
        VersionHistory::PatchPlusOneMinor,
        CommitPattern::OneMin,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_plus_one_minor_one_maj_expect_major(
        VersionHistory::PatchPlusOneMinor,
        CommitPattern::OneMaj,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    #[case::patch_plus_one_minor_multiple_rev_expect_patch(
        VersionHistory::PatchPlusOneMinor,
        CommitPattern::MultipleRev,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Patch
    )]
    #[case::patch_plus_one_minor_multiple_min_expect_minor(
        VersionHistory::PatchPlusOneMinor,
        CommitPattern::MultipleMin,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_plus_one_minor_multiple_maj_expect_major(
        VersionHistory::PatchPlusOneMinor,
        CommitPattern::MultipleMaj,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    // ============================================================================
    // PATCH + MULTIPLE MINORS SCENARIOS - Testing with multiple minor boundaries
    // ============================================================================
    #[case::patch_plus_multi_minors_one_rev_expect_patch(
        VersionHistory::PatchPlusMultiMinors,
        CommitPattern::OneRev,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Patch
    )]
    #[case::patch_plus_multi_minors_one_min_expect_minor(
        VersionHistory::PatchPlusMultiMinors,
        CommitPattern::OneMin,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_plus_multi_minors_one_maj_expect_major(
        VersionHistory::PatchPlusMultiMinors,
        CommitPattern::OneMaj,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    #[case::patch_plus_multi_minors_multiple_rev_expect_patch(
        VersionHistory::PatchPlusMultiMinors,
        CommitPattern::MultipleRev,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Patch
    )]
    #[case::patch_plus_multi_minors_multiple_min_expect_minor(
        VersionHistory::PatchPlusMultiMinors,
        CommitPattern::MultipleMin,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_plus_multi_minors_multiple_maj_expect_major(
        VersionHistory::PatchPlusMultiMinors,
        CommitPattern::MultipleMaj,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    #[case::patch_plus_multi_minors_multi_rev_multi_min_multi_maj_expect_major(
        VersionHistory::PatchPlusMultiMinors,
        CommitPattern::MultiRevMultiMinMultiMaj,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    // ============================================================================
    // PATCH + MULTIPLE MINORS + ONE MAJOR SCENARIOS
    // ============================================================================
    #[case::patch_multi_minors_plus_one_major_one_rev_expect_patch(
        VersionHistory::PatchMultiMinorsPlusOneMajor,
        CommitPattern::OneRev,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Patch
    )]
    #[case::patch_multi_minors_plus_one_major_one_min_expect_minor(
        VersionHistory::PatchMultiMinorsPlusOneMajor,
        CommitPattern::OneMin,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_multi_minors_plus_one_major_one_maj_expect_major(
        VersionHistory::PatchMultiMinorsPlusOneMajor,
        CommitPattern::OneMaj,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    #[case::patch_multi_minors_plus_one_major_multiple_rev_expect_patch(
        VersionHistory::PatchMultiMinorsPlusOneMajor,
        CommitPattern::MultipleRev,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Patch
    )]
    #[case::patch_multi_minors_plus_one_major_multiple_min_expect_minor(
        VersionHistory::PatchMultiMinorsPlusOneMajor,
        CommitPattern::MultipleMin,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_multi_minors_plus_one_major_multiple_maj_expect_major(
        VersionHistory::PatchMultiMinorsPlusOneMajor,
        CommitPattern::MultipleMaj,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    // ============================================================================
    // PATCH + MULTIPLE MINORS + MULTIPLE MAJORS SCENARIOS - Full complexity
    // ============================================================================
    #[case::patch_multi_minors_multi_majors_one_rev_expect_patch(
        VersionHistory::PatchMultiMinorsMultiMajors,
        CommitPattern::OneRev,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Patch
    )]
    #[case::patch_multi_minors_multi_majors_one_min_expect_minor(
        VersionHistory::PatchMultiMinorsMultiMajors,
        CommitPattern::OneMin,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_multi_minors_multi_majors_one_maj_expect_major(
        VersionHistory::PatchMultiMinorsMultiMajors,
        CommitPattern::OneMaj,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    #[case::patch_multi_minors_multi_majors_multiple_rev_expect_patch(
        VersionHistory::PatchMultiMinorsMultiMajors,
        CommitPattern::MultipleRev,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Patch
    )]
    #[case::patch_multi_minors_multi_majors_multiple_min_expect_minor(
        VersionHistory::PatchMultiMinorsMultiMajors,
        CommitPattern::MultipleMin,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_multi_minors_multi_majors_multiple_maj_expect_major(
        VersionHistory::PatchMultiMinorsMultiMajors,
        CommitPattern::MultipleMaj,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    // ============================================================================
    // MIXED COMMIT SCENARIOS - Testing complex commit combinations
    // ============================================================================
    #[case::no_versions_multi_rev_multi_min_expect_minor(
        VersionHistory::NoVersions,
        CommitPattern::MultiRevMultiMin,
        CommitPosition::Distributed,
        CurrentVersion::NoVersion,
        VersionPosition::NoVersions,
        BumpRule::Minor
    )]
    #[case::no_versions_multi_rev_multi_min_one_maj_expect_major(
        VersionHistory::NoVersions,
        CommitPattern::MultiRevMultiMinOneMaj,
        CommitPosition::Distributed,
        CurrentVersion::NoVersion,
        VersionPosition::NoVersions,
        BumpRule::Major
    )]
    #[case::no_versions_multi_rev_multi_min_multi_maj_expect_major(
        VersionHistory::NoVersions,
        CommitPattern::MultiRevMultiMinMultiMaj,
        CommitPosition::Distributed,
        CurrentVersion::NoVersion,
        VersionPosition::NoVersions,
        BumpRule::Major
    )]
    #[case::patch_only_multi_rev_multi_min_expect_minor(
        VersionHistory::PatchOnly,
        CommitPattern::MultiRevMultiMin,
        CommitPosition::Distributed,
        CurrentVersion::PatchVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_only_multi_rev_multi_min_one_maj_expect_major(
        VersionHistory::PatchOnly,
        CommitPattern::MultiRevMultiMinOneMaj,
        CommitPosition::Distributed,
        CurrentVersion::PatchVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    #[case::patch_only_multi_rev_multi_min_multi_maj_expect_major(
        VersionHistory::PatchOnly,
        CommitPattern::MultiRevMultiMinMultiMaj,
        CommitPosition::Distributed,
        CurrentVersion::PatchVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    // ============================================================================
    // POSITION VARIATION SCENARIOS - Testing commit order independence
    // ============================================================================
    #[case::patch_plus_multi_minors_multi_rev_one_min_front_expect_minor(
        VersionHistory::PatchPlusMultiMinors,
        CommitPattern::MultiRevOneMin,
        CommitPosition::Front,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_plus_multi_minors_multi_rev_one_min_back_expect_minor(
        VersionHistory::PatchPlusMultiMinors,
        CommitPattern::MultiRevOneMin,
        CommitPosition::Back,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_plus_multi_minors_multi_rev_one_min_middle_expect_minor(
        VersionHistory::PatchPlusMultiMinors,
        CommitPattern::MultiRevOneMin,
        CommitPosition::Middle,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_multi_minors_multi_majors_multi_rev_multi_min_one_maj_front_expect_major(
        VersionHistory::PatchMultiMinorsMultiMajors,
        CommitPattern::MultiRevMultiMinOneMaj,
        CommitPosition::Front,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    #[case::patch_multi_minors_multi_majors_multi_rev_multi_min_one_maj_back_expect_major(
        VersionHistory::PatchMultiMinorsMultiMajors,
        CommitPattern::MultiRevMultiMinOneMaj,
        CommitPosition::Back,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    #[case::patch_multi_minors_multi_majors_multi_rev_multi_min_one_maj_middle_expect_major(
        VersionHistory::PatchMultiMinorsMultiMajors,
        CommitPattern::MultiRevMultiMinOneMaj,
        CommitPosition::Middle,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Major
    )]
    // ============================================================================
    // CURRENT VERSION VARIATION SCENARIOS - Testing different starting versions
    // ============================================================================
    #[case::patch_only_one_min_from_no_version_expect_minor(
        VersionHistory::PatchOnly,
        CommitPattern::OneMin,
        CommitPosition::Distributed,
        CurrentVersion::NoVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_only_one_min_from_patch_version_expect_minor(
        VersionHistory::PatchOnly,
        CommitPattern::OneMin,
        CommitPosition::Distributed,
        CurrentVersion::PatchVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_only_one_min_from_minor_version_expect_minor(
        VersionHistory::PatchOnly,
        CommitPattern::OneMin,
        CommitPosition::Distributed,
        CurrentVersion::MinorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    #[case::patch_only_one_min_from_major_version_expect_minor(
        VersionHistory::PatchOnly,
        CommitPattern::OneMin,
        CommitPosition::Distributed,
        CurrentVersion::MajorVersion,
        VersionPosition::Distributed,
        BumpRule::Minor
    )]
    fn test_comprehensive_algorithm_matrix(
        #[case] version_history: VersionHistory,
        #[case] commit_pattern: CommitPattern,
        #[case] position: CommitPosition,
        #[case] current_version: CurrentVersion,
        #[case] version_position: VersionPosition,
        #[case] expected_bump: BumpRule,
    ) {
        let rules = vec![
            (CommitType::Fix, BumpRule::Patch),
            (CommitType::Feat, BumpRule::Minor),
            (CommitType::Custom("feat!".to_string()), BumpRule::Major),
        ];

        // Create version history (older commits with version boundaries)
        let mut all_commits = create_version_history(version_history, version_position);

        // Add new commits (newer commits without version boundaries)
        let new_commits = create_commit_pattern(commit_pattern, position);
        all_commits.extend(new_commits);

        // Test the algorithm with the specified current version
        let current_version = get_current_version(current_version);

        let result = collect_changelog_commits(all_commits, current_version, &rules);

        // Basic validation - ensure we get some commits back for most scenarios
        // Note: Some scenarios might legitimately return empty results
        if version_history == VersionHistory::NoVersions {
            // This is a valid scenario that might return empty results
            assert!(
                !result.is_empty(),
                "Should collect commits for pattern {:?} with history {:?}, current version {:?}, version position {:?}",
                commit_pattern,
                version_history,
                current_version,
                version_position
            );
        }

        // Verify the expected bump rule is calculated correctly
        let actual_bump = result
            .iter()
            .fold(BumpRule::default(), |max_bump, commit| max_bump.max(commit.rule(&rules)));

        assert_eq!(
            actual_bump, expected_bump,
            "Expected bump {:?} but got {:?} for pattern {:?} with history {:?}, current version {:?}, version position {:?}",
            expected_bump, actual_bump, commit_pattern, version_history, current_version, version_position
        );

        // Additional validations based on the test scenario
        match commit_pattern {
            CommitPattern::OneRev => {
                assert_eq!(actual_bump, BumpRule::Patch, "Single revision should result in patch bump");
            }
            CommitPattern::OneMin => {
                assert_eq!(actual_bump, BumpRule::Minor, "Single minor should result in minor bump");
            }
            CommitPattern::OneMaj => {
                assert_eq!(actual_bump, BumpRule::Major, "Single major should result in major bump");
            }
            CommitPattern::MultiRevOneMin => {
                assert_eq!(actual_bump, BumpRule::Minor, "Multiple revisions + one minor should result in minor bump");
            }
            CommitPattern::MultiRevMultiMinMultiMaj => {
                assert_eq!(actual_bump, BumpRule::Major, "Mixed commits with major should result in major bump");
            }
            _ => {
                // For other patterns, just verify the expected bump matches
                assert_eq!(actual_bump, expected_bump);
            }
        }
    }
}
