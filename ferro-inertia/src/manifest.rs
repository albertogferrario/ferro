//! Vite manifest resolution for production asset paths.
//!
//! Reads Vite's `manifest.json` to resolve hashed asset filenames.
//! The parsed manifest is cached with `OnceLock` so the file is read at most once.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Resolved JS and CSS asset paths for an entry point.
#[derive(Debug, Clone)]
pub struct ResolvedAssets {
    /// JS file path (absolute, e.g., `/assets/main-BiRGL2vC.js`).
    pub js: String,
    /// CSS file paths (absolute, e.g., `/assets/main-DXEjMDJ3.css`).
    pub css: Vec<String>,
}

/// Single entry in Vite's manifest.json.
#[derive(Debug, Deserialize)]
struct ManifestEntry {
    file: String,
    css: Option<Vec<String>>,
    #[serde(rename = "isEntry")]
    #[allow(dead_code)]
    is_entry: Option<bool>,
}

/// Parsed Vite manifest mapping entry point keys to their build outputs.
#[derive(Debug, Deserialize)]
struct ViteManifest(HashMap<String, ManifestEntry>);

impl ViteManifest {
    /// Load and parse manifest.json from disk. Returns `None` if the file
    /// doesn't exist or can't be parsed.
    fn load(path: &str) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Look up an entry point and return its resolved asset paths.
    fn resolve(&self, entry_point: &str) -> Option<ResolvedAssets> {
        let entry = self.0.get(entry_point)?;

        let js = format!("/{}", entry.file);
        let css = entry
            .css
            .as_ref()
            .map(|paths| paths.iter().map(|p| format!("/{p}")).collect())
            .unwrap_or_default();

        Some(ResolvedAssets { js, css })
    }
}

/// Global cache for the parsed manifest.
static MANIFEST: OnceLock<Option<ViteManifest>> = OnceLock::new();

/// Verify — without caching — that `entry_point` resolves in the manifest at
/// `manifest_path`. Returns a human-readable error describing exactly what's
/// missing. Used for fail-fast boot validation (see `InertiaConfig::verify_production_assets`).
pub fn verify_manifest(manifest_path: &str, entry_point: &str) -> Result<(), String> {
    match ViteManifest::load(manifest_path) {
        None => Err(format!(
            "Vite manifest not found or unparseable at '{manifest_path}'. \
             Run the frontend build (`npm run build`)."
        )),
        Some(m) => {
            if m.resolve(entry_point).is_some() {
                Ok(())
            } else {
                Err(format!(
                    "entry point '{entry_point}' is not present in the Vite manifest \
                     '{manifest_path}'. Check the Vite `rollupOptions.input` and that \
                     the build output matches InertiaConfig::entry_point."
                ))
            }
        }
    }
}

