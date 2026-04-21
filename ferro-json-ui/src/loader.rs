//! # Page Loader
//!
//! Runtime file-loading pipeline for v2 JSON-UI specs (Phase 119).
//!
//! This module exposes `load_cached`, a cache-aware loader that:
//!
//! 1. Canonicalizes the input path (`fs::canonicalize`) — used as the cache key.
//! 2. Reads the file contents (`fs::read_to_string`).
//! 3. Parses into a `Spec` via `Spec::from_json` (structural validation).
//! 4. Validates against `global_catalog().validate(&spec)` (component + envelope).
//! 5. Inserts `(Arc<Spec>, mtime)` into a process-level `OnceLock<RwLock<HashMap>>`.
//!
//! In production (`reload_if_changed = false`), entries are never evicted after
//! first load. In development (`reload_if_changed = true`), each request performs
//! a single `fs::metadata(path).modified()` syscall — if the mtime has advanced
//! past the cached mtime, the entry is reloaded. No background thread, no
//! `notify` crate (119-CONTEXT D-03).
//!
//! Errors are returned as [`LoadError`] with three variants: `Io` (read or
//! canonicalize), `Parse` (structural), `Catalog` (component schema).
//!
//! The cache follows the same `OnceLock<RwLock<...>>` pattern used by
//! [`crate::catalog::global_catalog`] and [`crate::layout::global_registry`].

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::SystemTime;

use thiserror::Error;

use crate::catalog::{global_catalog, CatalogError};
use crate::spec::{Spec, SpecError};

// ── Errors ────────────────────────────────────────────────────────────────────

/// Errors returned by [`load_cached`] and related loader entry points.
///
/// Three variants track the three failure modes of the load pipeline:
/// - [`LoadError::Io`] — filesystem read or canonicalize failure (missing path,
///   permission denied, etc.).
/// - [`LoadError::Parse`] — the file is present but its contents are not a
///   structurally valid v2 Spec. Wraps [`SpecError`].
/// - [`LoadError::Catalog`] — the spec parses structurally but fails catalog
///   validation (unknown component type, invalid props, etc.). Wraps a
///   `Vec<CatalogError>`; note that `Vec<CatalogError>` does NOT implement
///   `std::error::Error`, so `#[from]` cannot be used for this variant.
#[derive(Debug, Error)]
pub enum LoadError {
    /// Filesystem read failure (including canonicalize failure for missing paths).
    #[error("failed to read spec file: {0}")]
    Io(#[from] std::io::Error),

    /// Spec fails structural validation (JSON syntax, duplicate IDs, cycles, etc.).
    #[error("failed to parse spec: {0}")]
    Parse(#[from] SpecError),

    /// Spec is structurally valid but fails catalog validation.
    #[error("spec failed catalog validation: {0:?}")]
    Catalog(Vec<CatalogError>),
}

// ── Cache ─────────────────────────────────────────────────────────────────────

type SpecCache = HashMap<PathBuf, (Arc<Spec>, SystemTime)>;

static SPEC_CACHE: OnceLock<RwLock<SpecCache>> = OnceLock::new();

/// Access the process-level spec cache.
///
/// Keyed by canonical PathBuf. Value is `(Arc<Spec>, mtime)` so the cached
/// spec can be cheaply cloned into per-request handlers without duplicating
/// the element HashMap.
fn global_spec_cache() -> &'static RwLock<SpecCache> {
    SPEC_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Best-effort mtime — returns `UNIX_EPOCH` on platforms or filesystems that
/// do not report modification time. In dev mode this conservatively forces a
/// reload on the next access (119-RESEARCH §Pitfall 5).
fn current_mtime(path: &Path) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Load a spec from `path`, using the process-level cache.
///
/// # Parameters
///
/// - `path` — filesystem path to a v2 JSON-UI spec file.
/// - `reload_if_changed` — when `true`, checks the file's mtime against the
///   cached mtime; if the file has changed, evicts and reloads. Set this from
///   `!Config::is_production()` at the framework integration layer.
///
/// # Returns
///
/// `Ok(Arc<Spec>)` on cache hit or successful load. The `Arc` is cloned from
/// the cache entry — callers who need an owned `Spec` (e.g. for
/// `Spec::merge_data`) must call `(*arc).clone()` to get a `Spec`
/// (119-RESEARCH §Pitfall 1).
///
/// # Errors
///
/// - [`LoadError::Io`] — file missing, unreadable, or canonicalize fails.
/// - [`LoadError::Parse`] — file contents are not a structurally valid Spec.
/// - [`LoadError::Catalog`] — spec parses but fails catalog validation.
///
/// # Concurrency
///
/// Reads use a shared read lock. Writes (first load, reload) briefly acquire
/// a write lock AFTER parsing and validation have completed — no fallible
/// code runs inside the write guard, so the lock cannot be poisoned by a
/// panic-throwing parser (119-RESEARCH §Pitfall 2).
pub fn load_cached(path: &Path, reload_if_changed: bool) -> Result<Arc<Spec>, LoadError> {
    let canonical = fs::canonicalize(path)?;

    // Fast path: read lock.
    {
        let cache = global_spec_cache()
            .read()
            .expect("spec cache RwLock poisoned");
        if let Some((arc_spec, cached_mtime)) = cache.get(&canonical) {
            if !reload_if_changed {
                return Ok(Arc::clone(arc_spec));
            }
            let current = current_mtime(&canonical);
            if current <= *cached_mtime {
                return Ok(Arc::clone(arc_spec));
            }
            // mtime advanced — fall through to reload path.
        }
    }

    // Miss or stale: parse + validate outside any lock.
    let content = fs::read_to_string(&canonical)?;
    let spec = Spec::from_json(&content).map_err(LoadError::Parse)?;
    global_catalog()
        .validate(&spec)
        .map_err(LoadError::Catalog)?;

    let mtime = current_mtime(&canonical);
    let arc_spec = Arc::new(spec);

    // Write lock holds only for the insert — no fallible code inside.
    global_spec_cache()
        .write()
        .expect("spec cache RwLock poisoned")
        .insert(canonical, (Arc::clone(&arc_spec), mtime));

    Ok(arc_spec)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    /// Write `content` to a unique tempfile and return its path.
    ///
    /// Uses `std::env::temp_dir()` with a uniquifier derived from a static
    /// counter so concurrent tests do not collide. We do not pull in the
    /// `tempfile` crate — ferro-json-ui has no dev-dependency on it and
    /// Phase 119 forbids new deps.
    fn write_temp(name: &str, content: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut path = std::env::temp_dir();
        path.push(format!("ferro-json-ui-loader-{name}-{n}.json"));
        let mut f = std::fs::File::create(&path).expect("create tempfile");
        f.write_all(content.as_bytes()).expect("write tempfile");
        f.sync_all().expect("sync tempfile");
        path
    }

    const VALID_SPEC: &str = r#"{
        "$schema": "ferro-json-ui/v2",
        "root": "r",
        "elements": { "r": { "type": "Text", "props": { "content": "hi" } } }
    }"#;

    const VALID_SPEC_ALT: &str = r#"{
        "$schema": "ferro-json-ui/v2",
        "root": "other",
        "elements": { "other": { "type": "Text", "props": { "content": "changed" } } }
    }"#;

