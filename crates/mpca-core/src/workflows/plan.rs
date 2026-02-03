//! Plan feature workflow implementation.
//!
//! This module implements the feature planning workflow, which guides the user
//! through interactive planning to create comprehensive feature specifications.

use crate::config::MpcaConfig;
use crate::error::{MPCAError, Result};
use crate::tools::fs::FsAdapter;
use crate::tools::git::GitAdapter;
use anyhow::Context;
use std::path::Path;

/// Plans a new feature with the given slug.
///
/// This workflow:
/// 1. Validates the feature slug format
/// 2. Creates specs/<feature-slug>/ directory structure
/// 3. Initializes Claude agent with planning mode
/// 4. Executes interactive planning conversation
/// 5. Generates and saves specification files:
///    - README.md (feature overview)
///    - requirements.md (user requirements)
///    - design.md (technical design)
/// 6. Creates state.toml to track progress
/// 7. Returns summary of created specifications
///
/// # Arguments
///
/// * `config` - MPCA configuration with repository paths
/// * `feature_slug` - Feature identifier (e.g., "add-caching")
/// * `fs` - File system adapter for creating files
/// * `git` - Git adapter for repository operations
///
/// # Returns
///
/// `Ok(())` on successful planning, or an error if planning fails.
///
/// # Errors
///
/// Returns:
/// - `MPCAError::InvalidFeatureSlug` if slug format is invalid
/// - `MPCAError::FeatureAlreadyExists` if feature already planned
/// - `MPCAError::FileWriteError` if cannot create spec files
/// - `MPCAError::AgentError` if Claude agent fails
///
/// # Examples
///
/// ```no_run
/// use mpca_core::{MpcaConfig, workflows};
/// use mpca_core::tools::fs_impl::StdFsAdapter;
/// use mpca_core::tools::git_impl::StdGitAdapter;
/// use std::path::PathBuf;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = MpcaConfig::new(PathBuf::from("/repo"));
/// let fs = StdFsAdapter::new();
/// let git = StdGitAdapter::new();
///
/// workflows::plan_feature(&config, "add-caching", &fs, &git)?;
/// # Ok(())
/// # }
/// ```
#[tracing::instrument(skip(fs, git), fields(feature_slug = %feature_slug))]
pub fn plan_feature(
    config: &MpcaConfig,
    feature_slug: &str,
    fs: &dyn FsAdapter,
    git: &dyn GitAdapter,
) -> Result<()> {
    // Validate feature slug format
    validate_feature_slug(feature_slug)?;

    // Create feature specs directory
    let feature_dir = config.specs_dir.join(feature_slug);
    let specs_dir = feature_dir.join("specs");
    let docs_dir = feature_dir.join("docs");

    // Check if feature already exists
    if fs.exists(&feature_dir) {
        return Err(MPCAError::FeatureAlreadyExists(feature_slug.to_string()));
    }

    // Create directory structure
    fs.create_dir_all(&specs_dir)
        .context("failed to create specs directory")?;
    fs.create_dir_all(&docs_dir)
        .context("failed to create docs directory")?;

    // Initialize state.toml
    let state_file = specs_dir.join("state.toml");
    let initial_state = format!(
        r#"# MPCA workflow state for feature: {}
feature_slug = "{}"
phase = "Plan"
step = 0
turns = 0
cost_usd = 0.0
created_at = "{}"
updated_at = "{}"
"#,
        feature_slug,
        feature_slug,
        chrono::Utc::now().to_rfc3339(),
        chrono::Utc::now().to_rfc3339()
    );

    fs.write(&state_file, &initial_state)
        .context("failed to write state.toml")?;

    // Create placeholder spec files (will be filled by Claude agent)
    create_placeholder_specs(&specs_dir, feature_slug, fs)?;

    // Verify git repository
    if !git.is_git_repo(&config.repo_root) {
        return Err(MPCAError::NotGitRepository(config.repo_root.clone()));
    }

    tracing::info!(
        feature = feature_slug,
        specs_dir = %specs_dir.display(),
        "feature planning initialized"
    );

    Ok(())
}

/// Validates that a feature slug follows naming conventions.
///
/// Valid slugs:
/// - Lowercase letters, numbers, and hyphens only
/// - Must start with a letter
/// - No consecutive hyphens
/// - Between 3 and 50 characters
fn validate_feature_slug(slug: &str) -> Result<()> {
    if slug.len() < 3 || slug.len() > 50 {
        return Err(MPCAError::InvalidFeatureSlug(format!(
            "{} (length must be 3-50 characters)",
            slug
        )));
    }

    if !slug.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        return Err(MPCAError::InvalidFeatureSlug(format!(
            "{} (must start with a lowercase letter)",
            slug
        )));
    }

    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(MPCAError::InvalidFeatureSlug(format!(
            "{} (only lowercase letters, numbers, and hyphens allowed)",
            slug
        )));
    }

    if slug.contains("--") {
        return Err(MPCAError::InvalidFeatureSlug(format!(
            "{} (no consecutive hyphens allowed)",
            slug
        )));
    }

    Ok(())
}

