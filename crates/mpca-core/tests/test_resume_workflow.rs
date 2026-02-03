//! Integration tests for workflow resume capability.
//!
//! Tests interruption and resumption of workflows with state persistence.

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
fn test_resume_after_execution_start() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    runtime.init_project().unwrap();
    runtime.plan_feature("resumable").unwrap();
    runtime.run_feature("resumable").unwrap();

    // Read state before resume
    let state_file = temp_dir
        .path()
        .join(".mpca/specs/resumable/specs/state.toml");
    let _state_before = fs::read_to_string(&state_file).unwrap();

    // Create a new runtime instance (simulating restart)
    let new_config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let new_runtime = AgentRuntime::new(new_config).unwrap();

    // Resume execution
    let result = new_runtime.run_feature("resumable");
    assert!(result.is_ok());

    // State should still be present
    let state_after = fs::read_to_string(&state_file).unwrap();
    assert!(state_after.contains("phase = \"Run\""));
}

#[test]
fn test_state_persistence() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    runtime.init_project().unwrap();
    runtime.plan_feature("persistent").unwrap();

    let state_file = temp_dir
        .path()
        .join(".mpca/specs/persistent/specs/state.toml");

    // Verify state exists after planning
    assert!(state_file.exists());
    let state = fs::read_to_string(&state_file).unwrap();
    assert!(state.contains("phase = \"Plan\""));

    // Execute and verify state updates
    runtime.run_feature("persistent").unwrap();
    let state_after = fs::read_to_string(&state_file).unwrap();
    assert!(state_after.contains("phase = \"Run\""));
    assert!(state_after.contains("updated_at"));
}

#[test]
fn test_worktree_preserved_on_resume() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    runtime.init_project().unwrap();
    runtime.plan_feature("preserved").unwrap();
    runtime.run_feature("preserved").unwrap();

    let worktree_dir = temp_dir.path().join(".trees/preserved");
    assert!(worktree_dir.exists());

    // Add a file to worktree
    fs::write(worktree_dir.join("test.txt"), "preserved content").unwrap();

    // Resume
    let new_config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let new_runtime = AgentRuntime::new(new_config).unwrap();
    new_runtime.run_feature("preserved").unwrap();

    // Verify file still exists
    assert!(worktree_dir.join("test.txt").exists());
    let content = fs::read_to_string(worktree_dir.join("test.txt")).unwrap();
    assert_eq!(content, "preserved content");
}
