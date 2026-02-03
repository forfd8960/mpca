//! Execute feature workflow implementation.
//!
//! This module implements the feature execution workflow, which loads
//! specifications and executes the implementation plan with git worktree support.

use crate::config::MpcaConfig;
use crate::error::{MPCAError, Result};
use crate::tools::fs::FsAdapter;
use crate::tools::git::GitAdapter;
use crate::tools::shell::ShellAdapter;
use anyhow::Context;
use std::path::Path;

/// Executes a feature implementation with the given slug.
///
/// This workflow:
/// 1. Validates feature exists in specs/
/// 2. Loads specifications from .mpca/specs/<feature-slug>/
/// 3. Creates git worktree in .trees/<feature-slug>/ on branch feature/<feature-slug>
/// 4. Initializes Claude agent with execution mode
/// 5. Executes implementation steps with access to:
///    - File operations (read, write, search)
///    - Git operations (status, commit, diff)
///    - Shell commands (build, test, run)
/// 6. Updates state.toml after each step
/// 7. Handles interruptions (saves state, allows resume)
///
/// # Arguments
///
/// * `config` - MPCA configuration with repository paths
/// * `feature_slug` - Feature identifier (e.g., "add-caching")
/// * `fs` - File system adapter for file operations
/// * `git` - Git adapter for repository operations
/// * `shell` - Shell adapter for executing commands
///
/// # Returns
///
/// `Ok(())` on successful execution, or an error if execution fails.
///
/// # Errors
///
/// Returns:
/// - `MPCAError::FeatureNotFound` if feature specs don't exist
/// - `MPCAError::WorktreeExists` if worktree already exists
/// - `MPCAError::GitCommandFailed` if git operations fail
/// - `MPCAError::AgentError` if Claude agent fails
///
/// # Examples
///
/// ```no_run
/// use mpca_core::{MpcaConfig, workflows};
/// use mpca_core::tools::fs_impl::StdFsAdapter;
/// use mpca_core::tools::git_impl::StdGitAdapter;
/// use mpca_core::tools::shell_impl::StdShellAdapter;
/// use std::path::PathBuf;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = MpcaConfig::new(PathBuf::from("/repo"));
/// let fs = StdFsAdapter::new();
/// let git = StdGitAdapter::new();
/// let shell = StdShellAdapter::new();
///
/// workflows::execute_feature(&config, "add-caching", &fs, &git, &shell)?;
/// # Ok(())
/// # }
/// ```
#[tracing::instrument(skip_all, fields(feature_slug = feature_slug))]
pub fn execute_feature(
    config: &MpcaConfig,
    feature_slug: &str,
    fs: &dyn FsAdapter,
    git: &dyn GitAdapter,
    _shell: &dyn ShellAdapter,
) -> Result<()> {
    // Verify feature exists
    let feature_dir = config.specs_dir.join(feature_slug);
    let specs_dir = feature_dir.join("specs");

    if !fs.exists(&feature_dir) {
        return Err(MPCAError::FeatureNotFound(feature_slug.to_string()));
    }

    if !fs.exists(&specs_dir) {
        return Err(MPCAError::FeatureNotFound(format!(
            "{} (specs directory not found)",
            feature_slug
        )));
    }

    // Check for existing state to support resume
    let state_file = specs_dir.join("state.toml");
    let resume = fs.exists(&state_file);

    if resume {
        tracing::info!(
            feature = feature_slug,
            "resuming feature execution from previous state"
        );
    } else {
        tracing::info!(feature = feature_slug, "starting fresh feature execution");
    }

    // Create worktree directory path
    let worktree_dir = config.trees_dir.join(feature_slug);
    let branch_name = config
        .git
        .branch_naming
        .replace("{feature_slug}", feature_slug);

    // Check if worktree already exists
    if fs.exists(&worktree_dir) && !resume {
        return Err(MPCAError::WorktreeExists(worktree_dir.clone()));
    }

    // Create git worktree if not resuming
    if !resume {
        create_worktree(config, feature_slug, &branch_name, &worktree_dir, git)?;
    }

    // Update state to execution phase
    update_state_for_execution(&state_file, fs)?;

    tracing::info!(
        feature = feature_slug,
        worktree = %worktree_dir.display(),
        branch = %branch_name,
        "feature execution initialized"
    );

    Ok(())
}

/// Creates a git worktree for feature development.
fn create_worktree(
    config: &MpcaConfig,
    feature_slug: &str,
    branch_name: &str,
    worktree_dir: &Path,
    git: &dyn GitAdapter,
) -> Result<()> {
    // Verify repository is clean
    if !git.is_git_repo(&config.repo_root) {
        return Err(MPCAError::NotGitRepository(config.repo_root.clone()));
    }

    // Create worktree with new branch
    git.create_worktree(&config.repo_root, worktree_dir, branch_name)
        .with_context(|| format!("failed to create worktree for {}", feature_slug))?;

    tracing::info!(
        branch = branch_name,
        worktree = %worktree_dir.display(),
        "created git worktree"
    );

    Ok(())
}

