//! v2 Spec types: flat element map with ID-keyed references.
//!
//! A [`Spec`] is a top-level JSON-UI document consisting of a `$schema` version tag,
//! a root element ID, and an `elements` map of type-erased [`Element`] values.
//! Each `Element` carries a `type_name: String`, a `props: serde_json::Value` payload,
//! a `children: Vec<String>` list of child IDs (not nested structures), and optional
//! `action` / `visible` fields.
//!
//! [`Spec::from_json`] and [`SpecBuilder::build`] both run the same parse-time structural
//! validation: duplicate IDs, ID format, root existence, dangling refs, cycles, depth
//! ≤ [`MAX_NESTING_DEPTH`]. Malformed specs surface as typed [`SpecError`] variants;
//! `from_json` never panics on arbitrary input.

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::de::{Deserialize as DeserializeTrait, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

use crate::action::Action;
use crate::visibility::Visibility;

// ---------------------------------------------------------------------------
// Section A — Constants
// ---------------------------------------------------------------------------

/// Schema version string embedded in every v2 [`Spec`] under the `$schema` JSON key.
pub const SCHEMA_VERSION: &str = "ferro-json-ui/v2";

/// Maximum allowed nesting depth from the root element.
///
/// Matches the Screen > Section > Component hierarchy documented by the
/// SDUI research in `115-CONTEXT.md` (D-09). Paths exceeding this depth
/// surface as [`SpecError::DepthExceeded`].
pub const MAX_NESTING_DEPTH: usize = 3;

// ---------------------------------------------------------------------------
// Section B — Types (Spec, Element, SpecError)
// ---------------------------------------------------------------------------

/// Top-level v2 JSON-UI document.
///
/// A `Spec` is a flat element map keyed by ID with a single `root` pointer.
/// Children are referenced by string ID, not by nesting, which keeps the
/// structure human-auditable and preserves a stable anchor for every element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    /// Schema version tag (`"ferro-json-ui/v2"`).
    #[serde(rename = "$schema")]
    pub schema: String,
    /// ID of the root element (must exist as a key in `elements`).
    pub root: String,
    /// Flat map of element ID to element body. Element IDs are
    /// `^[A-Za-z_][A-Za-z0-9_-]{0,127}$`.
    pub elements: HashMap<String, Element>,
    /// Optional document title (used by layouts to populate `<title>`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional layout name (e.g. `"dashboard"`, `"app"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    /// Arbitrary data payload consumed by data-path references inside elements.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub data: Value,
}

/// A single type-erased UI element.
///
/// The `type_name` is an unrestricted string; catalog / renderer layers decide
/// whether the name resolves to a built-in or plugin component. `props` is a
/// free-form JSON value carrying component-specific fields; validation of
/// per-component props is deferred to the Phase 117 catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    /// Component type name (renamed from `type` to avoid Rust keyword collision).
    #[serde(rename = "type")]
    pub type_name: String,
    /// Component-specific props payload.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub props: Value,
    /// IDs of child elements (must resolve in the parent `Spec.elements` map).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
    /// Optional action attached to this element.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    /// Optional visibility rule governing whether this element renders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<Visibility>,
}

/// Errors returned by [`Spec::from_json`] and [`SpecBuilder::build`].
///
/// Variants carry structured payloads (not formatted strings) so tooling
/// can pinpoint the offending element by ID.
#[derive(Debug, Error)]
pub enum SpecError {
    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("duplicate element ID in spec: {0}")]
    DuplicateId(String),
    #[error("root element '{0}' not found in elements map")]
    RootMissing(String),
    #[error("element '{element}' references child '{child}' which does not exist")]
    DanglingChild { element: String, child: String },
    #[error("cycle detected in element graph: {}", path.join(" -> "))]
    Cycle { path: Vec<String> },
    #[error(
        "nesting depth exceeds maximum of {max}: found depth {found} at {}",
        path.join(" -> ")
    )]
    DepthExceeded {
        max: usize,
        found: usize,
        path: Vec<String>,
    },
    #[error("invalid element ID '{0}' — must match ^[A-Za-z_][A-Za-z0-9_-]{{0,127}}$")]
    InvalidId(String),
}

