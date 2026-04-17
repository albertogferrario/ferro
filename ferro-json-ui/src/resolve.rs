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
}
