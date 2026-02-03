//! Integration tests for the plan workflow.
//!
//! Tests feature planning including directory creation, state management,
//! and spec file generation.

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
}

#[test]
fn test_plan_workflow_creates_structure() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    // Initialize project first
    runtime.init_project().unwrap();

    // Plan a new feature
    let result = runtime.plan_feature("add-caching");
    assert!(result.is_ok(), "Planning failed: {:?}", result.err());

    // Verify directory structure
    let feature_dir = temp_dir.path().join(".mpca/specs/add-caching");
    assert!(feature_dir.exists());
    assert!(feature_dir.join("specs").exists());
    assert!(feature_dir.join("docs").exists());

    // Verify spec files
    assert!(feature_dir.join("specs/state.toml").exists());
    assert!(feature_dir.join("specs/README.md").exists());
    assert!(feature_dir.join("specs/requirements.md").exists());
    assert!(feature_dir.join("specs/design.md").exists());
    assert!(feature_dir.join("specs/verify.md").exists());
}

#[test]
fn test_plan_workflow_state_file() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    runtime.init_project().unwrap();
    runtime.plan_feature("test-feature").unwrap();

    // Read and verify state file
    let state_file = temp_dir
        .path()
        .join(".mpca/specs/test-feature/specs/state.toml");
    let state_content = fs::read_to_string(state_file).unwrap();

    assert!(state_content.contains("feature_slug = \"test-feature\""));
    assert!(state_content.contains("phase = \"Plan\""));
    assert!(state_content.contains("step = 0"));
    assert!(state_content.contains("turns = 0"));
    assert!(state_content.contains("cost_usd = 0.0"));
}

#[test]
fn test_plan_duplicate_feature_fails() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    runtime.init_project().unwrap();
    runtime.plan_feature("duplicate").unwrap();

    // Try to plan same feature again
    let result = runtime.plan_feature("duplicate");
    assert!(result.is_err());
}

#[test]
fn test_plan_invalid_slug_fails() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    runtime.init_project().unwrap();

    // Invalid slugs
    assert!(runtime.plan_feature("Invalid-Slug").is_err());
    assert!(runtime.plan_feature("ab").is_err());
    assert!(runtime.plan_feature("feature_name").is_err());
    assert!(runtime.plan_feature("123-start").is_err());
}

#[test]
fn test_plan_multiple_features() {
    let temp_dir = TempDir::new().unwrap();
    init_test_repo(temp_dir.path());

    let config = MpcaConfig::new(temp_dir.path().to_path_buf());
    let runtime = AgentRuntime::new(config).unwrap();

    runtime.init_project().unwrap();

    // Plan multiple features
    runtime.plan_feature("feature-one").unwrap();
    runtime.plan_feature("feature-two").unwrap();
    runtime.plan_feature("feature-three").unwrap();

    // Verify all exist
    let specs_dir = temp_dir.path().join(".mpca/specs");
    assert!(specs_dir.join("feature-one").exists());
    assert!(specs_dir.join("feature-two").exists());
    assert!(specs_dir.join("feature-three").exists());
}
