---
phase: 118-server-side-expressions
reviewed: 2026-04-19T00:00:00Z
depth: standard
files_reviewed: 3
files_reviewed_list:
  - ferro-json-ui/src/expression.rs
  - ferro-json-ui/src/lib.rs
  - framework/src/json_ui/mod.rs
findings:
  critical: 0
  warning: 0
  info: 4
  total: 4
status: issues_found
---

# Phase 118: Code Review Report

**Reviewed:** 2026-04-19
**Depth:** standard
**Files Reviewed:** 3
**Status:** issues_found (informational only — no bugs, no security issues)

## Summary

The resolver implementation is tight and matches the Phase 118 design decisions accurately.
All 28 unit tests and 5 integration tests exercise the documented invariants (single-pass,
type preservation, escape handling, scope restrictions, pipeline ordering). No bugs found.
No security issues introduced — HTML escaping remains the walker's responsibility per Phase
116, and the resolver honors the `$data` / `$template`-only cap that was architected against
the inner-platform risk.

Findings below are all **Info** — minor test-coverage gaps and one allocation pattern the
user explicitly asked about. None are blocking; each is a small-polish opportunity.

Cross-checked against `118-CONTEXT.md` D-01 through D-14: every decision is honored by the
implementation (scope restricted to `Element.props`, infallible, single-pass, type-preserving,
plugin props walk identically, `$data`/`$template` only, no fast-path pre-scan).

## Info

### IN-01: `spec.data.clone()` at resolver entry is avoidable via disjoint-field borrow

**File:** `ferro-json-ui/src/expression.rs:36`
**Issue:** `resolve_expressions` clones `spec.data` up-front so the resolver body can pass
`&Value` into the recursive walker while mutating `spec.elements`. For specs with large
embedded data payloads (e.g., a `DataTable` with thousands of rows in `spec.data`), this is a
full deep clone per render. Rust's borrow checker allows disjoint-field borrows through
`&mut Spec`, so the clone is not structurally necessary.

**Fix:**
```rust
pub fn resolve_expressions(spec: &mut Spec) {
    let data = &spec.data;
    for el in spec.elements.values_mut() {
        resolve_value(&mut el.props, data);
    }
}
```
This compiles because `spec.data` and `spec.elements` are disjoint fields of `Spec`; the
immutable borrow of `data` and the mutable iteration of `elements` do not overlap. Preserves
behavior exactly (resolver already does not touch `spec.data` per D-04, verified by
`does_not_touch_spec_data`).

Noting as Info rather than Warning: performance is explicitly out-of-scope for v1 per review
policy, and the user flagged this as a question rather than an assertion.

### IN-02: No test for malformed-sibling-keys object containing a nested valid expression

**File:** `ferro-json-ui/src/expression.rs:44-57` (behavior), `:226-238` (test gap)
**Issue:** When `is_data_expr` / `is_template_expr` return `None` because the object has
sibling keys (D-06 malformed-expression passthrough), the `else` branch at line 54 recurses
into `map.values_mut()`. This means a malformed outer expression with a nested valid
expression inside one of its values **will have the nested expression resolved**. For
example:
```json
{"$data": "/x", "inner": {"$data": "/y"}}
```
The outer object stays literal (correct per D-06), but `inner` is replaced by the resolved
`/y` value. This is consistent with "walk every value unconditionally" but may surprise
authors who assume malformed containers are fully inert. The `data_sibling_keys` test only
covers the shallow string-sibling case; the nested-expression case is untested.

**Fix:** Add a unit test that locks in the intended behavior (whichever way the team wants
it):
```rust
#[test]
fn sibling_keys_still_recurse_into_nested_expressions() {
    let out = run(
        json!({ "x": "resolved_x", "y": "resolved_y" }),
        json!({ "$data": "/x", "inner": { "$data": "/y" } }),
    );
    // Document the current behavior: outer stays literal, inner resolves.
    assert_eq!(
        out,
        json!({ "$data": "/x", "inner": "resolved_y" }),
    );
}
```
If the desired semantics is "malformed containers are fully inert," the fix is in
`resolve_value` (skip recursion when any `$data`/`$template` key is present with siblings).
Either direction, pick one and pin it with a test.

### IN-03: No test for backslash character inside a placeholder body

**File:** `ferro-json-ui/src/expression.rs:107-125` (behavior), test-file-wide (test gap)
**Issue:** D-02 specifies the placeholder regex as `\{[^{}\\]*\}` — "no nested braces or
backslashes inside." The implementation's inner-placeholder loop at line 110-116 consumes
every character up to `}` with no backslash awareness, so a template like `"{a\\b}"` treats
`a\b` as the path and hands it to `resolve_path_string`. No existing test covers this, so the
spec/implementation gap is invisible.

**Fix:** Either tighten the scanner to match the documented regex, or (simpler) relax D-02's
documentation and pin the lenient behavior with a test:
```rust
#[test]
fn template_backslash_inside_placeholder_is_part_of_path() {
    let out = run(json!({}), json!({ "$template": "{a\\b}" }));
    // Backslash inside placeholder is treated as a literal path character;
    // resolve fails, placeholder interpolates as empty.
    assert_eq!(out, json!(""));
}
```
Low impact — authors are unlikely to put backslashes inside placeholder bodies — but the gap
between D-02's stated regex and the actual scanner is a latent surprise vector.

### IN-04: Test suite covers `does_not_touch_visible` and `does_not_touch_children` but not `action`, `title`, or `layout`

**File:** `ferro-json-ui/src/expression.rs:332-405` (scope-restriction tests)
**Issue:** D-04 locks expression resolution to `Element.props` only — `Spec.title`,
`Spec.layout`, `Element.action` are all out of scope. The inline tests cover `spec.data`,
`children`, and `visible` explicitly but not `action`, `title`, or `layout`. Current
implementation doesn't touch them (it only iterates `spec.elements.values_mut()` and walks
`el.props`), so no bug — just an under-guarded invariant.

**Fix:** Add three tiny guard tests, e.g.:
```rust
#[test]
fn does_not_touch_action() {
    let action = Action {
        handler: "route.name".to_string(),
        url: Some("/literal/{/path}".to_string()), // $template-like syntax
        method: HttpMethod::Get,
        confirm: None, on_success: None, on_error: None, target: None,
    };
    let mut spec = Spec::builder()
        .data(json!({ "path": "resolved" }))
        .element("root", Element::new("Button").prop("x", json!("lit")).action(action.clone()))
        .build().unwrap();
    resolve_expressions(&mut spec);
    assert_eq!(spec.elements.get("root").unwrap().action, Some(action));
}

#[test]
fn does_not_touch_title() {
    let mut spec = Spec::builder()
        .title("literal {/ignored}")
        .data(json!({ "ignored": "resolved" }))
        .element("root", Element::new("Text").prop("x", json!("lit")))
        .build().unwrap();
    resolve_expressions(&mut spec);
    assert_eq!(spec.title.as_deref(), Some("literal {/ignored}"));
}

#[test]
fn does_not_touch_layout() {
    let mut spec = Spec::builder()
        .layout("app")
        .data(json!({ "x": "resolved" }))
        .element("root", Element::new("Text").prop("x", json!("lit")))
        .build().unwrap();
    resolve_expressions(&mut spec);
    assert_eq!(spec.layout.as_deref(), Some("app"));
}
```
Makes D-04 a structural guarantee rather than a documentation hope — same rationale as
existing `does_not_touch_spec_data` / `does_not_touch_visible`.

---

_Reviewed: 2026-04-19_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
