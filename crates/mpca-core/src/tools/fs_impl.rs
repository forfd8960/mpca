//! Standard file system adapter implementation.
//!
//! This module provides a concrete implementation of the `FsAdapter` trait
//! using `std::fs` for real file system operations.

use crate::error::{MPCAError, Result};
use crate::tools::fs::FsAdapter;
use std::path::Path;

/// Standard file system adapter using `std::fs`.
///
/// This adapter provides real file system access and is the default
/// implementation used by MPCA in production. For testing, use a mock
/// implementation instead.
#[derive(Debug, Default)]
pub struct StdFsAdapter;

impl StdFsAdapter {
    /// Creates a new standard file system adapter.
    ///
    /// # Returns
    ///
    /// A new `StdFsAdapter` instance.
    pub fn new() -> Self {
        Self
    }
}

impl FsAdapter for StdFsAdapter {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        std::fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                MPCAError::PathNotFound(path.to_path_buf())
            } else {
                MPCAError::FileReadError(format!("{}: {}", path.display(), e))
            }
        })
    }

    fn write(&self, path: &Path, content: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            self.create_dir_all(parent)?;
        }

        std::fs::write(path, content).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                MPCAError::PermissionDenied(path.display().to_string())
            } else {
                MPCAError::FileWriteError(format!("{}: {}", path.display(), e))
            }
        })
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        if !path.exists() {
            return Err(MPCAError::PathNotFound(path.to_path_buf()));
        }

        if !path.is_dir() {
            return Err(MPCAError::InvalidPath(path.to_path_buf()));
        }

        std::fs::read_dir(path)
            .map_err(|e| MPCAError::FileReadError(format!("{}: {}", path.display(), e)))?
            .map(|entry| {
                entry
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .map_err(|e| {
                        MPCAError::FileReadError(format!("failed to read directory entry: {}", e))
                    })
            })
            .collect()
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                MPCAError::PermissionDenied(path.display().to_string())
            } else {
                MPCAError::FileWriteError(format!("{}: {}", path.display(), e))
            }
        })
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_write() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = StdFsAdapter::new();
        let file_path = temp_dir.path().join("test.txt");

        // Write content
        adapter.write(&file_path, "Hello, MPCA!").unwrap();

        // Read content back
        let content = adapter.read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, MPCA!");
    }

    #[test]
    fn test_read_nonexistent() {
        let adapter = StdFsAdapter::new();
        let result = adapter.read_to_string(Path::new("/nonexistent/file.txt"));

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MPCAError::PathNotFound(_)));
    }

    #[test]
    fn test_create_dir_all() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = StdFsAdapter::new();
        let nested_dir = temp_dir.path().join("a").join("b").join("c");

        adapter.create_dir_all(&nested_dir).unwrap();

        assert!(adapter.exists(&nested_dir));
        assert!(adapter.is_dir(&nested_dir));
    }

    #[test]
    fn test_list_dir() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = StdFsAdapter::new();

        // Create some files
        adapter
            .write(&temp_dir.path().join("file1.txt"), "content1")
            .unwrap();
        adapter
            .write(&temp_dir.path().join("file2.txt"), "content2")
            .unwrap();

        let entries = adapter.list_dir(temp_dir.path()).unwrap();

        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&"file1.txt".to_string()));
        assert!(entries.contains(&"file2.txt".to_string()));
    }

    #[test]
    fn test_exists_and_is_checks() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = StdFsAdapter::new();
        let file_path = temp_dir.path().join("test.txt");

        // File doesn't exist yet
        assert!(!adapter.exists(&file_path));
        assert!(!adapter.is_file(&file_path));

        // Create file
        adapter.write(&file_path, "content").unwrap();

        // File exists and is a file
        assert!(adapter.exists(&file_path));
        assert!(adapter.is_file(&file_path));
        assert!(!adapter.is_dir(&file_path));

        // Directory checks
        assert!(adapter.is_dir(temp_dir.path()));
        assert!(!adapter.is_file(temp_dir.path()));
    }

    #[test]
    fn test_write_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = StdFsAdapter::new();
        let file_path = temp_dir.path().join("nested").join("dirs").join("file.txt");

        adapter.write(&file_path, "content").unwrap();

        assert!(adapter.exists(&file_path));
        assert!(adapter.is_file(&file_path));
    }
}