/// Resolve production asset paths from the Vite manifest.
///
/// On first call, reads and parses `manifest_path`. Subsequent calls return
/// cached results. Falls back to `/assets/main.js` and `/assets/main.css`
/// if the manifest is missing or the entry point is not found.
pub fn resolve_assets(manifest_path: &str, entry_point: &str) -> ResolvedAssets {
    let manifest = MANIFEST.get_or_init(|| ViteManifest::load(manifest_path));

    if let Some(assets) = manifest.as_ref().and_then(|m| m.resolve(entry_point)) {
        return assets;
    }

    // Production asset resolution failed. Rather than silently serve a guessed
    // `/assets/main.js` (which 404s and leaves a blank page with no clue), warn
    // once with the exact cause. This runs only on the production path — the dev
    // path uses the Vite server and never calls `resolve_assets`.
    let reason = if manifest.is_none() {
        format!("manifest not found or unparseable at '{manifest_path}'")
    } else {
        format!("entry point '{entry_point}' not present in '{manifest_path}'")
    };
    eprintln!(
        "[ferro-inertia] WARNING: {reason}. Falling back to '/assets/main.js'. \
         Run the frontend build (`npm run build`) and check that Vite's outDir + \
         manifest path line up with InertiaConfig::manifest_path and the static root."
    );

    ResolvedAssets {
        js: "/assets/main.js".to_string(),
        css: vec!["/assets/main.css".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_manifest_and_resolve_entry() {
        let manifest_json = r#"{
            "src/main.tsx": {
                "file": "assets/main-BiRGL2vC.js",
                "isEntry": true,
                "css": ["assets/main-DXEjMDJ3.css"]
            }
        }"#;

        let manifest: ViteManifest = serde_json::from_str(manifest_json).unwrap();
        let resolved = manifest.resolve("src/main.tsx").unwrap();

        assert_eq!(resolved.js, "/assets/main-BiRGL2vC.js");
        assert_eq!(resolved.css, vec!["/assets/main-DXEjMDJ3.css"]);
    }

    #[test]
    fn resolve_missing_entry_returns_none() {
        let manifest_json = r#"{
            "src/main.tsx": {
                "file": "assets/main-abc.js",
                "isEntry": true,
                "css": ["assets/main-def.css"]
            }
        }"#;

        let manifest: ViteManifest = serde_json::from_str(manifest_json).unwrap();
        assert!(manifest.resolve("src/other.tsx").is_none());
    }

    #[test]
    fn resolve_entry_without_css() {
        let manifest_json = r#"{
            "src/main.tsx": {
                "file": "assets/main-abc.js",
                "isEntry": true
            }
        }"#;

        let manifest: ViteManifest = serde_json::from_str(manifest_json).unwrap();
        let resolved = manifest.resolve("src/main.tsx").unwrap();

        assert_eq!(resolved.js, "/assets/main-abc.js");
        assert!(resolved.css.is_empty());
    }

    #[test]
    fn load_from_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(
            tmp,
            r#"{{"src/main.tsx":{{"file":"assets/main-xyz.js","isEntry":true,"css":["assets/main-xyz.css"]}}}}"#
        )
        .unwrap();

        let manifest = ViteManifest::load(tmp.path().to_str().unwrap()).unwrap();
        let resolved = manifest.resolve("src/main.tsx").unwrap();

        assert_eq!(resolved.js, "/assets/main-xyz.js");
        assert_eq!(resolved.css, vec!["/assets/main-xyz.css"]);
    }

    #[test]
    fn load_nonexistent_returns_none() {
        assert!(ViteManifest::load("/nonexistent/path/manifest.json").is_none());
    }

    #[test]
    fn verify_manifest_ok_and_errors() {
        // Missing manifest → error mentioning the build.
        let err = verify_manifest("/nonexistent/manifest.json", "src/main.tsx").unwrap_err();
        assert!(err.contains("npm run build"), "got: {err}");

        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(
            tmp,
            r#"{{"src/main.tsx":{{"file":"assets/main-x.js","isEntry":true}}}}"#
        )
        .unwrap();
        let p = tmp.path().to_str().unwrap();

        // Present entry → ok.
        assert!(verify_manifest(p, "src/main.tsx").is_ok());
        // Wrong entry → error naming the entry point.
        let err = verify_manifest(p, "src/other.tsx").unwrap_err();
        assert!(err.contains("src/other.tsx"), "got: {err}");
    }

    #[test]
    fn load_invalid_json_returns_none() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(tmp, "not json").unwrap();
        assert!(ViteManifest::load(tmp.path().to_str().unwrap()).is_none());
    }

    #[test]
    fn multiple_css_files() {
        let manifest_json = r#"{
            "src/main.tsx": {
                "file": "assets/main-abc.js",
                "isEntry": true,
                "css": ["assets/main-abc.css", "assets/vendor-def.css"]
            }
        }"#;

        let manifest: ViteManifest = serde_json::from_str(manifest_json).unwrap();
        let resolved = manifest.resolve("src/main.tsx").unwrap();

        assert_eq!(resolved.css.len(), 2);
        assert_eq!(resolved.css[0], "/assets/main-abc.css");
        assert_eq!(resolved.css[1], "/assets/vendor-def.css");
    }
}