    #[test]
    fn load_spec_valid() {
        let path = write_temp("valid", VALID_SPEC);
        let spec = load_cached(&path, false).expect("valid spec should load");
        assert_eq!(spec.root, "r");
    }

    #[test]
    fn load_spec_invalid_json() {
        let path = write_temp("invalid-json", "{ not valid json");
        let err = load_cached(&path, false).expect_err("must fail");
        assert!(
            matches!(err, LoadError::Parse(_)),
            "expected Parse, got {err:?}"
        );
    }

    #[test]
    fn load_spec_catalog_error() {
        // Spec parses structurally but the component type is unknown.
        let unknown = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "r",
            "elements": { "r": { "type": "NotARealComponent_119_loader" } }
        }"#;
        let path = write_temp("unknown-type", unknown);
        let err = load_cached(&path, false).expect_err("must fail");
        match err {
            LoadError::Catalog(errs) => {
                assert!(!errs.is_empty(), "catalog errors must be non-empty")
            }
            other => panic!("expected Catalog, got {other:?}"),
        }
    }

    #[test]
    fn load_spec_missing_file() {
        let path = PathBuf::from("/nonexistent/path-119-loader-test-does-not-exist.json");
        let err = load_cached(&path, false).expect_err("must fail");
        assert!(matches!(err, LoadError::Io(_)), "expected Io, got {err:?}");
    }

    #[test]
    fn cache_hit() {
        let path = write_temp("cache-hit", VALID_SPEC);
        let first = load_cached(&path, false).expect("first load");
        let second = load_cached(&path, false).expect("second load");
        assert!(
            Arc::ptr_eq(&first, &second),
            "second load must return the same Arc — cache hit"
        );
    }

    #[test]
    fn dev_mode_invalidation() {
        let path = write_temp("dev-mode", VALID_SPEC);
        let first = load_cached(&path, true).expect("first load");
        assert_eq!(first.root, "r");

        // Sleep 1.1s to guarantee SystemTime advances past 1-second filesystem
        // resolution (ext4/apfs default). Rewrite with different content.
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let mut f = std::fs::File::create(&path).expect("rewrite tempfile");
        f.write_all(VALID_SPEC_ALT.as_bytes()).expect("write");
        f.sync_all().expect("sync");

        let second = load_cached(&path, true).expect("second load after mtime advance");
        assert!(
            !Arc::ptr_eq(&first, &second),
            "mtime advance must produce a fresh Arc"
        );
        assert_eq!(
            second.root, "other",
            "reloaded spec must reflect post-write content"
        );
    }
}
