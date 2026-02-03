//! File system adapter trait and operations.
//!
//! This module defines the `FsAdapter` trait for file system operations,
//! allowing for both real file system access and mock implementations for testing.

use crate::error::Result;
use std::path::Path;

/// File system adapter trait.
///
/// Defines the interface for file system operations needed by MPCA workflows.
/// Implementations can be real (using `std::fs`) or mocked for testing.
pub trait FsAdapter: Send + Sync {
    /// Reads the contents of a file as a string.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read.
    ///
    /// # Returns
    ///
    /// The file contents as a `String`, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::PathNotFound` if the file doesn't exist,
    /// `MPCAError::FileReadError` if reading fails, or `MPCAError::Io`
    /// for other IO errors.
    fn read_to_string(&self, path: &Path) -> Result<String>;

    /// Writes a string to a file, creating it if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to write.
    /// * `content` - Content to write to the file.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::FileWriteError` if writing fails,
    /// `MPCAError::PermissionDenied` if lacking write permissions,
    /// or `MPCAError::Io` for other IO errors.
    fn write(&self, path: &Path, content: &str) -> Result<()>;

    /// Lists all entries in a directory.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory to list.
    ///
    /// # Returns
    ///
    /// A vector of entry names (not full paths), or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::PathNotFound` if the directory doesn't exist,
    /// `MPCAError::InvalidPath` if the path is not a directory,
    /// or `MPCAError::Io` for other IO errors.
    fn list_dir(&self, path: &Path) -> Result<Vec<String>>;

    /// Checks if a path exists.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check.
    ///
    /// # Returns
    ///
    /// `true` if the path exists (file or directory), `false` otherwise.
    fn exists(&self, path: &Path) -> bool;

    /// Creates a directory and all missing parent directories.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory to create.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::FileWriteError` if creation fails,
    /// `MPCAError::PermissionDenied` if lacking write permissions,
    /// or `MPCAError::Io` for other IO errors.
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Checks if a path is a directory.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check.
    ///
    /// # Returns
    ///
    /// `true` if the path exists and is a directory, `false` otherwise.
    fn is_dir(&self, path: &Path) -> bool;

    /// Checks if a path is a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check.
    ///
    /// # Returns
    ///
    /// `true` if the path exists and is a file, `false` otherwise.
    fn is_file(&self, path: &Path) -> bool;
}
