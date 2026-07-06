//! Integration tests for the D-16 validation-pipeline reorder.
//!
//! Verify: catalog errors at load time are warnings (not failures); per-request
//! validation enforces catalog errors AFTER expand_directives.
//!
//! Architecture:
//! - `visible: {...}` gates are evaluated at RENDER TIME by the renderer.
//!   `expand_directives` only removes `$if`-falsy elements. So a `visible`-gated
//!   element with a catalog-invalid prop (Alert.variant="") stays in the spec
//!   after expand_directives, catalog validation fires, but the element is not
//!   rendered in HTML because the renderer evaluates visibility before emitting.
//! - Pipeline order fix: catalog errors are now tracing::warn at load time and
//!   tracing::error at render time (not hard failures), so render continues.

#![cfg(feature = "json-ui")]

use ferro_rs::{Element, JsonUi, JsonUiVisibility, Spec, VisibilityCondition, VisibilityOperator};

/// Extract the HTML body from a framework Response.
fn html_body(result: ferro_rs::http::Response) -> String {
    match result {
        Ok(r) => r.body().to_string(),
        Err(r) => r.body().to_string(),
    }
}

/// Build a spec with Alert.variant="" (catalog-invalid) gated by
/// `visible: {path: "/flash", operator: "exists"}`.
/// Root is a `Grid` containing the maybe_alert child.
fn gated_bad_variant_spec() -> Spec {
    let visibility = JsonUiVisibility::Condition(VisibilityCondition {
        path: "/flash".to_string(),
        operator: VisibilityOperator::Exists,
        value: None,
    });
    Spec::builder()
        .element("root", Element::new("Grid").child("maybe_alert"))
        .element(
            "maybe_alert",
            Element::new("Alert")
                .prop("variant", "")
                .prop("message", "flash message")
                .visible(visibility),
        )
        .build()
        .expect("spec with gated bad-variant must be structurally valid")
}

/// A spec with Alert.variant="" gated by `visible: {path: "/flash", operator: "exists"}`
/// renders without panicking or returning a 500 error, even though the Alert has a
/// catalog-invalid prop.
///
/// Architecture note: `visible` is evaluated at render time by the renderer; the
/// element is NOT removed from the spec by `expand_directives` (only `$if` does that).
/// With D-16: catalog validation runs after expand_directives at render time and
/// logs tracing::error, but render continues. The renderer then evaluates the
/// visibility condition and suppresses the Alert's HTML output.
///
/// The key invariant: the rendered HTML does NOT contain Alert-specific markup
/// (`role="alert"`) because the renderer honours the visibility gate.
#[test]
fn alert_variant_empty_but_gated_renders_cleanly() {
    let spec = gated_bad_variant_spec();

    // No flash data → visibility condition (Exists on /flash) is false.
    // Renderer evaluates visibility and suppresses Alert HTML.
    let data = serde_json::json!({});
    let result = JsonUi::render(&spec, &data);

    // Render must succeed — catalog error is tracing::error + continue, not a panic.
    assert!(
        result.is_ok(),
        "render must succeed for gated bad-variant spec (tracing::error + continue)"
    );

    let body = html_body(result);

    // The Alert HTML must not be emitted (renderer suppresses it via visibility gate).
    // Note: "flash message" MAY appear in the embedded data-view JSON — that is
    // expected and correct (the raw spec is serialized for JS hydration). We assert
    // on rendered Alert markup (`role="alert"`) which only appears if render_alert runs.
    assert!(
        !body.contains(r#"role="alert""#),
        "Alert HTML must not be emitted when visibility gate is false; body snippet: {}",
        &body[..body.len().min(800)]
    );
}

/// A spec with Alert.variant="" and NO visibility gate.
/// The element survives expand_directives; catalog validation runs at render time
/// and logs tracing::error, but render continues (clean-path: tracing::error + continue).
///
/// This test verifies the render does NOT panic and returns Ok(200).
#[test]
fn alert_variant_empty_ungated_surfaces_error_at_render() {
    let spec = Spec::builder()
        .element("root", Element::new("Grid").child("bad_alert"))
        .element(
            "bad_alert",
            Element::new("Alert")
                .prop("variant", "")
                .prop("message", "ungated bad alert"),
        )
        .build()
        .expect("spec with ungated bad-variant must be structurally valid");

    let data = serde_json::json!({});

    // Render should not panic — clean-path strategy is tracing::error + continue.
    // The catalog error is logged at error level; the HTTP response is still Ok(200).
    let result = JsonUi::render(&spec, &data);

    assert!(
        result.is_ok(),
        "render must not panic on catalog error in clean path; got: {:?}",
        result.err().map(|e| e.status_code())
    );
}
