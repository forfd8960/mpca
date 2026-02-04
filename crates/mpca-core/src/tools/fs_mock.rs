//! Mock file system adapter for testing.
//!
//! This module provides a mock implementation of the `FsAdapter` trait
//! for use in tests. The mock uses an in-memory HashMap to simulate
//! file system operations.

use crate::error::{MPCAError, Result};
use crate::tools::fs::FsAdapter;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Mock file system adapter for testing.
///
/// Uses an in-memory HashMap to simulate file system operations.
/// All operations are thread-safe via Arc<Mutex>.
///
/// # Examples
///
/// ```
/// use mpca_core::tools::fs_mock::MockFsAdapter;
/// use mpca_core::tools::fs::FsAdapter;
/// use std::path::Path;
///
/// let fs = MockFsAdapter::new();
/// fs.write(Path::new("/test.txt"), "content").unwrap();
/// assert_eq!(fs.read_to_string(Path::new("/test.txt")).unwrap(), "content");
/// ```
#[derive(Debug, Clone, Default)]
pub struct MockFsAdapter {
    /// In-memory file system storage (path -> content)
    files: Arc<Mutex<HashMap<PathBuf, String>>>,
    /// In-memory directory storage
    dirs: Arc<Mutex<Vec<PathBuf>>>,
}

impl MockFsAdapter {
    /// Creates a new mock file system adapter.
    ///
    /// # Returns
    ///
    /// A new `MockFsAdapter` with empty file system.
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            dirs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Pre-populates the mock file system with files.
    ///
    /// # Arguments
    ///
    /// * `files` - HashMap of paths to file contents
    ///
    /// # Examples
    ///
    /// ```
    /// use mpca_core::tools::fs_mock::MockFsAdapter;
    /// use std::collections::HashMap;
    /// use std::path::PathBuf;
    ///
    /// let mut files = HashMap::new();
    /// files.insert(PathBuf::from("/README.md"), "# Test".to_string());
    ///
    /// let fs = MockFsAdapter::with_files(files);
    /// ```
    pub fn with_files(files: HashMap<PathBuf, String>) -> Self {
        Self {
            files: Arc::new(Mutex::new(files)),
            dirs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Returns a copy of all files in the mock file system.
    ///
    /// # Returns
    ///
    /// HashMap of all files (path -> content).
    pub fn get_all_files(&self) -> HashMap<PathBuf, String> {
        self.files.lock().unwrap().clone()
    }

    /// Returns all directory paths in the mock file system.
    ///
    /// # Returns
    ///
    /// Vector of all directory paths.
    pub fn get_all_dirs(&self) -> Vec<PathBuf> {
        self.dirs.lock().unwrap().clone()
    }

    /// Clears all files and directories from the mock file system.
    pub fn clear(&self) {
        self.files.lock().unwrap().clear();
        self.dirs.lock().unwrap().clear();
    }
}

impl FsAdapter for MockFsAdapter {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| MPCAError::PathNotFound(path.to_path_buf()))
    }