// ---------------------------------------------------------------------------
// Section C — Builder
// ---------------------------------------------------------------------------

impl Spec {
    /// Entry point for fluent construction of a [`Spec`].
    ///
    /// The first `.element(id, _)` call sets the `root` if it has not been
    /// explicitly set via `.root()`. The terminal `.build()` runs the same
    /// structural validation as [`Spec::from_json`].
    pub fn builder() -> SpecBuilder {
        SpecBuilder::new()
    }

    /// Parse a v2 spec from its JSON representation.
    ///
    /// Returns `Ok(spec)` only if the JSON is well-formed AND the element
    /// graph passes every structural check. Never panics on arbitrary input.
    pub fn from_json(json: &str) -> Result<Spec, SpecError> {
        let raw: SpecWire = match serde_json::from_str::<SpecWire>(json) {
            Ok(r) => r,
            Err(e) => {
                // Intercept the custom duplicate-ID sentinel and convert to DuplicateId.
                let msg = e.to_string();
                if let Some(idx) = msg.find(DUP_ID_SENTINEL) {
                    let after = &msg[idx + DUP_ID_SENTINEL.len()..];
                    let id: String = after
                        .chars()
                        .take_while(|c| !c.is_whitespace() && *c != '"' && *c != '\'' && *c != ',')
                        .collect();
                    return Err(SpecError::DuplicateId(id));
                }
                return Err(SpecError::Json(e));
            }
        };
        let spec = Spec {
            schema: raw.schema,
            root: raw.root,
            elements: raw.elements.0,
            title: raw.title,
            layout: raw.layout,
            data: raw.data,
        };
        validate_structure(&spec)?;
        Ok(spec)
    }
}

impl Element {
    /// Start building an [`Element`] with the given type name.
    ///
    /// Returns an [`ElementBuilder`] rather than `Self` because element
    /// construction is fluent; the terminal call is consumed by
    /// [`SpecBuilder::element`] which invokes the crate-private `build`.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(type_name: impl Into<String>) -> ElementBuilder {
        ElementBuilder {
            type_name: type_name.into(),
            props: Map::new(),
            children: Vec::new(),
            action: None,
            visible: None,
        }
    }
}

/// Fluent builder for [`Spec`].
#[derive(Debug, Default)]
pub struct SpecBuilder {
    title: Option<String>,
    layout: Option<String>,
    data: Value,
    root: Option<String>,
    elements: HashMap<String, Element>,
}

impl SpecBuilder {
    fn new() -> Self {
        Self {
            title: None,
            layout: None,
            data: Value::Null,
            root: None,
            elements: HashMap::new(),
        }
    }

    /// Set the document title.
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    /// Set the layout name.
    pub fn layout(mut self, l: impl Into<String>) -> Self {
        self.layout = Some(l.into());
        self
    }

    /// Attach the data payload.
    pub fn data(mut self, d: Value) -> Self {
        self.data = d;
        self
    }

    /// Explicitly set the root element ID.
    ///
    /// If omitted, the root defaults to the ID of the first element added.
    pub fn root(mut self, id: impl Into<String>) -> Self {
        self.root = Some(id.into());
        self
    }

    /// Add an element to the spec. The first call (absent an explicit
    /// [`SpecBuilder::root`]) establishes the root.
    pub fn element(mut self, id: impl Into<String>, el: ElementBuilder) -> Self {
        let id: String = id.into();
        if self.root.is_none() {
            self.root = Some(id.clone());
        }
        self.elements.insert(id, el.build());
        self
    }

