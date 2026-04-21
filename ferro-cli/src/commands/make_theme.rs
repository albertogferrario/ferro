//! make:theme command — scaffold theme files for the Ferro theme system
//!
//! Scaffolds plain CSS variable declarations (`:root { ... }`) compatible
//! with ferro-json-ui's `<style>` injection path.

use console::style;
use std::fs;
use std::path::Path;

/// Generate theme scaffold files in `themes/{name}/`.
///
/// Creates:
/// - `themes/{name}/tokens.css` — plain CSS `:root { ... }` block with all 23 semantic token slots
/// - `themes/{name}/theme.json` — empty JSON object for partial intent template overrides
///
/// Returns an error if `themes/{name}/` already exists.
pub fn make_theme(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    make_theme_in_dir(name, Path::new("."))
}

/// Core logic — generate in a given base directory (enables testability).
pub fn make_theme_in_dir(name: &str, base: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let theme_dir = base.join("themes").join(name);

    if theme_dir.exists() {
        return Err(format!("Theme directory '{}' already exists", theme_dir.display()).into());
    }

    fs::create_dir_all(&theme_dir)?;

    // Write tokens.css — plain CSS variables, injectable into <style> without Tailwind processing
    let tokens_path = theme_dir.join("tokens.css");
    fs::write(&tokens_path, tokens_css_template())?;

    // Write theme.json — empty object for partial intent template overrides
    let theme_json_path = theme_dir.join("theme.json");
    fs::write(&theme_json_path, "{}\n")?;

    println!(
        "{} Created theme '{}' at {}/",
        style("✓").green(),
        style(name).cyan().bold(),
        theme_dir.display()
    );
    println!();
    println!("Next steps:");
    println!(
        "  {} Edit {}/tokens.css to customize colors, shapes, and typography",
        style("1.").dim(),
        theme_dir.display()
    );
    println!(
        "  {} Add intent template overrides to {}/theme.json",
        style("2.").dim(),
        theme_dir.display()
    );
    println!(
        "  {} Activate the theme via ThemeMiddleware in your application",
        style("3.").dim()
    );
    println!();

    Ok(())
}

/// Public entry point called from main.rs.
pub fn run(name: &str) {
    if let Err(e) = make_theme(name) {
        eprintln!("{} {}", style("Error:").red().bold(), e);
        std::process::exit(1);
    }
}

fn tokens_css_template() -> &'static str {
    r#"/* Theme tokens — plain CSS variables.
 *
 * Injected into <style> by ferro-json-ui at render time.
 * MUST use standard CSS (:root { ... }) — not Tailwind's @theme syntax,
 * which only works under the Tailwind browser runtime (dev-only).
 */