    fn write(&self, path: &Path, content: &str) -> Result<()> {
        // Auto-create parent directories
        if let Some(parent) = path.parent() {
            let mut dirs = self.dirs.lock().unwrap();
            if !dirs.contains(&parent.to_path_buf()) {
                dirs.push(parent.to_path_buf());
            }
        }

        self.files
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), content.to_string());
        Ok(())
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        let files = self.files.lock().unwrap();
        let dirs = self.dirs.lock().unwrap();

        // Check if directory exists
        if !dirs.contains(&path.to_path_buf()) {
            return Err(MPCAError::PathNotFound(path.to_path_buf()));
        }

        // List all files/dirs in this directory
        let mut entries = Vec::new();

        // Find all files under this path
        for file_path in files.keys() {
            if let Some(parent) = file_path.parent()
                && parent == path
                && let Some(name) = file_path.file_name()
            {
                entries.push(name.to_string_lossy().to_string());
            }
        }

        // Find all subdirectories
        for dir_path in dirs.iter() {
            if let Some(parent) = dir_path.parent()
                && parent == path
                && dir_path != path
                && let Some(name) = dir_path.file_name()
            {
                let name_str = name.to_string_lossy().to_string();
                if !entries.contains(&name_str) {
                    entries.push(name_str);
                }
            }
        }

        Ok(entries)
    }

    fn exists(&self, path: &Path) -> bool {
        self.files.lock().unwrap().contains_key(path)
            || self.dirs.lock().unwrap().contains(&path.to_path_buf())
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        let mut dirs = self.dirs.lock().unwrap();

        // Add all parent directories
        let mut current = path.to_path_buf();
        let mut parents_to_add = Vec::new();

        while let Some(parent) = current.parent() {
            if parent.as_os_str().is_empty() || parent == Path::new("/") {
                break;
            }
            parents_to_add.push(parent.to_path_buf());
            current = parent.to_path_buf();
        }

        // Add in reverse order (root to leaf)
        for parent in parents_to_add.into_iter().rev() {
            if !dirs.contains(&parent) {
                dirs.push(parent);
            }
        }

        // Add the target directory itself
        if !dirs.contains(&path.to_path_buf()) {
            dirs.push(path.to_path_buf());
        }

        Ok(())
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.lock().unwrap().contains(&path.to_path_buf())
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.lock().unwrap().contains_key(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_fs_read_write() {
        let fs = MockFsAdapter::new();
        let path = Path::new("/test.txt");

        fs.write(path, "hello world").unwrap();
        let content = fs.read_to_string(path).unwrap();

        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_mock_fs_file_not_found() {
        let fs = MockFsAdapter::new();
        let result = fs.read_to_string(Path::new("/nonexistent.txt"));

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MPCAError::PathNotFound(_)));
    }

    #[test]
    fn test_mock_fs_exists() {
        let fs = MockFsAdapter::new();
        let path = Path::new("/test.txt");

        assert!(!fs.exists(path));
        fs.write(path, "content").unwrap();
        assert!(fs.exists(path));
    }

    #[test]
    fn test_mock_fs_create_dir() {
        let fs = MockFsAdapter::new();
        let dir = Path::new("/test/dir");

        fs.create_dir_all(dir).unwrap();
        assert!(fs.is_dir(dir));
        assert!(!fs.is_file(dir));
    }

    #[test]
    fn test_mock_fs_list_dir() {
        let fs = MockFsAdapter::new();
        let dir = Path::new("/test");

        fs.create_dir_all(dir).unwrap();
        fs.write(&dir.join("file1.txt"), "content1").unwrap();
        fs.write(&dir.join("file2.txt"), "content2").unwrap();

        let entries = fs.list_dir(dir).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&"file1.txt".to_string()));
        assert!(entries.contains(&"file2.txt".to_string()));
    }

    #[test]
    fn test_mock_fs_with_files() {
        let mut files = HashMap::new();
        files.insert(PathBuf::from("/README.md"), "# Test".to_string());
        files.insert(PathBuf::from("/src/main.rs"), "fn main() {}".to_string());

        let fs = MockFsAdapter::with_files(files.clone());

        assert_eq!(
            fs.read_to_string(Path::new("/README.md")).unwrap(),
            "# Test"
        );
        assert_eq!(
            fs.read_to_string(Path::new("/src/main.rs")).unwrap(),
            "fn main() {}"
        );
    }

    #[test]
    fn test_mock_fs_clear() {
        let fs = MockFsAdapter::new();
        fs.write(Path::new("/test.txt"), "content").unwrap();
        fs.create_dir_all(Path::new("/test")).unwrap();

        assert!(fs.exists(Path::new("/test.txt")));
        assert!(fs.exists(Path::new("/test")));

        fs.clear();

        assert!(!fs.exists(Path::new("/test.txt")));
        assert!(!fs.exists(Path::new("/test")));
    }
}
