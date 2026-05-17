//! Resolvers for v2 JSON-UI Spec element maps.
//!
//! Walks a `Spec`'s flat element map and resolves action handler names to
//! URLs, or populates per-field validation errors on form-like elements.
//!
//! Phase 115: flat iteration only. No tree descent — children are ID
//! strings, not nested structs. Action resolution is per-element.

use std::collections::HashMap;

use serde_json::Value;

use crate::action::{Action, ActionHandler};
use crate::spec::{Element, Spec};

/// Resolve a single action using the callback. Literal paths (starting
/// with '/') are passed through as-is so callers can use
/// `Action::get("/dashboard/...")` without registering a named route.
///
/// `data` is the `spec.data` payload — used to resolve `ActionHandler::Binding`
/// references (F14, Phase 165). Binding-shape handlers resolve against the
/// JSON pointer in `spec.data`; the resolved string is then treated as a
/// literal handler (resolver lookup or `/path` pass-through).
fn resolve_action(
    action: &mut Action,
    data: &serde_json::Value,
    resolver: &impl Fn(&str) -> Option<String>,
) {
    if action.url.is_some() {
        return;
    }
    // Materialise the handler to a literal string. Binding-shape handlers
    // resolve against spec.data; missing bindings degrade to no-op so the
    // diagnostic comment surfaces in render.
    let literal: Option<String> = match &action.handler {
        ActionHandler::Literal(s) => Some(s.clone()),
        ActionHandler::Binding(d) => crate::data::resolve_path(data, &d.data)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    };
    let Some(s) = literal else { return };

    if s.starts_with('/') {
        action.url = Some(s);
        return;
    }
    if let Some(url) = resolver(&s) {
        action.url = Some(url);
    }
}

/// Resolve every `Element.action` via the provided resolver closure.
///
/// Mutates in place. Silent on missing handlers — use
/// `resolve_actions_strict` if you want to collect missing names.
pub fn resolve_actions(spec: &mut Spec, resolver: impl Fn(&str) -> Option<String>) {
    let data = spec.data.clone();
    for el in spec.elements.values_mut() {
        if let Some(action) = el.action.as_mut() {
            resolve_action(action, &data, &resolver);
        }
    }
}

