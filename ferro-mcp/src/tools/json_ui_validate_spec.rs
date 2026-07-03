//! JSON-UI validate-spec tool. Wraps `Spec::from_json` + `Catalog::validate`
//! so MCP-using agents see the same diagnostics the framework produces at
//! server startup.
//!
//! Per Phase 164 D-04 + D-16 (two-stage validation):
//! - Structural errors (depth, footer IDs, element refs, directives) → structural_errors
//! - Catalog errors (per-component enum-shape validation) → catalog_errors
//! - Other diagnostics that don't fail validation → warnings
//!
//! Both error vecs may be populated for a single spec; `valid` is true iff
//! both are empty.

use ferro_json_ui::{global_catalog, Spec};
use serde::Serialize;

/// Response returned by `json_ui_validate_spec`.
///
/// Two-stage validation mirrors the server-startup pipeline:
/// 1. Structural validation via `Spec::from_json` — catches malformed element
///    refs, depth violations, missing root, directive errors, and footer gaps.
/// 2. Catalog validation via `Catalog::validate` — catches per-component prop
///    schema violations (wrong enum variant, missing required field, bad type).
#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    /// True iff both `structural_errors` and `catalog_errors` are empty.
    pub valid: bool,
    /// Errors from `Spec::from_json` (structural). Non-empty → spec cannot be
    /// loaded at all; catalog validation is skipped.
    pub structural_errors: Vec<String>,
    /// Errors from `Catalog::validate` (per-component prop validation).
    /// May be non-empty even when structural validation passes.
    pub catalog_errors: Vec<String>,
    /// Non-fatal diagnostics (reserved for future use; currently empty).
    pub warnings: Vec<String>,
}

/// Validate a JSON-UI v2 spec string against the structural validator and the
/// global component catalog.
///
/// Returns a [`ValidateResponse`] with separate `structural_errors` and
/// `catalog_errors` vecs so callers can distinguish parse-time failures from
/// component-prop failures.
pub fn execute(spec_json: &str) -> ValidateResponse {
    let mut response = ValidateResponse {
        valid: true,
        structural_errors: Vec::new(),
        catalog_errors: Vec::new(),
        warnings: Vec::new(),
    };

    let spec = match Spec::from_json(spec_json) {
        Ok(s) => s,
        Err(e) => {
            response.valid = false;
            response.structural_errors.push(e.to_string());
            return response;
        }
    };

    if let Err(errs) = global_catalog().validate(&spec) {
        response.valid = false;
        response.catalog_errors = errs.into_iter().map(|e| e.to_string()).collect();
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_spec() {
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "x",
            "elements": {
                "x": {"type": "Text", "props": {"content": "hello"}}
            }
        }"#;
        let r = execute(json);
        assert!(r.valid, "expected valid, got: {r:?}");
        assert!(r.structural_errors.is_empty());
        assert!(r.catalog_errors.is_empty());
    }

    #[test]
    fn reports_structural_error_on_missing_root() {
        // Root element "nope" not present in elements map.
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "nope",
            "elements": {
                "x": {"type": "Text", "props": {"content": "hello"}}
            }
        }"#;
        let r = execute(json);
        assert!(!r.valid);
        assert!(
            !r.structural_errors.is_empty(),
            "expected structural error, got: {r:?}"
        );
        assert!(r.catalog_errors.is_empty());
    }

    #[test]
    fn reports_catalog_error_on_bad_variant() {
        // Alert.tone="" is catalog-invalid (empty string is not a valid Tone).
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "alert",
            "elements": {
                "alert": {"type": "Alert", "props": {"tone": "", "message": "x"}}
            }
        }"#;
        let r = execute(json);
        assert!(!r.valid);
        assert!(
            r.structural_errors.is_empty(),
            "should parse OK, got structural: {r:?}"
        );
        assert!(
            !r.catalog_errors.is_empty(),
            "expected catalog error, got: {r:?}"
        );
    }

    #[test]
    fn reports_both_vecs_addressable_on_any_spec() {
        // Structurally valid spec that may or may not have catalog issues.
        // Test just confirms: execute() returns a well-formed response without panic,
        // and structural_errors / catalog_errors are independently addressable.
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "x",
            "elements": {
                "x": {"type": "Alert", "props": {"variant": "info", "message": "hello"}}
            }
        }"#;
        let r = execute(json);
        let _ = r.valid;
        let _ = r.structural_errors.len();
        let _ = r.catalog_errors.len();
        let _ = r.warnings.len();
    }
}
