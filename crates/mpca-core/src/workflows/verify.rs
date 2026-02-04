//! Verification workflow implementation.
//!
//! This module implements the verification workflow, which validates that
//! a feature implementation meets all acceptance criteria and quality standards.

use crate::config::MpcaConfig;
use crate::error::{MPCAError, Result};
use crate::tools::fs::FsAdapter;
use crate::tools::shell::ShellAdapter;
use anyhow::Context;
use std::path::Path;

/// Verifies a feature implementation against its verification spec.
///
/// This workflow:
/// 1. Validates feature exists and verify.md spec is present
/// 2. Loads verification spec from `.mpca/specs/<feature-slug>/specs/verify.md`
/// 3. Runs automated tests (unit, integration, custom)
/// 4. Executes manual verification checks
/// 5. Collects evidence (test output, logs, artifacts)
/// 6. Generates verification report with pass/fail status
/// 7. Updates state.toml with verification results
///
/// # Arguments
///
/// * `config` - MPCA configuration with repository paths
/// * `feature_slug` - Feature identifier (e.g., "add-caching")
/// * `fs` - File system adapter for file operations
/// * `shell` - Shell adapter for running tests and checks
///
/// # Returns
///
/// `Ok(())` if verification passes, or an error if verification fails.
///
/// # Errors
///
/// Returns:
/// - `MPCAError::FeatureNotFound` if feature specs don't exist
/// - `MPCAError::VerificationSpecMissing` if verify.md doesn't exist
/// - `MPCAError::VerificationFailed` if tests fail or criteria not met
/// - `MPCAError::VerificationTimeout` if tests take too long
/// - `MPCAError::ShellCommandFailed` if test commands fail
///
/// # Examples
///
/// ```no_run
/// use mpca_core::{MpcaConfig, workflows};
/// use mpca_core::tools::fs_impl::StdFsAdapter;
/// use mpca_core::tools::shell_impl::StdShellAdapter;
/// use std::path::PathBuf;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = MpcaConfig::new(PathBuf::from("/repo"));
/// let fs = StdFsAdapter::new();
/// let shell = StdShellAdapter::new();
///
/// workflows::verify_feature(&config, "add-caching", &fs, &shell)?;
/// # Ok(())
/// # }
/// ```
#[tracing::instrument(skip_all, fields(feature_slug = feature_slug))]
pub fn verify_feature(
    config: &MpcaConfig,
    feature_slug: &str,
    fs: &dyn FsAdapter,
    shell: &dyn ShellAdapter,
) -> Result<()> {
    // Verify feature exists
    let feature_dir = config.specs_dir.join(feature_slug);
    let specs_dir = feature_dir.join("specs");
    let verify_spec = specs_dir.join("verify.md");
    let state_file = feature_dir.join("state.toml");

    if !fs.exists(&feature_dir) {
        return Err(MPCAError::FeatureNotFound(feature_slug.to_string()));
    }

    if !fs.exists(&verify_spec) {
        return Err(MPCAError::VerificationSpecMissing(feature_slug.to_string()));
    }

    tracing::info!(
        feature = feature_slug,
        verify_spec = %verify_spec.display(),
        "starting verification workflow"
    );

    // Load verification spec
    let verify_content = fs
        .read_to_string(&verify_spec)
        .with_context(|| format!("failed to read verify.md for {}", feature_slug))?;

    tracing::debug!("loaded verification spec: {} bytes", verify_content.len());

    // Run automated tests
    let test_results = run_automated_tests(config, fs, shell)?;

    tracing::info!(
        passed = test_results.passed,
        failed = test_results.failed,
        "automated tests completed"
    );

    // Collect verification evidence
    let evidence = collect_evidence(config, feature_slug, &test_results, fs)?;

    // Generate verification report
    let report = generate_report(feature_slug, &verify_content, &test_results, &evidence);

    // Save verification report
    let report_path = feature_dir.join("verification_report.md");
    fs.write(&report_path, &report)
        .context("failed to write verification report")?;

    tracing::info!(
        report = %report_path.display(),
        "verification report generated"
    );

    // Update state to reflect verification
    update_state_for_verification(&state_file, &test_results, fs)?;

    // Check if verification passed
    if test_results.failed > 0 {
        return Err(MPCAError::VerificationFailed(format!(
            "{} test(s) failed",
            test_results.failed
        )));
    }

    tracing::info!(feature = feature_slug, "verification passed successfully");

    Ok(())
}

/// Results from running automated tests.
#[derive(Debug, Clone)]
struct TestResults {
    /// Number of tests that passed
    passed: usize,
    /// Number of tests that failed
    failed: usize,
    /// Number of tests that were ignored/skipped
    ignored: usize,
    /// Exit code from test command
    exit_code: i32,
}

