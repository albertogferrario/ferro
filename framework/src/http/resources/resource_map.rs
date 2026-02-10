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
    pub fn when(
        mut self,
        key: &str,
        condition: bool,
        value: impl FnOnce() -> Value,
    ) -> Self {
        if condition {
            self.fields
                .push((key.to_string(), ResourceValue::Present(value())));
        } else {
            self.fields
                .push((key.to_string(), ResourceValue::Missing));
        }
        self
    }

    /// Include field only when `condition` is false (opposite of `when`).
    pub fn unless(
        self,
        key: &str,
        condition: bool,
        value: impl FnOnce() -> Value,
    ) -> Self {
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
                ResourceValue::Present(
                    serde_json::to_value(v).unwrap_or(Value::Null),
                ),
            ));
        } else {
            self.fields
                .push((key.to_string(), ResourceValue::Missing));
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
