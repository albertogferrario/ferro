//! Resolvers for v2 JSON-UI Spec element maps.
//!
//! Walks a `Spec`'s flat element map and resolves action handler names to
//! URLs, or populates per-field validation errors on form-like elements.
//!
//! Phase 115: flat iteration only. No tree descent — children are ID
//! strings, not nested structs. Action resolution is per-element.

use std::collections::HashMap;

use serde_json::Value;

use crate::action::Action;
use crate::spec::{Element, Spec};

/// Resolve a single action using the callback. Literal paths (starting
/// with '/') are passed through as-is so callers can use
/// `Action::get("/dashboard/...")` without registering a named route.
fn resolve_action(action: &mut Action, resolver: &impl Fn(&str) -> Option<String>) {
    if action.url.is_none() {
        if action.handler.starts_with('/') {
            action.url = Some(action.handler.clone());
            return;
        }
        if let Some(url) = resolver(&action.handler) {
            action.url = Some(url);
        }
    }
}

/// Resolve every `Element.action` via the provided resolver closure.
///
/// Mutates in place. Silent on missing handlers — use
/// `resolve_actions_strict` if you want to collect missing names.
pub fn resolve_actions(spec: &mut Spec, resolver: impl Fn(&str) -> Option<String>) {
    for el in spec.elements.values_mut() {
        if let Some(action) = el.action.as_mut() {
            resolve_action(action, &resolver);
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
    let mut missing: Vec<String> = Vec::new();
    for el in spec.elements.values_mut() {
        if let Some(action) = el.action.as_mut() {
            resolve_action(action, &resolver);
            if action.url.is_none() {
                missing.push(action.handler.clone());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, HttpMethod};
    use crate::spec::{Element, Spec};

    fn action(handler: &str) -> Action {
        Action {
            handler: handler.to_string(),
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
        assert!(el.if_.is_none(), "if_ stripped post-expansion for idempotency");
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
