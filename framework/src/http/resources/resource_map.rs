use serde_json::{Map, Value};

/// Internal representation for conditional field inclusion.
enum ResourceValue {
    Present(Value),
    Missing,
}

/// Builder for constructing JSON resource representations with conditional field support.
///
/// `ResourceMap` collects field name-value pairs and supports conditional inclusion
/// via `when()`, `unless()`, `merge_when()`, and `when_some()`. Fields are output
/// in insertion order.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::ResourceMap;
/// use serde_json::json;
///
/// let value = ResourceMap::new()
///     .field("id", json!(1))
///     .field("name", json!("Alice"))
///     .when("email", is_admin, || json!("alice@example.com"))
///     .build();
/// ```
pub struct ResourceMap {
    fields: Vec<(String, ResourceValue)>,
}

impl ResourceMap {
    /// Create an empty resource map.
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Always include this field in the output.
    pub fn field(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.fields
            .push((key.to_string(), ResourceValue::Present(value.into())));
        self
    }

    /// Include field only when `condition` is true.
    /// The value closure is only evaluated when the condition holds.
    pub fn when(mut self, key: &str, condition: bool, value: impl FnOnce() -> Value) -> Self {
        if condition {
            self.fields
                .push((key.to_string(), ResourceValue::Present(value())));
        } else {
            self.fields.push((key.to_string(), ResourceValue::Missing));
        }
        self
    }

    /// Include field only when `condition` is false (opposite of `when`).
    pub fn unless(self, key: &str, condition: bool, value: impl FnOnce() -> Value) -> Self {
        self.when(key, !condition, value)
    }

    /// Conditionally merge multiple fields at once.
    /// When the condition is true, all fields from the closure are included.
    pub fn merge_when(
        mut self,
        condition: bool,
        fields: impl FnOnce() -> Vec<(&'static str, Value)>,
    ) -> Self {
        if condition {
            for (key, value) in fields() {
                self.fields
                    .push((key.to_string(), ResourceValue::Present(value)));
            }
        }
        self
    }

    /// Include field only if the `Option` is `Some`.
    pub fn when_some<T: serde::Serialize>(mut self, key: &str, value: &Option<T>) -> Self {
        if let Some(v) = value {
            self.fields.push((
                key.to_string(),
                ResourceValue::Present(serde_json::to_value(v).unwrap_or(Value::Null)),
            ));
        } else {
            self.fields.push((key.to_string(), ResourceValue::Missing));
        }
        self
    }

    /// Finalize into a JSON object, stripping fields marked as missing.
    /// Preserves insertion order.
    pub fn build(self) -> Value {
        let mut map = Map::new();
        for (key, value) in self.fields {
            if let ResourceValue::Present(v) = value {
                map.insert(key, v);
            }
        }
        Value::Object(map)
    }
}

impl Default for ResourceMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_field_basic() {
        let value = ResourceMap::new()
            .field("id", json!(1))
            .field("name", json!("Alice"))
            .build();

        assert_eq!(value, json!({"id": 1, "name": "Alice"}));
    }

    #[test]
    fn test_when_true() {
        let value = ResourceMap::new()
            .field("id", json!(1))
            .when("email", true, || json!("a@b.com"))
            .build();

        assert_eq!(value, json!({"id": 1, "email": "a@b.com"}));
    }

    #[test]
    fn test_when_false() {
        let value = ResourceMap::new()
            .field("id", json!(1))
            .when("email", false, || json!("a@b.com"))
            .build();

        assert_eq!(value, json!({"id": 1}));
    }

    #[test]
    fn test_unless_true() {
        let value = ResourceMap::new()
            .field("id", json!(1))
            .unless("debug", true, || json!(true))
            .build();

        assert_eq!(value, json!({"id": 1}));
    }

    #[test]
    fn test_unless_false() {
        let value = ResourceMap::new()
            .field("id", json!(1))
            .unless("debug", false, || json!(true))
            .build();

        assert_eq!(value, json!({"id": 1, "debug": true}));
    }

    #[test]
    fn test_merge_when_true() {
        let value = ResourceMap::new()
            .field("id", json!(1))
            .merge_when(true, || vec![("a", json!(1)), ("b", json!(2))])
            .build();

        assert_eq!(value, json!({"id": 1, "a": 1, "b": 2}));
    }

    #[test]
    fn test_merge_when_false() {
        let value = ResourceMap::new()
            .field("id", json!(1))
            .merge_when(false, || vec![("a", json!(1)), ("b", json!(2))])
            .build();

        assert_eq!(value, json!({"id": 1}));
    }

    #[test]
    fn test_when_some_present() {
        let bio: Option<&str> = Some("hello");
        let value = ResourceMap::new()
            .field("id", json!(1))
            .when_some("bio", &bio)
            .build();

        assert_eq!(value, json!({"id": 1, "bio": "hello"}));
    }

    #[test]
    fn test_when_some_none() {
        let bio: Option<String> = None;
        let value = ResourceMap::new()
            .field("id", json!(1))
            .when_some("bio", &bio)
            .build();

        assert_eq!(value, json!({"id": 1}));
    }

    #[test]
    fn test_field_order_preserved() {
        let value = ResourceMap::new()
            .field("c", json!(3))
            .field("a", json!(1))
            .field("b", json!(2))
            .build();

        let keys: Vec<&String> = value.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["c", "a", "b"]);
    }
}
