//! Standard git adapter implementation.
//!
//! This module provides a concrete implementation of the `GitAdapter` trait
//! using `std::process::Command` to execute git commands.

use crate::error::{MPCAError, Result};
use crate::tools::git::GitAdapter;
use std::path::Path;
use std::process::Command;

/// Standard git adapter using `git` command-line tool.
///
/// This adapter executes real git commands via `std::process::Command`.
/// For testing, use a mock implementation instead.
#[derive(Debug, Default)]
pub struct StdGitAdapter;

impl StdGitAdapter {
    /// Creates a new standard git adapter.
    ///
    /// # Returns
    ///
    /// A new `StdGitAdapter` instance.
    pub fn new() -> Self {
        Self
    }

    /// Helper to run a git command and capture output.
    fn run_git(&self, args: &[&str], cwd: Option<&Path>) -> Result<String> {
        let mut cmd = Command::new("git");
        cmd.args(args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let output = cmd
            .output()
            .map_err(|e| MPCAError::GitCommandFailed(format!("failed to execute git: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MPCAError::GitCommandFailed(format!(
                "git {} failed: {}",
                args.join(" "),
                stderr.trim()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl GitAdapter for StdGitAdapter {
    fn is_git_repo(&self, path: &Path) -> bool {
        // Check if .git directory exists in the given path
        path.join(".git").exists()
    }

    fn get_repo_root(&self, path: &Path) -> Result<String> {
        if !self.is_git_repo(path) {
            return Err(MPCAError::NotGitRepository(path.to_path_buf()));
        }

        self.run_git(&["rev-parse", "--show-toplevel"], Some(path))
    }

    fn create_worktree(
        &self,
        repo_root: &Path,
        worktree_path: &Path,
        branch_name: &str,
    ) -> Result<()> {
        // Check if worktree already exists
        if worktree_path.exists() {
            return Err(MPCAError::WorktreeExists(worktree_path.to_path_buf()));
        }

        // Check if branch already exists
        let branch_check = Command::new("git")
            .args(["rev-parse", "--verify", branch_name])
            .current_dir(repo_root)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if branch_check {
            return Err(MPCAError::BranchExists(branch_name.to_string()));
        }

        // Create worktree with new branch
        self.run_git(
            &[
                "worktree",
                "add",
                "-b",
                branch_name,
                worktree_path
                    .to_str()
                    .ok_or_else(|| MPCAError::InvalidPath(worktree_path.to_path_buf()))?,
            ],
            Some(repo_root),
        )?;

        Ok(())
    }

    fn remove_worktree(&self, repo_root: &Path, worktree_path: &Path) -> Result<()> {
        if !worktree_path.exists() {
            return Err(MPCAError::WorktreeNotFound(worktree_path.to_path_buf()));
        }

        self.run_git(
            &[
                "worktree",
                "remove",
                worktree_path
                    .to_str()
                    .ok_or_else(|| MPCAError::InvalidPath(worktree_path.to_path_buf()))?,
            ],
            Some(repo_root),
        )?;

        Ok(())
    }

    fn commit(&self, path: &Path, message: &str) -> Result<()> {
        // Add all changes
        self.run_git(&["add", "-A"], Some(path))?;

        // Check if there's anything to commit
        if !self.has_uncommitted_changes(path) {
            return Ok(());
        }

        // Commit with message
        self.run_git(&["commit", "-m", message], Some(path))?;

        Ok(())
    }

    fn status(&self, path: &Path) -> Result<Vec<String>> {
        let output = self.run_git(&["status", "--porcelain"], Some(path))?;

        if output.is_empty() {
            return Ok(Vec::new());
        }

        Ok(output
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                // Format is "XY filename", we want just the filename
                line.split_whitespace()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect())
    }

    fn has_uncommitted_changes(&self, path: &Path) -> bool {
        // Only check if it's a git repo
        if !self.is_git_repo(path) {
            return false;
        }

        // Check for modified tracked files
        let has_diff = Command::new("git")
            .args(["diff", "--quiet", "HEAD"])
            .current_dir(path)
            .output()
            .map(|output| !output.status.success())
            .unwrap_or(false);

        // Check for staged changes
        let has_cached = Command::new("git")
            .args(["diff", "--cached", "--quiet"])
            .current_dir(path)
            .output()
            .map(|output| !output.status.success())
            .unwrap_or(false);

        // Check for untracked files
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
            .unwrap_or_default();

        has_diff || has_cached || !status_output.is_empty()
    }

    fn diff(&self, path: &Path) -> Result<String> {
        self.run_git(&["diff", "HEAD"], Some(path))
    }

    fn add(&self, path: &Path, files: &[&str]) -> Result<()> {
        let mut args = vec!["add"];
        args.extend(files);

        self.run_git(&args, Some(path))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn init_test_repo(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .env("PRE_COMMIT_ALLOW_NO_CONFIG", "1")
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(dir)
            .env("PRE_COMMIT_ALLOW_NO_CONFIG", "1")
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir)
            .env("PRE_COMMIT_ALLOW_NO_CONFIG", "1")
            .output()
            .unwrap();

        // Create initial commit
        fs::write(dir.join("README.md"), "# Test Repo").unwrap();
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(dir)
            .env("PRE_COMMIT_ALLOW_NO_CONFIG", "1")
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(dir)
            .env("PRE_COMMIT_ALLOW_NO_CONFIG", "1")
            .output()
            .unwrap();
    }

    #[test]
    fn test_is_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = StdGitAdapter::new();

        // Not a git repo initially
        assert!(!adapter.is_git_repo(temp_dir.path()));

        // Initialize git repo
        init_test_repo(temp_dir.path());

        // Now it's a git repo
        assert!(adapter.is_git_repo(temp_dir.path()));
    }

    #[test]
    fn test_get_repo_root() {
        let temp_dir = TempDir::new().unwrap();
        init_test_repo(temp_dir.path());

        let adapter = StdGitAdapter::new();
        let root = adapter.get_repo_root(temp_dir.path()).unwrap();

        // The root should be the temp directory (paths might differ in canonicalization)
        assert!(root.ends_with(temp_dir.path().file_name().unwrap().to_str().unwrap()));
    }

    #[test]
    fn test_commit() {
        let temp_dir = TempDir::new().unwrap();
        init_test_repo(temp_dir.path());

        let adapter = StdGitAdapter::new();

        // Create a new file
        fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

        // Commit it
        adapter.commit(temp_dir.path(), "Add test file").unwrap();

        // Should have no uncommitted changes now
        assert!(!adapter.has_uncommitted_changes(temp_dir.path()));
    }

    #[test]
    fn test_status() {
        let temp_dir = TempDir::new().unwrap();
        init_test_repo(temp_dir.path());

        let adapter = StdGitAdapter::new();

        // Create a new file
        fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

        let status = adapter.status(temp_dir.path()).unwrap();
        assert!(!status.is_empty());
        assert!(status.iter().any(|s| s.contains("test.txt")));
    }

    #[test]
    fn test_has_uncommitted_changes() {
        let temp_dir = TempDir::new().unwrap();
        init_test_repo(temp_dir.path());

        let adapter = StdGitAdapter::new();

        // No changes initially
        assert!(!adapter.has_uncommitted_changes(temp_dir.path()));

        // Create a new file
        fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

        // Should have uncommitted changes
        assert!(adapter.has_uncommitted_changes(temp_dir.path()));
    }
}
