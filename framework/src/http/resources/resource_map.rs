use std::collections::HashMap;
use std::hash::Hash;

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

    /// Include field if `lookup_key` exists in the given `HashMap` (belongs_to / has_one).
    ///
    /// When the key is found, `transform` converts the value to JSON.
    /// When absent the field is omitted from output.
    pub fn when_loaded<K, M>(
        mut self,
        key: &str,
        lookup_key: &K,
        map: &HashMap<K, M>,
        transform: impl FnOnce(&M) -> Value,
    ) -> Self
    where
        K: Eq + Hash,
    {
        if let Some(item) = map.get(lookup_key) {
            self.fields
                .push((key.to_string(), ResourceValue::Present(transform(item))));
        } else {
            self.fields.push((key.to_string(), ResourceValue::Missing));
        }
        self
    }

    /// Include field if `lookup_key` exists in the given `HashMap<K, Vec<M>>` (has_many).
    ///
    /// When the key is found, `transform` receives the vec slice.
    /// When absent the field is omitted from output.
    /// An empty vec is still included (loaded but empty).
    pub fn when_loaded_many<K, M>(
        mut self,
        key: &str,
        lookup_key: &K,
        map: &HashMap<K, Vec<M>>,
        transform: impl FnOnce(&[M]) -> Value,
    ) -> Self
    where
        K: Eq + Hash,
    {
        if let Some(items) = map.get(lookup_key) {
            self.fields
                .push((key.to_string(), ResourceValue::Present(transform(items))));
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
    fn test_when_loaded_present() {
        let mut authors = HashMap::new();
        authors.insert(10, "Alice".to_string());

        let value = ResourceMap::new()
            .field("id", json!(1))
            .when_loaded("author", &10, &authors, |name| json!(name))
            .build();

        assert_eq!(value, json!({"id": 1, "author": "Alice"}));
    }

    #[test]
    fn test_when_loaded_missing() {
        let authors: HashMap<i32, String> = HashMap::new();

        let value = ResourceMap::new()
            .field("id", json!(1))
            .when_loaded("author", &10, &authors, |name| json!(name))
            .build();

        assert_eq!(value, json!({"id": 1}));
    }

    #[test]
    fn test_when_loaded_many_present() {
        let mut tags = HashMap::new();
        tags.insert(1, vec!["rust", "web"]);

        let value = ResourceMap::new()
            .field("id", json!(1))
            .when_loaded_many("tags", &1, &tags, |t| json!(t))
            .build();

        assert_eq!(value, json!({"id": 1, "tags": ["rust", "web"]}));
    }

    #[test]
    fn test_when_loaded_many_missing() {
        let tags: HashMap<i32, Vec<&str>> = HashMap::new();

        let value = ResourceMap::new()
            .field("id", json!(1))
            .when_loaded_many("tags", &1, &tags, |t| json!(t))
            .build();

        assert_eq!(value, json!({"id": 1}));
    }

    #[test]
    fn test_when_loaded_many_empty_vec() {
        let mut tags: HashMap<i32, Vec<&str>> = HashMap::new();
        tags.insert(1, vec![]);

        let value = ResourceMap::new()
            .field("id", json!(1))
            .when_loaded_many("tags", &1, &tags, |t| json!(t))
            .build();

        assert_eq!(value, json!({"id": 1, "tags": []}));
    }

    #[test]
    fn test_when_loaded_combined() {
        let mut authors = HashMap::new();
        authors.insert(10, "Alice".to_string());

        let mut tags: HashMap<i32, Vec<&str>> = HashMap::new();
        tags.insert(1, vec!["rust", "web"]);

        // author present, tags present
        let value = ResourceMap::new()
            .field("id", json!(1))
            .when_loaded("author", &10, &authors, |name| json!(name))
            .when_loaded_many("tags", &1, &tags, |t| json!(t))
            .build();

        assert_eq!(
            value,
            json!({"id": 1, "author": "Alice", "tags": ["rust", "web"]})
        );

        // author missing, tags present
        let value = ResourceMap::new()
            .field("id", json!(2))
            .when_loaded("author", &99, &authors, |name| json!(name))
            .when_loaded_many("tags", &1, &tags, |t| json!(t))
            .build();

        assert_eq!(value, json!({"id": 2, "tags": ["rust", "web"]}));

        // author present, tags missing
        let value = ResourceMap::new()
            .field("id", json!(3))
            .when_loaded("author", &10, &authors, |name| json!(name))
            .when_loaded_many("tags", &99, &tags, |t| json!(t))
            .build();

        assert_eq!(value, json!({"id": 3, "author": "Alice"}));
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
