//! Data path resolution for JSON-UI.
//!
//! Resolves slash-separated paths against JSON values, enabling components
//! to reference dynamic data from handler payloads. Path format: `/segment/segment/...`
//! where segments are object keys or array indices (numeric strings).

use serde_json::Value;

/// Resolves a slash-separated path against a JSON value.
///
/// Path format: `/segment/segment/...` where each segment is an object key
/// or array index (numeric string). Leading slash is required for non-empty
/// paths. Empty path or `"/"` returns the root value. Returns `None` if any
/// segment fails to resolve.
///
/// # Examples
///
/// ```
/// use ferro_json_ui::resolve_path;
/// use serde_json::json;
///
/// let data = json!({"user": {"name": "Alice"}});
/// let result = resolve_path(&data, "/user/name");
/// assert_eq!(result, Some(&json!("Alice")));
/// ```
pub fn resolve_path<'a>(data: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() || path == "/" {
        return Some(data);
    }

    let trimmed = path.strip_prefix('/').unwrap_or(path);
    let segments: Vec<&str> = trimmed.split('/').collect();

    let mut current = data;
    for segment in segments {
        if segment.is_empty() {
            continue;
        }
        match current {
            Value::Object(map) => {
                current = map.get(segment)?;
            }
            Value::Array(arr) => {
                let index: usize = segment.parse().ok()?;
                current = arr.get(index)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

/// Resolves a path and converts the result to a string representation.
///
/// For `String` values, returns the string directly. For numbers and booleans,
/// uses `to_string()`. For `null`, returns `None`. For objects and arrays,
/// returns their JSON serialization.
///
/// # Examples
///
/// ```
/// use ferro_json_ui::resolve_path_string;
/// use serde_json::json;
///
/// let data = json!({"count": 42, "name": "Alice"});
/// assert_eq!(resolve_path_string(&data, "/name"), Some("Alice".to_string()));
/// assert_eq!(resolve_path_string(&data, "/count"), Some("42".to_string()));
/// ```
pub fn resolve_path_string(data: &Value, path: &str) -> Option<String> {
    let value = resolve_path(data, path)?;
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => None,
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn simple_key_resolution() {
        let data = json!({"name": "Alice"});
        assert_eq!(resolve_path(&data, "/name"), Some(&json!("Alice")));
    }

    #[test]
    fn nested_key_resolution() {
        let data = json!({"user": {"name": "Bob"}});
        assert_eq!(resolve_path(&data, "/user/name"), Some(&json!("Bob")));
    }

    #[test]
    fn array_index_resolution() {
        let data = json!({"users": [{"name": "Carol"}]});
        assert_eq!(resolve_path(&data, "/users/0/name"), Some(&json!("Carol")));
    }

    #[test]
    fn missing_key_returns_none() {
        let data = json!({"name": "Alice"});
        assert_eq!(resolve_path(&data, "/missing"), None);
    }

    #[test]
    fn empty_path_returns_root() {
        let data = json!({"name": "Alice"});
        assert_eq!(resolve_path(&data, ""), Some(&data));
    }

    #[test]
    fn root_slash_returns_root() {
        let data = json!({"name": "Alice"});
        assert_eq!(resolve_path(&data, "/"), Some(&data));
    }

    #[test]
    fn numeric_value_resolution() {
        let data = json!({"count": 42});
        let result = resolve_path(&data, "/count");
        assert_eq!(result, Some(&json!(42)));
        assert!(result.unwrap().is_number());
    }

    #[test]
    fn boolean_resolution() {
        let data = json!({"active": true});
        let result = resolve_path(&data, "/active");
        assert_eq!(result, Some(&json!(true)));
        assert!(result.unwrap().is_boolean());
    }

    #[test]
    fn null_value_resolve_path() {
        let data = json!({"deleted_at": null});
        let result = resolve_path(&data, "/deleted_at");
        assert_eq!(result, Some(&Value::Null));
    }

    #[test]
    fn null_value_resolve_path_string_returns_none() {
        let data = json!({"deleted_at": null});
        assert_eq!(resolve_path_string(&data, "/deleted_at"), None);
    }

    #[test]
    fn deep_nesting() {
        let data = json!({"a": {"b": {"c": {"d": "deep"}}}});
        assert_eq!(resolve_path(&data, "/a/b/c/d"), Some(&json!("deep")));
    }

    #[test]
    fn invalid_array_index_returns_none() {
        let data = json!({"items": [1, 2, 3]});
        assert_eq!(resolve_path(&data, "/items/5"), None);
        assert_eq!(resolve_path(&data, "/items/abc"), None);
    }

    #[test]
    fn resolve_path_string_for_string() {
        let data = json!({"name": "Alice"});
        assert_eq!(
            resolve_path_string(&data, "/name"),
            Some("Alice".to_string())
        );
    }

    #[test]
    fn resolve_path_string_for_number() {
        let data = json!({"count": 42});
        assert_eq!(resolve_path_string(&data, "/count"), Some("42".to_string()));
    }

    #[test]
    fn resolve_path_string_for_boolean() {
        let data = json!({"active": true});
        assert_eq!(
            resolve_path_string(&data, "/active"),
            Some("true".to_string())
        );
    }

    #[test]
    fn resolve_path_string_for_object() {
        let data = json!({"user": {"name": "Alice"}});
        let result = resolve_path_string(&data, "/user");
        assert_eq!(result, Some(r#"{"name":"Alice"}"#.to_string()));
    }

    #[test]
    fn resolve_path_string_for_array() {
        let data = json!({"items": [1, 2, 3]});
        let result = resolve_path_string(&data, "/items");
        assert_eq!(result, Some("[1,2,3]".to_string()));
    }

    #[test]
    fn resolve_path_string_missing_returns_none() {
        let data = json!({"name": "Alice"});
        assert_eq!(resolve_path_string(&data, "/missing"), None);
    }

    #[test]
    fn resolve_path_on_non_object_non_array() {
        let data = json!("just a string");
        assert_eq!(resolve_path(&data, "/anything"), None);
    }

    #[test]
    fn nested_array_access() {
        let data = json!({"matrix": [[1, 2], [3, 4]]});
        assert_eq!(resolve_path(&data, "/matrix/1/0"), Some(&json!(3)));
    }
}
