//! Project structure analyzer for convention detection.
//!
//! Scans project directories to detect existing patterns and conventions,
//! enabling smart defaults for CLI commands like make:scaffold.

// Allow unused code warnings as this module is used by make_scaffold in subsequent tasks
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

/// Information about a detected foreign key relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKeyInfo {
    /// The field name (e.g., "user_id")
    pub field_name: String,
    /// The target model name in PascalCase (e.g., "User")
    pub target_model: String,
    /// The target table name in snake_case plural (e.g., "users")
    pub target_table: String,
    /// Whether the target model exists in the project
    pub validated: bool,
}

/// Pattern for test file organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestPattern {
    /// No test files detected
    None,
    /// Tests organized per controller (e.g., user_controller_test.rs)
    PerController,
    /// Tests in a unified test file
    Unified,
}

/// Pattern for factory file organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactoryPattern {
    /// No factory files detected
    None,
    /// Factories organized per model (e.g., user_factory.rs)
    PerModel,
    /// Factories in a unified factory file
    Unified,
}

/// Detected project conventions and structure.
#[derive(Debug, Clone)]
pub struct ProjectConventions {
    /// Whether src/tests/ directory exists
    pub has_tests_dir: bool,
    /// Whether src/factories/ directory exists
    pub has_factories_dir: bool,
    /// Whether frontend/src/pages/ directory exists with content
    pub has_inertia_pages: bool,
    /// List of existing model names (from src/models/)
    pub existing_models: Vec<String>,
    /// Detected test file organization pattern
    pub test_pattern: TestPattern,
    /// Detected factory file organization pattern
    pub factory_pattern: FactoryPattern,
    /// Number of existing test files
    pub test_file_count: usize,
    /// Number of existing factory files
    pub factory_file_count: usize,
}

impl Default for ProjectConventions {
    fn default() -> Self {
        Self {
            has_tests_dir: false,
            has_factories_dir: false,
            has_inertia_pages: false,
            existing_models: Vec::new(),
            test_pattern: TestPattern::None,
            factory_pattern: FactoryPattern::None,
            test_file_count: 0,
            factory_file_count: 0,
        }
    }
}

/// Analyzer for detecting project conventions and patterns.
pub struct ProjectAnalyzer {
    root: PathBuf,
}

impl ProjectAnalyzer {
    /// Create a new analyzer for the given project root.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Create an analyzer for the current working directory.
    pub fn current_dir() -> Self {
        Self::new(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    /// Analyze the project structure and return detected conventions.
    pub fn analyze(&self) -> ProjectConventions {
        let mut conventions = ProjectConventions::default();

        // Detect tests directory and pattern
        self.detect_tests(&mut conventions);

        // Detect factories directory and pattern
        self.detect_factories(&mut conventions);

        // Detect Inertia pages
        self.detect_inertia_pages(&mut conventions);

        // Detect existing models
        self.detect_models(&mut conventions);

        conventions
    }

    /// Detect test directory presence and pattern.
    fn detect_tests(&self, conventions: &mut ProjectConventions) {
        let tests_dir = self.root.join("src/tests");

        if !tests_dir.exists() || !tests_dir.is_dir() {
            return;
        }

        conventions.has_tests_dir = true;

        // Count test files and detect pattern
        let test_files = self.count_files_matching(&tests_dir, "_controller_test.rs");
        let unified_test = self.file_exists(&tests_dir, "tests.rs");

        conventions.test_file_count = test_files;

        if test_files > 0 {
            conventions.test_pattern = TestPattern::PerController;
        } else if unified_test {
            conventions.test_pattern = TestPattern::Unified;
        }
    }

    /// Detect factories directory presence and pattern.
    fn detect_factories(&self, conventions: &mut ProjectConventions) {
        let factories_dir = self.root.join("src/factories");

        if !factories_dir.exists() || !factories_dir.is_dir() {
            return;
        }

        conventions.has_factories_dir = true;

        // Count factory files and detect pattern
        let factory_files = self.count_files_matching(&factories_dir, "_factory.rs");
        let unified_factory = self.file_exists(&factories_dir, "factory.rs");

        conventions.factory_file_count = factory_files;

        if factory_files > 0 {
            conventions.factory_pattern = FactoryPattern::PerModel;
        } else if unified_factory {
            conventions.factory_pattern = FactoryPattern::Unified;
        }
    }

    /// Detect Inertia pages directory presence and content.
    fn detect_inertia_pages(&self, conventions: &mut ProjectConventions) {
        let pages_dir = self.root.join("frontend/src/pages");

        if !pages_dir.exists() || !pages_dir.is_dir() {
            return;
        }

        // Check if there are any .tsx files (indicating Inertia pages)
        if let Ok(entries) = fs::read_dir(&pages_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // Check for either .tsx files or subdirectories with .tsx files
                if path.is_dir() {
                    if self.has_tsx_files(&path) {
                        conventions.has_inertia_pages = true;
                        return;
                    }
                } else if path.extension().is_some_and(|ext| ext == "tsx") {
                    conventions.has_inertia_pages = true;
                    return;
                }
            }
        }
    }

    /// Detect existing models from src/models/ directory.
    fn detect_models(&self, conventions: &mut ProjectConventions) {
        let models_dir = self.root.join("src/models");

        if !models_dir.exists() || !models_dir.is_dir() {
            return;
        }

        if let Ok(entries) = fs::read_dir(&models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_stem() {
                        let name_str = name.to_string_lossy().to_string();
                        // Skip mod.rs
                        if name_str != "mod" {
                            conventions.existing_models.push(name_str);
                        }
                    }
                }
            }
        }

