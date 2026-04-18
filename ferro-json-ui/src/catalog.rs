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

use serde_json::Value;

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
// Fields are populated in Plan 02; suppress dead_code until then.
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

// ── Catalog impl ───────────────────────────────────────────────────────────────

impl Catalog {
    /// Build the catalog from the static built-in specs and the current plugin registry.
    ///
    /// Called once by [`global_catalog`]. Returns `Err` if a plugin's `props_schema()`
    /// is not a valid JSON Schema or if the assembled full schema fails to compile.
    ///
    /// # Errors
    ///
    /// - [`CatalogError::BuildFailed`] — jsonschema compilation failure.
    /// - [`CatalogError::SchemaSerialization`] — serde_json serialization failure.
    pub fn build() -> Result<Self, CatalogError> {
        unimplemented!("Plan 02 populates BUILTIN_SPECS and implements build")
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
mod tests {
    #[test]
    fn builtin_types_count_is_39() {
        // Drift guard — if this fails, Phase 116's BUILTIN_TYPES changed
        // without a corresponding catalog update. See Plan 02.
        assert_eq!(crate::render::BUILTIN_TYPES.len(), 39);
    }
}