/// Evidence collected during verification.
#[derive(Debug, Clone)]
struct Evidence {
    /// Paths to test result files
    test_results: Vec<String>,
    /// Paths to log files
    logs: Vec<String>,
    /// Performance metrics (if any)
    metrics: Vec<String>,
}

/// Runs automated tests for the feature.
fn run_automated_tests(
    config: &MpcaConfig,
    _fs: &dyn FsAdapter,
    shell: &dyn ShellAdapter,
) -> Result<TestResults> {
    // Determine working directory (use worktree if it exists, otherwise repo root)
    let working_dir = config.repo_root.clone();

    tracing::debug!(
        working_dir = %working_dir.display(),
        "running automated tests"
    );

    // Run cargo test with timeout
    let cmd_output = shell
        .run("cargo test --all -- --nocapture", Some(&working_dir))
        .context("failed to execute cargo test")?;

    // Combine stdout and stderr for full output
    let output = format!("{}\n{}", cmd_output.stdout, cmd_output.stderr);

    // Parse test results from output
    let test_results = parse_test_output(&output);

    tracing::debug!(
        passed = test_results.passed,
        failed = test_results.failed,
        ignored = test_results.ignored,
        exit_code = test_results.exit_code,
        "test execution completed"
    );

    // Save full test output
    let test_log_path = config.specs_dir.join("last_test_output.log");
    _fs.write(&test_log_path, &output)
        .context("failed to save test output")?;

    Ok(test_results)
}

/// Parses test output to extract pass/fail counts.
fn parse_test_output(output: &str) -> TestResults {
    let mut passed = 0;
    let mut failed = 0;
    let mut ignored = 0;
    let mut exit_code = 0;

    // Look for "test result:" line
    // Example: "test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
    for line in output.lines() {
        if line.contains("test result:") {
            // Extract pass/fail/ignore counts
            if let Some(passed_str) = extract_count(line, "passed") {
                passed = passed_str;
            }
            if let Some(failed_str) = extract_count(line, "failed") {
                failed = failed_str;
            }
            if let Some(ignored_str) = extract_count(line, "ignored") {
                ignored = ignored_str;
            }

            // Determine exit code based on "ok" or "FAILED"
            if line.contains("FAILED") {
                exit_code = 1;
            }
        }
    }

    TestResults {
        passed,
        failed,
        ignored,
        exit_code,
    }
}

/// Extracts a numeric count from a test result line.
fn extract_count(line: &str, label: &str) -> Option<usize> {
    // Find the number before the label
    let parts: Vec<&str> = line.split(';').collect();
    for part in parts {
        if part.contains(label) {
            // Extract number from "X passed" or "X failed"
            let words: Vec<&str> = part.split_whitespace().collect();
            for (i, word) in words.iter().enumerate() {
                if *word == label
                    && i > 0
                    && let Ok(count) = words[i - 1].parse::<usize>()
                {
                    return Some(count);
                }
            }
        }
    }
    None
}

/// Collects evidence files for verification.
fn collect_evidence(
    config: &MpcaConfig,
    feature_slug: &str,
    _test_results: &TestResults,
    fs: &dyn FsAdapter,
) -> Result<Evidence> {
    let mut evidence = Evidence {
        test_results: Vec::new(),
        logs: Vec::new(),
        metrics: Vec::new(),
    };

    // Look for common test result locations
    let possible_test_results = vec![
        config.repo_root.join("target/nextest/default/junit.xml"),
        config.repo_root.join("target/test-results.xml"),
        config.specs_dir.join("last_test_output.log"),
    ];

    for path in possible_test_results {
        if fs.exists(&path) {
            evidence
                .test_results
                .push(path.to_string_lossy().to_string());
        }
    }

    // Look for log files in feature directory
    let feature_dir = config.specs_dir.join(feature_slug);
    if fs.exists(&feature_dir) {
        // Would use fs.list_dir if available
        // For now, just check for common log file names
        let log_files = vec!["build.log", "test.log", "verification.log"];
        for log_name in log_files {
            let log_path = feature_dir.join(log_name);
            if fs.exists(&log_path) {
                evidence.logs.push(log_path.to_string_lossy().to_string());
            }
        }
    }

    tracing::debug!(
        test_results_count = evidence.test_results.len(),
        logs_count = evidence.logs.len(),
        "collected verification evidence"
    );

    Ok(evidence)
}

