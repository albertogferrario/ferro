//! # Component Catalog
//!
//! Machine-readable registry of every built-in and plugin JSON-UI component.
//!
//! Phase 117 replaces the hand-maintained `COMPONENT_CATALOG` const string with
//! this auto-derived catalog that reads per-component JSON Schema from the
//! `#[derive(JsonSchema)]` attributes already present on every `*Props` struct
//! (Phase 115). The Catalog pre-computes five artifacts at build time:
//!
//! - `components` — built-in component specs keyed by type name (D-01, D-03)
//! - `plugin_components` — plugin specs pulled from the global registry (D-08)
//! - `full_schema` — assembled JSON Schema document for the full Spec shape (D-13)
//! - `per_component_schemas` — per-component Props schemas for targeted validation (D-12)
//! - `validator` — a `jsonschema::Validator` compiled once from `full_schema` (D-12)
//!
//! Consumers access the singleton via [`global_catalog()`] (D-04). The catalog is
//! frozen after first construction; late-registered plugins do not propagate (D-04).
//!
//! See CONTEXT D-01..D-04, D-11 for the full design rationale.
//!
//! Plan 02 populates `BUILTIN_SPECS` and implements `Catalog::build()` fully.

use std::collections::HashMap;
use std::sync::OnceLock;

use schemars::schema_for;
use serde_json::{to_value, Value};

use crate::component::{
    ActionCardProps, AlertProps, AvatarProps, BadgeProps, BreadcrumbProps, ButtonGroupProps,
    ButtonProps, CalendarCellProps, CardProps, CheckboxProps, ChecklistProps, CollapsibleProps,
    DataTableProps, DescriptionListProps, DropdownMenuProps, EmptyStateProps, FormProps,
    FormSectionProps, GridProps, HeaderProps, ImageProps, InputProps, KanbanBoardProps, ModalProps,
    NotificationDropdownProps, PageHeaderProps, PaginationProps, ProductTileProps, ProgressProps,
    SelectProps, SeparatorProps, SidebarProps, SkeletonProps, StatCardProps, SwitchProps,
    TableProps, TabsProps, TextProps, ToastProps,
};

// ── Public types ───────────────────────────────────────────────────────────────

/// Metadata and JSON Schema for a single JSON-UI component.
///
/// Built by [`Catalog::build`] from the static `BUILTIN_SPECS` table (built-ins)
/// or from [`crate::plugin::JsonUiPlugin::props_schema`] (plugins).
pub struct ComponentSpec {
    /// Component type name as it appears in the Spec's `"type"` field.
    pub name: String,
    /// Short imperative description matching the voice of the legacy `COMPONENT_CATALOG`.
    pub description: String,
    /// JSON Schema object for the component's Props struct (schemars output).
    pub props_schema: Value,
    /// `true` for plugin components; `false` for built-ins.
    pub is_plugin: bool,
    /// Names of fields whose values are `Vec<String>` of element IDs (slot fields).
    ///
    /// Examples: `["footer"]` for Card, `["children"]` for Tabs' per-tab model,
    /// `[]` for leaf components with no children slots.
    pub slot_fields: Vec<String>,
}

/// Pre-computed, immutable registry of all JSON-UI components and their schemas.
///
/// Constructed once via [`Catalog::build`] and accessed globally through
/// [`global_catalog`]. All fields are `pub(crate)` — external callers use the
/// accessor methods added in Plans 02–05.
// components, plugin_components, per_component_schemas, and validator are consumed
// by Plan 04's validate() pipeline and MCP tooling. Suppress dead_code until then.
#[allow(dead_code)]
pub struct Catalog {
    /// Built-in components keyed by type name.
    pub(crate) components: HashMap<String, ComponentSpec>,
    /// Plugin components keyed by type name.
    pub(crate) plugin_components: HashMap<String, ComponentSpec>,
    /// Full Spec JSON Schema document (root + elements + oneOf over all components).
    pub(crate) full_schema: Value,
    /// Per-component Props schemas keyed by type name.
    pub(crate) per_component_schemas: HashMap<String, Value>,
    /// Compiled validator over `full_schema`. Reused across `validate()` calls.
    pub(crate) validator: jsonschema::Validator,
}

