//! Prompt manager implementation using minijinja.

use crate::{
    context::PromptContext,
    engine::PromptEngine,
    error::{PromptError, Result},
};
use serde::Serialize;
use std::path::PathBuf;

/// Manager for loading and rendering prompt templates.
///
/// `PromptManager` wraps the minijinja template engine and provides
/// a convenient interface for rendering system prompts and custom templates.
///
/// # Examples
///
/// ```no_run
/// use mpca_pm::{PromptManager, PromptContext, PromptEngine};
/// use std::path::PathBuf;
///
/// let template_dir = PathBuf::from("./templates");
/// let manager = PromptManager::new(template_dir)?;
///
/// let context = PromptContext::new(PathBuf::from("/repo"));
/// let prompt = manager.render("init", &context)?;
/// # Ok::<(), mpca_pm::PromptError>(())
/// ```
#[derive(Debug)]
pub struct PromptManager {
    /// Directory containing template files.
    pub templates_dir: PathBuf,
    /// Minijinja environment for template rendering.
    env: minijinja::Environment<'static>,
}

impl PromptManager {
    /// Creates a new `PromptManager` with the specified template directory.
    ///
    /// This method initializes the minijinja environment and verifies that
    /// the template directory exists.
    ///
    /// # Arguments
    ///
    /// * `templates_dir` - Path to the directory containing `.j2` template files
    ///
    /// # Errors
    ///
    /// Returns an error if the template directory does not exist or is not accessible.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mpca_pm::PromptManager;
    /// use std::path::PathBuf;
    ///
    /// let manager = PromptManager::new(PathBuf::from("./templates"))?;
    /// # Ok::<(), mpca_pm::PromptError>(())
    /// ```
    pub fn new(templates_dir: PathBuf) -> Result<Self> {
        // Verify template directory exists
        if !templates_dir.exists() {
            return Err(PromptError::TemplateDirectoryNotFound(
                templates_dir.clone(),
            ));
        }

        if !templates_dir.is_dir() {
            return Err(PromptError::TemplateDirectoryNotFound(
                templates_dir.clone(),
            ));
        }

        // Create environment with path loader
        let mut env = minijinja::Environment::new();
        env.set_loader(minijinja::path_loader(&templates_dir));

        Ok(Self { templates_dir, env })
    }

    /// Loads a template by name.
    ///
    /// Templates are expected to have a `.j2` extension in the templates directory.
    ///
    /// # Arguments
    ///
    /// * `name` - Template name without extension (e.g., "init", "plan")
    ///
    /// # Errors
    ///
    /// Returns an error if the template file does not exist.
    fn load_template(&self, name: &str) -> Result<minijinja::Template<'_, '_>> {
        let template_name = format!("{name}.j2");
        self.env
            .get_template(&template_name)
            .map_err(|e| PromptError::TemplateNotFound(format!("{name}: {e}")))
    }
}

impl PromptEngine for PromptManager {
    fn render<T: Serialize>(&self, template: &str, ctx: &T) -> Result<String> {
        // Load and render template directly with serializable context
        let tmpl = self.load_template(template)?;
        tmpl.render(ctx)
            .map_err(|e| PromptError::TemplateRenderError(format!("{template}: {e}")))
    }

    fn get_system_prompt(&self, role: &str) -> Result<String> {
        // Use empty context for system prompts that don't require dynamic data
        let empty_context = PromptContext::default();
        self.render(role, &empty_context)
    }

    fn list_templates(&self) -> Result<Vec<String>> {
        let entries = std::fs::read_dir(&self.templates_dir).map_err(|source| {
            PromptError::TemplateListError {
                path: self.templates_dir.clone(),
                source,
            }
        })?;

        let mut templates = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|source| PromptError::TemplateListError {
                path: self.templates_dir.clone(),
                source,
            })?;

            let path = entry.path();

