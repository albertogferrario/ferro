//! Server-side expression resolver for v2 JSON-UI Specs.
//!
//! Walks every `Element.props` in a spec's flat element map and substitutes
//! two narrow expression object shapes against `Spec.data`:
//!
//! - `{"$data": "/slash/path"}` — replaced with the JSON value at the path,
//!   preserving type. Missing paths resolve to `Value::Null`.
//! - `{"$template": "literal {/path} more"}` — replaced with a `Value::String`
//!   built by substituting every `{slash-path}` placeholder via
//!   `resolve_path_string`. Missing placeholders interpolate to `""`.
//!
//! **Infallible:** malformed expressions (non-string value, sibling keys)
//! degrade to literal JSON — no panic, no log, no `Result`.
//!
//! **Single-pass:** expressions inside resolved `$data` output are NOT
//! re-resolved. This is the inner-platform-effect firewall (Phase 118 D-07).
//!
//! **Scope:** only `el.props` is walked. `Spec.data`, `Spec.title`,
//! `Spec.layout`, `el.children`, `el.action`, `el.visible` are untouched.
//!
//! **Pipeline order:** must run after `resolve_actions` and before
//! `Catalog::validate` (Phase 118 D-08).

use serde_json::{Map, Value};

use crate::data::{resolve_path, resolve_path_string};
use crate::spec::Spec;

const EXPR_DATA_KEY: &str = "$data";
const EXPR_TEMPLATE_KEY: &str = "$template";

/// Resolve every `$data` and `$template` expression in `spec.elements[*].props`.
///
/// Mutates in place. Returns `()` — infallible.
pub fn resolve_expressions(spec: &mut Spec) {
    let data = spec.data.clone();
    for el in spec.elements.values_mut() {
        resolve_value(&mut el.props, &data);
    }
}

fn resolve_value(val: &mut Value, data: &Value) {
    match val {
        Value::Object(map) => {
            if let Some(path) = is_data_expr(map) {
                let path = path.to_owned();
                *val = resolve_path(data, &path).cloned().unwrap_or(Value::Null);
                // Single-pass: do NOT recurse into the resolved value (D-07).
            } else if let Some(tmpl) = is_template_expr(map) {
                let tmpl = tmpl.to_owned();
                *val = Value::String(substitute_template(&tmpl, data));
                // Single-pass: do NOT recurse into the resolved string.
            } else {
                for v in map.values_mut() {
                    resolve_value(v, data);
                }
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                resolve_value(v, data);
            }
        }
        _ => {}
    }
}

fn is_data_expr(obj: &Map<String, Value>) -> Option<&str> {
    if obj.len() == 1 {
        if let Some(Value::String(path)) = obj.get(EXPR_DATA_KEY) {
            return Some(path.as_str());
        }
    }
    None
}

fn is_template_expr(obj: &Map<String, Value>) -> Option<&str> {
    if obj.len() == 1 {
        if let Some(Value::String(tmpl)) = obj.get(EXPR_TEMPLATE_KEY) {
            return Some(tmpl.as_str());
        }
    }
    None
}

