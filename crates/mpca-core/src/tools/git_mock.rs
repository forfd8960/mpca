//! Mock git adapter for testing.
//!
//! This module provides a mock implementation of the `GitAdapter` trait
//! for use in tests. The mock simulates git operations without requiring
//! a real git repository.

use crate::error::{MPCAError, Result};
use crate::tools::git::GitAdapter;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Mock git adapter for testing.
///
/// Simulates git operations using in-memory state. Tracks repositories,
/// worktrees, branches, and commits.
///
/// # Examples
///
/// ```
/// use mpca_core::tools::git_mock::MockGitAdapter;
/// use mpca_core::tools::git::GitAdapter;
/// use std::path::Path;
///
/// let git = MockGitAdapter::new();
/// git.init_repository(Path::new("/repo")).unwrap();
/// assert!(git.is_git_repo(Path::new("/repo")));
/// ```
#[derive(Debug, Clone, Default)]
pub struct MockGitAdapter {
    /// Set of paths that are git repositories
    repos: Arc<Mutex<HashSet<PathBuf>>>,
    /// Map of worktree paths to their branch names
    worktrees: Arc<Mutex<HashMap<PathBuf, String>>>,
    /// Set of branch names
    branches: Arc<Mutex<HashSet<String>>>,
    /// Whether the repo is "clean" (no uncommitted changes)
    clean: Arc<Mutex<bool>>,
}

impl MockGitAdapter {
    /// Creates a new mock git adapter.
    ///
    /// # Returns
    ///
    /// A new `MockGitAdapter` with no repositories.
    pub fn new() -> Self {
        Self {
            repos: Arc::new(Mutex::new(HashSet::new())),
            worktrees: Arc::new(Mutex::new(HashMap::new())),
            branches: Arc::new(Mutex::new(HashSet::new())),
            clean: Arc::new(Mutex::new(true)),
        }
    }

    /// Creates a mock with a repository pre-initialized.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to the repository
    ///
    /// # Returns
    ///
    /// A `MockGitAdapter` with one repository.
    pub fn with_repo(repo_path: PathBuf) -> Self {
        let adapter = Self::new();
        adapter.repos.lock().unwrap().insert(repo_path);
        adapter.branches.lock().unwrap().insert("main".to_string());
        adapter
    }

    /// Initializes a mock repository at the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the repository should be initialized
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    pub fn init_repository(&self, path: &Path) -> Result<()> {
        self.repos.lock().unwrap().insert(path.to_path_buf());
        self.branches.lock().unwrap().insert("main".to_string());
        Ok(())
    }

    /// Sets whether the repository has uncommitted changes.
    ///
    /// # Arguments
    ///
    /// * `clean` - `true` if repo is clean, `false` if it has uncommitted changes
    pub fn set_clean(&self, clean: bool) {
        *self.clean.lock().unwrap() = clean;
    }

    /// Returns all worktrees created by this mock.
    ///
    /// # Returns
    ///
    /// HashMap of worktree paths to branch names.
    pub fn get_worktrees(&self) -> HashMap<PathBuf, String> {
        self.worktrees.lock().unwrap().clone()
    }

    /// Returns all branches in the mock repository.
    ///
    /// # Returns
    ///
    /// Set of branch names.
    pub fn get_branches(&self) -> HashSet<String> {
        self.branches.lock().unwrap().clone()
    }

    /// Clears all state from the mock.
    pub fn clear(&self) {
        self.repos.lock().unwrap().clear();
        self.worktrees.lock().unwrap().clear();
        self.branches.lock().unwrap().clear();
        *self.clean.lock().unwrap() = true;
    }
}

impl GitAdapter for MockGitAdapter {
    fn is_git_repo(&self, path: &Path) -> bool {
        self.repos.lock().unwrap().contains(path)
    }

    fn get_repo_root(&self, path: &Path) -> Result<String> {
        // For mock, return the path itself if it's a repo
        if self.is_git_repo(path) {
            Ok(path.to_string_lossy().to_string())
        } else {
            Err(MPCAError::NotGitRepository(path.to_path_buf()))
        }
    }

    fn create_worktree(&self, _repo: &Path, worktree_path: &Path, branch: &str) -> Result<()> {
        let mut worktrees = self.worktrees.lock().unwrap();
        let mut branches = self.branches.lock().unwrap();

        // Check if worktree already exists
        if worktrees.contains_key(worktree_path) {
            return Err(MPCAError::WorktreeExists(worktree_path.to_path_buf()));
        }

        // Check if branch already exists
        if branches.contains(branch) {
            return Err(MPCAError::BranchExists(branch.to_string()));
        }

        worktrees.insert(worktree_path.to_path_buf(), branch.to_string());
        branches.insert(branch.to_string());

        Ok(())
    }

    fn remove_worktree(&self, _repo: &Path, worktree_path: &Path) -> Result<()> {
        let mut worktrees = self.worktrees.lock().unwrap();

        if !worktrees.contains_key(worktree_path) {
            return Err(MPCAError::WorktreeNotFound(worktree_path.to_path_buf()));
        }

        worktrees.remove(worktree_path);
        Ok(())
    }