            // Only include .j2 files
            if path.is_file()
                && let Some(ext) = path.extension()
                && ext == "j2"
                && let Some(name) = path.file_stem()
                && let Some(name_str) = name.to_str()
            {
                templates.push(name_str.to_string());
            }
        }

        templates.sort();
        Ok(templates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_template_dir() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let templates_path = temp_dir.path().join("templates");
        fs::create_dir(&templates_path).expect("failed to create templates dir");

        // Create test templates
        fs::write(templates_path.join("test.j2"), "Hello {{ name }}!")
            .expect("failed to write test template");

        fs::write(
            templates_path.join("context.j2"),
            "Repo: {{ repo_root }}\nFeature: {{ feature_slug }}",
        )
        .expect("failed to write context template");

        (temp_dir, templates_path)
    }

    #[test]
    fn test_new_with_valid_directory() {
        let (_temp, templates_path) = create_test_template_dir();
        let manager = PromptManager::new(templates_path.clone());
        assert!(manager.is_ok());
        assert_eq!(manager.unwrap().templates_dir, templates_path);
    }

    #[test]
    fn test_new_with_nonexistent_directory() {
        let result = PromptManager::new(PathBuf::from("/nonexistent/path"));
        assert!(result.is_err());
        match result.unwrap_err() {
            PromptError::TemplateDirectoryNotFound(_) => {}
            _ => panic!("expected TemplateDirectoryNotFound error"),
        }
    }

    #[test]
    fn test_render_with_simple_context() {
        let (_temp, templates_path) = create_test_template_dir();
        let manager = PromptManager::new(templates_path).expect("failed to create manager");

        #[derive(Serialize)]
        struct TestContext {
            name: String,
        }

        let ctx = TestContext {
            name: "World".to_string(),
        };

        let result = manager.render("test", &ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World!");
    }

    #[test]
    fn test_render_with_prompt_context() {
        let (_temp, templates_path) = create_test_template_dir();
        let manager = PromptManager::new(templates_path).expect("failed to create manager");

        let ctx = PromptContext::new(PathBuf::from("/my/repo")).with_feature("my-feature");

        let result = manager.render("context", &ctx);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.contains("/my/repo"));
        assert!(rendered.contains("my-feature"));
    }

    #[test]
    fn test_render_template_not_found() {
        let (_temp, templates_path) = create_test_template_dir();
        let manager = PromptManager::new(templates_path).expect("failed to create manager");

        let ctx = PromptContext::default();
        let result = manager.render("nonexistent", &ctx);

        assert!(result.is_err());
        match result.unwrap_err() {
            PromptError::TemplateNotFound(_) => {}
            _ => panic!("expected TemplateNotFound error"),
        }
    }

    #[test]
    fn test_get_system_prompt() {
        let (_temp, templates_path) = create_test_template_dir();
        let manager = PromptManager::new(templates_path).expect("failed to create manager");

        let result = manager.get_system_prompt("test");
        assert!(result.is_ok());
        // Should use empty context, so name should be empty
        assert_eq!(result.unwrap(), "Hello !");
    }

    #[test]
    fn test_list_templates() {
        let (_temp, templates_path) = create_test_template_dir();
        let manager = PromptManager::new(templates_path).expect("failed to create manager");

        let result = manager.list_templates();
        assert!(result.is_ok());

        let templates = result.unwrap();
        assert_eq!(templates.len(), 2);
        assert!(templates.contains(&"test".to_string()));
        assert!(templates.contains(&"context".to_string()));
    }

    #[test]
    fn test_list_templates_empty_directory() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let templates_path = temp_dir.path().join("empty");
        fs::create_dir(&templates_path).expect("failed to create templates dir");

        let manager = PromptManager::new(templates_path).expect("failed to create manager");
        let result = manager.list_templates();

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_context_builder_pattern() {
        let context = PromptContext::new(PathBuf::from("/repo"))
            .with_feature("test-feature")
            .with_spec_paths(vec![PathBuf::from("specs/design.md")])
            .with_resume(true);

        assert_eq!(context.repo_root, PathBuf::from("/repo"));
        assert_eq!(context.feature_slug, Some("test-feature".to_string()));
        assert_eq!(context.spec_paths.len(), 1);
        assert!(context.resume);
    }
}
