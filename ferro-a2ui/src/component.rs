//! A2UI component wire type — one entry in a surface's flat adjacency list.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A single component in a surface's flat component list.
///
/// Wire shape: `{ "id": "…", "component": "Text", …props }` — props are
/// flattened to the top level. Parents reference children by ID via the
/// `children` / `child` props; containers may bind list children with
/// `{ "path": …, "componentId": … }` template bindings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    /// Unique component ID within the surface. Exactly one component per
    /// surface carries the ID `root`.
    pub id: String,
    /// Component type name (e.g. `Text`, `Row`, `Button`).
    pub component: String,
    /// Type-specific props, flattened onto the wire object.
    #[serde(flatten)]
    pub props: Map<String, Value>,
}

impl Component {
    /// Creates a component with the given ID and type name.
    pub fn new(id: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            component: type_name.into(),
            props: Map::new(),
        }
    }

    /// Sets a literal prop.
    pub fn prop(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.props.insert(key.into(), value.into());
        self
    }

    /// Sets the single-child reference (`child`).
    pub fn child(self, id: impl Into<String>) -> Self {
        let id: String = id.into();
        self.prop("child", id)
    }

    /// Sets the child-ID list (`children`).
    pub fn children_ids<I, S>(self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let ids: Vec<Value> = ids.into_iter().map(|s| Value::String(s.into())).collect();
        self.prop("children", ids)
    }

    /// Sets a template child binding: the client instantiates `component_id`
    /// once per item of the list bound at `path`.
    pub fn template_children(
        self,
        path: impl Into<String>,
        component_id: impl Into<String>,
    ) -> Self {
        self.prop(
            "children",
            serde_json::json!({"path": path.into(), "componentId": component_id.into()}),
        )
    }

    /// Sets a data-bound prop: `{ "path": <json-pointer> }`.
    pub fn bound_prop(self, key: impl Into<String>, path: impl Into<String>) -> Self {
        self.prop(key, serde_json::json!({"path": path.into()}))
    }

    /// Sets the `action` prop (see [`crate::actions`]).
    pub fn action(self, action: Value) -> Self {
        self.prop("action", action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_serializes_flat_with_props_at_top_level() {
        let c = Component::new("title", "Text")
            .prop("text", "Order")
            .prop("variant", "heading");
        let v = serde_json::to_value(&c).unwrap();
        assert_eq!(
            v,
            serde_json::json!({"id": "title", "component": "Text", "text": "Order", "variant": "heading"})
        );
    }

    #[test]
    fn template_children_binding_shape() {
        let c = Component::new("items", "List").template_children("/items", "row");
        let v = serde_json::to_value(&c).unwrap();
        assert_eq!(
            v["children"],
            serde_json::json!({"path": "/items", "componentId": "row"})
        );
    }

    #[test]
    fn children_ids_and_single_child() {
        let row = Component::new("root", "Column").children_ids(["a", "b"]);
        assert_eq!(
            serde_json::to_value(&row).unwrap()["children"],
            serde_json::json!(["a", "b"])
        );
        let card = Component::new("card", "Card").child("row");
        assert_eq!(
            serde_json::to_value(&card).unwrap()["child"],
            serde_json::json!("row")
        );
    }

    #[test]
    fn bound_prop_emits_json_pointer_binding() {
        let c = Component::new("t", "Text").bound_prop("text", "/entity/name");
        assert_eq!(
            serde_json::to_value(&c).unwrap()["text"],
            serde_json::json!({"path": "/entity/name"})
        );
    }

    #[test]
    fn component_round_trips() {
        let c = Component::new("b", "Button")
            .child("l")
            .prop("variant", "primary");
        let json = serde_json::to_string(&c).unwrap();
        let back: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }
}
