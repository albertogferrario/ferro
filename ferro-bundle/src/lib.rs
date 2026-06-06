//! In-memory immutable byte blobs with content-hashed URLs and one-year immutable caching.
//!
//! See the crate README for the bundle-vs-filesystem split: ferro-bundle handles
//! compile-time-embedded immutable assets; the framework's filesystem static-file
//! handler at `ferro_rs::static_files` handles mutable on-disk tenant assets.
//!
//! The public API will be added in Plan 02 (`Bundle::new`, `.content_type`,
//! `.with_alias`, `.hashed_url`, `Bundle::serve`).

// ── RED-phase unit tests (Plan 02 Task 1) ──────────────────────────────
//
// These tests reference symbols that do NOT yet exist in this crate. The
// RED-phase commit captures a failing build: `Bundle`, `Error`, and
// `reset()` are all undefined here. The GREEN-phase commit lands the full
// implementation and these tests pass.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        reset();
        let b = Bundle::new("test1", b"hello").content_type("text/plain");
        // SHA-256 of "hello" = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        // First 8 chars = 2cf24dba
        assert_eq!(b.hashed_url(), "/bundles/test1.2cf24dba.txt");
    }

    #[test]
    fn default_content_type_is_octet_stream() {
        reset();
        let b = Bundle::new("test2", b"x");
        let url = b.hashed_url();
        assert!(
            url.starts_with("/bundles/test2."),
            "expected /bundles/test2. prefix, got {}",
            url
        );
        assert!(
            !url.ends_with(".txt") && !url.ends_with(".js") && !url.ends_with(".css"),
            "default URL should not have a known extension; got {}",
            url
        );
        let suffix = url.strip_prefix("/bundles/test2.").unwrap();
        assert_eq!(suffix.len(), 8, "expected 8-char short hash; got {}", suffix);
    }

    #[test]
    #[should_panic(expected = "duplicate")]
    fn duplicate_name_panics() {
        reset();
        Bundle::new("dup", b"a");
        Bundle::new("dup", b"a");
    }

    #[test]
    fn error_not_found_displays_message() {
        let e = Error::NotFound("/x".to_string());
        assert_eq!(e.to_string(), "bundle not found at path: /x");
    }

    #[test]
    fn error_duplicate_name_displays_message() {
        let e = Error::DuplicateName("dup".to_string());
        assert_eq!(e.to_string(), "duplicate bundle name: dup already registered");
    }
}