/// Strict variant: returns `Err(missing_handlers)` if any handler did not
/// resolve to a URL. Literal `/path` handlers are always considered
/// resolved.
pub fn resolve_actions_strict(
    spec: &mut Spec,
    resolver: impl Fn(&str) -> Option<String>,
) -> Result<(), Vec<String>> {
    let data = spec.data.clone();
    let mut missing: Vec<String> = Vec::new();
    for el in spec.elements.values_mut() {
        if let Some(action) = el.action.as_mut() {
            resolve_action(action, &data, &resolver);
            if action.url.is_none() {
                missing.push(action.handler.as_str().to_string());
            }
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

/// Populate validation errors onto any element whose props contain a
/// `"name"` field (or `"field"` field) matching an error key.
pub fn resolve_errors(spec: &mut Spec, errors: &HashMap<String, Vec<String>>) {
    for el in spec.elements.values_mut() {
        attach_errors(el, errors, false);
    }
}

/// Variant that writes the full error bag onto every element (regardless
/// of name match).
pub fn resolve_errors_all(spec: &mut Spec, errors: &HashMap<String, Vec<String>>) {
    for el in spec.elements.values_mut() {
        attach_errors(el, errors, true);
    }
}

fn attach_errors(el: &mut Element, errors: &HashMap<String, Vec<String>>, all: bool) {
    let Some(props_obj) = el.props.as_object_mut() else {
        return;
    };
    // Match by either `name` or `field` prop (inputs use `field`, other
    // elements commonly use `name`).
    let key = props_obj
        .get("name")
        .or_else(|| props_obj.get("field"))
        .and_then(|v| v.as_str())
        .map(String::from);
    if let Some(k) = key {
        if let Some(msgs) = errors.get(&k) {
            props_obj.insert(
                "errors".to_string(),
                Value::Array(msgs.iter().cloned().map(Value::String).collect()),
            );
        }
    } else if all {
        if let Ok(errors_value) = serde_json::to_value(errors) {
            props_obj.insert("errors".to_string(), errors_value);
        }
    }
}

// ---------------------------------------------------------------------------
// Section — Directive expansion (Phase 163: $each, $if)
// ---------------------------------------------------------------------------

/// Expand `$each` / `$if` directives in `spec` against `spec.data`.
///
/// Pass order:
/// 1. `$if`-falsy elements are REMOVED from `spec.elements`.
/// 2. `$each` templated elements are REPLACED by N clones with
///    auto-suffixed IDs (`{tmpl_id}-0` .. `{tmpl_id}-(N-1)`).
/// 3. Every parent's `children` list is rewritten to reference the new clone
///    IDs (or pruned if the child was `$if`-deleted or `$each` with empty
///    rows).
///
/// When `$if` AND `$each` co-occur on the same element, `$if` is evaluated
/// first; if false, the element is removed before `$each` expansion (no
/// clones produced). This is the planner-locked ordering (163-03-PLAN #5).
///
/// Correlated child indexes: when a templated element's child points to
/// another templated element with the SAME `{path, as}` directive, the
/// cloned parent at index `i` references the cloned child at index `i`.
/// Mismatched-each child references are caught at validation time
/// (`SpecError::MismatchedEach` — Plan 04).
///
/// This pass is IDEMPOTENT: after expansion, every clone has its directive
/// fields cleared (`each = None`, `if_ = None`), so a second call is a no-op.
///
/// Pipeline position: must run BEFORE `resolve_actions` and
/// `resolve_expressions` so those passes operate on the post-expansion
/// element set.
pub fn expand_directives(spec: &mut Spec) {
    let data = spec.data.clone();
    // Pass 1: remove $if-falsy elements (templates included — a falsy $if on
    // a templated element prevents its $each expansion entirely).
    let if_removed = remove_if_falsy(spec, &data);
    // Pass 2: expand $each templated elements (those that survived $if).
    let each_expanded = expand_each(spec, &data);
    // Pass 3: rewrite parent children lists to reference new IDs / prune removed ones.
    rewrite_parent_children(spec, &if_removed, &each_expanded);
}

/// Remove elements whose `$if` predicate evaluates false; return the set of
/// removed IDs so `rewrite_parent_children` can prune dangling references.
/// Surviving elements have `if_` cleared for idempotency.
fn remove_if_falsy(spec: &mut Spec, data: &serde_json::Value) -> std::collections::HashSet<String> {
    let mut to_delete: Vec<String> = Vec::new();
    for (id, el) in spec.elements.iter() {
        if let Some(predicate) = &el.if_ {
            // SOLE predicate evaluator — D-04 reuse mandate.
            if !predicate.evaluate(data) {
                to_delete.push(id.clone());
            }
        }
    }
    let removed: std::collections::HashSet<String> = to_delete.iter().cloned().collect();
    for id in &to_delete {
        spec.elements.remove(id);
    }
    // Strip if_ on the survivors so a second pass is a no-op.
    for el in spec.elements.values_mut() {
        if el.if_.is_some() {
            el.if_ = None;
        }
    }
    removed
}

/// Expand every `$each`-templated element into N clones; return a map from
/// each template ID to its ordered list of clone IDs (empty Vec if the
/// data array was missing / empty / non-array).
fn expand_each(
    spec: &mut Spec,
    data: &serde_json::Value,
) -> std::collections::HashMap<String, Vec<String>> {
    // Snapshot templates BEFORE mutation so iteration order is deterministic
    // and we can read sibling directives during correlated-child rewriting.
    let templates: Vec<(String, Element)> = spec
        .elements
        .iter()
        .filter_map(|(id, el)| el.each.as_ref().map(|_| (id.clone(), el.clone())))
        .collect();

    let template_directives: std::collections::HashMap<String, crate::spec::EachDirective> =
        templates
            .iter()
            .map(|(id, el)| (id.clone(), el.each.clone().unwrap()))
            .collect();

    let mut expanded: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for (tmpl_id, tmpl_el) in &templates {
        let each = tmpl_el.each.as_ref().unwrap();
        let rows: Vec<serde_json::Value> = crate::data::resolve_path(data, &each.path)
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mut clone_ids: Vec<String> = Vec::with_capacity(rows.len());
        for (i, row) in rows.iter().enumerate() {
            let clone_id = format!("{tmpl_id}-{i}");
            let mut clone = tmpl_el.clone();
            clone.each = None; // strip directive — idempotent
            clone.if_ = None;
            // Pre-resolve /{as}/... paths in props against the current row.
            inline_resolve_row_paths(&mut clone.props, &each.as_, row);
            // Phase 165 F14: pre-resolve /{as}/... action.handler bindings to
            // a literal string so per-row navigation works (handler accepts
            // {"$data": "/{as}/action_url"} → ActionHandler::Literal("/dashboard/.../...")).
            if let Some(action) = clone.action.as_mut() {
                inline_resolve_row_action(action, &each.as_, row);
            }
            // Rewrite this clone's children list for correlated-each siblings.
            for child in clone.children.iter_mut() {
                if let Some(child_each) = template_directives.get(child) {
                    if child_each.path == each.path && child_each.as_ == each.as_ {
                        *child = format!("{child}-{i}");
                    }
                    // Different {path, as} would be caught by the validator
                    // (MismatchedEach, Plan 04). At runtime we leave the ID
                    // literal; the render layer emits a missing-id comment
                    // if it does not exist.
                }
            }
            spec.elements.insert(clone_id.clone(), clone);
            clone_ids.push(clone_id);
        }
        spec.elements.remove(tmpl_id);
        expanded.insert(tmpl_id.clone(), clone_ids);
    }

    expanded
}

/// Walk `value` and replace every `{"$data": "/{as}/..."}` expression that
/// references the loop variable with the literal value from `row`. Paths NOT
/// starting with `/{as}/` are left untouched (they resolve against
/// `spec.data` later via `resolve_expressions`).
fn inline_resolve_row_paths(value: &mut serde_json::Value, as_name: &str, row: &serde_json::Value) {
    let prefix = format!("/{as_name}/");
    inline_walk(value, &prefix, row, as_name);
}

/// Phase 165 F14: pre-resolve `ActionHandler::Binding` references that point
/// into the current `$each` row.
///
/// `{"$data": "/{as}/field"}` becomes `ActionHandler::Literal(row.field)`.
/// `{"$data": "/global_field"}` is left untouched — `resolve_actions` will
/// resolve it against `spec.data` later.
fn inline_resolve_row_action(
    action: &mut crate::action::Action,
    as_name: &str,
    row: &serde_json::Value,
) {
    use crate::action::ActionHandler;
    let prefix = format!("/{as_name}/");
    if let ActionHandler::Binding(d) = &action.handler {
        if let Some(rest) = d.data.strip_prefix(&prefix) {
            let resolved = crate::data::resolve_path(row, &format!("/{rest}"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if let Some(s) = resolved {
                action.handler = ActionHandler::Literal(s);
            }
            // Missing field: leave binding; resolve_actions will degrade to None.
        } else if d.data == format!("/{as_name}") {
            // `{"$data": "/cell"}` — whole row as string when row is a string.
            if let Some(s) = row.as_str() {
                action.handler = ActionHandler::Literal(s.to_string());
            }
        }
        // Non-row paths stay as Binding; resolve_actions handles them later.
    }
}

fn inline_walk(
    value: &mut serde_json::Value,
    prefix: &str,
    row: &serde_json::Value,
    as_name: &str,
) {
    match value {
        serde_json::Value::Object(map) => {
            if map.len() == 1 {
                // Single-key {"$data": "/{as}/..."} → row-scoped literal.
                if let Some(serde_json::Value::String(path)) = map.get("$data") {
                    if let Some(rest) = path.strip_prefix(prefix) {
                        let resolved = crate::data::resolve_path(row, &format!("/{rest}"))
                            .cloned()
                            .unwrap_or(serde_json::Value::Null);
                        *value = resolved;
                        return;
                    } else if path == &format!("/{as_name}") {
                        // `{"$data": "/order"}` — bind the whole row.
                        *value = row.clone();
                        return;
                    }
                }
                // Single-key {"$template": "... {/{as}/field} ..."} → interpolate
                // row-scoped placeholders inline; leave non-row markers for
                // downstream resolve_expressions.
                if let Some(serde_json::Value::String(tpl)) = map.get("$template") {
                    let interpolated = interpolate_row_template(tpl, prefix, row, as_name);
                    if !contains_template_marker(&interpolated) {
                        *value = serde_json::Value::String(interpolated);
                        return;
                    } else {
                        map.insert(
                            "$template".to_string(),
                            serde_json::Value::String(interpolated),
                        );
                        return;
                    }
                }
            }
            for v in map.values_mut() {
                inline_walk(v, prefix, row, as_name);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                inline_walk(v, prefix, row, as_name);
            }
        }
        _ => {}
    }
}

fn interpolate_row_template(
    tpl: &str,
    prefix: &str,
    row: &serde_json::Value,
    as_name: &str,
) -> String {
    let mut out = String::with_capacity(tpl.len());
    let mut chars = tpl.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut path = String::new();
            let mut closed = false;
            for nc in chars.by_ref() {
                if nc == '}' {
                    closed = true;
                    break;
                }
                path.push(nc);
            }
            if closed {
                if let Some(rest) = path.strip_prefix(prefix) {
                    let resolved = crate::data::resolve_path(row, &format!("/{rest}"))
                        .map(value_to_string)
                        .unwrap_or_default();
                    out.push_str(&resolved);
                } else if path == format!("/{as_name}") {
                    out.push_str(&value_to_string(row));
                } else {
                    // Non-row template marker — leave intact for the
                    // downstream resolve_expressions pass.
                    out.push('{');
                    out.push_str(&path);
                    out.push('}');
                }
            } else {
                out.push('{');
                out.push_str(&path);
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn contains_template_marker(s: &str) -> bool {
    // Heuristic: any `{/...}` substring left in the string indicates a
    // downstream-resolvable placeholder.
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' && matches!(chars.peek(), Some('/')) {
            return true;
        }
    }
    false
}

fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Rewrite every element's `children` list so:
/// - IDs of `$if`-removed elements are pruned.
/// - IDs of `$each`-templated elements are replaced by their ordered clone
///   IDs (or dropped if the array was empty).
/// - Other IDs pass through unchanged.
///
/// Skips children that were ALREADY rewritten to correlated-child suffixes
/// by `expand_each` (those reference clones that exist in the map and are
/// NOT keys in `each_expanded`).
fn rewrite_parent_children(
    spec: &mut Spec,
    if_removed: &std::collections::HashSet<String>,
    each_expanded: &std::collections::HashMap<String, Vec<String>>,
) {
    for el in spec.elements.values_mut() {
        let mut new_children: Vec<String> = Vec::with_capacity(el.children.len());
        for child in el.children.drain(..) {
            if if_removed.contains(&child) {
                continue; // pruned
            }
            if let Some(clones) = each_expanded.get(&child) {
                new_children.extend(clones.iter().cloned());
            } else {
                new_children.push(child);
            }
        }
        el.children = new_children;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, HttpMethod};
    use crate::spec::{Element, Spec};

    fn action(handler: &str) -> Action {
        Action {
            handler: ActionHandler::Literal(handler.to_string()),
            url: None,
            method: HttpMethod::Post,
            confirm: None,
            on_success: None,
            on_error: None,
            target: None,
        }
    }

    #[test]
    fn resolve_actions_populates_url_from_resolver() {
        let mut spec = Spec::builder()
            .element("btn", Element::new("Button").action(action("users.create")))
            .build()
            .unwrap();

        resolve_actions(&mut spec, |h| {
            if h == "users.create" {
                Some("/users".to_string())
            } else {
                None
            }
        });

        let el = spec.elements.get("btn").unwrap();
        assert_eq!(el.action.as_ref().unwrap().url.as_deref(), Some("/users"));
    }

    // ── F14: ActionHandler binding resolution ───────────────────────────

    fn action_with_binding(path: &str) -> Action {
        Action {
            handler: ActionHandler::Binding(crate::spec::DataRef {
                data: path.to_string(),
            }),
            url: None,
            method: HttpMethod::Get,
            confirm: None,
            on_success: None,
            on_error: None,
            target: None,
        }
    }

    #[test]
    fn resolve_actions_resolves_binding_to_literal_path_via_spec_data() {
        let mut spec = Spec::builder()
            .data(serde_json::json!({ "row": { "url": "/dashboard/orders/42" } }))
            .element(
                "btn",
                Element::new("Button").action(action_with_binding("/row/url")),
            )
            .build()
            .unwrap();

        resolve_actions(&mut spec, |_| None);

        let el = spec.elements.get("btn").unwrap();
        assert_eq!(
            el.action.as_ref().unwrap().url.as_deref(),
            Some("/dashboard/orders/42"),
            "binding pointing to a `/path` string in spec.data resolves to action.url"
        );
    }

    #[test]
    fn resolve_actions_resolves_binding_to_named_handler_via_resolver() {
        let mut spec = Spec::builder()
            .data(serde_json::json!({ "row": { "handler": "users.show" } }))
            .element(
                "btn",
                Element::new("Button").action(action_with_binding("/row/handler")),
            )
            .build()
            .unwrap();

        resolve_actions(&mut spec, |name| {
            (name == "users.show").then(|| "/users/show".to_string())
        });

        let el = spec.elements.get("btn").unwrap();
        assert_eq!(
            el.action.as_ref().unwrap().url.as_deref(),
            Some("/users/show"),
            "binding resolved to handler name flows through the resolver"
        );
    }

    #[test]
    fn resolve_actions_binding_missing_data_leaves_url_unset() {
        let mut spec = Spec::builder()
            .data(serde_json::json!({}))
            .element(
                "btn",
                Element::new("Button").action(action_with_binding("/missing")),
            )
            .build()
            .unwrap();

        resolve_actions(&mut spec, |_| Some("UNEXPECTED".to_string()));

        let el = spec.elements.get("btn").unwrap();
        assert!(
            el.action.as_ref().unwrap().url.is_none(),
            "missing binding data leaves url unset (renderer emits diagnostic)"
        );
    }

    #[test]
    fn resolve_actions_skips_when_url_already_resolved() {
        let mut spec = Spec::builder()
            .data(serde_json::json!({ "row": { "url": "/from-data" } }))
            .element("btn", {
                let mut a = action_with_binding("/row/url");
                a.url = Some("/preset".to_string());
                Element::new("Button").action(a)
            })
            .build()
            .unwrap();

        resolve_actions(&mut spec, |_| None);

        let el = spec.elements.get("btn").unwrap();
        assert_eq!(el.action.as_ref().unwrap().url.as_deref(), Some("/preset"));
    }

    #[test]
    fn each_inlines_row_action_url_to_literal() {
        // $each over /cells; each row has action_url; per-row CalendarCell-style
        // action.handler binding points at /{as}/action_url. expand_each must
        // pre-resolve it to ActionHandler::Literal(row.action_url) so
        // resolve_actions can then set action.url.
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "grid",
            "elements": {
                "grid": {
                    "type": "Grid",
                    "props": {},
                    "children": ["cell"]
                },
                "cell": {
                    "type": "CalendarCell",
                    "$each": {"path": "/cells", "as": "c"},
                    "props": {"day": {"$data": "/c/day"}},
                    "action": {
                        "handler": {"$data": "/c/action_url"},
                        "method": "GET"
                    }
                }
            },
            "data": {
                "cells": [
                    {"day": 17, "action_url": "/dashboard/calendario?date=2026-05-17"},
                    {"day": 18, "action_url": "/dashboard/calendario?date=2026-05-18"}
                ]
            }
        }));

        expand_directives(&mut spec);
        resolve_actions(&mut spec, |_| None);

        let cell0 = spec.elements.get("cell-0").expect("expanded cell-0");
        let cell1 = spec.elements.get("cell-1").expect("expanded cell-1");
        assert_eq!(
            cell0.action.as_ref().unwrap().url.as_deref(),
            Some("/dashboard/calendario?date=2026-05-17"),
            "$each inlines /c/action_url to a literal; resolve_actions promotes literal to url"
        );
        assert_eq!(
            cell1.action.as_ref().unwrap().url.as_deref(),
            Some("/dashboard/calendario?date=2026-05-18")
        );
    }

    #[test]
    fn each_leaves_non_row_action_bindings_for_resolve_actions() {
        // Binding to /global_field (not /{as}/...) survives $each unchanged,
        // then resolve_actions resolves it against spec.data.
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "grid",
            "elements": {
                "grid": {
                    "type": "Grid",
                    "props": {},
                    "children": ["item"]
                },
                "item": {
                    "type": "Button",
                    "$each": {"path": "/items", "as": "i"},
                    "props": {"label": {"$data": "/i/label"}},
                    "action": {
                        "handler": {"$data": "/global_link"},
                        "method": "GET"
                    }
                }
            },
            "data": {
                "items": [{"label": "a"}, {"label": "b"}],
                "global_link": "/dashboard/shared"
            }
        }));

        expand_directives(&mut spec);
        resolve_actions(&mut spec, |_| None);

        let i0 = spec.elements.get("item-0").unwrap();
        let i1 = spec.elements.get("item-1").unwrap();
        assert_eq!(i0.action.as_ref().unwrap().url.as_deref(), Some("/dashboard/shared"));
        assert_eq!(i1.action.as_ref().unwrap().url.as_deref(), Some("/dashboard/shared"));
    }

    #[test]
    fn resolve_actions_passes_through_literal_paths() {
        let mut spec = Spec::builder()
            .element("btn", Element::new("Button").action(action("/dashboard")))
            .build()
            .unwrap();

        resolve_actions(&mut spec, |_| None);

        let el = spec.elements.get("btn").unwrap();
        assert_eq!(
            el.action.as_ref().unwrap().url.as_deref(),
            Some("/dashboard")
        );
    }

    #[test]
    fn resolve_actions_strict_reports_missing() {
        let mut spec = Spec::builder()
            .element(
                "btn",
                Element::new("Button").action(action("missing.handler")),
            )
            .build()
            .unwrap();

        let result = resolve_actions_strict(&mut spec, |_| None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec!["missing.handler".to_string()]);
    }

    #[test]
    fn resolve_errors_matches_by_name_prop() {
        let mut spec = Spec::builder()
            .element("email", Element::new("Input").prop("name", "email"))
            .build()
            .unwrap();

        let mut errors: HashMap<String, Vec<String>> = HashMap::new();
        errors.insert("email".to_string(), vec!["required".to_string()]);

        resolve_errors(&mut spec, &errors);

        let el = spec.elements.get("email").unwrap();
        let err_val = el.props.as_object().unwrap().get("errors").unwrap();
        assert_eq!(err_val, &serde_json::json!(["required"]));
    }

    #[test]
    fn resolve_errors_matches_by_field_prop() {
        let mut spec = Spec::builder()
            .element("email", Element::new("Input").prop("field", "email"))
            .build()
            .unwrap();

        let mut errors: HashMap<String, Vec<String>> = HashMap::new();
        errors.insert("email".to_string(), vec!["required".to_string()]);

        resolve_errors(&mut spec, &errors);

        let el = spec.elements.get("email").unwrap();
        let err_val = el.props.as_object().unwrap().get("errors").unwrap();
        assert_eq!(err_val, &serde_json::json!(["required"]));
    }

    #[test]
    fn resolve_errors_all_writes_full_bag_when_no_match() {
        let mut spec = Spec::builder()
            .element("card", Element::new("Card").prop("title", "t"))
            .build()
            .unwrap();

        let mut errors: HashMap<String, Vec<String>> = HashMap::new();
        errors.insert("email".to_string(), vec!["required".to_string()]);

        resolve_errors_all(&mut spec, &errors);

        let el = spec.elements.get("card").unwrap();
        let err_val = el.props.as_object().unwrap().get("errors").unwrap();
        assert_eq!(err_val["email"], serde_json::json!(["required"]));
    }

    // -----------------------------------------------------------------------
    // expand_directives tests (Phase 163 Plan 03) — $each / $if expansion
    // -----------------------------------------------------------------------

    fn parse_spec(json: serde_json::Value) -> Spec {
        serde_json::from_value::<Spec>(json).expect("spec parses")
    }

    #[test]
    fn expand_if_falsy_deletes_element() {
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "btn",
            "elements": {
                "btn": {
                    "type": "Button",
                    "$if": {"path": "/show", "operator": "eq", "value": true},
                    "props": {"label": "Hi"}
                }
            },
            "data": {"show": false}
        }));
        expand_directives(&mut spec);
        assert!(!spec.elements.contains_key("btn"));
    }

    #[test]
    fn expand_if_truthy_retains_element() {
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "btn",
            "elements": {
                "btn": {
                    "type": "Button",
                    "$if": {"path": "/show", "operator": "eq", "value": true},
                    "props": {"label": "Hi"}
                }
            },
            "data": {"show": true}
        }));
        expand_directives(&mut spec);
        let el = spec.elements.get("btn").expect("btn retained");
        assert!(
            el.if_.is_none(),
            "if_ stripped post-expansion for idempotency"
        );
    }

    #[test]
    fn expand_if_uses_visibility_evaluate() {
        // Compound And — exercises Visibility::And evaluation path verbatim.
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "btn",
            "elements": {
                "btn": {
                    "type": "Button",
                    "$if": {"and": [
                        {"path": "/a", "operator": "eq", "value": true},
                        {"path": "/b", "operator": "eq", "value": true}
                    ]},
                    "props": {"label": "Hi"}
                }
            },
            "data": {"a": true, "b": false}
        }));
        expand_directives(&mut spec);
        // And of (true, false) is false → element removed.
        assert!(!spec.elements.contains_key("btn"));
    }

    #[test]
    fn expand_each_produces_n_elements() {
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "order_card",
            "elements": {
                "order_card": {
                    "type": "Card",
                    "$each": {"path": "/orders", "as": "order"},
                    "props": {"title": {"$data": "/order/order_number"}}
                }
            },
            "data": {"orders": [
                {"order_number": "ORD-1"},
                {"order_number": "ORD-2"},
                {"order_number": "ORD-3"}
            ]}
        }));
        expand_directives(&mut spec);
        assert!(spec.elements.contains_key("order_card-0"));
        assert!(spec.elements.contains_key("order_card-1"));
        assert!(spec.elements.contains_key("order_card-2"));
        assert!(!spec.elements.contains_key("order_card"));
        let c0 = spec.elements.get("order_card-0").unwrap();
        assert_eq!(c0.props.get("title").unwrap(), &serde_json::json!("ORD-1"));
    }

    #[test]
    fn expand_each_auto_suffixes_ids() {
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "order_card",
            "elements": {
                "order_card": {
                    "type": "Card",
                    "$each": {"path": "/orders", "as": "order"},
                    "props": {}
                }
            },
            "data": {"orders": [{"x":1},{"x":2}]}
        }));
        expand_directives(&mut spec);
        for id in ["order_card-0", "order_card-1"] {
            let el = spec.elements.get(id).unwrap();
            assert!(el.each.is_none(), "{id} should have each stripped");
            assert!(el.if_.is_none(), "{id} should have if_ stripped");
        }
    }

    #[test]
    fn expand_each_pre_resolves_row_paths() {
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "order_card",
            "elements": {
                "order_card": {
                    "type": "Card",
                    "$each": {"path": "/orders", "as": "order"},
                    "props": {"title": {"$data": "/order/order_number"}}
                }
            },
            "data": {"orders": [{"order_number": "ORD-7"}]}
        }));
        expand_directives(&mut spec);
        let c0 = spec.elements.get("order_card-0").unwrap();
        assert_eq!(
            c0.props.get("title").unwrap(),
            &serde_json::json!("ORD-7"),
            "/order/X must be pre-resolved to a literal value"
        );
    }

    #[test]
    fn expand_each_correlates_child_indexes() {
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "root",
            "elements": {
                "root": {
                    "type": "Grid",
                    "props": {},
                    "children": ["card"]
                },
                "card": {
                    "type": "Card",
                    "$each": {"path": "/orders", "as": "order"},
                    "props": {},
                    "children": ["badge"]
                },
                "badge": {
                    "type": "Badge",
                    "$each": {"path": "/orders", "as": "order"},
                    "props": {"label": {"$data": "/order/status"}}
                }
            },
            "data": {"orders": [{"status": "A"}, {"status": "B"}]}
        }));
        expand_directives(&mut spec);
        let card0 = spec.elements.get("card-0").unwrap();
        assert_eq!(card0.children, vec!["badge-0".to_string()]);
        let card1 = spec.elements.get("card-1").unwrap();
        assert_eq!(card1.children, vec!["badge-1".to_string()]);
        let root = spec.elements.get("root").unwrap();
        assert_eq!(
            root.children,
            vec!["card-0".to_string(), "card-1".to_string()]
        );
    }

    #[test]
    fn expand_parent_children_rewritten_for_each() {
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "root",
            "elements": {
                "root": {
                    "type": "Grid",
                    "props": {},
                    "children": ["card"]
                },
                "card": {
                    "type": "Card",
                    "$each": {"path": "/orders", "as": "order"},
                    "props": {}
                }
            },
            "data": {"orders": [{"x":1},{"x":2},{"x":3}]}
        }));
        expand_directives(&mut spec);
        let root = spec.elements.get("root").unwrap();
        assert_eq!(
            root.children,
            vec![
                "card-0".to_string(),
                "card-1".to_string(),
                "card-2".to_string()
            ]
        );
    }

    #[test]
    fn expand_parent_children_pruned_for_if() {
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "root",
            "elements": {
                "root": {
                    "type": "Grid",
                    "props": {},
                    "children": ["btn"]
                },
                "btn": {
                    "type": "Button",
                    "$if": {"path": "/flag", "operator": "eq", "value": true},
                    "props": {"label": "Hi"}
                }
            },
            "data": {"flag": false}
        }));
        expand_directives(&mut spec);
        let root = spec.elements.get("root").unwrap();
        assert!(root.children.is_empty(), "pruned $if-false child");
        assert!(!spec.elements.contains_key("btn"));
    }

    #[test]
    fn expand_if_first_then_each() {
        // Element has BOTH $if (falsy) AND $each. $if removes the template before $each runs.
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "card",
            "elements": {
                "card": {
                    "type": "Card",
                    "$if": {"path": "/show", "operator": "eq", "value": true},
                    "$each": {"path": "/orders", "as": "order"},
                    "props": {}
                }
            },
            "data": {"show": false, "orders": [{"x":1},{"x":2}]}
        }));
        expand_directives(&mut spec);
        for id in ["card", "card-0", "card-1"] {
            assert!(
                !spec.elements.contains_key(id),
                "{id} must not exist when $if removed the template"
            );
        }
    }

    #[test]
    fn expand_each_empty_array_produces_zero_clones() {
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "root",
            "elements": {
                "root": {
                    "type": "Grid",
                    "props": {},
                    "children": ["card"]
                },
                "card": {
                    "type": "Card",
                    "$each": {"path": "/orders", "as": "order"},
                    "props": {}
                }
            },
            "data": {"orders": []}
        }));
        expand_directives(&mut spec);
        assert!(!spec.elements.contains_key("card"));
        let root = spec.elements.get("root").unwrap();
        assert!(root.children.is_empty());
    }

    #[test]
    fn expand_directives_idempotent() {
        let mut spec = parse_spec(serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "root",
            "elements": {
                "root": {
                    "type": "Grid",
                    "props": {},
                    "children": ["card"]
                },
                "card": {
                    "type": "Card",
                    "$each": {"path": "/orders", "as": "order"},
                    "props": {"title": {"$data": "/order/name"}}
                }
            },
            "data": {"orders": [{"name": "A"}, {"name": "B"}]}
        }));
        expand_directives(&mut spec);
        let snapshot_after_first = serde_json::to_value(&spec.elements).unwrap();
        expand_directives(&mut spec);
        let snapshot_after_second = serde_json::to_value(&spec.elements).unwrap();
        assert_eq!(
            snapshot_after_first, snapshot_after_second,
            "expand_directives must be idempotent"
        );
    }
}
