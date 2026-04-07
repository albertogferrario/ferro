//! Parses `ignore_patterns.toml` (single source of truth, D-18) and renders
//! both `.dockerignore` (D-19) and `.gitignore` from it.
//!
//! See `ferro ignore:sync` (D-20) for the reconciler that consumes these
//! renderers in existing user projects.

use serde::Deserialize;

/// Compile-time embedded copy of the canonical ignore patterns file.
pub const IGNORE_PATTERNS_TOML: &str = include_str!("files/root/ignore_patterns.toml");

const HEADER: &str =
    "# Generated from templates/files/root/ignore_patterns.toml — edit there, run ferro ignore:sync";

#[derive(Debug, Clone, Deserialize)]
pub struct IgnorePatterns {
    #[serde(rename = "category", default)]
    pub categories: Vec<Category>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Category {
    pub name: String,
    #[serde(default)]
    pub for_git: bool,
    #[serde(default)]
    pub for_docker: bool,
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub git_patterns: Vec<String>,
    #[serde(default)]
    pub docker_patterns: Vec<String>,
}

/// Parse a TOML string into [`IgnorePatterns`]. Returns the toml error message
/// stringified on failure (callers don't need richer typing).
pub fn parse_ignore_patterns(toml_str: &str) -> Result<IgnorePatterns, String> {
    toml::from_str::<IgnorePatterns>(toml_str).map_err(|e| e.to_string())
}

/// Parse the embedded canonical ignore_patterns.toml. Panics on malformed
/// input — the file is shipped with the binary, so a parse error is a
/// compile-time-equivalent bug.
pub fn load_default() -> IgnorePatterns {
    parse_ignore_patterns(IGNORE_PATTERNS_TOML)
        .expect("embedded ignore_patterns.toml must be valid TOML")
}

/// Render the .dockerignore file from `p`. Output is byte-deterministic given
/// stable input ordering: header, then for each category in declaration order
/// where `for_docker` is true, a blank line, a `# {name}` comment, and then
/// the merged `patterns` + `docker_patterns` (in that order, declaration order
/// preserved). Trailing newline at EOF.
pub fn render_dockerignore(p: &IgnorePatterns) -> String {
    render(p, RenderTarget::Docker)
}

/// Render the .gitignore file from `p`. Same shape as
/// [`render_dockerignore`], but for `for_git` categories using `patterns` +
/// `git_patterns`.
pub fn render_gitignore(p: &IgnorePatterns) -> String {
    render(p, RenderTarget::Git)
}

#[derive(Copy, Clone)]
enum RenderTarget {
    Docker,
    Git,
}

fn render(p: &IgnorePatterns, target: RenderTarget) -> String {
    let mut out = String::new();
    out.push_str(HEADER);
    out.push('\n');

    for cat in &p.categories {
        let included = match target {
            RenderTarget::Docker => cat.for_docker,
            RenderTarget::Git => cat.for_git,
        };
        if !included {
            continue;
        }

        let mut lines: Vec<&str> = cat.patterns.iter().map(String::as_str).collect();
        match target {
            RenderTarget::Docker => lines.extend(cat.docker_patterns.iter().map(String::as_str)),
            RenderTarget::Git => lines.extend(cat.git_patterns.iter().map(String::as_str)),
        }

        if lines.is_empty() {
            continue;
        }

        out.push('\n');
        out.push_str("# ");
        out.push_str(&cat.name);
        out.push('\n');
        for line in lines {
            out.push_str(line);
            out.push('\n');
        }
    }

    out
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_patterns() -> IgnorePatterns {
        load_default()
    }

    #[test]
    fn parses_embedded_toml() {
        let p = default_patterns();
        assert!(!p.categories.is_empty());
        let names: Vec<&str> = p.categories.iter().map(|c| c.name.as_str()).collect();
        for required in [
            "rust",
            "node",
            "ide",
            "env",
            "sqlite",
            "planning",
            "storage",
            "secrets",
        ] {
            assert!(
                names.contains(&required),
                "missing required category: {required}"
            );
        }
    }

    #[test]
    fn dockerignore_includes_docker_only_patterns() {
        let out = render_dockerignore(&default_patterns());
        // docker category exists only in dockerignore
        assert!(out.contains("Dockerfile"));
        assert!(out.contains(".dockerignore"));
        // git category (.git/) is dockerignore-only
        assert!(out.contains(".git/"));
        // planning is dockerignore-only
        assert!(out.contains(".planning/"));
        // storage is dockerignore-only
        assert!(out.contains("storage/"));
        assert!(out.contains("data/"));
    }

    #[test]
    fn gitignore_includes_git_only_patterns() {
        let out = render_gitignore(&default_patterns());
        // /target with leading slash is git-only
        assert!(out.contains("/target"));
        assert!(out.contains("Cargo.lock"));
        // generated_types is git-only
        assert!(out.contains("frontend/src/types/"));
        // /public/assets (leading slash) is git-only
        assert!(out.contains("/public/assets"));
    }

    #[test]
    fn dockerignore_excludes_git_only_categories() {
        let out = render_dockerignore(&default_patterns());
        assert!(!out.contains("frontend/src/types/"));
        assert!(!out.contains("/target"));
        assert!(!out.contains("Cargo.lock"));
    }

    #[test]
    fn gitignore_excludes_docker_only_categories() {
        let out = render_gitignore(&default_patterns());
        assert!(!out.contains("Dockerfile"));
        assert!(!out.contains(".dockerignore"));
        assert!(!out.contains(".planning/"));
        assert!(!out.contains("storage/"));
        // sqlite is docker-only in our SoT
        assert!(!out.contains("database.db"));
    }

    #[test]
    fn render_is_byte_deterministic() {
        let p = default_patterns();
        let a = render_dockerignore(&p);
        let b = render_dockerignore(&p);
        assert_eq!(a, b);
        let g1 = render_gitignore(&p);
        let g2 = render_gitignore(&p);
        assert_eq!(g1, g2);
    }

    #[test]
    fn render_starts_with_header_and_ends_with_newline() {
        let out = render_dockerignore(&default_patterns());
        assert!(out.starts_with(HEADER));
        assert!(out.ends_with('\n'));
        let out = render_gitignore(&default_patterns());
        assert!(out.starts_with(HEADER));
        assert!(out.ends_with('\n'));
    }

    /// Migration safety net: every non-comment, non-blank line currently in
    /// dockerignore.tpl must be present in the rendered output.
    #[test]
    fn dockerignore_migration_safety_net() {
        let old = include_str!("files/docker/dockerignore.tpl");
        let new = render_dockerignore(&default_patterns());
        assert_subset(old, &new, "dockerignore");
    }

    /// Migration safety net: every non-comment, non-blank line currently in
    /// gitignore.tpl must be present in the rendered output.
    #[test]
    fn gitignore_migration_safety_net() {
        let old = include_str!("files/root/gitignore.tpl");
        let new = render_gitignore(&default_patterns());
        assert_subset(old, &new, "gitignore");
    }

    fn assert_subset(old: &str, new: &str, label: &str) {
        for line in old.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            assert!(
                new.lines().any(|l| l.trim() == trimmed),
                "{label}: pattern '{trimmed}' from existing template missing in regenerated output\n--- regenerated ---\n{new}"
            );
        }
    }

    /// Helper test (run with `--ignored`) to overwrite both .tpl files from
    /// the canonical TOML. Used in Task 2 to perform the one-shot regen.
    #[test]
    #[ignore]
    fn regenerate_ignore_templates() {
        use std::path::PathBuf;
        let p = default_patterns();
        let docker = render_dockerignore(&p);
        let git = render_gitignore(&p);

        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let docker_path = crate_root.join("src/templates/files/docker/dockerignore.tpl");
        let git_path = crate_root.join("src/templates/files/root/gitignore.tpl");

        std::fs::write(&docker_path, &docker).unwrap();
        std::fs::write(&git_path, &git).unwrap();

        eprintln!("wrote {}", docker_path.display());
        eprintln!("wrote {}", git_path.display());
    }
}