        conventions.existing_models.sort();
    }

    /// Count files in a directory matching a suffix pattern.
    fn count_files_matching(&self, dir: &Path, suffix: &str) -> usize {
        let Ok(entries) = fs::read_dir(dir) else {
            return 0;
        };

        entries
            .filter_map(Result::ok)
            .filter(|e| {
                e.path().is_file()
                    && e.path()
                        .file_name()
                        .is_some_and(|n| n.to_string_lossy().ends_with(suffix))
            })
            .count()
    }

    /// Check if a specific file exists in a directory.
    fn file_exists(&self, dir: &Path, filename: &str) -> bool {
        dir.join(filename).exists()
    }

    /// Check if a directory contains any .tsx files.
    fn has_tsx_files(&self, dir: &Path) -> bool {
        let Ok(entries) = fs::read_dir(dir) else {
            return false;
        };

        entries.filter_map(Result::ok).any(|e| {
            let path = e.path();
            path.is_file() && path.extension().is_some_and(|ext| ext == "tsx")
        })
    }

    /// List all existing model names (snake_case) in the project.
    pub fn list_models(&self) -> Vec<String> {
        let models_dir = self.root.join("src/models");

        if !models_dir.exists() || !models_dir.is_dir() {
            return Vec::new();
        }

        let mut models = Vec::new();
        if let Ok(entries) = fs::read_dir(&models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_stem() {
                        let name_str = name.to_string_lossy().to_string();
                        // Skip mod.rs
                        if name_str != "mod" {
                            models.push(name_str);
                        }
                    }
                }
            }
        }
        models.sort();
        models
    }

    /// Check if a model exists in the project (case-insensitive).
    pub fn model_exists(&self, name: &str) -> bool {
        let models = self.list_models();
        let name_lower = name.to_lowercase();

        models.iter().any(|m| {
            m.to_lowercase() == name_lower || to_pascal_case(m).to_lowercase() == name_lower
        })
    }

    /// Detect foreign key relationships from a list of fields.
    ///
    /// Fields ending in `_id` are considered potential foreign keys.
    /// Returns information about each detected FK including whether
    /// the target model exists in the project.
    pub fn detect_foreign_keys(&self, fields: &[(&str, &str)]) -> Vec<ForeignKeyInfo> {
        let mut fks = Vec::new();

        for (field_name, _field_type) in fields {
            if let Some(prefix) = field_name.strip_suffix("_id") {
                // Skip "id" field itself
                if prefix.is_empty() {
                    continue;
                }

                let target_model = to_pascal_case(prefix);
                let target_table = to_plural(prefix);
                let validated = self.model_exists(&target_model);

                fks.push(ForeignKeyInfo {
                    field_name: field_name.to_string(),
                    target_model,
                    target_table,
                    validated,
                });
            }
        }

        fks
    }
}

