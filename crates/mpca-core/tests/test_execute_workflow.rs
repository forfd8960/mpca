//! Integration tests for the execute workflow.
//!
//! Tests feature execution including worktree creation, state updates,
//! and resume capability.

use mpca_core::{AgentRuntime, MpcaConfig};
use std::fs;
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
    fs::write(dir.join("README.md"), "# Test").unwrap();
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
fn test_execute_workflow_creates_worktree() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    // Initialize and plan
    runtime.init_project().unwrap();
    runtime.plan_feature("test-feature").unwrap();

    // Execute
    let result = runtime.run_feature("test-feature");
    assert!(result.is_ok(), "Execute failed: {:?}", result.err());

    // Verify worktree created
    let worktree_dir = temp_dir.path().join(".trees/test-feature");
    assert!(worktree_dir.exists());
    assert!(worktree_dir.join(".git").exists());
}

#[test]
fn test_execute_workflow_updates_state() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    runtime.init_project().unwrap();
    runtime.plan_feature("test-feature").unwrap();
    runtime.run_feature("test-feature").unwrap();

    // Verify state updated to Run phase
    let state_file = temp_dir
        .path()
        .join(".mpca/specs/test-feature/specs/state.toml");
    let state_content = fs::read_to_string(state_file).unwrap();

    assert!(state_content.contains("phase = \"Run\""));
}

#[test]
fn test_execute_nonexistent_feature_fails() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    runtime.init_project().unwrap();

    let result = runtime.run_feature("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_execute_workflow_resume() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    runtime.init_project().unwrap();
    runtime.plan_feature("test-feature").unwrap();

    // First execution
    runtime.run_feature("test-feature").unwrap();

    // Second execution should resume without error
    let result = runtime.run_feature("test-feature");
    assert!(result.is_ok());
}

#[test]
fn test_execute_workflow_branch_naming() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    runtime.init_project().unwrap();
    runtime.plan_feature("my-feature").unwrap();
    runtime.run_feature("my-feature").unwrap();

    // Check that branch was created with correct name
    let output = Command::new("git")
        .args(["branch", "--list"])
        .current_dir(temp_dir.path())
        .env("PRE_COMMIT_ALLOW_NO_CONFIG", "1")
        .output()
        .unwrap();

    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(branches.contains("feature/my-feature"));
}