/// Errors that can occur during catalog construction or spec validation.
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    /// A Spec element references a component type not in the catalog.
    #[error("unknown component type '{type_name}' at element '{element_id}'")]
    UnknownType {
        /// The element's ID field.
        element_id: String,
        /// The unrecognized `"type"` value.
        type_name: String,
    },
    /// An element's `props` object fails JSON Schema validation for its component.
    #[error("props invalid for '{type_name}' at element '{element_id}': {errors:?}")]
    PropsInvalid {
        /// The element's ID field.
        element_id: String,
        /// The component type that owns the failing props.
        type_name: String,
        /// Human-readable validation error messages from the jsonschema crate.
        errors: Vec<String>,
    },
    /// The full Spec fails the top-level JSON Schema (missing `$schema`, bad `root`, etc.).
    #[error("spec invalid: {errors:?}")]
    SpecInvalid {
        /// Human-readable validation error messages.
        errors: Vec<String>,
    },
    /// Catalog construction failed (e.g., a plugin returned an invalid JSON Schema).
    #[error("catalog build failed: {0}")]
    BuildFailed(String),
    /// JSON serialization error during schema assembly.
    #[error("schema serialization error: {0}")]
    SchemaSerialization(#[from] serde_json::Error),
}

// ── Static built-in table ─────────────────────────────────────────────────────

type SchemaFn = fn() -> Value;