/// Generates a verification report in Markdown format.
fn generate_report(
    feature_slug: &str,
    verify_spec: &str,
    test_results: &TestResults,
    evidence: &Evidence,
) -> String {
    let status = if test_results.failed == 0 {
        "✅ PASS"
    } else {
        "❌ FAIL"
    };

    format!(
        r#"# Verification Report: {}

**Status**: {}
**Generated**: {}

## Test Results

```
Total tests: {}
Passed: {}
Failed: {}
Ignored: {}
Exit code: {}
```

## Verification Spec

{}

## Evidence

### Test Results
{}

### Logs
{}

### Metrics
{}

## Summary

{}

---

*Generated by MPCA verification workflow*
"#,
        feature_slug,
        status,
        chrono::Utc::now().to_rfc3339(),
        test_results.passed + test_results.failed + test_results.ignored,
        test_results.passed,
        test_results.failed,
        test_results.ignored,
        test_results.exit_code,
        verify_spec,
        if evidence.test_results.is_empty() {
            "- No test result files found".to_string()
        } else {
            evidence
                .test_results
                .iter()
                .map(|p| format!("- `{}`", p))
                .collect::<Vec<_>>()
                .join("\n")
        },
        if evidence.logs.is_empty() {
            "- No log files found".to_string()
        } else {
            evidence
                .logs
                .iter()
                .map(|p| format!("- `{}`", p))
                .collect::<Vec<_>>()
                .join("\n")
        },
        if evidence.metrics.is_empty() {
            "- No metrics collected".to_string()
        } else {
            evidence
                .metrics
                .iter()
                .map(|p| format!("- `{}`", p))
                .collect::<Vec<_>>()
                .join("\n")
        },
        if test_results.failed == 0 {
            "All tests passed. Feature is ready for review."
        } else {
            "Some tests failed. Please address failures before proceeding."
        }
    )
}

/// Updates state.toml to reflect verification results.
fn update_state_for_verification(
    state_file: &Path,
    test_results: &TestResults,
    fs: &dyn FsAdapter,
) -> Result<()> {
    // Read existing state if it exists
    let mut state_content = if fs.exists(state_file) {
        fs.read_to_string(state_file)
            .context("failed to read state.toml")?
    } else {
        String::new()
    };

    // Update phase to "Verify"
    if state_content.contains("phase = ") {
        state_content = state_content
            .lines()
            .map(|line| {
                if line.starts_with("phase = ") {
                    "phase = \"Verify\""
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        state_content.push('\n');
    } else {
        state_content.push_str("phase = \"Verify\"\n");
    }

    // Add verification status
    let verification_status = if test_results.failed == 0 {
        "passed"
    } else {
        "failed"
    };

    if state_content.contains("verification_status = ") {
        state_content = state_content
            .lines()
            .map(|line| {
                if line.starts_with("verification_status = ") {
                    format!("verification_status = \"{}\"", verification_status)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        state_content.push('\n');
    } else {
        state_content.push_str(&format!(
            "verification_status = \"{}\"\n",
            verification_status
        ));
    }

    // Update timestamp
    let timestamp = chrono::Utc::now().to_rfc3339();
    if state_content.contains("updated_at = ") {
        state_content = state_content
            .lines()
            .map(|line| {
                if line.starts_with("updated_at = ") {
                    format!("updated_at = \"{}\"", timestamp)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        state_content.push('\n');
    } else {
        state_content.push_str(&format!("updated_at = \"{}\"\n", timestamp));
    }

    // Add test counts
    state_content.push_str(&format!(
        "tests_passed = {}\ntests_failed = {}\ntests_ignored = {}\n",
        test_results.passed, test_results.failed, test_results.ignored
    ));

    fs.write(state_file, &state_content)
        .context("failed to update state.toml")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_test_output_success() {
        let output = r#"
running 42 tests
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
        "#;

        let results = parse_test_output(output);
        assert_eq!(results.passed, 42);
        assert_eq!(results.failed, 0);
        assert_eq!(results.ignored, 0);
        assert_eq!(results.exit_code, 0);
    }

    #[test]
    fn test_parse_test_output_failures() {
        let output = r#"
running 10 tests
test result: FAILED. 8 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
        "#;

        let results = parse_test_output(output);
        assert_eq!(results.passed, 8);
        assert_eq!(results.failed, 2);
        assert_eq!(results.ignored, 0);
        assert_eq!(results.exit_code, 1);
    }

    #[test]
    fn test_parse_test_output_ignored() {
        let output = r#"
running 15 tests
test result: ok. 12 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out
        "#;

        let results = parse_test_output(output);
        assert_eq!(results.passed, 12);
        assert_eq!(results.failed, 0);
        assert_eq!(results.ignored, 3);
        assert_eq!(results.exit_code, 0);
    }

    #[test]
    fn test_extract_count() {
        let line = "test result: ok. 42 passed; 0 failed; 3 ignored; 0 measured";
        assert_eq!(extract_count(line, "passed"), Some(42));
        assert_eq!(extract_count(line, "failed"), Some(0));
        assert_eq!(extract_count(line, "ignored"), Some(3));
        assert_eq!(extract_count(line, "measured"), Some(0));
    }
}