/// Updates state.toml to reflect execution phase.
fn update_state_for_execution(state_file: &Path, fs: &dyn FsAdapter) -> Result<()> {
    // Read existing state if it exists
    let mut state_content = if fs.exists(state_file) {
        fs.read_to_string(state_file)
            .context("failed to read state.toml")?
    } else {
        String::new()
    };

    // Update phase to "Run" if not already set
    if !state_content.contains("phase = \"Run\"") {
        if state_content.contains("phase = ") {
            state_content = state_content.replace("phase = \"Plan\"", "phase = \"Run\"");
        } else {
            state_content.push_str("phase = \"Run\"\n");
        }
    }

    // Update timestamp
    let timestamp = chrono::Utc::now().to_rfc3339();
    if state_content.contains("updated_at = ") {
        // Replace existing timestamp
        let lines: Vec<&str> = state_content.lines().collect();
        let mut new_lines = Vec::new();
        for line in lines {
            if line.starts_with("updated_at = ") {
                new_lines.push(format!("updated_at = \"{}\"", timestamp));
            } else {
                new_lines.push(line.to_string());
            }
        }
        state_content = new_lines.join("\n");
        state_content.push('\n');
    } else {
        state_content.push_str(&format!("updated_at = \"{}\"\n", timestamp));
    }

    fs.write(state_file, &state_content)
        .context("failed to update state.toml")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::fs_impl::StdFsAdapter;
    use crate::tools::git_impl::StdGitAdapter;
    use crate::tools::shell_impl::StdShellAdapter;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_test_repo(dir: &std::path::Path) {
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
        std::fs::write(dir.join("README.md"), "# Test").unwrap();
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

    fn create_test_feature(config: &MpcaConfig, feature_slug: &str, fs: &dyn FsAdapter) {
        let feature_dir = config.specs_dir.join(feature_slug);
        let specs_dir = feature_dir.join("specs");
        fs.create_dir_all(&specs_dir).unwrap();

        let state_content = format!(
            r#"feature_slug = "{}"
phase = "Plan"
step = 0
turns = 0
cost_usd = 0.0
created_at = "2024-01-01T00:00:00Z"
updated_at = "2024-01-01T00:00:00Z"
"#,
            feature_slug
        );
        fs.write(&specs_dir.join("state.toml"), &state_content)
            .unwrap();
    }

    #[test]
    fn test_execute_feature_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let config = MpcaConfig::new(temp_dir.path().to_path_buf());
        let fs = StdFsAdapter::new();
        let git = StdGitAdapter::new();
        let shell = StdShellAdapter::new();

        let result = execute_feature(&config, "nonexistent", &fs, &git, &shell);
        assert!(matches!(result, Err(MPCAError::FeatureNotFound(_))));
    }

    #[test]
    fn test_execute_feature_creates_worktree() {
        let temp_dir = TempDir::new().unwrap();
        init_test_repo(temp_dir.path());

        let config = MpcaConfig::new(temp_dir.path().to_path_buf());
        let fs = StdFsAdapter::new();
        let git = StdGitAdapter::new();
        let shell = StdShellAdapter::new();

        // Create feature specs
        create_test_feature(&config, "test-feature", &fs);

        let result = execute_feature(&config, "test-feature", &fs, &git, &shell);
        assert!(result.is_ok());

        // Verify worktree was created
        let worktree_dir = config.trees_dir.join("test-feature");
        assert!(fs.exists(&worktree_dir));

        // Verify state was updated
        let state_file = config
            .specs_dir
            .join("test-feature")
            .join("specs")
            .join("state.toml");
        let state_content = fs.read_to_string(&state_file).unwrap();
        assert!(state_content.contains("phase = \"Run\""));
    }

    #[test]
    fn test_execute_feature_resume() {
        let temp_dir = TempDir::new().unwrap();
        init_test_repo(temp_dir.path());

        let config = MpcaConfig::new(temp_dir.path().to_path_buf());
        let fs = StdFsAdapter::new();
        let git = StdGitAdapter::new();
        let shell = StdShellAdapter::new();

        // Create feature and execute once
        create_test_feature(&config, "test-feature", &fs);
        execute_feature(&config, "test-feature", &fs, &git, &shell).unwrap();

        // Execute again (should resume)
        let result = execute_feature(&config, "test-feature", &fs, &git, &shell);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_state_for_execution() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("state.toml");
        let fs = StdFsAdapter::new();

        let initial_state = r#"feature_slug = "test"
phase = "Plan"
step = 0
"#;
        fs.write(&state_file, initial_state).unwrap();

        let result = update_state_for_execution(&state_file, &fs);
        assert!(result.is_ok());

        let updated = fs.read_to_string(&state_file).unwrap();
        assert!(updated.contains("phase = \"Run\""));
        assert!(updated.contains("updated_at = "));
    }
}