/// `(type_name, description, schema_fn, slot_fields)`
///
/// Descriptions per CONTEXT D-06. Slot fields per Phase 116 / CONTEXT D-05.
/// Order MUST match `crate::render::BUILTIN_TYPES` exactly (see drift guard in
/// `Catalog::build`).
static BUILTIN_SPECS: &[(&str, &str, SchemaFn, &[&str])] = &[
    // === Leaves (atoms.rs) ===
    (
        "Text",
        "Semantic text element (p / h1 / h2 / h3 / span / div / section).",
        || to_value(schema_for!(TextProps)).unwrap(),
        &[],
    ),
    (
        "Button",
        "Interactive button with variant, size, optional icon, and disabled state.",
        || to_value(schema_for!(ButtonProps)).unwrap(),
        &[],
    ),
    (
        "Badge",
        "Small variant-styled label.",
        || to_value(schema_for!(BadgeProps)).unwrap(),
        &[],
    ),
    (
        "Alert",
        "Inline notice with info / success / warning / error variants.",
        || to_value(schema_for!(AlertProps)).unwrap(),
        &[],
    ),
    (
        "Separator",
        "Horizontal or vertical divider between content sections.",
        || to_value(schema_for!(SeparatorProps)).unwrap(),
        &[],
    ),
    (
        "Progress",
        "Progress bar with 0–100 percentage value and optional label.",
        || to_value(schema_for!(ProgressProps)).unwrap(),
        &[],
    ),
    (
        "Avatar",
        "Circular user image with fallback initials and size variants.",
        || to_value(schema_for!(AvatarProps)).unwrap(),
        &[],
    ),
    (
        "Image",
        "Image with optional aspect ratio and skeleton fallback on load error.",
        || to_value(schema_for!(ImageProps)).unwrap(),
        &[],
    ),
    (
        "Skeleton",
        "Loading placeholder with configurable width / height / rounding.",
        || to_value(schema_for!(SkeletonProps)).unwrap(),
        &[],
    ),
    (
        "Breadcrumb",
        "Navigation trail of label + optional URL items.",
        || to_value(schema_for!(BreadcrumbProps)).unwrap(),
        &[],
    ),
    (
        "Pagination",
        "Page navigation for paginated data (current / per_page / total).",
        || to_value(schema_for!(PaginationProps)).unwrap(),
        &[],
    ),
    (
        "DescriptionList",
        "Key-value pairs displayed as a description list with optional format.",
        || to_value(schema_for!(DescriptionListProps)).unwrap(),
        &[],
    ),
    (
        "EmptyState",
        "Standardized empty view with title, description, and optional CTA.",
        || to_value(schema_for!(EmptyStateProps)).unwrap(),
        &[],
    ),
    (
        "StatCard",
        "Live-updatable metric card with label, value, icon, SSE target.",
        || to_value(schema_for!(StatCardProps)).unwrap(),
        &[],
    ),
    (
        "Checklist",
        "Onboarding-style checklist with dismissal and server-side state.",
        || to_value(schema_for!(ChecklistProps)).unwrap(),
        &[],
    ),
    (
        "Toast",
        "Declarative notification intent consumed by the runtime JS via data attributes.",
        || to_value(schema_for!(ToastProps)).unwrap(),
        &[],
    ),
    (
        "NotificationDropdown",
        "Dropdown listing notification items with icons, timestamps, read state.",
        || to_value(schema_for!(NotificationDropdownProps)).unwrap(),
        &[],
    ),
    (
        "Sidebar",
        "Dashboard sidebar with fixed top / bottom items and collapsible nav groups.",
        || to_value(schema_for!(SidebarProps)).unwrap(),
        &[],
    ),
    (
        "Header",
        "Dashboard top bar with business name, notification badge, user menu.",
        || to_value(schema_for!(HeaderProps)).unwrap(),
        &[],
    ),
    (
        "DropdownMenu",
        "Trigger button with an absolutely-positioned kebab-style action panel.",
        || to_value(schema_for!(DropdownMenuProps)).unwrap(),
        &[],
    ),
    (
        "CalendarCell",
        "Single day in a month grid with today highlight, out-of-month muting, event dots.",
        || to_value(schema_for!(CalendarCellProps)).unwrap(),
        &[],
    ),
    (
        "ActionCard",
        "Clickable row with icon, title, description, chevron, and variant-colored border.",
        || to_value(schema_for!(ActionCardProps)).unwrap(),
        &[],
    ),
    (
        "ProductTile",
        "Touch-friendly POS tile with name, price, and +/- quantity controls.",
        || to_value(schema_for!(ProductTileProps)).unwrap(),
        &[],
    ),
    // === Containers (containers.rs) ===
    (
        "Card",
        "Content container with title, description, body children, and optional footer slot.",
        || to_value(schema_for!(CardProps)).unwrap(),
        &["footer"],
    ),
    (
        "Modal",
        "Dialog overlay with title, description, body children, and optional footer slot.",
        || to_value(schema_for!(ModalProps)).unwrap(),
        &["footer"],
    ),
    (
        "Tabs",
        "Tabbed content; per-tab children live in TabsProps.tabs[i].children.",
        || to_value(schema_for!(TabsProps)).unwrap(),
        &[],
    ),
    (
        "KanbanBoard",
        "Horizontally scrollable kanban columns on desktop, tab-switched on mobile.",
        || to_value(schema_for!(KanbanBoardProps)).unwrap(),
        &[],
    ),
    (
        "PageHeader",
        "Page title with optional breadcrumb and action button slot.",
        || to_value(schema_for!(PageHeaderProps)).unwrap(),
        &["actions"],
    ),
    (
        "Grid",
        "Responsive multi-column grid with configurable breakpoint columns, gap, scroll.",
        || to_value(schema_for!(GridProps)).unwrap(),
        &[],
    ),
    (
        "Collapsible",
        "Expandable <details> / <summary> section.",
        || to_value(schema_for!(CollapsibleProps)).unwrap(),
        &[],
    ),
    (
        "FormSection",
        "Visual grouping within a form with title, description, and layout variant.",
        || to_value(schema_for!(FormSectionProps)).unwrap(),
        &[],
    ),
    (
        "ButtonGroup",
        "Horizontal button row with configurable gap.",
        || to_value(schema_for!(ButtonGroupProps)).unwrap(),
        &[],
    ),
    // === Form controls (form.rs) ===
    (
        "Form",
        "Form container with action binding and field components.",
        || to_value(schema_for!(FormProps)).unwrap(),
        &[],
    ),
    (
        "Input",
        "Text input with type variants, validation error, data_path pre-fill.",
        || to_value(schema_for!(InputProps)).unwrap(),
        &[],
    ),
    (
        "Select",
        "Dropdown select with options, error, data_path pre-fill.",
        || to_value(schema_for!(SelectProps)).unwrap(),
        &[],
    ),
    (
        "Checkbox",
        "Boolean checkbox with label, description, data binding.",
        || to_value(schema_for!(CheckboxProps)).unwrap(),
        &[],
    ),
    (
        "Switch",
        "Toggle switch (visual alternative to Checkbox); auto-submit when `action` set.",
        || to_value(schema_for!(SwitchProps)).unwrap(),
        &[],
    ),
    // === Data displays (data.rs) ===
    (
        "Table",
        "Data table with columns, row_actions, sorting, empty_message.",
        || to_value(schema_for!(TableProps)).unwrap(),
        &[],
    ),
    (
        "DataTable",
        "Stripe-style alternating-row table with per-row DropdownMenu and mobile card fallback.",
        || to_value(schema_for!(DataTableProps)).unwrap(),
        &[],
    ),
];

// ── Schema sanitizer ──────────────────────────────────────────────────────────