    /// Finalize the spec. Runs the same structural validation as
    /// [`Spec::from_json`].
    pub fn build(self) -> Result<Spec, SpecError> {
        let root = self.root.ok_or_else(|| {
            // An empty builder with no elements has no meaningful root; surface
            // it as RootMissing("") so the error surface is uniform with
            // from_json.
            SpecError::RootMissing(String::new())
        })?;
        let spec = Spec {
            schema: SCHEMA_VERSION.to_string(),
            root,
            elements: self.elements,
            title: self.title,
            layout: self.layout,
            data: self.data,
        };
        validate_structure(&spec)?;
        Ok(spec)
    }
}

/// Fluent builder for [`Element`].
#[derive(Debug)]
pub struct ElementBuilder {
    type_name: String,
    props: Map<String, Value>,
    children: Vec<String>,
    action: Option<Action>,
    visible: Option<Visibility>,
}

impl ElementBuilder {
    /// Set a prop on the element.
    pub fn prop(mut self, k: impl Into<String>, v: impl Into<Value>) -> Self {
        self.props.insert(k.into(), v.into());
        self
    }

    /// Append a child ID.
    pub fn child(mut self, id: impl Into<String>) -> Self {
        self.children.push(id.into());
        self
    }

    /// Attach an action.
    pub fn action(mut self, a: Action) -> Self {
        self.action = Some(a);
        self
    }

    /// Attach a visibility rule.
    pub fn visible(mut self, v: Visibility) -> Self {
        self.visible = Some(v);
        self
    }

    pub(crate) fn build(self) -> Element {
        let props = if self.props.is_empty() {
            Value::Null
        } else {
            Value::Object(self.props)
        };
        Element {
            type_name: self.type_name,
            props,
            children: self.children,
            action: self.action,
            visible: self.visible,
        }
    }
}

// ---------------------------------------------------------------------------
// Section D — Validation and parse-wire types
// ---------------------------------------------------------------------------

/// Sentinel string smuggled through `serde::de::Error::custom` so
/// [`Spec::from_json`] can distinguish duplicate-ID errors from other parse
/// failures without relying on a forked serde_json. The string is chosen to
/// not appear in legitimate JSON.
const DUP_ID_SENTINEL: &str = "__FERRO_DUPLICATE_ID__";

/// Internal wire struct. Wraps `elements` in [`ElementsMap`] so duplicate keys
/// are rejected during deserialization rather than silently overwritten.
#[derive(Deserialize)]
struct SpecWire {
    #[serde(rename = "$schema", default = "default_schema")]
    schema: String,
    root: String,
    elements: ElementsMap,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    layout: Option<String>,
    #[serde(default)]
    data: Value,
}

fn default_schema() -> String {
    SCHEMA_VERSION.to_string()
}

/// Wrapper around `HashMap<String, Element>` that fails deserialization on
/// duplicate keys. The underlying `serde_json::Map` default silently overwrites
/// — we want authors to see the mistake.
struct ElementsMap(HashMap<String, Element>);

impl<'de> DeserializeTrait<'de> for ElementsMap {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = ElementsMap;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a JSON object with unique element IDs")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut m: M) -> Result<ElementsMap, M::Error> {
                let mut map: HashMap<String, Element> = HashMap::new();
                while let Some(k) = m.next_key::<String>()? {
                    if map.contains_key(&k) {
                        return Err(serde::de::Error::custom(format!("{DUP_ID_SENTINEL}{k}")));
                    }
                    let v: Element = m.next_value()?;
                    map.insert(k, v);
                }
                Ok(ElementsMap(map))
            }
        }
        d.deserialize_map(V)
    }
}

