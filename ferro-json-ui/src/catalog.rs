//! # Component Catalog
//!
//! Machine-readable registry of every built-in and plugin JSON-UI component.
//!
//! Phase 117 replaces the hand-maintained component reference string with
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
    ButtonProps, CalendarCellProps, CardProps, CheckboxListProps, CheckboxProps, ChecklistProps,
    CollapsibleProps, DataTableProps, DescriptionListProps, DetailPageProps, DropdownMenuProps,
    EmptyStateProps, FormProps, FormSectionProps, GridProps, HeaderProps, ImageProps, InputProps,
    KanbanBoardProps, MediaCardGridProps, ModalProps, NotificationDropdownProps, PageHeaderProps,
    PaginationProps, ProductTileProps, ProgressProps, RawHtmlProps, SegmentedControlProps,
    SelectProps, SeparatorProps, SidebarLayoutProps, SidebarProps, SkeletonProps, StatCardProps,
    StreamTextProps, SwitchProps, TableProps, TabsProps, TextProps, ToastProps,
};

// ── Public types ───────────────────────────────────────────────────────────────

/// Metadata and JSON Schema for a single JSON-UI component.
///
/// Built by [`Catalog::build`] from the static `BUILTIN_SPECS` table (built-ins)
/// or from [`crate::plugin::JsonUiPlugin::props_schema`] (plugins).
pub struct ComponentSpec {
    /// Component type name as it appears in the Spec's `"type"` field.
    pub name: String,
    /// Short imperative description used in prompt output and catalog tooling.
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
    (
        "RawHtml",
        "Server-injected HTML island. CONSUMER is responsible for sanitization — see docs/src/json-ui/plugins.md.",
        || to_value(schema_for!(RawHtmlProps)).unwrap(),
        &[],
    ),
    (
        "StreamText",
        "Connects to a server-sent-events endpoint and renders token-by-token output as plain text. The SSE endpoint must emit `event: done` on completion to prevent auto-reconnect.",
        || to_value(schema_for!(StreamTextProps)).unwrap(),
        &[],
    ),
    // === Containers (containers.rs) ===
    (
        "Card",
        "Content container with title, description, optional badge and subtitle, body children, and optional footer slot.",
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
        "DetailPage",
        "Canonical resource-detail skeleton: PageHeader chrome, optional info Card slot, and stacked body sections from Element.children.",
        || to_value(schema_for!(DetailPageProps)).unwrap(),
        &["actions", "info"],
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
    (
        "SegmentedControl",
        "Connected button cluster — date scrollers, view toggles, mode pickers. Items via literal or data_path.",
        || to_value(schema_for!(SegmentedControlProps)).unwrap(),
        &[],
    ),
    (
        "SidebarLayout",
        "Two-column layout with sticky vertical nav (left) and main content slot (right). Mobile-collapsing.",
        || to_value(schema_for!(SidebarLayoutProps)).unwrap(),
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
    (
        "CheckboxList",
        "Multi-select checkbox group from static options or data-driven array. \
         Each checked option submits as field=value.",
        || to_value(schema_for!(CheckboxListProps)).unwrap(),
        &[],
    ),
    (
        "CheckboxGroup",
        "Multi-select checkbox group (alias for CheckboxList). Each checked option \
         submits as field=value with array-submit semantics. Identical props to \
         CheckboxList; see that entry for full schema.",
        || to_value(schema_for!(CheckboxListProps)).unwrap(),
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
    (
        "MediaCardGrid",
        "Responsive card grid backed by a data array. Each card shows an optional screenshot image, title, description, status badge, and per-row dropdown actions.",
        || to_value(schema_for!(MediaCardGridProps)).unwrap(),
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

    /// Validate a [`crate::spec::Spec`] against the catalog.
    ///
    /// Three-stage pipeline (CONTEXT D-10):
    ///
    /// 1. **Type-name whitelist** — every `element.type_name` must resolve to a
    ///    built-in or plugin component. Unknown names return [`CatalogError::UnknownType`]
    ///    and **short-circuit** the rest of the pipeline. This avoids noisy `oneOf`
    ///    errors from Stage 3 when a component name is simply wrong (RESEARCH §5).
    ///
    /// 2. **Per-element Props validation** — for each element, look up
    ///    `per_component_schemas[type_name]` and validate `element.props` against it
    ///    using on-demand [`jsonschema::validator_for`]. Errors accumulate as
    ///    [`CatalogError::PropsInvalid`]. Plugin schemas are accepted per CONTEXT D-20.
    ///
    /// 3. **Envelope check** — serialize the full `Spec` and run it through the
    ///    cached `self.validator` (compiled once in [`Catalog::build`], SCHEMA-03).
    ///    Errors become [`CatalogError::SpecInvalid`].
    ///
    /// Errors accumulate across Stages 2 and 3 so a caller sees every issue at once.
    pub fn validate(&self, spec: &crate::spec::Spec) -> Result<(), Vec<CatalogError>> {
        let mut errors: Vec<CatalogError> = Vec::new();

        // === Stage 1: type_name whitelist (O(1) per element) ===
        for (id, el) in &spec.elements {
            let known = self.components.contains_key(&el.type_name)
                || self.plugin_components.contains_key(&el.type_name);
            if !known {
                errors.push(CatalogError::UnknownType {
                    element_id: id.clone(),
                    type_name: el.type_name.clone(),
                });
            }
        }
        // SHORT-CIRCUIT: if any type is unknown, skip Stages 2 & 3.
        // Rationale: Stage 3's full-spec oneOf would emit dozens of
        // "no variant matched" errors for unknown types, obscuring the signal.
        // Stage 2 would skip unknowns anyway.
        if !errors.is_empty() {
            return Err(errors);
        }

        // === Stage 2: per-element Props validation ===
        for (id, el) in &spec.elements {
            if let Some(schema) = self.per_component_schemas.get(&el.type_name) {
                // Skip null props — null means "no props provided"; the schema's
                // `required` list is the gate for required fields. When props is
                // null the element carries no props object, which the envelope
                // schema permits (props is optional per the element allOf shape).
                if el.props.is_null() {
                    continue;
                }
                // On-demand compile (CONTEXT D-12). Schemas are small (~50–200 LOC
                // JSON); compile cost < 1 ms per component. Cache as
                // HashMap<String, Validator> if profiling demands it.
                let v = match jsonschema::validator_for(schema) {
                    Ok(v) => v,
                    Err(e) => {
                        errors.push(CatalogError::BuildFailed(format!(
                            "compiling per-component schema for '{}': {e}",
                            el.type_name
                        )));
                        continue;
                    }
                };
                // Strip $data/$template expression objects before schema validation.
                // Expressions are resolved at render time against handler data — the
                // static catalog validator cannot know the resolved type. We substitute
                // expression objects with "" so string-typed fields pass; the runtime
                // resolver (resolve_expressions) enforces the actual type via data binding.
                let validation_props = strip_expr_objects(&el.props);
                let mut per_elem_errs: Vec<String> = Vec::new();
                for err in v.iter_errors(&validation_props) {
                    per_elem_errs.push(format!("{}: {}", err.instance_path(), err));
                }
                if !per_elem_errs.is_empty() {
                    errors.push(CatalogError::PropsInvalid {
                        element_id: id.clone(),
                        type_name: el.type_name.clone(),
                        errors: per_elem_errs,
                    });
                }
            }
        }

        // === Stage 3: full-spec envelope validation (cached validator, SCHEMA-03) ===
        let spec_value = match serde_json::to_value(spec) {
            Ok(v) => v,
            Err(e) => {
                errors.push(CatalogError::SchemaSerialization(e));
                return Err(errors);
            }
        };
        // Strip expression objects in the serialized spec for the same reason as Stage 2.
        let stripped_spec_value = strip_expr_objects(&spec_value);
        let mut envelope_errs: Vec<String> = Vec::new();
        for err in self.validator.iter_errors(&stripped_spec_value) {
            envelope_errs.push(format!("{}: {}", err.instance_path(), err));
        }
        if !envelope_errs.is_empty() {
            errors.push(CatalogError::SpecInvalid {
                errors: envelope_errs,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Return the per-component Props JSON Schema for `type_name`, or `None`
    /// if the name is not registered as a built-in or plugin component.
    ///
    /// The returned schema is Props-only (NOT wrapped in the Element envelope
    /// used by [`Self::json_schema`]). This is the schema shape Phase 120 AI
    /// structured-output generation consumes, and what `ferro json-ui:schema
    /// --component <name>` prints.
    ///
    /// The reference has the same lifetime as `&self` — zero-copy (CONTEXT D-15).
    ///
    /// Lookup is unified across built-ins and plugins via the
    /// `per_component_schemas` map populated in [`Self::build`] (CONTEXT D-20
    /// — plugin schemas are stored identically after meta-validation).
    pub fn component_schema(&self, type_name: &str) -> Option<&Value> {
        self.per_component_schemas.get(type_name)
    }

    /// Iterate built-in [`ComponentSpec`] entries sorted by name (ascending).
    ///
    /// Deterministic ordering is required by CONTEXT D-18 so that
    /// [`Self::json_schema`], `prompt()` (Plan 06), and ferro-mcp
    /// `json_ui_catalog` output (Plan 06 migration) produce byte-stable
    /// results for snapshot tests.
    pub fn components_sorted(&self) -> impl Iterator<Item = &ComponentSpec> {
        let mut entries: Vec<&ComponentSpec> = self.components.values().collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries.into_iter()
    }

    /// Iterate plugin [`ComponentSpec`] entries sorted by name (ascending).
    ///
    /// Separate from built-ins so consumers can format them in a distinct
    /// section (ferro-mcp `json_ui_catalog.CatalogResponse` preserves the
    /// `components` / `plugin_components` split per CONTEXT D-24).
    pub fn plugin_components_sorted(&self) -> impl Iterator<Item = &ComponentSpec> {
        let mut entries: Vec<&ComponentSpec> = self.plugin_components.values().collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries.into_iter()
    }

    /// Generate a concise text system prompt summarizing every component.
    ///
    /// Format: `## Component Catalog` header, a one-line note explaining the
    /// slot convention, then one `### <Name>` section per component (built-ins
    /// then plugins, both sorted by name). Each section contains the
    /// description, a single `Props:` line with `name (Type)` tuples, and
    /// (when non-empty) a `Slots:` line listing slot field names only.
    ///
    /// The prompt is intentionally CONCISE (≤ 10 KB, CONTEXT D-17) — the full
    /// JSON Schema is NOT embedded. Consumers wanting machine-readable schemas
    /// use [`Self::json_schema`] or [`Self::component_schema`] (Plan 07 CLI).
    ///
    /// Deterministic (CONTEXT D-18): two builds of the same catalog yield
    /// byte-identical output; order within sections follows alphabetical order
    /// via [`Self::components_sorted`] and [`Self::plugin_components_sorted`].
    pub fn prompt(&self) -> String {
        let mut out = String::with_capacity(8 * 1024);
        out.push_str("## Component Catalog\n\n");
        out.push_str("Slot fields are Vec<String> of element IDs; body children come from Element.children.\n\n");
        for spec in self.components_sorted() {
            render_component_section(&mut out, spec);
        }
        if self.plugin_components.is_empty() {
            return out;
        }
        out.push_str("## Plugin Components\n\n");
        for spec in self.plugin_components_sorted() {
            render_component_section(&mut out, spec);
        }
        out
    }
}

// ── Prompt generation helpers ─────────────────────────────────────────────────

/// Append a single component section to `out`.
///
/// Shape:
/// ```text
/// ### Card
/// Content container with title and optional footer slot.
/// Props: title (String), description (Option<String>), ...
/// Slots: footer
///
/// ```
///
/// The slot semantics (Vec<String> of element IDs, body children from
/// Element.children) are declared once in the catalog header by
/// [`Catalog::prompt`] rather than repeated per section.
fn render_component_section(out: &mut String, spec: &ComponentSpec) {
    out.push_str("### ");
    out.push_str(&spec.name);
    out.push('\n');
    out.push_str(&spec.description);
    out.push('\n');

    let props_line = render_props_line(&spec.props_schema);
    if !props_line.is_empty() {
        out.push_str("Props: ");
        out.push_str(&props_line);
        out.push('\n');
    }
    if !spec.slot_fields.is_empty() {
        out.push_str("Slots: ");
        out.push_str(&spec.slot_fields.join(", "));
        out.push('\n');
    }
    out.push('\n');
}

/// Render the `Props:` line for a schemars-derived Props schema.
///
/// Walks `schema.properties` in serde-emit order. For each field:
/// - `Option<T>` schemas (schemars emits `anyOf: [{...}, {type: null}]`) render as `Option<T>`.
/// - Enum fields with ≤ 8 `enum` entries render inline as `name (a|b|c)`.
/// - Enum fields with > 8 entries render as `name (one of N — see schema)`.
/// - Plain scalar fields render as `name (String)` / `(i64)` / `(bool)`.
/// - Array types render as `name (Vec<T>)`.
///
/// Returns an empty string if the schema has no `properties` map.
fn render_props_line(schema: &Value) -> String {
    let Some(obj) = schema.as_object() else {
        return String::new();
    };
    let Some(props) = obj.get("properties").and_then(|v| v.as_object()) else {
        return String::new();
    };
    let required: std::collections::HashSet<&str> = obj
        .get("required")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<std::collections::HashSet<_>>()
        })
        .unwrap_or_default();

    let parts: Vec<String> = props
        .iter()
        .map(|(name, field_schema)| {
            let ty = render_field_type(field_schema, required.contains(name.as_str()));
            format!("{name} ({ty})")
        })
        .collect();
    parts.join(", ")
}

/// Render a single field's type string from its JSON Schema.
fn render_field_type(schema: &Value, is_required: bool) -> String {
    // 1) Detect enum inline: {type: "string", enum: [...]} or {enum: [...]}
    if let Some(variants) = schema.get("enum").and_then(|v| v.as_array()) {
        let names: Vec<&str> = variants.iter().filter_map(|v| v.as_str()).collect();
        let inner = render_enum_inline(&names);
        return wrap_optional(inner, is_required);
    }
    // 2) anyOf / oneOf with null → Option<T>
    for key in ["anyOf", "oneOf"] {
        if let Some(arr) = schema.get(key).and_then(|v| v.as_array()) {
            let has_null = arr
                .iter()
                .any(|v| v.get("type").and_then(|t| t.as_str()) == Some("null"));
            let non_null: Vec<&Value> = arr
                .iter()
                .filter(|v| v.get("type").and_then(|t| t.as_str()) != Some("null"))
                .collect();
            if has_null && non_null.len() == 1 {
                let inner = render_field_type(non_null[0], true);
                return format!("Option<{inner}>");
            }
        }
    }
    // 3) type: ["T", "null"] → Option<T>
    if let Some(types) = schema.get("type").and_then(|v| v.as_array()) {
        let non_null: Vec<&str> = types
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|s| *s != "null")
            .collect();
        let has_null = types.iter().any(|v| v.as_str() == Some("null"));
        if has_null && non_null.len() == 1 {
            return format!("Option<{}>", rust_for_json_type(non_null[0], schema));
        }
    }
    // 4) Plain type
    if let Some(t) = schema.get("type").and_then(|v| v.as_str()) {
        let inner = rust_for_json_type(t, schema);
        return wrap_optional(inner, is_required);
    }
    // 5) Fallback: $ref or complex
    wrap_optional("<see schema>".to_string(), is_required)
}

/// Map a JSON Schema `type` + optional `items` to a Rust-ish type name.
fn rust_for_json_type(t: &str, schema: &Value) -> String {
    match t {
        "string" => "String".to_string(),
        "integer" => "i64".to_string(),
        "number" => "f64".to_string(),
        "boolean" => "bool".to_string(),
        "array" => {
            if let Some(items) = schema.get("items") {
                let inner = render_field_type(items, true);
                format!("Vec<{inner}>")
            } else {
                "Vec<Value>".to_string()
            }
        }
        "object" => "Object".to_string(),
        other => other.to_string(),
    }
}

/// Render an enum's variants inline when count ≤ 8, else collapse.
fn render_enum_inline(variants: &[&str]) -> String {
    if variants.len() <= 8 {
        variants.join("|")
    } else {
        format!("one of {} — see schema", variants.len())
    }
}

/// Wrap inner type in `Option<...>` when the field is not required.
fn wrap_optional(inner: String, is_required: bool) -> String {
    if is_required {
        inner
    } else {
        format!("Option<{inner}>")
    }
}

/// Replace every `$data` / `$template` expression object in a value tree with `""`.
///
/// Used by [`Catalog::validate`] so that specs with runtime data-binding placeholders
/// pass static schema validation. Expression objects have the shape
/// `{"$data": "/path"}` or `{"$template": "literal {/path}"}` — single-key objects
/// whose key is the expression marker. They are resolved at render time by
/// [`crate::expression::resolve_expressions`]; the catalog validator must not reject
/// them for failing type checks that only apply to the resolved value.
fn strip_expr_objects(val: &Value) -> Value {
    match val {
        Value::Object(map) => {
            if map.len() == 1 && (map.contains_key("$data") || map.contains_key("$template")) {
                Value::String(String::new())
            } else {
                Value::Object(
                    map.iter()
                        .map(|(k, v)| (k.clone(), strip_expr_objects(v)))
                        .collect(),
                )
            }
        }
        Value::Array(arr) => Value::Array(arr.iter().map(strip_expr_objects).collect()),
        other => other.clone(),
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
    pub(crate) fn build_builtins_only() -> Result<Self, CatalogError> {
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
        // Updated to 40 in Phase 162 Plan 01 (CheckboxList added).
        // Updated to 42 when DetailPage was added.
        // Updated to 43 in Phase 175 Plan 04 (CheckboxGroup alias added).
        // Updated to 44 when MediaCardGrid was added.
        // Updated to 45 in Phase 169 Plan 02 (StreamText added).
        assert_eq!(crate::render::BUILTIN_TYPES.len(), 45);
    }

    #[test]
    fn builtin_specs_len_matches_dispatch() {
        assert_eq!(BUILTIN_SPECS.len(), crate::render::BUILTIN_TYPES.len());
        assert_eq!(BUILTIN_SPECS.len(), 45);
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
        // Use build_builtins_only() to avoid pollution from BadPlugin_117.
        let cat = Catalog::build_builtins_only().expect("build succeeds");
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
        // Use build_builtins_only() to avoid pollution from BadPlugin_117.
        let cat = Catalog::build_builtins_only().expect("build succeeds");
        let card = &cat.components["Card"];
        assert_eq!(card.slot_fields, vec!["footer"]);
    }

    #[test]
    fn build_modal_has_footer_slot() {
        // Use build_builtins_only() to avoid pollution from BadPlugin_117.
        let cat = Catalog::build_builtins_only().expect("build succeeds");
        let modal = &cat.components["Modal"];
        assert_eq!(modal.slot_fields, vec!["footer"]);
    }

    #[test]
    fn build_pageheader_has_actions_slot() {
        // Use build_builtins_only() to avoid pollution from BadPlugin_117.
        let cat = Catalog::build_builtins_only().expect("build succeeds");
        let ph = &cat.components["PageHeader"];
        assert_eq!(ph.slot_fields, vec!["actions"]);
    }

    #[test]
    fn build_text_has_no_slots() {
        // Use build_builtins_only() to avoid pollution from BadPlugin_117.
        let cat = Catalog::build_builtins_only().expect("build succeeds");
        assert!(cat.components["Text"].slot_fields.is_empty());
    }

    #[test]
    fn build_populates_per_component_schemas() {
        // Use build_builtins_only() to avoid pollution from BadPlugin_117.
        let cat = Catalog::build_builtins_only().expect("build succeeds");
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

    // ── validate() tests (Plan 04) ────────────────────────────────────────────

    /// Build a minimal valid Spec with one Element of the given type + props.
    fn test_spec_with(type_name: &str, props: Value) -> crate::spec::Spec {
        use crate::spec::{Element, Spec};
        use std::collections::HashMap;
        let mut elements = HashMap::new();
        elements.insert(
            "r".to_string(),
            Element {
                type_name: type_name.to_string(),
                props,
                children: Vec::new(),
                action: None,
                visible: None,
                each: None,
                if_: None,
            },
        );
        Spec {
            schema: crate::spec::SCHEMA_VERSION.to_string(),
            root: "r".to_string(),
            elements,
            title: None,
            layout: None,
            data: Value::Null,
        }
    }

    #[test]
    fn validate_positive_per_type() {
        // Representative subset of built-ins — confirms validate() passes for
        // minimally valid elements. Full 39-type coverage lives in Plan 07.
        let cat = Catalog::build_builtins_only().expect("build");
        let cases: Vec<(&str, Value)> = vec![
            ("Text", serde_json::json!({ "content": "hi" })),
            ("Button", serde_json::json!({ "label": "Save" })),
            ("Badge", serde_json::json!({ "label": "New" })),
            ("Separator", serde_json::json!({})),
        ];
        for (ty, props) in cases {
            let spec = test_spec_with(ty, props.clone());
            match cat.validate(&spec) {
                Ok(()) => {}
                Err(errs) => panic!("validate({ty}) failed: {errs:?}"),
            }
        }
    }

    #[test]
    fn validate_unknown_type() {
        let cat = Catalog::build_builtins_only().expect("build");
        let spec = test_spec_with("NotARealComponent", serde_json::json!({}));
        let errs = cat.validate(&spec).expect_err("should fail");
        assert!(
            errs.iter().any(|e| matches!(
                e,
                CatalogError::UnknownType { type_name, .. } if type_name == "NotARealComponent"
            )),
            "expected UnknownType for NotARealComponent; got {errs:?}"
        );
    }

    #[test]
    fn validate_missing_required_prop() {
        // CardProps.title is required (no Option, no #[serde(default)]).
        // Passing {} props should produce PropsInvalid.
        let cat = Catalog::build_builtins_only().expect("build");
        let spec = test_spec_with("Card", serde_json::json!({}));
        let errs = cat.validate(&spec).expect_err("should fail");
        assert!(
            errs.iter().any(|e| matches!(
                e,
                CatalogError::PropsInvalid { type_name, .. } if type_name == "Card"
            )),
            "expected PropsInvalid for missing required 'title'; got {errs:?}"
        );
    }

    #[test]
    fn validate_bad_schema_version() {
        let cat = Catalog::build_builtins_only().expect("build");
        let mut spec = test_spec_with("Text", serde_json::json!({ "content": "hi" }));
        spec.schema = "ferro-json-ui/v99-wrong".to_string();
        let errs = cat.validate(&spec).expect_err("should fail");
        assert!(
            errs.iter()
                .any(|e| matches!(e, CatalogError::SpecInvalid { .. })),
            "expected SpecInvalid for wrong $schema version; got {errs:?}"
        );
    }

    #[test]
    fn validate_pre_dispatch_short_circuits() {
        // Stage 1 must short-circuit: unknown type + malformed envelope →
        // only UnknownType surfaces (not SpecInvalid or PropsInvalid).
        let cat = Catalog::build_builtins_only().expect("build");
        let mut spec = test_spec_with("NotARealComponent", serde_json::json!({}));
        spec.schema = "ferro-json-ui/v99-wrong".to_string();
        let errs = cat.validate(&spec).expect_err("should fail");

        let has_unknown = errs
            .iter()
            .any(|e| matches!(e, CatalogError::UnknownType { .. }));
        let has_spec_invalid = errs
            .iter()
            .any(|e| matches!(e, CatalogError::SpecInvalid { .. }));
        let has_props_invalid = errs
            .iter()
            .any(|e| matches!(e, CatalogError::PropsInvalid { .. }));

        assert!(has_unknown, "expected UnknownType");
        assert!(
            !has_spec_invalid,
            "Stage 3 ran despite Stage 1 failing: {errs:?}"
        );
        assert!(
            !has_props_invalid,
            "Stage 2 ran despite Stage 1 failing: {errs:?}"
        );
    }

    #[test]
    fn validator_is_cached_not_recompiled() {
        // Structural guarantee: self.validator is a plain field, not recompiled
        // per validate() call. This test approximates that by running validate()
        // 100 times against a single Catalog without panic or regression.
        let cat = Catalog::build_builtins_only().expect("build");
        for _ in 0..100 {
            let spec = test_spec_with("Text", serde_json::json!({ "content": "x" }));
            assert!(cat.validate(&spec).is_ok());
        }
    }

    #[test]
    fn validate_accumulates_multiple_errors_across_elements() {
        // Two elements with missing required props → two PropsInvalid errors.
        use crate::spec::{Element, Spec};
        use std::collections::HashMap;
        let cat = Catalog::build_builtins_only().expect("build");
        let mut elements = HashMap::new();
        elements.insert(
            "a".to_string(),
            Element {
                type_name: "Card".to_string(),
                props: serde_json::json!({}), // missing required "title"
                children: Vec::new(),
                action: None,
                visible: None,
                each: None,
                if_: None,
            },
        );
        elements.insert(
            "b".to_string(),
            Element {
                type_name: "Button".to_string(),
                props: serde_json::json!({}), // missing required "label"
                children: Vec::new(),
                action: None,
                visible: None,
                each: None,
                if_: None,
            },
        );
        let spec = Spec {
            schema: crate::spec::SCHEMA_VERSION.to_string(),
            root: "a".to_string(),
            elements,
            title: None,
            layout: None,
            data: Value::Null,
        };
        let errs = cat.validate(&spec).expect_err("should fail");
        let props_invalid_count = errs
            .iter()
            .filter(|e| matches!(e, CatalogError::PropsInvalid { .. }))
            .count();
        assert!(
            props_invalid_count >= 2,
            "expected at least 2 PropsInvalid errors; got {errs:?}"
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

    // ── component_schema / sorted accessor tests (Plan 05) ───────────────────

    #[test]
    fn component_schema_returns_props_only() {
        // ROADMAP SC-5 canonical example: catalog.component_schema("Card") must
        // return the CardProps schema (NOT the Element wrapper from
        // $defs/Element.properties.props in the full schema).
        let cat = Catalog::build_builtins_only().expect("build");
        let schema = cat
            .component_schema("Card")
            .expect("Card is a built-in component");

        // A Props schema is an object with a `properties` map. The Element
        // envelope would have a `type` + `props` + `children` layout — we
        // assert the Props-only shape by checking for CardProps fields.
        let obj = schema
            .as_object()
            .expect("Card props schema is a JSON object");

        // Expect "type": "object" or equivalent (schemars uses `type` or `oneOf`).
        assert!(
            obj.contains_key("type") || obj.contains_key("oneOf") || obj.contains_key("anyOf"),
            "CardProps schema should be a structural object schema; got {obj:?}"
        );

        // Expect CardProps-specific field "title" exists in `properties`.
        if let Some(props) = obj.get("properties").and_then(|v| v.as_object()) {
            assert!(
                props.contains_key("title"),
                "CardProps schema.properties should include 'title'; got keys: {:?}",
                props.keys().collect::<Vec<_>>()
            );
        } else {
            panic!(
                "CardProps schema missing top-level 'properties' map — \
                 sanitizer or Plan 02 may be wrong. Got: {}",
                serde_json::to_string_pretty(schema).unwrap_or_default()
            );
        }

        // Must NOT be the Element envelope (would mean we accidentally returned
        // full_schema["$defs"]["Element"] or similar — CONTEXT D-19).
        let is_element_wrapper = obj
            .get("properties")
            .and_then(|v| v.as_object())
            .map(|p| p.contains_key("children") && p.contains_key("props"))
            .unwrap_or(false);
        assert!(
            !is_element_wrapper,
            "component_schema('Card') returned an Element wrapper; must be Props-only (CONTEXT D-19)"
        );
    }

    #[test]
    fn component_schema_none_for_unknown() {
        let cat = Catalog::build_builtins_only().expect("build");
        assert!(
            cat.component_schema("NotARealComponent_117_05").is_none(),
            "unknown component must return None"
        );
        // Empty string is also "unknown".
        assert!(cat.component_schema("").is_none());
    }

    #[test]
    fn component_schema_resolves_every_builtin() {
        // Parallel safety net for SC-5: every name in BUILTIN_TYPES must have a
        // per-component schema. If any is missing, Plan 02's BUILTIN_SPECS table
        // or the build loop dropped an entry.
        let cat = Catalog::build_builtins_only().expect("build");
        for name in crate::render::BUILTIN_TYPES.iter() {
            assert!(
                cat.component_schema(name).is_some(),
                "built-in '{name}' has no per-component schema"
            );
        }
    }

    #[test]
    fn components_sorted_yields_ascending_by_name() {
        let cat = Catalog::build_builtins_only().expect("build");
        let names: Vec<String> = cat
            .components_sorted()
            .map(|spec| spec.name.clone())
            .collect();
        assert_eq!(names.len(), crate::render::BUILTIN_TYPES.len());
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(
            names, sorted,
            "components_sorted must yield ascending order"
        );

        // plugin_components_sorted returns the plugin side; may be empty.
        let plugin_names: Vec<String> = cat
            .plugin_components_sorted()
            .map(|spec| spec.name.clone())
            .collect();
        let mut plugin_sorted = plugin_names.clone();
        plugin_sorted.sort();
        assert_eq!(
            plugin_names, plugin_sorted,
            "plugin_components_sorted must yield ascending order"
        );
    }

    // ── prompt() tests (Plan 06) ─────────────────────────────────────────────

    #[test]
    fn prompt_under_size_budget() {
        let cat = Catalog::build_builtins_only().expect("build");
        let prompt = cat.prompt();
        let bytes = prompt.len();
        // Budget bumped from 8 KB to 9 KB in Phase 162 Plan 01 (CheckboxList added, 40 components).
        // Budget bumped from 9 KB to 10 KB in Phase 175 Plan 04 (CheckboxGroup alias added, 43 components).
        // Budget bumped from 10 KB to 11 KB in Phase 169 Plan 02 (StreamText added, 45 components).
        assert!(
            bytes <= 11 * 1024,
            "prompt() is {bytes} bytes, exceeds 11 KB budget (CONTEXT D-17)"
        );
    }

    #[test]
    fn prompt_mentions_every_builtin() {
        let cat = Catalog::build_builtins_only().expect("build");
        let prompt = cat.prompt();
        for name in crate::render::BUILTIN_TYPES.iter() {
            let heading = format!("### {name}\n");
            assert!(
                prompt.contains(&heading),
                "prompt() missing section heading for '{name}'"
            );
        }
    }

    #[test]
    fn prompt_is_deterministic() {
        let cat1 = Catalog::build_builtins_only().expect("build 1");
        let cat2 = Catalog::build_builtins_only().expect("build 2");
        assert_eq!(
            cat1.prompt(),
            cat2.prompt(),
            "prompt() must be deterministic"
        );
    }

    #[test]
    fn prompt_documents_slot_fields() {
        // CardProps has slot_fields = ["footer"] (set in Plan 02). The prompt
        // must include a `Slots:` line for Card.
        let cat = Catalog::build_builtins_only().expect("build");
        let prompt = cat.prompt();
        let card_start = prompt.find("### Card\n").expect("Card section present");
        let card_slice = &prompt[card_start..];
        // End at the next ### heading (or EOF).
        let end = card_slice[3..]
            .find("### ")
            .map(|i| i + 3)
            .unwrap_or(card_slice.len());
        let card_section = &card_slice[..end];
        assert!(
            card_section.contains("Slots: footer"),
            "Card section missing 'Slots: footer' line:\n{card_section}"
        );
    }

    #[test]
    fn prompt_is_not_raw_json_schema() {
        let cat = Catalog::build_builtins_only().expect("build");
        let prompt = cat.prompt();
        assert!(
            prompt.starts_with("## Component Catalog"),
            "prompt() should start with Markdown header, not JSON"
        );
        assert!(
            !prompt.contains("\"$schema\""),
            "prompt() must not embed raw JSON Schema (ROADMAP caveat)"
        );
    }

    #[test]
    fn catalog_contains_checkbox_group() {
        let cat = Catalog::build_builtins_only().expect("build");
        assert!(
            cat.component_schema("CheckboxGroup").is_some(),
            "CheckboxGroup must be registered in BUILTIN_SPECS as an alias for CheckboxList"
        );
    }

    #[test]
    fn global_catalog_includes_stream_text() {
        let cat = Catalog::build_builtins_only().expect("build");
        assert!(
            cat.components.contains_key("StreamText"),
            "catalog must include StreamText"
        );
        let spec = &cat.components["StreamText"];
        assert_eq!(spec.name, "StreamText");
        assert!(
            spec.description.contains("event: done"),
            "StreamText description must mention 'event: done'; got: {}",
            spec.description
        );
        assert!(
            spec.props_schema.is_object(),
            "StreamText props_schema must be a JSON object"
        );
        assert!(!spec.is_plugin);
    }
}