/// Walk a JSON Schema tree and rewrite legacy `definitions` → `$defs`
/// (schemars 0.8 → 1.x draft key drift). Idempotent.
///
/// Also rewrites `$ref` strings that reference `#/definitions/X` → `#/$defs/X`
/// so that validator resolution does not break after the key rename (H-2).
fn sanitize_schema(mut schema: Value) -> Value {
    fn walk(v: &mut Value) {
        if let Some(obj) = v.as_object_mut() {
            if let Some(defs) = obj.remove("definitions") {
                obj.entry("$defs".to_string()).or_insert(defs);
            }
            if let Some(Value::String(ref_str)) = obj.get_mut("$ref") {
                if let Some(suffix) = ref_str.strip_prefix("#/definitions/") {
                    *ref_str = format!("#/$defs/{suffix}");
                }
            }
            // Collect keys first to avoid borrow conflicts.
            let keys: Vec<String> = obj.keys().cloned().collect();
            for k in keys {
                if let Some(child) = obj.get_mut(&k) {
                    walk(child);
                }
            }
        } else if let Some(arr) = v.as_array_mut() {
            for item in arr.iter_mut() {
                walk(item);
            }
        }
    }
    walk(&mut schema);
    schema
}

// ── Schema assembly ────────────────────────────────────────────────────────────

/// Hoist all `$defs` entries from a schemars-generated schema into a shared map.
///
/// schemars emits nested type definitions under `$defs` on the schema root. When
/// component schemas are embedded as `allOf[1]` in the oneOf, the `jsonschema`
/// validator resolves `$ref` pointers from the *assembled* root — so every
/// component-local `$defs` entry must be merged up to the top level.
fn hoist_defs(schema: &mut Value, shared_defs: &mut serde_json::Map<String, Value>) {
    if let Some(obj) = schema.as_object_mut() {
        if let Some(Value::Object(defs)) = obj.remove("$defs") {
            for (k, v) in defs {
                shared_defs.entry(k).or_insert(v);
            }
        }
    }
}

/// Hand-assemble the full spec JSON Schema document from per-component schemas.
///
/// Root: `$schema`, `root`, `elements` (HashMap<String, Element>), optional `title` /
/// `layout` / `data`. `$defs/Element` uses a `oneOf` at the element level —
/// each variant pins `"type": { "const": "X" }` on the element object itself and
/// validates `props` against that component's Props schema (CONTEXT D-13).
///
/// Variants are sorted by name to guarantee deterministic output (CONTEXT D-18).
///
/// `$defs` from every per-component schema are hoisted to the root so that `$ref`
/// pointers (e.g., `#/$defs/ConfirmDialog`) resolve against the assembled document.
fn assemble_full_schema(per_component: &HashMap<String, Value>) -> Result<Value, CatalogError> {
    // Start with Action and Visibility defs — their nested types ($defs) are hoisted too.
    let mut action_schema = sanitize_schema(to_value(schema_for!(crate::action::Action))?);
    let mut visibility_schema =
        sanitize_schema(to_value(schema_for!(crate::visibility::Visibility))?);

    // Collect shared $defs — starts with action + visibility nested types.
    let mut shared_defs: serde_json::Map<String, Value> = serde_json::Map::new();
    hoist_defs(&mut action_schema, &mut shared_defs);
    hoist_defs(&mut visibility_schema, &mut shared_defs);

    // Deterministic oneOf at the Element level — sorted by name (CONTEXT D-18).
    // Each variant describes a complete element object: pins `type` via const on the
    // element itself, then validates `props` against that component's Props schema.
    let mut names: Vec<&String> = per_component.keys().collect();
    names.sort();
    let one_of: Vec<Value> = names
        .into_iter()
        .map(|name| {
            let mut props_schema = per_component[name].clone();
            // Hoist component-local $defs so $ref pointers resolve from the assembled root.
            hoist_defs(&mut props_schema, &mut shared_defs);
            serde_json::json!({
                "allOf": [
                    {
                        "type": "object",
                        "required": ["type"],
                        "properties": {
                            "type": { "const": name }
                        }
                    },
                    {
                        "type": "object",
                        "properties": {
                            "props": props_schema,
                            "children": { "type": "array", "items": { "type": "string" } },
                            "action":   { "$ref": "#/$defs/Action" },
                            "visible":  { "$ref": "#/$defs/Visibility" }
                        }
                    }
                ]
            })
        })
        .collect();

    // Merge the framework-level $defs (Element, Action, Visibility) with the hoisted ones.
    // Framework entries take precedence and must not be overwritten by component defs.
    shared_defs
        .entry("Action".to_string())
        .or_insert(action_schema);
    shared_defs
        .entry("Visibility".to_string())
        .or_insert(visibility_schema);
    // Element is the discriminated union itself — oneOf over all component variants.
    shared_defs.insert(
        "Element".to_string(),
        serde_json::json!({ "oneOf": one_of }),
    );

    Ok(serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "ferro-json-ui/v2",
        "type": "object",
        "required": ["$schema", "root", "elements"],
        "properties": {
            "$schema":  { "const": "ferro-json-ui/v2" },
            "root":     { "type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_-]{0,127}$" },
            "elements": {
                "type": "object",
                "additionalProperties": { "$ref": "#/$defs/Element" }
            },
            "title":    { "type": ["string", "null"] },
            "layout":   { "type": ["string", "null"] },
            "data":     true
        },
        "$defs": shared_defs
    }))
}