/// Run every structural check on a freshly-parsed (or built) spec.
///
/// Order matters: ID format is checked first because every other variant
/// assumes well-formed IDs when it builds a path. After that, the cheapest
/// / most-specific failures come before the more expensive graph traversals.
fn validate_structure(spec: &Spec) -> Result<(), SpecError> {
    validate_ids(&spec.elements)?;
    if !spec.elements.contains_key(&spec.root) {
        return Err(SpecError::RootMissing(spec.root.clone()));
    }
    validate_no_dangling(&spec.elements)?;
    detect_cycle(&spec.elements, &spec.root)?;
    check_depth(&spec.elements, &spec.root)?;
    Ok(())
}

/// Check a single ID against the `^[A-Za-z_][A-Za-z0-9_-]{0,127}$` grammar.
fn is_valid_id(s: &str) -> bool {
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    let bytes = s.as_bytes();
    let first = bytes[0];
    let first_ok = first.is_ascii_alphabetic() || first == b'_';
    if !first_ok {
        return false;
    }
    bytes[1..]
        .iter()
        .all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn validate_ids(elements: &HashMap<String, Element>) -> Result<(), SpecError> {
    for (id, el) in elements {
        if !is_valid_id(id) {
            return Err(SpecError::InvalidId(id.clone()));
        }
        for child in &el.children {
            if !is_valid_id(child) {
                return Err(SpecError::InvalidId(child.clone()));
            }
        }
    }
    Ok(())
}

fn validate_no_dangling(elements: &HashMap<String, Element>) -> Result<(), SpecError> {
    for (id, el) in elements {
        for child in &el.children {
            if !elements.contains_key(child) {
                return Err(SpecError::DanglingChild {
                    element: id.clone(),
                    child: child.clone(),
                });
            }
        }
    }
    Ok(())
}

fn detect_cycle(elements: &HashMap<String, Element>, root: &str) -> Result<(), SpecError> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut on_stack: Vec<String> = Vec::new();
    dfs(root, elements, &mut visited, &mut on_stack)
}

fn dfs(
    node: &str,
    elements: &HashMap<String, Element>,
    visited: &mut HashSet<String>,
    on_stack: &mut Vec<String>,
) -> Result<(), SpecError> {
    if let Some(start) = on_stack.iter().position(|n| n == node) {
        let mut path: Vec<String> = on_stack[start..].to_vec();
        path.push(node.to_string());
        return Err(SpecError::Cycle { path });
    }
    if visited.contains(node) {
        return Ok(());
    }
    on_stack.push(node.to_string());
    if let Some(el) = elements.get(node) {
        for child in &el.children {
            dfs(child, elements, visited, on_stack)?;
        }
    }
    on_stack.pop();
    visited.insert(node.to_string());
    Ok(())
}

fn check_depth(elements: &HashMap<String, Element>, root: &str) -> Result<(), SpecError> {
    let mut path: Vec<String> = Vec::new();
    walk(root, elements, 1, &mut path)
}

fn walk(
    node: &str,
    elements: &HashMap<String, Element>,
    depth: usize,
    path: &mut Vec<String>,
) -> Result<(), SpecError> {
    path.push(node.to_string());
    if depth > MAX_NESTING_DEPTH {
        return Err(SpecError::DepthExceeded {
            max: MAX_NESTING_DEPTH,
            found: depth,
            path: path.clone(),
        });
    }
    if let Some(el) = elements.get(node) {
        for child in &el.children {
            walk(child, elements, depth + 1, path)?;
        }
    }
    path.pop();
    Ok(())
}

