//! Pinned Quill 2.0.3 CDN URLs and SHA-384 SRI hashes.
//!
//! Quill is loaded from jsDelivr at the exact pinned version `2.0.3`. The
//! SHA-384 integrity hashes were computed at phase-150 plan-02 execution
//! time from the live jsDelivr bytes. Browser SRI verification refuses to
//! load the assets if the bytes drift, providing a hard tamper-detection
//! contract.
//!
//! Bumping Quill is a deliberate phase: re-run the SRI computation and
//! update both the version in the URL and the corresponding hash. Do NOT
//! invent values.

/// Quill 2.0.3 main JS bundle.
// Used by render.rs (Plan 03 wires the asset injection).
#[allow(dead_code)]
pub(crate) const QUILL_JS_URL: &str = "https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.js";

/// Quill 2.0.3 Snow theme CSS.
#[allow(dead_code)]
pub(crate) const QUILL_CSS_URL: &str =
    "https://cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.snow.css";

/// SHA-384 SRI hash for the Quill 2.0.3 main JS bundle.
/// Computed from the bytes served by jsDelivr at phase-150 plan-02 execution.
#[allow(dead_code)]
pub(crate) const QUILL_JS_SRI: &str =
    "sha384-utBUCeG4SYaCm4m7GQZYr8Hy8Fpy3V4KGjBZaf4WTKOcwhCYpt/0PfeEe3HNlwx8";

/// SHA-384 SRI hash for the Quill 2.0.3 Snow theme CSS.
/// Computed from the bytes served by jsDelivr at phase-150 plan-02 execution.
#[allow(dead_code)]
pub(crate) const QUILL_CSS_SRI: &str =
    "sha384-ecIckRi4QlKYya/FQUbBUjS4qp65jF/J87Guw5uzTbO1C1Jfa/6kYmd6dXUF6D7i";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quill_constants_are_non_empty() {
        // Verify that none of the constants are accidentally set to an empty string.
        // Uses assert_ne! to avoid clippy::const_is_empty and clippy::len_zero lints.
        assert_ne!(QUILL_JS_URL, "", "QUILL_JS_URL must not be empty");
        assert_ne!(QUILL_CSS_URL, "", "QUILL_CSS_URL must not be empty");
        assert_ne!(QUILL_JS_SRI, "", "QUILL_JS_SRI must not be empty");
        assert_ne!(QUILL_CSS_SRI, "", "QUILL_CSS_SRI must not be empty");
    }

    #[test]
    fn quill_urls_pin_to_2_0_3() {
        // Locked by D-10 — version bump is a deliberate phase.
        assert!(QUILL_JS_URL.contains("@2.0.3/"));
        assert!(QUILL_CSS_URL.contains("@2.0.3/"));
        assert!(QUILL_JS_URL.ends_with("/quill.js"));
        assert!(QUILL_CSS_URL.ends_with("/quill.snow.css"));
        // jsDelivr — never unpkg or another mirror (locked by D-07).
        assert!(QUILL_JS_URL.starts_with("https://cdn.jsdelivr.net/"));
        assert!(QUILL_CSS_URL.starts_with("https://cdn.jsdelivr.net/"));
    }

    #[test]
    fn quill_sri_hashes_have_sha384_prefix_and_correct_length() {
        assert!(
            QUILL_JS_SRI.starts_with("sha384-"),
            "QUILL_JS_SRI must use sha384 algorithm"
        );
        assert!(
            QUILL_CSS_SRI.starts_with("sha384-"),
            "QUILL_CSS_SRI must use sha384 algorithm"
        );
        // Standard base64 of 48 SHA-384 bytes = 64 chars including padding.
        // Total string = "sha384-" (7 chars) + 64 = 71 chars.
        assert_eq!(
            QUILL_JS_SRI.len(),
            71,
            "QUILL_JS_SRI must be 71 chars: {QUILL_JS_SRI}"
        );
        assert_eq!(
            QUILL_CSS_SRI.len(),
            71,
            "QUILL_CSS_SRI must be 71 chars: {QUILL_CSS_SRI}"
        );
    }
}