// ── Catalog impl ───────────────────────────────────────────────────────────────

impl Catalog {
    /// Build the catalog from the static built-in specs and the current plugin registry.
    ///
    /// Called once by [`global_catalog`]. Returns `Err` if a plugin's `props_schema()`
    /// is not a valid JSON Schema or if the assembled full schema fails to compile.
    ///
    /// # Errors
    ///
    /// - [`CatalogError::BuildFailed`] — plugin meta-validation failure or jsonschema
    ///   compilation failure.
    /// - [`CatalogError::SchemaSerialization`] — serde_json serialization failure.
    pub fn build() -> Result<Self, CatalogError> {
        // === Runtime drift guard ===
        // BUILTIN_SPECS and BUILTIN_TYPES must stay in sync. If they diverge, the
        // catalog is incomplete and downstream validation would silently skip types.
        if BUILTIN_SPECS.len() != crate::render::BUILTIN_TYPES.len() {
            return Err(CatalogError::BuildFailed(format!(
                "BUILTIN_SPECS has {} entries but BUILTIN_TYPES has {} — \
                 add an entry to BUILTIN_SPECS or remove from BUILTIN_TYPES",
                BUILTIN_SPECS.len(),
                crate::render::BUILTIN_TYPES.len(),
            )));
        }

        // === Populate built-ins ===
        let mut components = HashMap::with_capacity(BUILTIN_SPECS.len());
        let mut per_component_schemas = HashMap::with_capacity(BUILTIN_SPECS.len() * 2);
        for (name, desc, schema_fn, slots) in BUILTIN_SPECS {
            let raw = schema_fn();
            let schema = sanitize_schema(raw);
            per_component_schemas.insert((*name).to_string(), schema.clone());
            components.insert(
                (*name).to_string(),
                ComponentSpec {
                    name: (*name).to_string(),
                    description: (*desc).to_string(),
                    props_schema: schema,
                    is_plugin: false,
                    slot_fields: slots.iter().map(|s| (*s).to_string()).collect(),
                },
            );
        }

        // === Populate plugins (H-3 meta-validation) ===
        // Plugins are developer-authored but their schemas are treated as untrusted
        // input (CONTEXT D-20, RESEARCH H-3). Each schema is compiled with
        // `jsonschema::validator_for` before wiring into the catalog; a bad schema
        // aborts build with the plugin name embedded in the error.
        let mut plugin_components = HashMap::new();
        for plugin_type in crate::plugin::registered_plugin_types() {
            // Built-ins take precedence; a plugin cannot shadow a built-in type (D-19).
            if components.contains_key(&plugin_type) {
                continue;
            }
            let raw = crate::plugin::with_plugin(&plugin_type, |p| p.props_schema())
                .unwrap_or(Value::Null);
            let schema = sanitize_schema(raw);
            // Meta-validate plugin schema (CONTEXT D-20, RESEARCH H-3).
            if jsonschema::validator_for(&schema).is_err() {
                return Err(CatalogError::BuildFailed(format!(
                    "plugin '{plugin_type}' returned an invalid JSON Schema"
                )));
            }
            per_component_schemas.insert(plugin_type.clone(), schema.clone());
            plugin_components.insert(
                plugin_type.clone(),
                ComponentSpec {
                    name: plugin_type,
                    description: String::from("Plugin component."),
                    props_schema: schema,
                    is_plugin: true,
                    slot_fields: Vec::new(),
                },
            );
        }

        // === Assemble full schema (CONTEXT D-13, D-14, D-15) ===
        let full_schema = assemble_full_schema(&per_component_schemas)?;

        // === Compile validator ONCE (SCHEMA-03) ===
        let validator = jsonschema::validator_for(&full_schema)
            .map_err(|e| CatalogError::BuildFailed(format!("compiling full spec schema: {e}")))?;

        Ok(Catalog {
            components,
            plugin_components,
            full_schema,
            per_component_schemas,
            validator,
        })
    }

