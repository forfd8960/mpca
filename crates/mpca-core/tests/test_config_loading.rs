//! Integration tests for configuration loading and management.
//!
//! Tests config file parsing, defaults, and loading behavior.

use mpca_core::MpcaConfig;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_load_defaults_when_missing() {
    let temp_dir = TempDir::new().unwrap();

    // Load config when file doesn't exist
    let config = MpcaConfig::load(temp_dir.path().to_path_buf()).unwrap();

    // Verify defaults
    assert_eq!(config.repo_root, temp_dir.path().to_path_buf());
    assert_eq!(config.trees_dir, temp_dir.path().join(".trees"));
    assert_eq!(config.specs_dir, temp_dir.path().join(".mpca/specs"));
    assert!(config.git.auto_commit);
    assert_eq!(config.git.branch_naming, "feature/{feature_slug}");
}

#[test]
fn test_config_load_from_file() {
    let temp_dir = TempDir::new().unwrap();
    let mpca_dir = temp_dir.path().join(".mpca");
    fs::create_dir_all(&mpca_dir).unwrap();

    // Create config file
    let config_content = r#"
[git]
auto_commit = false
branch_naming = "feat/{feature_slug}"

[review]
enabled = true
reviewers = ["alice", "bob"]
"#;

    fs::write(mpca_dir.join("config.toml"), config_content).unwrap();

    // Load config
    let config = MpcaConfig::load(temp_dir.path().to_path_buf()).unwrap();

    // Verify loaded values
    assert!(!config.git.auto_commit);
    assert_eq!(config.git.branch_naming, "feat/{feature_slug}");
    assert!(config.review.enabled);
    assert_eq!(config.review.reviewers, vec!["alice", "bob"]);
}

#[test]
fn test_config_invalid_toml_fails() {
    let temp_dir = TempDir::new().unwrap();
    let mpca_dir = temp_dir.path().join(".mpca");
    fs::create_dir_all(&mpca_dir).unwrap();

    // Create invalid TOML
    fs::write(mpca_dir.join("config.toml"), "invalid { toml").unwrap();

    // Load should fail
    let result = MpcaConfig::load(temp_dir.path().to_path_buf());
    assert!(result.is_err());
}

#[test]
fn test_config_partial_overrides() {
    let temp_dir = TempDir::new().unwrap();
    let mpca_dir = temp_dir.path().join(".mpca");
    fs::create_dir_all(&mpca_dir).unwrap();

    // Create config with partial overrides
    let config_content = r#"
[git]
auto_commit = false
"#;

    fs::write(mpca_dir.join("config.toml"), config_content).unwrap();

    let config = MpcaConfig::load(temp_dir.path().to_path_buf()).unwrap();

    // Overridden value
    assert!(!config.git.auto_commit);

    // Default value preserved
    assert_eq!(config.git.branch_naming, "feature/{feature_slug}");
}

#[test]
fn test_config_paths_always_canonical() {
    let temp_dir = TempDir::new().unwrap();
    let mpca_dir = temp_dir.path().join(".mpca");
    fs::create_dir_all(&mpca_dir).unwrap();

    // Create config with arbitrary values (should be overridden)
    let config_content = r#"
repo_root = "/some/other/path"
trees_dir = "/wrong/trees"
"#;

    fs::write(mpca_dir.join("config.toml"), config_content).unwrap();

    let config = MpcaConfig::load(temp_dir.path().to_path_buf()).unwrap();

    // Paths should be canonical based on actual repo_root
    assert_eq!(config.repo_root, temp_dir.path().to_path_buf());
    assert_eq!(config.trees_dir, temp_dir.path().join(".trees"));
}
