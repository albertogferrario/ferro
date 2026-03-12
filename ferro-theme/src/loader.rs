use std::path::Path;

use crate::error::ThemeError;
use crate::template::ThemeTemplates;

const DEFAULT_THEME_CSS: &str = include_str!("../assets/default.css");

/// A loaded theme: CSS tokens and optional intent template overrides.
///
/// Two construction paths:
/// - [`Theme::default_theme()`] — always available, embedded at compile time.
/// - [`Theme::from_path()`] — loads from a filesystem directory.
#[derive(Debug, Clone)]
pub struct Theme {
    /// CSS content using Tailwind v4 `@theme` syntax.
    pub css: String,

    /// Optional intent template overrides; all-`None` means built-in layouts apply.
    pub templates: ThemeTemplates,
}

impl Theme {
    /// Returns the embedded default theme.
    ///
    /// The CSS uses Tailwind v4 `@theme` with 23 semantic token slots (light + dark).
    /// Templates are all-`None` — built-in intent layouts apply unchanged.
    pub fn default_theme() -> Self {
        Self {
            css: DEFAULT_THEME_CSS.to_string(),
            templates: ThemeTemplates::default(),
        }
    }

    /// Loads a theme from a directory on the filesystem.
    ///
    /// Expects `tokens.css` to exist in the directory.
    /// `theme.json` is optional — if absent, templates default to empty (`ThemeTemplates::default()`).
    ///
    /// # Errors
    ///
    /// - [`ThemeError::NotFound`] — directory does not exist.
    /// - [`ThemeError::Io`] — `tokens.css` cannot be read.
    /// - [`ThemeError::Json`] — `theme.json` exists but cannot be parsed.
    pub fn from_path(path: &str) -> Result<Self, ThemeError> {
        let dir = Path::new(path);
        if !dir.exists() {
            return Err(ThemeError::NotFound(path.to_string()));
        }

        let css_path = dir.join("tokens.css");
        let css = std::fs::read_to_string(css_path)?;

        let templates_path = dir.join("theme.json");
        let templates = if templates_path.exists() {
            let json = std::fs::read_to_string(&templates_path)?;
            serde_json::from_str(&json)?
        } else {
            ThemeTemplates::default()
        };

        Ok(Self { css, templates })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_returns_non_empty_css_with_color_primary() {
        let theme = Theme::default_theme();
        assert!(!theme.css.is_empty());
        assert!(theme.css.contains("--color-primary"));
    }

    #[test]
    fn default_theme_returns_all_none_templates() {
        let theme = Theme::default_theme();
        assert!(theme.templates.browse.is_none());
        assert!(theme.templates.focus.is_none());
        assert!(theme.templates.collect.is_none());
        assert!(theme.templates.process.is_none());
        assert!(theme.templates.summarize.is_none());
        assert!(theme.templates.analyze.is_none());
        assert!(theme.templates.track.is_none());
    }

    #[test]
    fn from_path_loads_tokens_css_and_theme_json() {
        let dir = tempfile::tempdir().unwrap();
        let css_content = "@theme { --color-primary: oklch(55% 0.2 250); }";
        let json_content = r#"{"browse": {"display": {"slots": ["title", "fields"]}}}"#;
        std::fs::write(dir.path().join("tokens.css"), css_content).unwrap();
        std::fs::write(dir.path().join("theme.json"), json_content).unwrap();

        let theme = Theme::from_path(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(theme.css, css_content);
        assert!(theme.templates.browse.is_some());
        let browse = theme.templates.browse.unwrap();
        assert_eq!(browse.display.slots, vec!["title", "fields"]);
    }

    #[test]
    fn from_path_works_without_theme_json() {
        let dir = tempfile::tempdir().unwrap();
        let css_content = "@theme { --color-primary: oklch(55% 0.2 250); }";
        std::fs::write(dir.path().join("tokens.css"), css_content).unwrap();

        let theme = Theme::from_path(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(theme.css, css_content);
        assert!(theme.templates.browse.is_none());
    }

    #[test]
    fn from_path_returns_not_found_for_nonexistent_directory() {
        let result = Theme::from_path("/nonexistent/path/that/does/not/exist");
        assert!(matches!(result, Err(ThemeError::NotFound(_))));
    }

    #[test]
    fn from_path_returns_json_error_for_invalid_theme_json() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("tokens.css"), "@theme {}").unwrap();
        std::fs::write(dir.path().join("theme.json"), "not valid json{{").unwrap();

        let result = Theme::from_path(dir.path().to_str().unwrap());
        assert!(matches!(result, Err(ThemeError::Json(_))));
    }

    #[test]
    fn from_path_returns_io_error_when_tokens_css_missing() {
        let dir = tempfile::tempdir().unwrap();
        // Only create theme.json, not tokens.css
        std::fs::write(dir.path().join("theme.json"), "{}").unwrap();

        let result = Theme::from_path(dir.path().to_str().unwrap());
        assert!(matches!(result, Err(ThemeError::Io(_))));
    }
}