    fn commit(&self, _repo: &Path, _message: &str) -> Result<()> {
        // In mock, just mark as clean after commit
        *self.clean.lock().unwrap() = true;
        Ok(())
    }

    fn status(&self, _repo: &Path) -> Result<Vec<String>> {
        // Mock implementation returns empty list for clean repo
        if *self.clean.lock().unwrap() {
            Ok(Vec::new())
        } else {
            Ok(vec!["file.txt".to_string()])
        }
    }

    fn has_uncommitted_changes(&self, _repo: &Path) -> bool {
        !*self.clean.lock().unwrap()
    }

    fn diff(&self, _repo: &Path) -> Result<String> {
        // Mock implementation returns empty diff for clean repo
        if *self.clean.lock().unwrap() {
            Ok(String::new())
        } else {
            Ok("diff --git a/file.txt b/file.txt\n--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new".to_string())
        }
    }

    fn add(&self, _repo: &Path, _files: &[&str]) -> Result<()> {
        // Mock implementation: adding files doesn't change state
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_git_init() {
        let git = MockGitAdapter::new();
        let repo = Path::new("/repo");

        assert!(!git.is_git_repo(repo));
        git.init_repository(repo).unwrap();
        assert!(git.is_git_repo(repo));
    }

    #[test]
    fn test_mock_git_with_repo() {
        let repo = PathBuf::from("/repo");
        let git = MockGitAdapter::with_repo(repo.clone());

        assert!(git.is_git_repo(&repo));
        assert_eq!(git.get_repo_root(&repo).unwrap(), repo.to_string_lossy());
    }

    #[test]
    fn test_mock_git_create_worktree() {
        let repo = PathBuf::from("/repo");
        let git = MockGitAdapter::with_repo(repo.clone());

        let worktree = Path::new("/repo/.trees/feature");
        let branch = "feature/test";

        git.create_worktree(&repo, worktree, branch).unwrap();

        assert!(git.get_worktrees().contains_key(worktree));
        assert_eq!(git.get_worktrees().get(worktree).unwrap(), branch);
        assert!(git.get_branches().contains(branch));
    }

    #[test]
    fn test_mock_git_worktree_already_exists() {
        let repo = PathBuf::from("/repo");
        let git = MockGitAdapter::with_repo(repo.clone());

        let worktree = Path::new("/repo/.trees/feature");
        git.create_worktree(&repo, worktree, "feature/test")
            .unwrap();

        let result = git.create_worktree(&repo, worktree, "feature/other");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MPCAError::WorktreeExists(_)));
    }

    #[test]
    fn test_mock_git_branch_already_exists() {
        let repo = PathBuf::from("/repo");
        let git = MockGitAdapter::with_repo(repo.clone());

        git.create_worktree(&repo, Path::new("/trees/f1"), "feature/test")
            .unwrap();

        let result = git.create_worktree(&repo, Path::new("/trees/f2"), "feature/test");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MPCAError::BranchExists(_)));
    }

    #[test]
    fn test_mock_git_remove_worktree() {
        let repo = PathBuf::from("/repo");
        let git = MockGitAdapter::with_repo(repo.clone());

        let worktree = Path::new("/trees/feature");
        git.create_worktree(&repo, worktree, "feature/test")
            .unwrap();

        assert!(git.get_worktrees().contains_key(worktree));

        git.remove_worktree(&repo, worktree).unwrap();
        assert!(!git.get_worktrees().contains_key(worktree));
    }

    #[test]
    fn test_mock_git_is_clean() {
        let repo = PathBuf::from("/repo");
        let git = MockGitAdapter::with_repo(repo.clone());

        assert!(!git.has_uncommitted_changes(&repo));

        git.set_clean(false);
        assert!(git.has_uncommitted_changes(&repo));

        git.commit(&repo, "test commit").unwrap();
        assert!(!git.has_uncommitted_changes(&repo));
    }

    #[test]
    fn test_mock_git_diff() {
        let repo = PathBuf::from("/repo");
        let git = MockGitAdapter::with_repo(repo.clone());

        // Clean repo returns empty diff
        assert_eq!(git.diff(&repo).unwrap(), "");

        // Dirty repo returns diff
        git.set_clean(false);
        assert!(git.diff(&repo).unwrap().contains("diff --git"));
    }

    #[test]
    fn test_mock_git_clear() {
        let repo = PathBuf::from("/repo");
        let git = MockGitAdapter::with_repo(repo.clone());
        let worktree = Path::new("/trees/feature");

        git.create_worktree(&repo, worktree, "feature/test")
            .unwrap();
        git.set_clean(false);

        git.clear();

        assert!(!git.is_git_repo(&repo));
        assert!(git.get_worktrees().is_empty());
        assert!(git.get_branches().is_empty());
        assert!(!git.has_uncommitted_changes(&repo));
    }
}
