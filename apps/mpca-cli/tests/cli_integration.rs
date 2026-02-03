//! Integration tests for MPCA CLI.
//!
//! Tests each CLI subcommand with temporary git repositories to ensure
//! proper behavior and error handling.

use anyhow::Result;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a temporary git repository
fn create_test_repo() -> Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()?;

    // Configure git (required for commits)
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()?;

    // Create initial commit
    std::fs::write(temp_dir.path().join("README.md"), "# Test Repo")?;
    Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()?;
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()?;

    Ok(temp_dir)
}

/// Get the path to the mpca binary
fn mpca_bin() -> String {
    // Use cargo to find the binary
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--quiet", "--bin", "mpca"]);
    cmd.output().expect("Failed to build mpca binary");

    // Binary should be in target/debug/mpca
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/../../target/debug/mpca", manifest_dir)
}

#[test]
fn test_cli_version() -> Result<()> {
    let output = Command::new(mpca_bin()).arg("--version").output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("mpca"));

    Ok(())
}

#[test]
fn test_cli_help() -> Result<()> {
    let output = Command::new(mpca_bin()).arg("--help").output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("MPCA"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("plan"));
    assert!(stdout.contains("run"));

    Ok(())
}

#[test]
fn test_init_command_success() -> Result<()> {
    let temp_repo = create_test_repo()?;

    let output = Command::new(mpca_bin())
        .arg("init")
        .current_dir(temp_repo.path())
        .output()?;

    assert!(output.status.success(), "Init command failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Repository initialized"));

    // Verify directories were created
    assert!(temp_repo.path().join(".mpca").exists());
    assert!(temp_repo.path().join(".trees").exists());
    assert!(temp_repo.path().join(".mpca/config.toml").exists());

    Ok(())
}

#[test]
fn test_init_command_not_git_repo() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    let output = Command::new(mpca_bin())
        .arg("init")
        .current_dir(temp_dir.path())
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("git repository") || stderr.contains("Not a git repository"));

    Ok(())
}

#[test]
fn test_plan_command_requires_init() -> Result<()> {
    let temp_repo = create_test_repo()?;

    let output = Command::new(mpca_bin())
        .args(["plan", "test-feature"])
        .current_dir(temp_repo.path())
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("not initialized") || stderr.contains("mpca init"));

    Ok(())
}

#[test]
fn test_plan_command_after_init() -> Result<()> {
    let temp_repo = create_test_repo()?;

    // Initialize first
    Command::new(mpca_bin())
        .arg("init")
        .current_dir(temp_repo.path())
        .output()?;

    // Now plan
    let output = Command::new(mpca_bin())
        .args(["plan", "test-feature"])
        .current_dir(temp_repo.path())
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("test-feature"));

    Ok(())
}

#[test]
fn test_run_command_requires_init() -> Result<()> {
    let temp_repo = create_test_repo()?;

    let output = Command::new(mpca_bin())
        .args(["run", "test-feature"])
        .current_dir(temp_repo.path())
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("not initialized") || stderr.contains("mpca init"));

    Ok(())
}

#[test]
fn test_run_command_after_init() -> Result<()> {
    let temp_repo = create_test_repo()?;

    // Initialize first
    Command::new(mpca_bin())
        .arg("init")
        .current_dir(temp_repo.path())
        .output()?;

    // Now run
    let output = Command::new(mpca_bin())
        .args(["run", "test-feature"])
        .current_dir(temp_repo.path())
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("test-feature"));

    Ok(())
}

#[test]
fn test_review_command_requires_init() -> Result<()> {
    let temp_repo = create_test_repo()?;

    let output = Command::new(mpca_bin())
        .args(["review", "test-feature"])
        .current_dir(temp_repo.path())
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("not initialized") || stderr.contains("mpca init"));

    Ok(())
}

#[test]
fn test_review_command_after_init() -> Result<()> {
    let temp_repo = create_test_repo()?;

    // Initialize first
    Command::new(mpca_bin())
        .arg("init")
        .current_dir(temp_repo.path())
        .output()?;

    // Now review
    let output = Command::new(mpca_bin())
        .args(["review", "test-feature"])
        .current_dir(temp_repo.path())
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("test-feature"));

    Ok(())
}

#[test]
fn test_chat_command_requires_init() -> Result<()> {
    let temp_repo = create_test_repo()?;

    let output = Command::new(mpca_bin())
        .arg("chat")
        .current_dir(temp_repo.path())
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("not initialized") || stderr.contains("mpca init"));

    Ok(())
}

#[test]
fn test_chat_command_after_init() -> Result<()> {
    let temp_repo = create_test_repo()?;

    // Initialize first
    Command::new(mpca_bin())
        .arg("init")
        .current_dir(temp_repo.path())
        .output()?;

    // Now chat
    let output = Command::new(mpca_bin())
        .arg("chat")
        .current_dir(temp_repo.path())
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("not yet implemented") || stdout.contains("Chat mode"));

    Ok(())
}

#[test]
fn test_resume_command_requires_init() -> Result<()> {
    let temp_repo = create_test_repo()?;

    let output = Command::new(mpca_bin())
        .args(["resume", "test-feature"])
        .current_dir(temp_repo.path())
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("not initialized") || stderr.contains("mpca init"));

    Ok(())
}

#[test]
fn test_resume_command_after_init() -> Result<()> {
    let temp_repo = create_test_repo()?;

    // Initialize first
    Command::new(mpca_bin())
        .arg("init")
        .current_dir(temp_repo.path())
        .output()?;

    // Now resume
    let output = Command::new(mpca_bin())
        .args(["resume", "test-feature"])
        .current_dir(temp_repo.path())
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("test-feature"));

    Ok(())
}

#[test]
fn test_verbose_flag() -> Result<()> {
    let temp_repo = create_test_repo()?;

    let output = Command::new(mpca_bin())
        .args(["-v", "init"])
        .current_dir(temp_repo.path())
        .output()?;

    assert!(output.status.success());

    Ok(())
}
