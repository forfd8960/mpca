//! Git adapter trait and operations.
//!
//! This module defines the `GitAdapter` trait for git operations,
//! allowing for both real git command execution and mock implementations for testing.

use crate::error::Result;
use std::path::Path;

/// Git adapter trait.
///
/// Defines the interface for git operations needed by MPCA workflows.
/// Implementations can execute real git commands or provide mocked behavior for testing.
pub trait GitAdapter: Send + Sync {
    /// Checks if a directory is a git repository.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check (typically repository root).
    ///
    /// # Returns
    ///
    /// `true` if the path is a git repository, `false` otherwise.
    fn is_git_repo(&self, path: &Path) -> bool;

    /// Gets the root directory of a git repository.
    ///
    /// # Arguments
    ///
    /// * `path` - Any path within the repository.
    ///
    /// # Returns
    ///
    /// The absolute path to the repository root, or an error if not in a git repository.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::NotGitRepository` if the path is not within a git repository,
    /// or `MPCAError::GitCommandFailed` if the git command fails.
    fn get_repo_root(&self, path: &Path) -> Result<String>;

    /// Creates a new git worktree.
    ///
    /// # Arguments
    ///
    /// * `repo_root` - Root directory of the main repository.
    /// * `worktree_path` - Path where the worktree should be created.
    /// * `branch_name` - Name of the branch to create and checkout in the worktree.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::WorktreeExists` if a worktree already exists at the path,
    /// `MPCAError::BranchExists` if the branch already exists,
    /// or `MPCAError::GitCommandFailed` if the git command fails.
    fn create_worktree(
        &self,
        repo_root: &Path,
        worktree_path: &Path,
        branch_name: &str,
    ) -> Result<()>;

    /// Removes a git worktree.
    ///
    /// # Arguments
    ///
    /// * `repo_root` - Root directory of the main repository.
    /// * `worktree_path` - Path to the worktree to remove.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::WorktreeNotFound` if the worktree doesn't exist,
    /// or `MPCAError::GitCommandFailed` if the git command fails.
    fn remove_worktree(&self, repo_root: &Path, worktree_path: &Path) -> Result<()>;

    /// Commits changes in a repository or worktree.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the repository or worktree.
    /// * `message` - Commit message.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::GitCommandFailed` if the git command fails.
    fn commit(&self, path: &Path, message: &str) -> Result<()>;

    /// Gets the current git status (list of modified files).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the repository or worktree.
    ///
    /// # Returns
    ///
    /// A vector of file paths that have been modified, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::GitCommandFailed` if the git command fails.
    fn status(&self, path: &Path) -> Result<Vec<String>>;

    /// Checks if there are uncommitted changes in a repository or worktree.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the repository or worktree.
    ///
    /// # Returns
    ///
    /// `true` if there are uncommitted changes, `false` otherwise.
    fn has_uncommitted_changes(&self, path: &Path) -> bool;

    /// Gets the diff of uncommitted changes.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the repository or worktree.
    ///
    /// # Returns
    ///
    /// The diff output as a string, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::GitCommandFailed` if the git command fails.
    fn diff(&self, path: &Path) -> Result<String>;

    /// Adds files to the staging area.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the repository or worktree.
    /// * `files` - List of file paths to add (relative to `path`). Use `&["."]` to add all.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::GitCommandFailed` if the git command fails.
    fn add(&self, path: &Path, files: &[&str]) -> Result<()>;
}