    /// Return the fully-assembled spec JSON Schema document.
    ///
    /// Shape: root with `$schema`, `root`, `elements`, plus `$defs`
    /// containing `Element` (with a discriminated `oneOf` over all
    /// component Props) and `Action` / `Visibility` references.
    /// Zero-copy — the returned `&Value` lives as long as the Catalog.
    pub fn json_schema(&self) -> &Value {
        &self.full_schema
    }
}

// ── Global singleton ───────────────────────────────────────────────────────────

/// Access the global, immutable component catalog.
///
/// Lazily initialized on first call using the plugin registry state at that moment.
/// Subsequent plugin registrations do NOT propagate into the catalog (D-04).
///
/// # Panics
///
/// Panics if [`Catalog::build`] fails. In practice this only occurs if a registered
/// plugin returns a malformed JSON Schema from `props_schema()`. Built-in schemas
/// are derived at compile time and are always valid.
pub fn global_catalog() -> &'static Catalog {
    static GLOBAL_CATALOG: OnceLock<Catalog> = OnceLock::new();
    GLOBAL_CATALOG.get_or_init(|| {
        Catalog::build().expect("catalog build failed — see CatalogError for details")
    })
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
impl Catalog {
    /// Build a catalog from built-in specs only, skipping the global plugin registry.
    ///
    /// Tests that register plugins with invalid schemas pollute the global registry.
    /// This helper produces a clean, plugin-free catalog safe for use in any test order.
    fn build_builtins_only() -> Result<Self, CatalogError> {
        let mut components = HashMap::with_capacity(BUILTIN_SPECS.len());
        let mut per_component_schemas = HashMap::with_capacity(BUILTIN_SPECS.len());
        for (name, desc, schema_fn, slots) in BUILTIN_SPECS {
            let raw = schema_fn();
            let schema = sanitize_schema(raw);
            per_component_schemas.insert((*name).to_string(), schema.clone());
            components.insert(
                (*name).to_string(),
                ComponentSpec {
                    name: (*name).to_string(),
                    description: (*desc).to_string(),
                    props_schema: schema,
                    is_plugin: false,
                    slot_fields: slots.iter().map(|s| (*s).to_string()).collect(),
                },
            );
        }
        let full_schema = assemble_full_schema(&per_component_schemas)?;
        let validator = jsonschema::validator_for(&full_schema)
            .map_err(|e| CatalogError::BuildFailed(format!("compiling full spec schema: {e}")))?;
        Ok(Catalog {
            components,
            plugin_components: HashMap::new(),
            full_schema,
            per_component_schemas,
            validator,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_types_count_is_39() {
        // Drift guard — if this fails, Phase 116's BUILTIN_TYPES changed
        // without a corresponding catalog update. See Plan 02.
        assert_eq!(crate::render::BUILTIN_TYPES.len(), 39);
    }

    #[test]
    fn builtin_specs_len_matches_dispatch() {
        assert_eq!(BUILTIN_SPECS.len(), crate::render::BUILTIN_TYPES.len());
        assert_eq!(BUILTIN_SPECS.len(), 39);
    }

    #[test]
    fn builtin_specs_names_match_dispatch() {
        use std::collections::HashSet;
        let specs: HashSet<&str> = BUILTIN_SPECS.iter().map(|(n, ..)| *n).collect();
        let types: HashSet<&str> = crate::render::BUILTIN_TYPES.iter().copied().collect();
        assert_eq!(specs, types, "BUILTIN_SPECS names must match BUILTIN_TYPES");
    }

    #[test]
    fn build_populates_all_builtins() {
        let cat = Catalog::build().expect("build succeeds");
        for name in crate::render::BUILTIN_TYPES.iter() {
            assert!(
                cat.components.contains_key(*name),
                "built-in '{name}' missing from catalog.components"
            );
            let spec = &cat.components[*name];
            assert_eq!(spec.name, *name);
            assert!(
                !spec.description.is_empty(),
                "'{name}' has empty description"
            );
            assert!(
                spec.props_schema.is_object(),
                "'{name}' props_schema is not a JSON object"
            );
            assert!(!spec.is_plugin);
        }
    }

    #[test]
    fn build_card_has_footer_slot() {
        let cat = Catalog::build().expect("build succeeds");
        let card = &cat.components["Card"];
        assert_eq!(card.slot_fields, vec!["footer"]);
    }

    #[test]
    fn build_modal_has_footer_slot() {
        let cat = Catalog::build().expect("build succeeds");
        let modal = &cat.components["Modal"];
        assert_eq!(modal.slot_fields, vec!["footer"]);
    }

    #[test]
    fn build_pageheader_has_actions_slot() {
        let cat = Catalog::build().expect("build succeeds");
        let ph = &cat.components["PageHeader"];
        assert_eq!(ph.slot_fields, vec!["actions"]);
    }

    #[test]
    fn build_text_has_no_slots() {
        let cat = Catalog::build().expect("build succeeds");
        assert!(cat.components["Text"].slot_fields.is_empty());
    }

    #[test]
    fn build_populates_per_component_schemas() {
        let cat = Catalog::build().expect("build succeeds");
        assert_eq!(
            cat.per_component_schemas.len(),
            BUILTIN_SPECS.len() + cat.plugin_components.len()
        );
    }

    #[test]
    fn sanitize_schema_rewrites_definitions_to_dollar_defs() {
        let raw = serde_json::json!({
            "type": "object",
            "definitions": { "Foo": { "type": "string" } },
            "properties": {
                "x": { "$ref": "#/definitions/Foo" }
            }
        });
        let out = sanitize_schema(raw);
        assert!(out.get("definitions").is_none());
        assert!(out.get("$defs").is_some());
        assert_eq!(
            out["properties"]["x"]["$ref"].as_str().unwrap(),
            "#/$defs/Foo"
        );
    }

    #[test]
    fn sanitize_schema_is_idempotent() {
        let raw = serde_json::json!({
            "type": "object",
            "$defs": { "Foo": { "type": "string" } },
            "properties": {
                "x": { "$ref": "#/$defs/Foo" }
            }
        });
        let once = sanitize_schema(raw.clone());
        let twice = sanitize_schema(once.clone());
        assert_eq!(once, twice);
        // Existing $defs should remain, no definitions key introduced.
        assert!(twice.get("definitions").is_none());
        assert!(twice.get("$defs").is_some());
    }

    #[test]
    fn json_schema_has_spec_envelope_shape() {
        // Use build_builtins_only() to avoid global plugin registry pollution
        // from build_discovers_plugins_and_rejects_invalid_schema (BadPlugin_117).
        let cat = Catalog::build_builtins_only().expect("build");
        let schema = cat.json_schema();
        assert_eq!(schema["$id"], "ferro-json-ui/v2");
        assert_eq!(schema["type"], "object");
        let required: Vec<&str> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(required.contains(&"$schema"));
        assert!(required.contains(&"root"));
        assert!(required.contains(&"elements"));
    }

    #[test]
    fn json_schema_has_action_and_visibility_defs() {
        let cat = Catalog::build_builtins_only().expect("build");
        let schema = cat.json_schema();
        assert!(
            schema["$defs"]["Action"].is_object(),
            "$defs/Action missing"
        );
        assert!(
            schema["$defs"]["Visibility"].is_object(),
            "$defs/Visibility missing"
        );
        assert!(
            schema["$defs"]["Element"].is_object(),
            "$defs/Element missing"
        );
    }

    #[test]
    fn json_schema_oneof_covers_all_builtins() {
        let cat = Catalog::build_builtins_only().expect("build");
        let schema = cat.json_schema();
        // oneOf is at the Element level (discriminates on element.type, not props.type).
        let one_of = schema["$defs"]["Element"]["oneOf"]
            .as_array()
            .expect("Element.oneOf is an array");

        // Extract every const discriminator from the allOf[0] branch.
        let mut discriminators: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for variant in one_of {
            let c = variant["allOf"][0]["properties"]["type"]["const"]
                .as_str()
                .expect("every variant pins a type const");
            discriminators.insert(c.to_string());
        }

        for name in crate::render::BUILTIN_TYPES.iter() {
            assert!(
                discriminators.contains(*name),
                "oneOf is missing discriminator for '{name}'"
            );
        }

        // Built-ins only — exactly BUILTIN_TYPES.len() variants.
        assert_eq!(
            discriminators.len(),
            crate::render::BUILTIN_TYPES.len(),
            "oneOf variant count mismatch"
        );
    }

    #[test]
    fn json_schema_is_valid() {
        use jsonschema::draft202012;
        let cat = Catalog::build_builtins_only().expect("build");
        let schema = cat.json_schema();
        assert!(
            draft202012::meta::is_valid(schema),
            "assembled full_schema did not meta-validate as Draft 2020-12"
        );
    }

    #[test]
    fn validator_is_compiled_once_and_usable() {
        let cat = Catalog::build_builtins_only().expect("build");
        // The validator field is private — we prove it's real by validating
        // a minimal valid spec value. If the validator were stale / null /
        // placeholder, this would fail or mis-report.
        let minimal_valid = serde_json::json!({
            "$schema": "ferro-json-ui/v2",
            "root": "r",
            "elements": {
                "r": { "type": "Text", "props": { "content": "hi" } }
            }
        });
        // Should succeed — full-schema envelope accepts this shape.
        assert!(cat.validator.is_valid(&minimal_valid));
    }

    #[test]
    fn validator_rejects_wrong_schema_version() {
        let cat = Catalog::build_builtins_only().expect("build");
        let wrong_version = serde_json::json!({
            "$schema": "ferro-json-ui/v99-wrong",
            "root": "r",
            "elements": {
                "r": { "type": "Text", "props": { "content": "hi" } }
            }
        });
        assert!(
            !cat.validator.is_valid(&wrong_version),
            "validator should reject unknown $schema version via const"
        );
    }

    #[test]
    fn oneof_variants_are_deterministic_sorted() {
        let cat1 = Catalog::build_builtins_only().expect("build 1");
        let cat2 = Catalog::build_builtins_only().expect("build 2");
        // Byte-exact equality guarantees deterministic output (CONTEXT D-18).
        assert_eq!(
            serde_json::to_string(cat1.json_schema()).unwrap(),
            serde_json::to_string(cat2.json_schema()).unwrap()
        );
    }

    /// Combined plugin discovery + invalid schema rejection test.
    ///
    /// Uses unique names (`GoodPlugin_117`, `BadPlugin_117`) to avoid collisions
    /// with other test registrations. The bad plugin is registered after the good
    /// one to confirm discovery of the good plugin precedes the rejection.
    #[test]
    fn build_discovers_plugins_and_rejects_invalid_schema() {
        use crate::plugin::{register_plugin, Asset, JsonUiPlugin};

        struct GoodPlugin;
        impl JsonUiPlugin for GoodPlugin {
            fn component_type(&self) -> &str {
                "GoodPlugin_117"
            }
            fn props_schema(&self) -> Value {
                serde_json::json!({ "type": "object" })
            }
            fn render(&self, _: &Value, _: &Value) -> String {
                String::new()
            }
            fn css_assets(&self) -> Vec<Asset> {
                vec![]
            }
            fn js_assets(&self) -> Vec<Asset> {
                vec![]
            }
            fn init_script(&self) -> Option<String> {
                None
            }
        }

        register_plugin(GoodPlugin);

        // Positive discovery: GoodPlugin should appear in plugin_components.
        let cat = Catalog::build().expect("build succeeds with valid plugin only");
        assert!(
            cat.plugin_components.contains_key("GoodPlugin_117"),
            "plugin 'GoodPlugin_117' should have been discovered"
        );
        assert!(cat.plugin_components["GoodPlugin_117"].is_plugin);

        // Now register a bad plugin and confirm build fails with the plugin name embedded.
        struct BadPlugin;
        impl JsonUiPlugin for BadPlugin {
            fn component_type(&self) -> &str {
                "BadPlugin_117"
            }
            fn props_schema(&self) -> Value {
                // JSON Schema requires `type` to be a string or array of strings.
                // Number 42 is invalid → validator_for() rejects it.
                serde_json::json!({ "type": 42 })
            }
            fn render(&self, _: &Value, _: &Value) -> String {
                String::new()
            }
            fn css_assets(&self) -> Vec<Asset> {
                vec![]
            }
            fn js_assets(&self) -> Vec<Asset> {
                vec![]
            }
            fn init_script(&self) -> Option<String> {
                None
            }
        }

        register_plugin(BadPlugin);
        match Catalog::build() {
            Err(CatalogError::BuildFailed(msg)) => {
                assert!(
                    msg.contains("BadPlugin_117"),
                    "error should mention plugin name, got: {msg}"
                );
            }
            Err(other) => panic!("expected BuildFailed mentioning BadPlugin_117, got: {other:?}"),
            Ok(_) => panic!("expected build to fail due to invalid plugin schema"),
        }
    }
}