:root {
  /* Surface tokens */
  --color-background: oklch(100% 0 0);
  --color-surface: oklch(97% 0 0);
  --color-card: oklch(95% 0 0);
  --color-border: oklch(90% 0 0);
  --color-text: oklch(15% 0 0);
  --color-text-muted: oklch(50% 0 0);

  /* Role tokens */
  --color-primary: oklch(55% 0.2 250);
  --color-primary-foreground: oklch(100% 0 0);
  --color-secondary: oklch(70% 0.05 250);
  --color-secondary-foreground: oklch(15% 0 0);
  --color-accent: oklch(65% 0.15 200);
  --color-destructive: oklch(55% 0.22 25);
  --color-success: oklch(55% 0.18 145);
  --color-warning: oklch(70% 0.18 80);

  /* Shape tokens */
  --radius-sm: 0.25rem;
  --radius-md: 0.375rem;
  --radius-lg: 0.5rem;
  --radius-full: 9999px;

  /* Shadow tokens */
  --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1);

  /* Typography tokens */
  --font-sans: "Inter", ui-sans-serif, system-ui, sans-serif;
  --font-mono: ui-monospace, monospace;
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-background: oklch(12% 0 0);
    --color-surface: oklch(17% 0 0);
    --color-card: oklch(20% 0 0);
    --color-border: oklch(30% 0 0);
    --color-text: oklch(95% 0 0);
    --color-text-muted: oklch(60% 0 0);
    --color-primary: oklch(65% 0.2 250);
    --color-primary-foreground: oklch(100% 0 0);
    --color-secondary: oklch(60% 0.05 250);
    --color-secondary-foreground: oklch(95% 0 0);
    --color-accent: oklch(60% 0.15 200);
    --color-destructive: oklch(60% 0.22 25);
    --color-success: oklch(60% 0.18 145);
    --color-warning: oklch(65% 0.18 80);
  }
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn read_file(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
    }

    #[test]
    fn test_make_theme_creates_directory_structure() {
        let tmp = TempDir::new().unwrap();
        make_theme_in_dir("myapp", tmp.path()).unwrap();

        assert!(tmp.path().join("themes/myapp").exists());
        assert!(tmp.path().join("themes/myapp/tokens.css").exists());
        assert!(tmp.path().join("themes/myapp/theme.json").exists());
    }

    #[test]
    fn test_make_theme_tokens_css_has_all_23_token_slots() {
        let tmp = TempDir::new().unwrap();
        make_theme_in_dir("test", tmp.path()).unwrap();

        let css = read_file(&tmp.path().join("themes/test/tokens.css"));

        // Surface tokens (6)
        assert!(
            css.contains("--color-background:"),
            "missing --color-background"
        );
        assert!(css.contains("--color-surface:"), "missing --color-surface");
        assert!(css.contains("--color-card:"), "missing --color-card");
        assert!(css.contains("--color-border:"), "missing --color-border");
        assert!(css.contains("--color-text:"), "missing --color-text");
        assert!(
            css.contains("--color-text-muted:"),
            "missing --color-text-muted"
        );

        // Role tokens (8)
        assert!(css.contains("--color-primary:"), "missing --color-primary");
        assert!(
            css.contains("--color-primary-foreground:"),
            "missing --color-primary-foreground"
        );
        assert!(
            css.contains("--color-secondary:"),
            "missing --color-secondary"
        );
        assert!(
            css.contains("--color-secondary-foreground:"),
            "missing --color-secondary-foreground"
        );
        assert!(css.contains("--color-accent:"), "missing --color-accent");
        assert!(
            css.contains("--color-destructive:"),
            "missing --color-destructive"
        );
        assert!(css.contains("--color-success:"), "missing --color-success");
        assert!(css.contains("--color-warning:"), "missing --color-warning");

        // Shape tokens (4)
        assert!(css.contains("--radius-sm:"), "missing --radius-sm");
        assert!(css.contains("--radius-md:"), "missing --radius-md");
        assert!(css.contains("--radius-lg:"), "missing --radius-lg");
        assert!(css.contains("--radius-full:"), "missing --radius-full");

        // Shadow tokens (3)
        assert!(css.contains("--shadow-sm:"), "missing --shadow-sm");
        assert!(css.contains("--shadow-md:"), "missing --shadow-md");
        assert!(css.contains("--shadow-lg:"), "missing --shadow-lg");

        // Typography tokens (2)
        assert!(css.contains("--font-sans:"), "missing --font-sans");
        assert!(css.contains("--font-mono:"), "missing --font-mono");
    }

    #[test]
    fn test_make_theme_tokens_css_has_root_block_and_no_tailwind_syntax() {
        let tmp = TempDir::new().unwrap();
        make_theme_in_dir("test", tmp.path()).unwrap();

        let css = read_file(&tmp.path().join("themes/test/tokens.css"));

        // Must NOT contain Tailwind-CDN-only syntax.
        assert!(
            !css.contains("@import \"tailwindcss\""),
            "scaffolded tokens.css must not contain @import \"tailwindcss\" — it is Tailwind-CDN-specific"
        );
        assert!(
            !css.contains("@theme {"),
            "scaffolded tokens.css must not contain @theme {{...}} — it is Tailwind-CDN-specific"
        );

        // Must contain plain CSS :root block with the primary token.
        assert!(css.contains(":root {"), "missing :root {{...}} block");
        assert!(
            css.contains("--color-primary:"),
            "missing --color-primary declaration"
        );
    }

    #[test]
    fn test_make_theme_tokens_css_has_dark_mode_block() {
        let tmp = TempDir::new().unwrap();
        make_theme_in_dir("test", tmp.path()).unwrap();

        let css = read_file(&tmp.path().join("themes/test/tokens.css"));

        assert!(
            css.contains("@media (prefers-color-scheme: dark)"),
            "missing dark mode @media"
        );
        assert!(
            css.contains("oklch(12%"),
            "missing dark mode background value"
        );
        // Dark mode block must use :root, not @theme { ... }.
        assert!(
            !css.contains("@theme {"),
            "dark-mode block must use :root {{...}}, not @theme {{...}}"
        );
    }

    #[test]
    fn test_make_theme_theme_json_is_empty_object() {
        let tmp = TempDir::new().unwrap();
        make_theme_in_dir("test", tmp.path()).unwrap();

        let json_content = read_file(&tmp.path().join("themes/test/theme.json"));
        let trimmed = json_content.trim();

        // Must be valid JSON that deserializes to an empty object
        let parsed: serde_json::Value =
            serde_json::from_str(trimmed).expect("theme.json must be valid JSON");
        assert_eq!(
            parsed,
            serde_json::json!({}),
            "theme.json must be empty object"
        );
    }

    #[test]
    fn test_make_theme_fails_if_directory_exists() {
        let tmp = TempDir::new().unwrap();

        // Create the directory first
        fs::create_dir_all(tmp.path().join("themes/duplicate")).unwrap();

        let result = make_theme_in_dir("duplicate", tmp.path());
        assert!(
            result.is_err(),
            "should return error for duplicate theme name"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("already exists"),
            "error should mention 'already exists'"
        );
    }

    #[test]
    fn test_make_theme_succeeds_once_fails_on_repeat() {
        let tmp = TempDir::new().unwrap();

        // First call succeeds
        make_theme_in_dir("myapp", tmp.path()).unwrap();

        // Second call with same name fails
        let result = make_theme_in_dir("myapp", tmp.path());
        assert!(result.is_err());
    }
}