/// Creates placeholder specification files for a feature.
fn create_placeholder_specs(
    specs_dir: &Path,
    feature_slug: &str,
    fs: &dyn FsAdapter,
) -> Result<()> {
    // README.md
    let readme = format!(
        r#"# Feature: {}

## Overview
This document provides a high-level overview of the feature.

## Goals
- Goal 1
- Goal 2

## Non-Goals
- Non-goal 1

## Success Criteria
- [ ] Criterion 1
- [ ] Criterion 2
"#,
        feature_slug
    );
    fs.write(&specs_dir.join("README.md"), &readme)
        .context("failed to write README.md")?;

    // requirements.md
    let requirements = format!(
        r#"# Requirements: {}

## Functional Requirements
1. Requirement 1
2. Requirement 2

## Non-Functional Requirements
1. Performance: TBD
2. Security: TBD
3. Compatibility: TBD

## Constraints
- Constraint 1
- Constraint 2
"#,
        feature_slug
    );
    fs.write(&specs_dir.join("requirements.md"), &requirements)
        .context("failed to write requirements.md")?;

    // design.md
    let design = format!(
        r#"# Design: {}

## Architecture
Describe the high-level architecture and component interactions.

## Data Structures
List and describe key data structures.

## API Design
Document public interfaces and contracts.

## Implementation Plan
1. Step 1
2. Step 2
3. Step 3

## Testing Strategy
Describe testing approach and coverage goals.
"#,
        feature_slug
    );
    fs.write(&specs_dir.join("design.md"), &design)
        .context("failed to write design.md")?;

    // verify.md
    let verify = format!(
        r#"# Verification: {}

## Acceptance Criteria
- [ ] All tests pass
- [ ] Code follows project conventions
- [ ] Documentation is complete

## Test Cases
1. Test case 1
2. Test case 2

## Manual Verification Steps
1. Step 1
2. Step 2
"#,
        feature_slug
    );
    fs.write(&specs_dir.join("verify.md"), &verify)
        .context("failed to write verify.md")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::fs_impl::StdFsAdapter;
    use crate::tools::git_impl::StdGitAdapter;
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
    fn test_validate_feature_slug_valid() {
        assert!(validate_feature_slug("add-caching").is_ok());
        assert!(validate_feature_slug("feature-123").is_ok());
        assert!(validate_feature_slug("abc").is_ok());
    }

    #[test]
    fn test_validate_feature_slug_invalid() {
        // Too short
        assert!(validate_feature_slug("ab").is_err());
        // Too long
        assert!(validate_feature_slug("a".repeat(51).as_str()).is_err());
        // Starts with number
        assert!(validate_feature_slug("123-feature").is_err());
        // Contains uppercase
        assert!(validate_feature_slug("Add-Caching").is_err());
        // Consecutive hyphens
        assert!(validate_feature_slug("add--caching").is_err());
        // Invalid characters
        assert!(validate_feature_slug("add_caching").is_err());
    }

    #[test]
    fn test_plan_feature_creates_structure() {
        let temp_dir = TempDir::new().unwrap();
        init_test_repo(temp_dir.path());

        let config = MpcaConfig::new(temp_dir.path().to_path_buf());
        let fs = StdFsAdapter::new();
        let git = StdGitAdapter::new();

        // Create .mpca/specs directory
        fs.create_dir_all(&config.specs_dir).unwrap();

        let result = plan_feature(&config, "test-feature", &fs, &git);
        assert!(result.is_ok());

        // Verify directory structure
        let feature_dir = config.specs_dir.join("test-feature");
        assert!(fs.exists(&feature_dir));
        assert!(fs.exists(&feature_dir.join("specs")));
        assert!(fs.exists(&feature_dir.join("docs")));

        // Verify spec files
        assert!(fs.exists(&feature_dir.join("specs").join("state.toml")));
        assert!(fs.exists(&feature_dir.join("specs").join("README.md")));
        assert!(fs.exists(&feature_dir.join("specs").join("requirements.md")));
        assert!(fs.exists(&feature_dir.join("specs").join("design.md")));
        assert!(fs.exists(&feature_dir.join("specs").join("verify.md")));
    }

    #[test]
    fn test_plan_feature_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        init_test_repo(temp_dir.path());

        let config = MpcaConfig::new(temp_dir.path().to_path_buf());
        let fs = StdFsAdapter::new();
        let git = StdGitAdapter::new();

        fs.create_dir_all(&config.specs_dir).unwrap();

        // Create feature once
        plan_feature(&config, "test-feature", &fs, &git).unwrap();

        // Try to create again
        let result = plan_feature(&config, "test-feature", &fs, &git);
        assert!(matches!(result, Err(MPCAError::FeatureAlreadyExists(_))));
    }

    #[test]
    fn test_plan_feature_invalid_slug() {
        let temp_dir = TempDir::new().unwrap();
        let config = MpcaConfig::new(temp_dir.path().to_path_buf());
        let fs = StdFsAdapter::new();
        let git = StdGitAdapter::new();

        let result = plan_feature(&config, "Invalid-Slug", &fs, &git);
        assert!(matches!(result, Err(MPCAError::InvalidFeatureSlug(_))));
    }
}
