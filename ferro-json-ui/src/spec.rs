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
    /// Optional iteration directive. When present, the element is treated as
    /// a template — resolve-time expansion (Plan 03) produces N clones with
    /// auto-suffixed IDs, one per row in the resolved data array.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "$each")]
    pub each: Option<EachDirective>,
    /// Optional conditional-emission directive. When the predicate evaluates
    /// false against [`Spec::data`] at resolve time (Plan 03), the element is
    /// REMOVED from the element map (no hidden DOM, no JS). Distinct from
    /// `visible` which renders the element with `hidden` semantics.
    ///
    /// Reuses the [`Visibility`] enum (D-04) — accepts flat conditions AND
    /// And/Or/Not composition because `Visibility` is `#[serde(untagged)]`.
    ///
    /// # Interaction with `$each`
    ///
    /// When both `$if` and `$each` are present on the same element, `$if` is
    /// evaluated FIRST. If false, the element is removed before `$each`
    /// expansion runs (no clones produced).
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "$if")]
    pub if_: Option<Visibility>,
}

/// Iteration directive on an [`Element`]: instantiate one element per row of
/// a JSON array resolved from [`Spec::data`].
///
/// At resolve time, the templated element is replaced by N clones with
/// auto-suffixed IDs (`{element_id}-0`, `{element_id}-1`, ...). The loop
/// variable bound by `as` (default name `"row"`) scopes `$data` paths
/// starting with `/{as}/...` to the current iteration row.
///
/// # Reserved names
///
/// The `as` field must NOT be one of `["data", "root", "_root", "_each",
/// "this", "self"]` — see `SpecError::EachAsReservedName` (validated by
/// `Spec::validate` in Plan 04).
///
/// # Wire format example
///
/// ```json
/// {
///   "type": "Card",
///   "$each": { "path": "/orders", "as": "order" },
///   "props": { "title": { "$data": "/order/order_number" } }
/// }
/// ```
///
/// # Resource bounds
///
/// At Plan 01, the directive is inert — no resolver runs yet. A hard cap on
/// expansion size is a follow-up concern; Phase 163 does not impose a fixed
/// limit. Spec authors are responsible for bounding the resolved array.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EachDirective {
    /// JSONPath-style slash-separated path to a JSON array in [`Spec::data`].
    pub path: String,
    /// Loop-variable name bound during expansion. Paths starting with
    /// `/{as}/...` in the templated element's props resolve to the current row.
    #[serde(rename = "as")]
    pub as_: String,
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
    #[error("element '{element_id}' has footer reference '{footer_id}' not found in elements")]
    FooterMissing {
        element_id: String,
        footer_id: String,
    },
    #[error("element '{element_id}' has `$each.path = \"{path}\"` resolving to a non-array value in spec.data")]
    EachPathNotArray { element_id: String, path: String },
    #[error("element '{element_id}' has `$if.path = \"{path}\"` referencing a key absent from spec.data")]
    IfPathMissing { element_id: String, path: String },
    #[error("element '{element_id}' has `$each.as = \"{name}\"` which is a reserved name (one of: data, root, _root, _each, this, self)")]
    EachAsReservedName { element_id: String, name: String },
    #[error("nested `$each` is not supported in Phase 163: element '{outer}' templates element '{inner}' which is also `$each`-templated")]
    NestedEach { outer: String, inner: String },
    #[error("element '{parent}' (`$each` over '{parent_path}') references child '{child}' which is `$each` over a different path '{child_path}' — mismatched each siblings")]
    MismatchedEach {
        parent: String,
        parent_path: String,
        child: String,
        child_path: String,
    },
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

    /// Merge handler-provided data into `spec.data` via a shallow top-level merge.
    ///
    /// If `handler_data` is a JSON Object, its keys are inserted into `self.data`,
    /// overwriting matching keys (handler wins — locked per 119-CONTEXT D-04).
    /// If `self.data` is `Value::Null` (the default for specs built without `.data(...)`),
    /// it is initialized to an empty object before inserting — otherwise `as_object_mut()`
    /// would return `None` and the handler keys would be silently dropped
    /// (119-RESEARCH §Pitfall 4).
    ///
    /// If `handler_data` is not an Object (Null, Array, String, Number, Bool), it is
    /// silently ignored — a `debug_assert!` fires in dev builds but production never
    /// panics (119-CONTEXT D-04).
    ///
    /// Consuming builder (`mut self -> Self`) for consistency with `SpecBuilder`.
    pub fn merge_data(mut self, handler_data: serde_json::Value) -> Self {
        debug_assert!(
            handler_data.is_null() || handler_data.is_object(),
            "merge_data expects an Object or Null; non-Object handler_data ignored"
        );
        if let Some(obj) = handler_data.as_object() {
            if self.data.is_null() {
                self.data = Value::Object(Map::new());
            }
            if let Some(data_map) = self.data.as_object_mut() {
                for (k, v) in obj {
                    data_map.insert(k.clone(), v.clone());
                }
            }
        }
        self
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
            each: None,
            if_: None,
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
    each: Option<EachDirective>,
    if_: Option<Visibility>,
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
            each: self.each,
            if_: self.if_,
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
    validate_directives(spec)?;
    validate_footer_ids(spec)?;
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

/// D-07: every footer-referenced ID must exist in `spec.elements`.
/// D-08: when an ID appears in both `props.footer` and `children` of the same
/// parent, emit an `eprintln!` warning — the element renders once (in footer)
/// and the duplicate listing is dead config.
fn validate_footer_ids(spec: &Spec) -> Result<(), SpecError> {
    for (element_id, el) in &spec.elements {
        // `props` is a generic Value; handle null/missing gracefully.
        let footer_ids: Vec<String> = el
            .props
            .get("footer")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        for footer_id in &footer_ids {
            if !spec.elements.contains_key(footer_id) {
                return Err(SpecError::FooterMissing {
                    element_id: element_id.clone(),
                    footer_id: footer_id.clone(),
                });
            }
            // D-08 warning — non-fatal, written to stderr.
            if el.children.iter().any(|c| c == footer_id) {
                eprintln!(
                    "ferro-json-ui: element '{element_id}' has '{footer_id}' in both \
                     props.footer and children — the element renders once (in footer); \
                     remove the duplicate from children"
                );
            }
        }
    }
    Ok(())
}

/// Reserved names for the `$each.as` loop variable.
const RESERVED_EACH_AS: &[&str] = &["data", "root", "_root", "_each", "this", "self"];

/// Validate `$each` and `$if` directives on every element.
///
/// Best-effort: path resolvability against `spec.data` is checked only when
/// `spec.data` is non-null. Per-request data is not visible at this stage.
fn validate_directives(spec: &Spec) -> Result<(), SpecError> {
    // First pass: collect each-templated element IDs + their directives.
    let templated: HashMap<&str, &EachDirective> = spec
        .elements
        .iter()
        .filter_map(|(id, el)| el.each.as_ref().map(|e| (id.as_str(), e)))
        .collect();

    for (id, el) in &spec.elements {
        // -- $each validation --
        if let Some(each) = &el.each {
            // (a) Reserved-name check.
            if RESERVED_EACH_AS.contains(&each.as_.as_str()) {
                return Err(SpecError::EachAsReservedName {
                    element_id: id.clone(),
                    name: each.as_.clone(),
                });
            }
            // (b) Path-resolves-to-array check (only when spec.data is non-null).
            if !spec.data.is_null() {
                if let Some(value) = crate::data::resolve_path(&spec.data, &each.path) {
                    if !value.is_array() {
                        return Err(SpecError::EachPathNotArray {
                            element_id: id.clone(),
                            path: each.path.clone(),
                        });
                    }
                }
            }
            // (c) Mismatched-each child check: scan direct children.
            for child in &el.children {
                if let Some(child_each) = templated.get(child.as_str()) {
                    if child_each.path != each.path || child_each.as_ != each.as_ {
                        return Err(SpecError::MismatchedEach {
                            parent: id.clone(),
                            parent_path: each.path.clone(),
                            child: child.clone(),
                            child_path: child_each.path.clone(),
                        });
                    }
                }
            }
            // (d) Nested-each check: walk transitive descendants beyond direct
            // children. Direct children with matching (path, as) are the
            // correlated-sibling case (valid). Transitive descendants that are
            // also $each-templated are nested — always rejected.
            let direct: HashSet<&str> = el.children.iter().map(|s| s.as_str()).collect();
            let mut visited: HashSet<&str> = HashSet::new();
            let mut stack: Vec<&str> = Vec::new();
            // Seed stack with grandchildren (skip direct children).
            for child in &el.children {
                if let Some(child_el) = spec.elements.get(child) {
                    for gc in &child_el.children {
                        stack.push(gc.as_str());
                    }
                }
            }
            while let Some(node) = stack.pop() {
                if !visited.insert(node) {
                    continue;
                }
                if templated.contains_key(node) && !direct.contains(node) {
                    return Err(SpecError::NestedEach {
                        outer: id.clone(),
                        inner: node.to_string(),
                    });
                }
                if let Some(node_el) = spec.elements.get(node) {
                    for c in &node_el.children {
                        stack.push(c.as_str());
                    }
                }
            }
        }
        // -- $if validation --
        // Walk all conditions inside the Visibility tree; for each, check path
        // against spec.data (best-effort, only when spec.data is non-null).
        if let Some(vis) = &el.if_ {
            if !spec.data.is_null() {
                check_visibility_paths(id, vis, &spec.data)?;
            }
        }
    }
    Ok(())
}

/// Recursively walk a Visibility tree, asserting every condition's path
/// resolves to a present key in `data` (Some(_)). Missing → IfPathMissing.
fn check_visibility_paths(
    element_id: &str,
    vis: &Visibility,
    data: &Value,
) -> Result<(), SpecError> {
    match vis {
        Visibility::And { and } => {
            for v in and {
                check_visibility_paths(element_id, v, data)?;
            }
        }
        Visibility::Or { or } => {
            for v in or {
                check_visibility_paths(element_id, v, data)?;
            }
        }
        Visibility::Not { not } => check_visibility_paths(element_id, not, data)?,
        Visibility::Condition(c) => {
            if crate::data::resolve_path(data, &c.path).is_none() {
                return Err(SpecError::IfPathMissing {
                    element_id: element_id.to_string(),
                    path: c.path.clone(),
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

    #[test]
    fn merge_data_handler_wins() {
        let spec = Spec::builder()
            .element("a", Element::new("Text"))
            .data(json!({"a": 1, "b": 2}))
            .build()
            .unwrap();
        let merged = spec.merge_data(json!({"b": 99, "c": 3}));
        assert_eq!(merged.data, json!({"a": 1, "b": 99, "c": 3}));
    }

    #[test]
    fn merge_data_ignores_non_object() {
        // Null is a no-op (allowed by debug_assert).
        let spec = Spec::builder()
            .element("a", Element::new("Text"))
            .data(json!({"a": 1}))
            .build()
            .unwrap();
        let merged = spec.merge_data(Value::Null);
        assert_eq!(merged.data, json!({"a": 1}));
        // Array / String / Number variants would trip debug_assert in debug mode,
        // so we exercise only the Null no-op here. Production behavior for those
        // variants is covered by inspection — they fall through to the `if let
        // Some(obj) = handler_data.as_object()` guard and are ignored.
    }

    #[test]
    fn merge_data_initializes_null_data() {
        let spec = Spec::builder()
            .element("a", Element::new("Text"))
            .build() // no .data(...) call → spec.data is Value::Null
            .unwrap();
        assert_eq!(spec.data, Value::Null);
        let merged = spec.merge_data(json!({"k": "v"}));
        assert_eq!(merged.data, json!({"k": "v"}));
    }

    #[test]
    fn merge_data_empty_handler_no_op() {
        let spec = Spec::builder()
            .element("a", Element::new("Text"))
            .data(json!({"a": 1}))
            .build()
            .unwrap();
        let merged = spec.merge_data(json!({}));
        assert_eq!(merged.data, json!({"a": 1}));
    }

    #[test]
    fn from_json_rejects_missing_footer_id() {
        let err = Spec::from_json(
            r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "card",
            "elements": {
                "card": {
                    "type": "Card",
                    "props": {"title": "T", "footer": ["ghost"]}
                }
            }
        }"#,
        )
        .unwrap_err();
        match err {
            SpecError::FooterMissing {
                element_id,
                footer_id,
            } => {
                assert_eq!(element_id, "card");
                assert_eq!(footer_id, "ghost");
            }
            other => panic!("expected FooterMissing, got {other:?}"),
        }
    }

    #[test]
    fn from_json_rejects_missing_modal_footer_id() {
        // validate_footer_ids walks `props.footer` for every element, including Modal.
        // This test pins Modal coverage explicitly even though the helper is generic.
        let err = Spec::from_json(
            r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "modal",
            "elements": {
                "modal": {
                    "type": "Modal",
                    "props": {"id": "m", "title": "T", "footer": ["ghost"]}
                }
            }
        }"#,
        )
        .unwrap_err();
        match err {
            SpecError::FooterMissing {
                element_id,
                footer_id,
            } => {
                assert_eq!(element_id, "modal");
                assert_eq!(footer_id, "ghost");
            }
            other => panic!("expected FooterMissing on Modal, got {other:?}"),
        }
    }

    #[test]
    fn spec_warns_duplicate_footer_child() {
        // D-08: duplicate footer+children entry produces a stderr warning,
        // but parsing must still succeed. We assert only the success path.
        let spec = Spec::from_json(
            r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "card",
            "elements": {
                "card": {
                    "type": "Card",
                    "props": {"title": "T", "footer": ["btn"]},
                    "children": ["btn"]
                },
                "btn": {
                    "type": "Button",
                    "props": {"label": "Save"}
                }
            }
        }"#,
        )
        .expect("D-08 warning is non-fatal; parse must succeed");
        assert_eq!(spec.root, "card");
    }

    #[test]
    fn each_directive_round_trips() {
        let json = serde_json::json!({"path": "/orders", "as": "order"});
        let parsed: EachDirective = serde_json::from_value(json.clone()).expect("decode");
        assert_eq!(parsed.path, "/orders");
        assert_eq!(parsed.as_, "order");
        let reserialized = serde_json::to_value(&parsed).expect("encode");
        assert_eq!(reserialized, json);
        // Confirm the wire-format uses "as", not "as_".
        assert!(reserialized.get("as").is_some());
        assert!(reserialized.get("as_").is_none());
    }

    #[test]
    fn element_with_each_round_trips() {
        let json = serde_json::json!({
            "type": "Card",
            "$each": {"path": "/orders", "as": "order"},
            "props": {"title": "x"}
        });
        let parsed: Element = serde_json::from_value(json.clone()).expect("decode");
        assert!(parsed.each.is_some());
        let each = parsed.each.as_ref().unwrap();
        assert_eq!(each.path, "/orders");
        assert_eq!(each.as_, "order");
        let reserialized = serde_json::to_value(&parsed).expect("encode");
        assert!(reserialized.get("$each").is_some());
    }

    #[test]
    fn element_without_each_omits_field() {
        // Build via Spec::builder() to mirror the canonical no-directive case.
        let spec = Spec::builder()
            .element("card", Element::new("Card").prop("title", "hello"))
            .build()
            .expect("spec is valid");
        let card = spec.elements.get("card").expect("card present");
        let json = serde_json::to_value(card).expect("encode");
        assert!(
            json.get("$each").is_none(),
            "expected $each to be omitted when None; got: {json}"
        );
    }

    #[test]
    fn if_directive_flat_condition_round_trips() {
        use crate::visibility::Visibility;
        let json = serde_json::json!({"path": "/can_advance", "operator": "eq", "value": true});
        let parsed: Visibility = serde_json::from_value(json.clone()).expect("decode");
        match &parsed {
            Visibility::Condition(c) => {
                assert_eq!(c.path, "/can_advance");
                assert_eq!(c.value, Some(serde_json::json!(true)));
            }
            _ => panic!("expected flat Condition variant, got: {parsed:?}"),
        }
        let reserialized = serde_json::to_value(&parsed).expect("encode");
        assert!(reserialized.get("path").is_some());
        assert!(reserialized.get("operator").is_some());
    }

    #[test]
    fn element_with_if_flat_round_trips() {
        let json = serde_json::json!({
            "type": "Button",
            "$if": {"path": "/can_advance", "operator": "eq", "value": true},
            "props": {"label": "x"}
        });
        let parsed: Element = serde_json::from_value(json.clone()).expect("decode");
        assert!(parsed.if_.is_some());
        let reserialized = serde_json::to_value(&parsed).expect("encode");
        assert!(reserialized.get("$if").is_some());
    }

    #[test]
    fn element_with_if_compound_round_trips() {
        use crate::visibility::Visibility;
        let json = serde_json::json!({
            "type": "Button",
            "$if": {"and": [
                {"path": "/a", "operator": "exists"},
                {"path": "/b", "operator": "eq", "value": true}
            ]},
            "props": {"label": "x"}
        });
        let parsed: Element = serde_json::from_value(json.clone()).expect("decode");
        match parsed.if_.as_ref() {
            Some(Visibility::And { and }) => assert_eq!(and.len(), 2),
            other => panic!("expected And variant, got: {other:?}"),
        }
        let reserialized = serde_json::to_value(&parsed).expect("encode");
        assert!(reserialized.get("$if").and_then(|v| v.get("and")).is_some());
    }

    #[test]
    fn element_without_if_omits_field() {
        let spec = Spec::builder()
            .element("btn", Element::new("Button").prop("label", "ok"))
            .build()
            .expect("spec is valid");
        let btn = spec.elements.get("btn").expect("btn present");
        let json = serde_json::to_value(btn).expect("encode");
        assert!(
            json.get("$if").is_none(),
            "expected $if to be omitted when None; got: {json}"
        );
    }

    // -----------------------------------------------------------------------
    // Directive validator tests (Plan 04)
    // -----------------------------------------------------------------------

    #[test]
    fn validate_each_path_not_array_fires() {
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "list",
            "elements": {
                "list": {
                    "type": "Card",
                    "$each": {"path": "/orders", "as": "order"},
                    "props": {}
                }
            },
            "data": {"orders": "not-an-array"}
        }"#;
        let err = Spec::from_json(json).expect_err("validator must reject non-array $each.path");
        match err {
            SpecError::EachPathNotArray { element_id, path } => {
                assert_eq!(element_id, "list");
                assert_eq!(path, "/orders");
            }
            other => panic!("expected EachPathNotArray, got: {other:?}"),
        }
    }

    #[test]
    fn validate_each_path_not_array_skipped_when_data_null() {
        // Same spec but data is null (absent) — validator is best-effort and skips.
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "list",
            "elements": {
                "list": {
                    "type": "Card",
                    "$each": {"path": "/orders", "as": "order"},
                    "props": {}
                }
            }
        }"#;
        // No data key → spec.data is Value::Null → no EachPathNotArray.
        Spec::from_json(json).expect("no error when data is null");
    }

    #[test]
    fn validate_each_as_reserved_data_rejected() {
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "list",
            "elements": {
                "list": {
                    "type": "Card",
                    "$each": {"path": "/items", "as": "data"},
                    "props": {}
                }
            }
        }"#;
        let err = Spec::from_json(json).expect_err("'data' is a reserved name");
        match err {
            SpecError::EachAsReservedName { element_id, name } => {
                assert_eq!(element_id, "list");
                assert_eq!(name, "data");
            }
            other => panic!("expected EachAsReservedName, got: {other:?}"),
        }
    }

    #[test]
    fn validate_each_as_reserved_root_rejected() {
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "list",
            "elements": {
                "list": {
                    "type": "Card",
                    "$each": {"path": "/items", "as": "root"},
                    "props": {}
                }
            }
        }"#;
        let err = Spec::from_json(json).expect_err("'root' is a reserved name");
        match err {
            SpecError::EachAsReservedName { element_id, name } => {
                assert_eq!(element_id, "list");
                assert_eq!(name, "root");
            }
            other => panic!("expected EachAsReservedName, got: {other:?}"),
        }
    }

    #[test]
    fn validate_each_as_non_reserved_accepted() {
        // "order" and "row" are not reserved — spec must parse OK.
        let json_order = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "list",
            "elements": {
                "list": {
                    "type": "Card",
                    "$each": {"path": "/items", "as": "order"},
                    "props": {}
                }
            },
            "data": {"items": []}
        }"#;
        Spec::from_json(json_order).expect("'order' is not reserved");

        let json_row = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "list",
            "elements": {
                "list": {
                    "type": "Card",
                    "$each": {"path": "/items", "as": "row"},
                    "props": {}
                }
            },
            "data": {"items": []}
        }"#;
        Spec::from_json(json_row).expect("'row' is not reserved");
    }

    #[test]
    fn validate_if_path_missing_fires() {
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "btn",
            "elements": {
                "btn": {
                    "type": "Button",
                    "$if": {"path": "/missing_key", "operator": "eq", "value": true},
                    "props": {"label": "Go"}
                }
            },
            "data": {"other": true}
        }"#;
        let err = Spec::from_json(json).expect_err("missing $if.path must error");
        match err {
            SpecError::IfPathMissing { element_id, path } => {
                assert_eq!(element_id, "btn");
                assert_eq!(path, "/missing_key");
            }
            other => panic!("expected IfPathMissing, got: {other:?}"),
        }
    }

    #[test]
    fn validate_if_path_missing_skipped_when_data_null() {
        // Same spec but data is null — validator is best-effort and skips.
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "btn",
            "elements": {
                "btn": {
                    "type": "Button",
                    "$if": {"path": "/missing_key", "operator": "eq", "value": true},
                    "props": {"label": "Go"}
                }
            }
        }"#;
        Spec::from_json(json).expect("no error when data is null");
    }

    #[test]
    fn validate_nested_each_rejected() {
        // Element A has $each; A's child B also has $each — nested, must fail.
        // B is a grandchild of A (A -> mid -> B) to exercise the transitive walk.
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "A",
            "elements": {
                "A": {
                    "type": "Card",
                    "$each": {"path": "/items", "as": "item"},
                    "children": ["mid"]
                },
                "mid": {
                    "type": "Section",
                    "children": ["B"]
                },
                "B": {
                    "type": "Card",
                    "$each": {"path": "/other_items", "as": "other"},
                    "props": {}
                }
            }
        }"#;
        let err = Spec::from_json(json).expect_err("nested $each must be rejected");
        match err {
            SpecError::NestedEach { outer, inner } => {
                assert_eq!(outer, "A");
                assert_eq!(inner, "B");
            }
            other => panic!("expected NestedEach, got: {other:?}"),
        }
    }

    #[test]
    fn validate_mismatched_each_child_rejected() {
        // A has $each over /items; direct child B has $each over /different — mismatch.
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "A",
            "elements": {
                "A": {
                    "type": "Card",
                    "$each": {"path": "/items", "as": "item"},
                    "children": ["B"]
                },
                "B": {
                    "type": "Text",
                    "$each": {"path": "/different_items", "as": "item"}
                }
            }
        }"#;
        let err = Spec::from_json(json).expect_err("mismatched $each child must be rejected");
        match err {
            SpecError::MismatchedEach {
                parent,
                parent_path,
                child,
                child_path,
            } => {
                assert_eq!(parent, "A");
                assert_eq!(parent_path, "/items");
                assert_eq!(child, "B");
                assert_eq!(child_path, "/different_items");
            }
            other => panic!("expected MismatchedEach, got: {other:?}"),
        }
    }

    #[test]
    fn validate_correlated_each_child_accepted() {
        // A and its direct child B both have $each with SAME (path, as) — correlated siblings, valid.
        let json = r#"{
            "$schema": "ferro-json-ui/v2",
            "root": "A",
            "elements": {
                "A": {
                    "type": "Card",
                    "$each": {"path": "/items", "as": "item"},
                    "children": ["B"]
                },
                "B": {
                    "type": "Text",
                    "$each": {"path": "/items", "as": "item"}
                }
            },
            "data": {"items": []}
        }"#;
        Spec::from_json(json).expect("correlated $each children with same (path, as) are valid");
    }

    #[test]
    fn validate_directives_called_between_no_dangling_and_cycle() {
        // Assert by code structure: validate_structure contains the literal call sequence
        // validate_no_dangling → validate_directives → detect_cycle.
        let src = include_str!("spec.rs");
        let validate_section = src
            .split("fn validate_structure")
            .nth(1)
            .expect("validate_structure body present");
        let body_end = validate_section
            .find("\nfn ")
            .unwrap_or(validate_section.len());
        let body = &validate_section[..body_end];
        let pos_no_dangling = body.find("validate_no_dangling").expect("no_dangling call");
        let pos_directives = body.find("validate_directives").expect("directives call");
        let pos_cycle = body.find("detect_cycle").expect("cycle call");
        assert!(
            pos_no_dangling < pos_directives,
            "validate_directives must be called AFTER validate_no_dangling"
        );
        assert!(
            pos_directives < pos_cycle,
            "validate_directives must be called BEFORE detect_cycle"
        );
    }
}
