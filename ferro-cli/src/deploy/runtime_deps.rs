//! Registry of crate→apt-package mappings and Cargo.toml scanner. Shared
//! source of truth between ferro-mcp `runtime_requirements` tool and
//! ferro-cli `docker:init --runtime-deps` (Phase 122). Per D-07..D-09.
//!
//! Pure functions. The only IO is `scan_runtime_deps(path)` which reads the
//! explicit Cargo.toml path passed in and delegates to the `_str` variant.

#![allow(dead_code, unused_imports)] // Consumed by ferro-mcp across crate boundary.

use std::fs;
use std::io;
use std::path::Path;
use toml::Value;

const DEP_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

/// Ordered, deterministic. One crate may map to multiple apt packages.
pub struct RuntimeDep {
    pub crate_name: &'static str,
    pub apt_packages: &'static [&'static str],
}

pub const RUNTIME_DEP_REGISTRY: &[RuntimeDep] = &[
    RuntimeDep {
        crate_name: "chromiumoxide",
        apt_packages: &["chromium", "fonts-liberation"],
    },
    RuntimeDep {
        crate_name: "headless_chrome",
        apt_packages: &["chromium", "fonts-liberation"],
    },
    RuntimeDep {
        crate_name: "ffmpeg-next",
        apt_packages: &["ffmpeg"],
    },
    RuntimeDep {
        crate_name: "pdfium",
        apt_packages: &["libpdfium"],
    },
];

/// Scan a Cargo.toml file; return deduped apt packages (sorted) for every
/// registry crate that appears in any `[dependencies*]` table.
pub fn scan_runtime_deps(cargo_toml: &Path) -> io::Result<Vec<String>> {
    let content = fs::read_to_string(cargo_toml)?;
    Ok(scan_runtime_deps_str(&content))
}

/// Same but operating on an already-loaded string (for testing).
pub fn scan_runtime_deps_str(content: &str) -> Vec<String> {
    let matches = scan_runtime_dep_matches(content);
    let mut out: Vec<String> = matches
        .iter()
        .flat_map(|m| m.apt_packages.iter().map(|p| (*p).to_string()))
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Expose the matched registry entries for richer MCP reports.
pub fn scan_runtime_dep_matches(content: &str) -> Vec<&'static RuntimeDep> {
    let parsed: Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut out: Vec<&'static RuntimeDep> = Vec::new();
    for table_name in DEP_TABLES {
        let Some(table) = parsed.get(*table_name).and_then(|v| v.as_table()) else {
            continue;
        };
        for key in table.keys() {
            if let Some(entry) = RUNTIME_DEP_REGISTRY
                .iter()
                .find(|d| d.crate_name == key.as_str())
            {
                if !out.iter().any(|e| e.crate_name == entry.crate_name) {
                    out.push(entry);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn chromiumoxide_maps_to_chromium_and_fonts() {
        let out = scan_runtime_deps_str(
            r#"
[dependencies]
chromiumoxide = "0.5"
"#,
        );
        assert_eq!(out, vec!["chromium", "fonts-liberation"]);
    }

    #[test]
    fn chromiumoxide_and_headless_chrome_dedup() {
        let out = scan_runtime_deps_str(
            r#"
[dependencies]
chromiumoxide = "0.5"
headless_chrome = "1"
"#,
        );
        assert_eq!(out, vec!["chromium", "fonts-liberation"]);
    }

    #[test]
    fn ffmpeg_next_table_form() {
        let out = scan_runtime_deps_str(
            r#"
[dependencies]
ffmpeg-next = { version = "6" }
"#,
        );
        assert_eq!(out, vec!["ffmpeg"]);
    }

    #[test]
    fn pdfium_string_and_table_forms() {
        let string_form = scan_runtime_deps_str(
            r#"
[dependencies]
pdfium = "1"
"#,
        );
        assert_eq!(string_form, vec!["libpdfium"]);

        let table_form = scan_runtime_deps_str(
            r#"
[dependencies]
pdfium = { version = "1" }
"#,
        );
        assert_eq!(table_form, vec!["libpdfium"]);
    }

    #[test]
    fn unknown_crates_return_empty() {
        let out = scan_runtime_deps_str(
            r#"
[dependencies]
serde = "1"
tokio = { version = "1", features = ["full"] }
"#,
        );
        assert!(out.is_empty());
    }

    #[test]
    fn malformed_toml_returns_empty() {
        let out = scan_runtime_deps_str("this is not = = valid toml [[[");
        assert!(out.is_empty());
    }

    #[test]
    fn scans_across_all_dep_tables() {
        let out = scan_runtime_deps_str(
            r#"
[dependencies]
chromiumoxide = "0.5"

[dev-dependencies]
ffmpeg-next = "6"

[build-dependencies]
pdfium = "1"
"#,
        );
        assert_eq!(
            out,
            vec!["chromium", "ffmpeg", "fonts-liberation", "libpdfium"]
        );
    }

    #[test]
    fn matches_expose_static_refs() {
        let matches = scan_runtime_dep_matches(
            r#"
[dependencies]
chromiumoxide = "0.5"
"#,
        );
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].crate_name, "chromiumoxide");
        assert_eq!(matches[0].apt_packages, &["chromium", "fonts-liberation"]);
    }

    #[test]
    fn path_variant_reads_file() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(
            br#"
[dependencies]
ffmpeg-next = "6"
"#,
        )
        .unwrap();
        let out = scan_runtime_deps(f.path()).unwrap();
        assert_eq!(out, vec!["ffmpeg"]);
    }
}