// ---------------------------------------------------------------------------
// Inline unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn default_schema_is_v2() {
        assert_eq!(default_schema(), SCHEMA_VERSION);
        assert_eq!(SCHEMA_VERSION, "ferro-json-ui/v2");
    }

    #[test]
    fn is_valid_id_edge_cases() {
        // Each row is (input, expected_valid).
        let cases: &[(&str, bool)] = &[
            ("", false),
            ("1abc", false),
            ("a", true),
            ("_", true),
            ("a_b-c", true),
            ("user form", false),
            ("ABC123", true),
            ("a.b", false),
            ("/path", false),
        ];
        for (s, ok) in cases {
            assert_eq!(is_valid_id(s), *ok, "mismatch on {s:?}");
        }
        // 128 chars ok, 129 chars rejected.
        let ok128: String = "a".repeat(128);
        let bad129: String = "a".repeat(129);
        assert!(is_valid_id(&ok128));
        assert!(!is_valid_id(&bad129));
    }

    #[test]
    fn builder_minimal_round_trips() {
        let spec = Spec::builder()
            .element("a", Element::new("Text").prop("content", "Hi"))
            .build()
            .unwrap();
        assert_eq!(spec.schema, SCHEMA_VERSION);
        assert_eq!(spec.root, "a");
        assert_eq!(spec.elements.len(), 1);
        let json = serde_json::to_string(&spec).unwrap();
        let back = Spec::from_json(&json).unwrap();
        assert_eq!(spec, back);
    }

    #[test]
    fn builder_parity_with_json() {
        let from_json = Spec::from_json(
            r#"{"$schema":"ferro-json-ui/v2","root":"a","elements":{"a":{"type":"Text","props":{"content":"Hi"}}}}"#,
        )
        .unwrap();
        let from_builder = Spec::builder()
            .element("a", Element::new("Text").prop("content", "Hi"))
            .build()
            .unwrap();
        assert_eq!(from_json, from_builder);
    }

    #[test]
    fn from_json_rejects_missing_root() {
        let err = Spec::from_json(
            r#"{"$schema":"ferro-json-ui/v2","root":"nope","elements":{"a":{"type":"Text"}}}"#,
        )
        .unwrap_err();
        match err {
            SpecError::RootMissing(id) => assert_eq!(id, "nope"),
            other => panic!("expected RootMissing, got {other:?}"),
        }
    }

    #[test]
    fn from_json_rejects_dangling_child() {
        let err = Spec::from_json(
            r#"{"$schema":"ferro-json-ui/v2","root":"a","elements":{"a":{"type":"Card","children":["ghost"]}}}"#,
        )
        .unwrap_err();
        match err {
            SpecError::DanglingChild { element, child } => {
                assert_eq!(element, "a");
                assert_eq!(child, "ghost");
            }
            other => panic!("expected DanglingChild, got {other:?}"),
        }
    }

    #[test]
    fn from_json_rejects_self_cycle() {
        let err = Spec::from_json(
            r#"{"$schema":"ferro-json-ui/v2","root":"A","elements":{"A":{"type":"Card","children":["A"]}}}"#,
        )
        .unwrap_err();
        match err {
            SpecError::Cycle { path } => {
                assert_eq!(path, vec!["A".to_string(), "A".to_string()]);
            }
            other => panic!("expected Cycle (self), got {other:?}"),
        }
    }

    #[test]
    fn from_json_rejects_two_cycle() {
        let err = Spec::from_json(
            r#"{"$schema":"ferro-json-ui/v2","root":"root","elements":{"root":{"type":"Card","children":["A"]},"A":{"type":"Card","children":["root"]}}}"#,
        )
        .unwrap_err();
        match err {
            SpecError::Cycle { path } => {
                assert!(path.len() >= 3);
                assert_eq!(path.first(), path.last());
            }
            other => panic!("expected Cycle, got {other:?}"),
        }
    }

    #[test]
    fn from_json_rejects_four_level_nesting() {
        let err = Spec::from_json(
            r#"{"$schema":"ferro-json-ui/v2","root":"root","elements":{
                "root":{"type":"Card","children":["A"]},
                "A":{"type":"Card","children":["B"]},
                "B":{"type":"Card","children":["C"]},
                "C":{"type":"Card","children":["D"]},
                "D":{"type":"Text"}
            }}"#,
        )
        .unwrap_err();
        match err {
            SpecError::DepthExceeded { max, found, path } => {
                assert_eq!(max, 3);
                assert!(found > 3, "found {found} must exceed 3");
                assert!(!path.is_empty());
            }
            other => panic!("expected DepthExceeded, got {other:?}"),
        }
    }

    #[test]
    fn from_json_rejects_invalid_id_space() {
        let err = Spec::from_json(
            r#"{"$schema":"ferro-json-ui/v2","root":"user form","elements":{"user form":{"type":"Text"}}}"#,
        )
        .unwrap_err();
        match err {
            SpecError::InvalidId(id) => assert_eq!(id, "user form"),
            other => panic!("expected InvalidId, got {other:?}"),
        }
    }

    #[test]
    fn from_json_rejects_duplicate_id() {
        // Raw JSON with two `"a"` keys; serde's default map would silently overwrite.
        let err = Spec::from_json(
            r#"{"$schema":"ferro-json-ui/v2","root":"a","elements":{"a":{"type":"Text"},"a":{"type":"Card"}}}"#,
        )
        .unwrap_err();
        match err {
            SpecError::DuplicateId(id) => assert_eq!(id, "a"),
            other => panic!("expected DuplicateId, got {other:?}"),
        }
    }

    #[test]
    fn from_json_accepts_three_level_nesting() {
        let spec = Spec::from_json(
            r#"{"$schema":"ferro-json-ui/v2","root":"root","elements":{
                "root":{"type":"Card","children":["section"]},
                "section":{"type":"FormSection","children":["leaf"]},
                "leaf":{"type":"Text"}
            }}"#,
        )
        .unwrap();
        assert_eq!(spec.elements.len(), 3);
    }

    #[test]
    fn from_json_accepts_diamond() {
        // A -> [B, C]; B -> D; C -> D. D is visited twice via different parents
        // but there's no cycle. Depth is 3 (A=1, B/C=2, D=3).
        let spec = Spec::from_json(
            r#"{"$schema":"ferro-json-ui/v2","root":"A","elements":{
                "A":{"type":"Card","children":["B","C"]},
                "B":{"type":"Card","children":["D"]},
                "C":{"type":"Card","children":["D"]},
                "D":{"type":"Text"}
            }}"#,
        )
        .unwrap();
        assert_eq!(spec.elements.len(), 4);
    }

    #[test]
    fn from_json_wraps_syntax_errors() {
        // Not a panic — malformed JSON becomes SpecError::Json.
        let err = Spec::from_json("{ this is not json ").unwrap_err();
        assert!(matches!(err, SpecError::Json(_)), "got {err:?}");
    }

    #[test]
    fn builder_rejects_forward_ref_without_target() {
        // Parent references a child that was never added.
        let err = Spec::builder()
            .element("root", Element::new("Card").child("ghost"))
            .build()
            .unwrap_err();
        match err {
            SpecError::DanglingChild { element, child } => {
                assert_eq!(element, "root");
                assert_eq!(child, "ghost");
            }
            other => panic!("expected DanglingChild, got {other:?}"),
        }
    }

    #[test]
    fn builder_data_payload_survives_round_trip() {
        let spec = Spec::builder()
            .element("a", Element::new("Text"))
            .data(json!({"user":{"name":"Alice"}}))
            .build()
            .unwrap();
        let json = serde_json::to_string(&spec).unwrap();
        let back = Spec::from_json(&json).unwrap();
        assert_eq!(back.data, json!({"user":{"name":"Alice"}}));
    }

    #[test]
    fn element_omits_optional_fields_when_absent() {
        let spec = Spec::builder()
            .element("bare", Element::new("Text"))
            .build()
            .unwrap();
        let json = serde_json::to_string(&spec).unwrap();
        // children is empty -> skipped; props is null -> skipped; action/visible absent -> skipped.
        assert!(!json.contains("children"));
        assert!(!json.contains("props"));
        assert!(!json.contains("action"));
        assert!(!json.contains("visible"));
    }
}