fn substitute_template(template: &str, data: &Value) -> String {
    let mut out = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => match chars.peek() {
                Some('{') => {
                    out.push('{');
                    chars.next();
                }
                Some('}') => {
                    out.push('}');
                    chars.next();
                }
                Some('\\') => {
                    out.push('\\');
                    chars.next();
                }
                _ => out.push('\\'),
            },
            '{' => {
                let mut path = String::new();
                let mut closed = false;
                for inner in chars.by_ref() {
                    if inner == '}' {
                        closed = true;
                        break;
                    }
                    path.push(inner);
                }
                if closed {
                    let trimmed = path.trim();
                    let resolved = resolve_path_string(data, trimmed).unwrap_or_default();
                    out.push_str(&resolved);
                } else {
                    out.push('{');
                    out.push_str(&path);
                }
            }
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{Element, Spec};
    use crate::visibility::{Visibility, VisibilityCondition, VisibilityOperator};
    use serde_json::json;

    /// Build a one-element spec with `props` stored under key "x" and run
    /// `resolve_expressions`. Returns the post-resolution value at `props.x`.
    fn run(data: Value, props: Value) -> Value {
        let mut spec = Spec::builder()
            .data(data)
            .element("root", Element::new("Text").prop("x", props))
            .build()
            .unwrap();
        resolve_expressions(&mut spec);
        spec.elements
            .get("root")
            .unwrap()
            .props
            .get("x")
            .cloned()
            .unwrap_or(Value::Null)
    }

    // --- $data: success cases -------------------------------------------------
    #[test]
    fn data_simple_path() {
        let out = run(json!({ "name": "Alice" }), json!({ "$data": "/name" }));
        assert_eq!(out, json!("Alice"));
    }

    #[test]
    fn data_nested_path() {
        let out = run(
            json!({ "user": { "name": "Bob" } }),
            json!({ "$data": "/user/name" }),
        );
        assert_eq!(out, json!("Bob"));
    }

    #[test]
    fn data_array_index() {
        let out = run(
            json!({ "items": ["x", "y"] }),
            json!({ "$data": "/items/0" }),
        );
        assert_eq!(out, json!("x"));
    }

    #[test]
    fn data_preserves_number() {
        let out = run(json!({ "count": 42 }), json!({ "$data": "/count" }));
        assert_eq!(out, json!(42));
        assert!(out.is_number());
    }

    #[test]
    fn data_preserves_bool() {
        let out = run(json!({ "active": true }), json!({ "$data": "/active" }));
        assert_eq!(out, json!(true));
        assert!(out.is_boolean());
    }

    #[test]
    fn data_preserves_object() {
        let payload = json!({ "user": { "name": "Carol", "age": 30 } });
        let out = run(payload.clone(), json!({ "$data": "/user" }));
        assert_eq!(out, json!({ "name": "Carol", "age": 30 }));
        assert!(out.is_object());
    }

    #[test]
    fn data_preserves_array() {
        let payload = json!({ "items": [1, 2, 3] });
        let out = run(payload, json!({ "$data": "/items" }));
        assert_eq!(out, json!([1, 2, 3]));
        assert!(out.is_array());
    }

    #[test]
    fn data_missing_path() {
        let out = run(json!({ "name": "Alice" }), json!({ "$data": "/missing" }));
        assert_eq!(out, Value::Null);
    }

    // --- $data: passthrough cases ---------------------------------------------
    #[test]
    fn data_non_string_value() {
        let input = json!({ "$data": 42 });
        let out = run(json!({}), input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn data_sibling_keys() {
        let input = json!({ "$data": "/x", "class": "y" });
        let out = run(json!({ "x": "resolved" }), input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn data_null_value() {
        let input = json!({ "$data": null });
        let out = run(json!({}), input.clone());
        assert_eq!(out, input);
    }

    // --- $template: success cases ---------------------------------------------
    #[test]
    fn template_single_placeholder() {
        let out = run(
            json!({ "name": "Alice" }),
            json!({ "$template": "Hi, {/name}!" }),
        );
        assert_eq!(out, json!("Hi, Alice!"));
    }

    #[test]
    fn template_multiple_placeholders() {
        let out = run(
            json!({ "a": "v1", "b": "v2" }),
            json!({ "$template": "{/a} and {/b}" }),
        );
        assert_eq!(out, json!("v1 and v2"));
    }

    #[test]
    fn template_no_placeholder() {
        let out = run(json!({}), json!({ "$template": "static text" }));
        assert_eq!(out, json!("static text"));
    }

    #[test]
    fn template_missing_placeholder() {
        let out = run(json!({}), json!({ "$template": "before {/missing} after" }));
        assert_eq!(out, json!("before  after"));
    }

    #[test]
    fn template_whitespace_trimmed() {
        let out = run(
            json!({ "name": "Alice" }),
            json!({ "$template": "{ /name }" }),
        );
        assert_eq!(out, json!("Alice"));
    }

    // --- $template: escape cases ----------------------------------------------
    #[test]
    fn template_escaped_open_brace() {
        let out = run(json!({}), json!({ "$template": "\\{not a placeholder}" }));
        assert_eq!(out, json!("{not a placeholder}"));
    }

    #[test]
    fn template_escaped_close_brace() {
        let out = run(json!({}), json!({ "$template": "text\\}" }));
        assert_eq!(out, json!("text}"));
    }

    #[test]
    fn template_escaped_backslash() {
        let out = run(json!({}), json!({ "$template": "a\\\\b" }));
        assert_eq!(out, json!("a\\b"));
    }

    #[test]
    fn template_unclosed_brace() {
        let out = run(json!({}), json!({ "$template": "{/missing_close" }));
        assert_eq!(out, json!("{/missing_close"));
    }

    // --- $template: passthrough cases -----------------------------------------
    #[test]
    fn template_non_string_value() {
        let input = json!({ "$template": 42 });
        let out = run(json!({}), input.clone());
        assert_eq!(out, input);
    }

    // --- nested expressions ---------------------------------------------------
    #[test]
    fn nested_in_array() {
        let out = run(
            json!({ "x": "resolved" }),
            json!([{ "key": "lit" }, { "$data": "/x" }]),
        );
        assert_eq!(out, json!([{ "key": "lit" }, "resolved"]));
    }

    #[test]
    fn nested_in_object_values() {
        let out = run(
            json!({ "x": "resolved" }),
            json!({ "inner": { "$data": "/x" } }),
        );
        assert_eq!(out, json!({ "inner": "resolved" }));
    }

    // --- scope restrictions ---------------------------------------------------
    #[test]
    fn does_not_touch_spec_data() {
        let data = json!({ "marker": { "$data": "/should_not_resolve" }, "target": "v" });
        let mut spec = Spec::builder()
            .data(data.clone())
            .element(
                "root",
                Element::new("Text").prop("x", json!({ "$data": "/target" })),
            )
            .build()
            .unwrap();
        resolve_expressions(&mut spec);
        assert_eq!(
            spec.data, data,
            "spec.data must be untouched even when it contains $data markers"
        );
        assert_eq!(
            spec.elements.get("root").unwrap().props.get("x"),
            Some(&json!("v"))
        );
    }

    #[test]
    fn single_pass_no_recursion() {
        let out = run(
            json!({ "outer": { "$data": "/inner" }, "inner": "never" }),
            json!({ "$data": "/outer" }),
        );
        assert_eq!(
            out,
            json!({ "$data": "/inner" }),
            "single-pass: $data output containing a marker must NOT be re-resolved"
        );
    }

    #[test]
    fn does_not_touch_children() {
        let mut spec = Spec::builder()
            .data(json!({ "child1": "resolved" }))
            .element(
                "root",
                Element::new("Text")
                    .prop("x", json!("literal"))
                    .child("child1"),
            )
            .element("child1", Element::new("Text").prop("y", json!("leaf")))
            .build()
            .unwrap();
        let before = spec.elements.get("root").unwrap().children.clone();
        resolve_expressions(&mut spec);
        assert_eq!(spec.elements.get("root").unwrap().children, before);
    }

    #[test]
    fn does_not_touch_visible() {
        let visible = Visibility::Condition(VisibilityCondition {
            path: "/flag".to_string(),
            operator: VisibilityOperator::Exists,
            value: None,
        });
        let mut spec = Spec::builder()
            .data(json!({ "flag": true }))
            .element(
                "root",
                Element::new("Text")
                    .prop("x", json!("lit"))
                    .visible(visible.clone()),
            )
            .build()
            .unwrap();
        resolve_expressions(&mut spec);
        assert_eq!(spec.elements.get("root").unwrap().visible, Some(visible));
    }

    #[test]
    fn plugin_props_walk_identically() {
        let mut spec = Spec::builder()
            .data(json!({ "x": "resolved" }))
            .element(
                "root",
                Element::new("MyPlugin").prop("x", json!({ "$data": "/x" })),
            )
            .build()
            .unwrap();
        resolve_expressions(&mut spec);
        assert_eq!(
            spec.elements.get("root").unwrap().props.get("x"),
            Some(&json!("resolved")),
            "plugin-typed props resolve identically to built-in props (D-14)"
        );
    }
}
