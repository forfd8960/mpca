//! Integration tests for the full init workflow.
//!
//! Tests the complete initialization flow from a fresh repository to a fully
//! initialized MPCA project.

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
fn test_full_init_workflow() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    // Execute init workflow
    let result = runtime.init_project();
    assert!(result.is_ok(), "Init failed: {:?}", result.err());

    // Verify all created artifacts
    assert!(temp_dir.path().join(".mpca").exists());
    assert!(temp_dir.path().join(".mpca/specs").exists());
    assert!(temp_dir.path().join(".trees").exists());
    assert!(temp_dir.path().join(".mpca/config.toml").exists());
    assert!(temp_dir.path().join("CLAUDE.md").exists());

    // Verify .gitignore was updated
    let gitignore_content = fs::read_to_string(temp_dir.path().join(".gitignore")).unwrap();
    assert!(gitignore_content.contains(".trees"));
}

#[test]
fn test_init_creates_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    runtime.init_project().unwrap();

    // Load the created config
    let config_path = temp_dir.path().join(".mpca/config.toml");
    let config_content = fs::read_to_string(&config_path).unwrap();

    // Verify it's valid TOML
    let _: toml::Value = toml::from_str(&config_content).unwrap();

    // Verify it can be loaded back
    let loaded_config = MpcaConfig::load(temp_dir.path().to_path_buf()).unwrap();
    assert_eq!(loaded_config.repo_root, temp_dir.path().to_path_buf());
}

#[test]
fn test_init_idempotent() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    // First init should succeed
    runtime.init_project().unwrap();

    // Second init should fail (already initialized)
    let result = runtime.init_project();
    assert!(result.is_err());
}

#[test]
fn test_init_outside_git_repo_fails() {
    let temp_dir = TempDir::new().unwrap();
    // Don't initialize git

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    let result = runtime.init_project();
    assert!(result.is_err());
}