/// Convert snake_case to PascalCase.
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Convert snake_case singular to snake_case plural.
fn to_plural(s: &str) -> String {
    if s.ends_with('s') || s.ends_with('x') || s.ends_with("ch") || s.ends_with("sh") {
        format!("{s}es")
    } else if s.ends_with('y')
        && !s.ends_with("ay")
        && !s.ends_with("ey")
        && !s.ends_with("oy")
        && !s.ends_with("uy")
    {
        format!("{}ies", &s[..s.len() - 1])
    } else {
        format!("{s}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn test_analyzer_detects_empty_project() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path().to_path_buf());
        let conventions = analyzer.analyze();

        assert!(!conventions.has_tests_dir);
        assert!(!conventions.has_factories_dir);
        assert!(!conventions.has_inertia_pages);
        assert!(conventions.existing_models.is_empty());
        assert_eq!(conventions.test_pattern, TestPattern::None);
        assert_eq!(conventions.factory_pattern, FactoryPattern::None);
    }

    #[test]
    fn test_analyzer_detects_tests_directory() {
        let temp = create_test_project();
        let tests_dir = temp.path().join("src/tests");
        fs::create_dir_all(&tests_dir).unwrap();
        fs::write(tests_dir.join("user_controller_test.rs"), "").unwrap();
        fs::write(tests_dir.join("post_controller_test.rs"), "").unwrap();

        let analyzer = ProjectAnalyzer::new(temp.path().to_path_buf());
        let conventions = analyzer.analyze();

        assert!(conventions.has_tests_dir);
        assert_eq!(conventions.test_pattern, TestPattern::PerController);
        assert_eq!(conventions.test_file_count, 2);
    }

    #[test]
    fn test_analyzer_detects_factories_directory() {
        let temp = create_test_project();
        let factories_dir = temp.path().join("src/factories");
        fs::create_dir_all(&factories_dir).unwrap();
        fs::write(factories_dir.join("user_factory.rs"), "").unwrap();
        fs::write(factories_dir.join("mod.rs"), "").unwrap();

        let analyzer = ProjectAnalyzer::new(temp.path().to_path_buf());
        let conventions = analyzer.analyze();

        assert!(conventions.has_factories_dir);
        assert_eq!(conventions.factory_pattern, FactoryPattern::PerModel);
        assert_eq!(conventions.factory_file_count, 1);
    }

    #[test]
    fn test_analyzer_detects_inertia_pages() {
        let temp = create_test_project();
        let pages_dir = temp.path().join("frontend/src/pages/users");
        fs::create_dir_all(&pages_dir).unwrap();
        fs::write(pages_dir.join("Index.tsx"), "").unwrap();

        let analyzer = ProjectAnalyzer::new(temp.path().to_path_buf());
        let conventions = analyzer.analyze();

        assert!(conventions.has_inertia_pages);
    }

    #[test]
    fn test_analyzer_detects_models() {
        let temp = create_test_project();
        let models_dir = temp.path().join("src/models");
        fs::create_dir_all(&models_dir).unwrap();
        fs::write(models_dir.join("user.rs"), "").unwrap();
        fs::write(models_dir.join("post.rs"), "").unwrap();
        fs::write(models_dir.join("mod.rs"), "").unwrap();

        let analyzer = ProjectAnalyzer::new(temp.path().to_path_buf());
        let conventions = analyzer.analyze();

        assert_eq!(conventions.existing_models, vec!["post", "user"]);
    }

    #[test]
    fn test_detect_foreign_keys_simple() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path().to_path_buf());

        let fields = [("user_id", "bigint"), ("title", "string")];
        let fks = analyzer.detect_foreign_keys(&fields);

        assert_eq!(fks.len(), 1);
        assert_eq!(fks[0].field_name, "user_id");
        assert_eq!(fks[0].target_model, "User");
        assert_eq!(fks[0].target_table, "users");
        assert!(!fks[0].validated); // No model exists
    }

    #[test]
    fn test_detect_foreign_keys_validated() {
        let temp = create_test_project();
        let models_dir = temp.path().join("src/models");
        fs::create_dir_all(&models_dir).unwrap();
        fs::write(models_dir.join("user.rs"), "").unwrap();

        let analyzer = ProjectAnalyzer::new(temp.path().to_path_buf());

        let fields = [("user_id", "bigint"), ("category_id", "bigint")];
        let fks = analyzer.detect_foreign_keys(&fields);

        assert_eq!(fks.len(), 2);

        // user_id should be validated (model exists)
        let user_fk = fks.iter().find(|f| f.field_name == "user_id").unwrap();
        assert!(user_fk.validated);

        // category_id should not be validated (no model)
        let category_fk = fks.iter().find(|f| f.field_name == "category_id").unwrap();
        assert!(!category_fk.validated);
    }

    #[test]
    fn test_detect_foreign_keys_compound_name() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path().to_path_buf());

        let fields = [("blog_post_id", "bigint")];
        let fks = analyzer.detect_foreign_keys(&fields);

        assert_eq!(fks.len(), 1);
        assert_eq!(fks[0].field_name, "blog_post_id");
        assert_eq!(fks[0].target_model, "BlogPost");
        assert_eq!(fks[0].target_table, "blog_posts");
    }

    #[test]
    fn test_detect_foreign_keys_ignores_id_field() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path().to_path_buf());

        // "id" field should not be detected as FK
        let fields = [("id", "bigint"), ("user_id", "bigint")];
        let fks = analyzer.detect_foreign_keys(&fields);

        assert_eq!(fks.len(), 1);
        assert_eq!(fks[0].field_name, "user_id");
    }

    #[test]
    fn test_detect_foreign_keys_pluralization() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path().to_path_buf());

        let fields = [
            ("category_id", "bigint"), // category -> categories
            ("status_id", "bigint"),   // status -> statuses
            ("box_id", "bigint"),      // box -> boxes
            ("company_id", "bigint"),  // company -> companies
            ("day_id", "bigint"),      // day -> days (vowel + y)
        ];
        let fks = analyzer.detect_foreign_keys(&fields);

        assert_eq!(fks.len(), 5);

        let tables: Vec<_> = fks.iter().map(|f| f.target_table.as_str()).collect();
        assert!(tables.contains(&"categories"));
        assert!(tables.contains(&"statuses"));
        assert!(tables.contains(&"boxes"));
        assert!(tables.contains(&"companies"));
        assert!(tables.contains(&"days"));
    }

    #[test]
    fn test_model_exists_case_insensitive() {
        let temp = create_test_project();
        let models_dir = temp.path().join("src/models");
        fs::create_dir_all(&models_dir).unwrap();
        fs::write(models_dir.join("user.rs"), "").unwrap();
        fs::write(models_dir.join("blog_post.rs"), "").unwrap();

        let analyzer = ProjectAnalyzer::new(temp.path().to_path_buf());

        // Should match by snake_case name
        assert!(analyzer.model_exists("user"));
        assert!(analyzer.model_exists("USER"));
        assert!(analyzer.model_exists("blog_post"));

        // Should match by PascalCase name
        assert!(analyzer.model_exists("User"));
        assert!(analyzer.model_exists("BlogPost"));

        // Should not match non-existent
        assert!(!analyzer.model_exists("category"));
        assert!(!analyzer.model_exists("Category"));
    }

    #[test]
    fn test_list_models() {
        let temp = create_test_project();
        let models_dir = temp.path().join("src/models");
        fs::create_dir_all(&models_dir).unwrap();
        fs::write(models_dir.join("user.rs"), "").unwrap();
        fs::write(models_dir.join("post.rs"), "").unwrap();
        fs::write(models_dir.join("mod.rs"), "").unwrap();

        let analyzer = ProjectAnalyzer::new(temp.path().to_path_buf());
        let models = analyzer.list_models();

        assert_eq!(models, vec!["post", "user"]);
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user"), "User");
        assert_eq!(to_pascal_case("blog_post"), "BlogPost");
        assert_eq!(
            to_pascal_case("user_profile_settings"),
            "UserProfileSettings"
        );
    }

    #[test]
    fn test_to_plural() {
        assert_eq!(to_plural("user"), "users");
        assert_eq!(to_plural("category"), "categories");
        assert_eq!(to_plural("status"), "statuses");
        assert_eq!(to_plural("box"), "boxes");
        assert_eq!(to_plural("day"), "days");
        assert_eq!(to_plural("key"), "keys");
    }
}
