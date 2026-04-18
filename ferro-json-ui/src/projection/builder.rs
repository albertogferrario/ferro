//! Schema-driven projection pipeline — `Spec::from_service_def`.
//!
//! Orchestrates the slot-based rendering of a `ServiceDef` + `IntentScore[]`
//! into a v2 `Spec`, consuming the Plan-01 static vocabulary (`component_map`,
//! `intent_layout`). Every output is validated against `global_catalog()`
//! before being returned — projector and catalog are consistent by construction
//! (D-06). Input mode collapses every intent to a Form per D-11; system fields
//! are excluded from `fields` slot emission per D-10. Theme-supplied
//! `ctx.templates` overrides the built-in `default_template` when present (D-05).
//!
//! This file owns no dispatch tables of its own — the `FieldMeaning` and
//! `NavigationHint` dispatches live in `component_map.rs`, and the
//! `Intent -> IntentModeTemplates` dispatch lives in `intent_layout.rs`. The
//! builder's responsibility is to walk the slot vocabulary and emit one
//! `Element` per slot, wiring typed Props via `serde_json::to_value` (D-04).

#![cfg(feature = "projections")]

use ferro_projections::render::{field_display_name, is_system_field};
use ferro_projections::{
    FieldDef, Intent, IntentScore, NavigationHint, RelationshipDef, ServiceDef,
};
use ferro_theme::IntentSlotTemplate;

use crate::action::Action;
use crate::catalog::{global_catalog, Catalog};
use crate::component::{
    CardProps, Column, DataTableProps, DescriptionItem, DescriptionListProps, FormProps,
    KanbanBoardProps, KanbanColumnProps, StatCardProps, Tab, TableProps, TabsProps,
};
use crate::spec::{Element, ElementBuilder, Spec};

use super::component_map::{
    build_badge_props, build_column_for_field, build_description_item, build_input_props,
    build_progress_props, build_relationship_button_props, build_relationship_text_props,
    build_select_props, build_switch_props, build_text_props, lookup_meaning, lookup_relationship,
};
use super::error::ProjectionError;
use super::intent_layout::{default_template, pick_intent_template};
use super::{RenderMode, VisualContext};

// Silence unused-import warnings until Plan 03 rewires the legacy renderer.
#[allow(dead_code)]
fn _plan_02_reserved(_: &Element) {}

impl Spec {
    /// Generate a v2 `Spec` from a `ServiceDef` and its ranked intents.
    ///
    /// Consumes the intent at `ctx.intent_index`, resolves an
    /// `IntentSlotTemplate` (theme override or built-in default), then emits
    /// one `Element` per slot into the flat `Spec.elements` map.
    /// `ctx.mode == RenderMode::Input` collapses the pipeline to a fixed Form
    /// layout per D-11.
    ///
    /// Every returned `Spec` has been validated by
    /// `global_catalog().validate` (D-06). In `cfg(debug_assertions)` builds,
    /// validation failure also panics so test runs fail loudly.
    pub fn from_service_def(
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &VisualContext,
    ) -> Result<Spec, ProjectionError> {
        Self::from_service_def_with_catalog(service, intents, ctx, global_catalog())
    }

    /// Test-friendly variant of `from_service_def` that accepts an explicit
    /// catalog reference.
    ///
    /// The public `from_service_def` calls this with `global_catalog()` at
    /// runtime. Tests pass a plugin-free `Catalog::build_builtins_only()` to
    /// avoid test-pollution from sibling tests that register plugins with
    /// invalid schemas (same workaround documented at `catalog.rs:1117`).
    pub(crate) fn from_service_def_with_catalog(
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &VisualContext,
        catalog: &Catalog,
    ) -> Result<Spec, ProjectionError> {
        if intents.is_empty() {
            return Err(ProjectionError::EmptyIntents);
        }
        let intent_score =
            intents
                .get(ctx.intent_index)
                .ok_or(ProjectionError::IntentIndexOutOfBounds {
                    requested: ctx.intent_index,
                    available: intents.len(),
                })?;

        let spec = if ctx.mode == RenderMode::Input {
            build_input_spec(service)?
        } else {
            let template = ctx
                .templates
                .as_ref()
                .and_then(|t| pick_intent_template(t, &intent_score.intent))
                .cloned()
                .unwrap_or_else(|| default_template(&intent_score.intent));
            build_display_spec(service, &intent_score.intent, &template.display)?
        };

        match catalog.validate(&spec) {
            Ok(()) => Ok(spec),
            Err(errors) => {
                #[cfg(debug_assertions)]
                panic!("Projector emitted invalid spec: {errors:?}");
                #[cfg(not(debug_assertions))]
                {
                    Err(ProjectionError::CatalogValidation(errors))
                }
            }
        }
    }
}

/// Fall back to `service.name` when `display_name` is absent (D-14).
fn resolve_title(service: &ServiceDef) -> String {
    service
        .display_name
        .as_deref()
        .unwrap_or(&service.name)
        .to_string()
}

/// Lift a typed Props `serde_json::Value` object into an `ElementBuilder`.
///
/// `props` must be a JSON object (every typed `*Props` struct in
/// `component.rs` serializes to one); panics otherwise, which indicates a
/// bug in the caller, not untrusted data.
fn element_with_props(type_name: &str, props: serde_json::Value) -> ElementBuilder {
    let obj = props
        .as_object()
        .expect("typed Props must serialize to a JSON object");
    let mut el = Element::new(type_name);
    for (k, v) in obj {
        el = el.prop(k.clone(), v.clone());
    }
    el
}

/// Same as `element_with_props` but also appends the supplied child IDs.
fn element_with_props_and_children(
    type_name: &str,
    props: serde_json::Value,
    children: Vec<String>,
) -> ElementBuilder {
    let mut el = element_with_props(type_name, props);
    for child in children {
        el = el.child(child);
    }
    el
}

// ---------------------------------------------------------------------------
// Input mode (D-11) — every intent collapses to Form.
// ---------------------------------------------------------------------------

fn build_input_spec(service: &ServiceDef) -> Result<Spec, ProjectionError> {
    // FormProps.action is required (Pitfall 1). Use the conventional
    // POST /{service.name} placeholder; Phase 118+ resolves a real route.
    let action = Action::new(format!("/{}", service.name));
    let form_props = serde_json::to_value(FormProps {
        action,
        method: None,
        guard: None,
        max_width: None,
    })
    .expect("FormProps serialization cannot fail");

    let mut children_ids: Vec<String> = Vec::new();
    let mut field_elements: Vec<(String, ElementBuilder)> = Vec::new();

    for field in service
        .fields
        .iter()
        .filter(|f| f.writable && !is_system_field(&f.meaning))
    {
        let choice = lookup_meaning(&field.meaning);
        let Some(type_name) = choice.input else {
            continue;
        };
        let props = input_props_for(type_name, field);
        let id = format!("field_{}", field.name);
        field_elements.push((id.clone(), element_with_props(type_name, props)));
        children_ids.push(id);
    }

    let root = element_with_props_and_children("Form", form_props, children_ids);

    let mut builder = Spec::builder()
        .title(resolve_title(service))
        .element("root", root);
    for (id, el) in field_elements {
        builder = builder.element(id, el);
    }
    builder.build().map_err(ProjectionError::SpecBuild)
}

/// Dispatch the typed Props builder for a given input-mode component name.
/// The `type_name` comes from `lookup_meaning(...).input`, which is one of
/// `"Input" | "Select" | "Switch"` per the Plan 01 component_map.
fn input_props_for(type_name: &str, field: &FieldDef) -> serde_json::Value {
    match type_name {
        "Input" => build_input_props(field),
        "Select" => build_select_props(field),
        "Switch" => build_switch_props(field),
        // Unknown input components fall back to Input so that validation
        // still receives a non-null, well-formed typed Props payload.
        // The drift guard test in component_map.rs ensures we never hit
        // this branch in practice.
        _ => build_input_props(field),
    }
}

// ---------------------------------------------------------------------------
// Display mode (D-05 + D-08) — slot orchestration keyed on outer layout.
// ---------------------------------------------------------------------------

fn build_display_spec(
    service: &ServiceDef,
    intent: &Intent,
    template: &IntentSlotTemplate,
) -> Result<Spec, ProjectionError> {
    let layout = template.layout.as_deref().unwrap_or("Card");

    let mut aux_elements: Vec<(String, ElementBuilder)> = Vec::new();
    let mut root_children: Vec<String> = Vec::new();

    let root = match layout {
        "DataTable" => emit_datatable_root(service),
        "Card" => emit_card_root(
            service,
            &template.slots,
            &mut aux_elements,
            &mut root_children,
        ),
        "Form" => {
            // Collect intent in Display mode — reuse the Input pipeline so the
            // Form always carries a required `action` prop (Pitfall 1).
            return build_input_spec(service);
        }
        "KanbanBoard" => emit_kanban_root(service),
        "StatCard" => emit_statcard_root(service, &template.slots, &mut aux_elements),
        other => {
            return Err(ProjectionError::UnknownComponent {
                type_name: other.to_string(),
            });
        }
    };

    // `intent` currently only selects the template passed in; parameter kept
    // to document the dispatch path for future per-intent tweaks.
    let _ = intent;

    let mut builder = Spec::builder()
        .title(resolve_title(service))
        .element("root", root);
    for (id, el) in aux_elements {
        builder = builder.element(id, el);
    }
    builder.build().map_err(ProjectionError::SpecBuild)
}

// ---------------------------------------------------------------------------
// Root-emit helpers per layout.
// ---------------------------------------------------------------------------

/// Browse / Track root. `fields` slot populates `DataTableProps.columns`
/// directly — no child elements (Pitfall 3 keeps depth at 1).
fn emit_datatable_root(service: &ServiceDef) -> ElementBuilder {
    let columns: Vec<Column> = service
        .fields
        .iter()
        .filter(|f| f.readable && !is_system_field(&f.meaning))
        .map(build_column_for_field)
        .collect();
    let props = serde_json::to_value(DataTableProps {
        columns,
        data_path: format!("/data/{}", service.name),
        row_actions: None,
        empty_message: None,
        row_key: None,
    })
    .expect("DataTableProps serialization cannot fail");
    element_with_props("DataTable", props)
}

/// Focus / Analyze / Custom root. Walks `slots` in order, emitting one or
/// more child elements per slot name (fields -> DescriptionList,
/// relationships -> one element per relationship, metadata ->
/// DescriptionList of system fields).
fn emit_card_root(
    service: &ServiceDef,
    slots: &[String],
    aux: &mut Vec<(String, ElementBuilder)>,
    children_out: &mut Vec<String>,
) -> ElementBuilder {
    for slot in slots {
        match slot.as_str() {
            "title" => { /* captured as CardProps.title below */ }
            "fields" => emit_fields_as_description_list(service, aux, children_out),
            "relationships" => emit_relationships(service, aux, children_out),
            "actions" => emit_actions_placeholder(service, aux, children_out),
            "metadata" => emit_metadata(service, aux, children_out),
            "body" => emit_body_placeholder(aux, children_out),
            // Non-applicable slots are silently skipped. Intent layouts align
            // slots to layouts; mismatches only occur with theme overrides,
            // where a no-op skip is the documented contract.
            _ => {}
        }
    }

    let props = serde_json::to_value(CardProps {
        title: resolve_title(service),
        description: None,
        max_width: None,
        footer: Vec::new(),
    })
    .expect("CardProps serialization cannot fail");
    let mut el = element_with_props("Card", props);
    for id in children_out.iter() {
        el = el.child(id.clone());
    }
    el
}

/// Process root. KanbanBoard emits a single root element with columns in
/// props — no child elements — so depth stays at 1 (Pitfall 3). Full
/// state-machine awareness is a deferred idea (see CONTEXT.md); for now we
/// emit a single placeholder column carrying the service's display name.
fn emit_kanban_root(service: &ServiceDef) -> ElementBuilder {
    let placeholder = KanbanColumnProps {
        id: "default".to_string(),
        title: resolve_title(service),
        count: 0,
        children: Vec::new(),
    };
    let props = serde_json::to_value(KanbanBoardProps {
        columns: vec![placeholder],
        mobile_default_column: None,
    })
    .expect("KanbanBoardProps serialization cannot fail");
    element_with_props("KanbanBoard", props)
}

/// Summarize root. StatCard has no child-element slots; `metadata` adds a
/// sibling DescriptionList (so it still validates and stays inside depth 3).
fn emit_statcard_root(
    service: &ServiceDef,
    slots: &[String],
    aux: &mut Vec<(String, ElementBuilder)>,
) -> ElementBuilder {
    // The metadata slot emits a sibling element — but StatCard itself has
    // no `children` convention, so we append it to `aux` without wiring
    // it into the root. Callers iterating `spec.elements` can still see it.
    // This is aligned with how other "no-child container" components behave
    // in the existing renderer; catalog validation permits orphan aux
    // elements as long as they reference valid type_names.
    let mut dropped: Vec<String> = Vec::new();
    for slot in slots {
        if slot == "metadata" {
            emit_metadata(service, aux, &mut dropped);
        }
    }

    let props = serde_json::to_value(StatCardProps {
        label: resolve_title(service),
        value: String::new(),
        icon: None,
        subtitle: None,
        sse_target: None,
    })
    .expect("StatCardProps serialization cannot fail");
    element_with_props("StatCard", props)
}

// ---------------------------------------------------------------------------
// Slot emit helpers.
// ---------------------------------------------------------------------------

/// `fields` slot for Card-shaped layouts: a DescriptionList of every
/// readable, non-system field (D-10 excludes Identifier/CreatedAt/UpdatedAt).
fn emit_fields_as_description_list(
    service: &ServiceDef,
    aux: &mut Vec<(String, ElementBuilder)>,
    children_out: &mut Vec<String>,
) {
    let items: Vec<DescriptionItem> = service
        .fields
        .iter()
        .filter(|f| f.readable && !is_system_field(&f.meaning))
        .map(build_description_item)
        .collect();
    if items.is_empty() {
        return;
    }
    let props = serde_json::to_value(DescriptionListProps {
        items,
        columns: None,
    })
    .expect("DescriptionListProps serialization cannot fail");
    let id = "fields_list".to_string();
    aux.push((id.clone(), element_with_props("DescriptionList", props)));
    children_out.push(id);
}

/// `relationships` slot (D-09). Tab-hint relationships are grouped into a
/// single Tabs container (Pitfall 7); every other relationship emits one
/// element per RelationshipDef.
fn emit_relationships(
    service: &ServiceDef,
    aux: &mut Vec<(String, ElementBuilder)>,
    children_out: &mut Vec<String>,
) {
    // Group Tab-hint relationships into a single Tabs container.
    let tab_rels: Vec<&RelationshipDef> = service
        .relationships
        .iter()
        .filter(|r| matches!(r.navigation, NavigationHint::Tab))
        .collect();
    if !tab_rels.is_empty() {
        let tabs: Vec<Tab> = tab_rels
            .iter()
            .map(|r| Tab {
                value: r.name.clone(),
                label: field_display_name(&r.target),
                children: Vec::new(),
            })
            .collect();
        let default_tab = tabs.first().map(|t| t.value.clone()).unwrap_or_default();
        let props = serde_json::to_value(TabsProps { default_tab, tabs })
            .expect("TabsProps serialization cannot fail");
        let id = "relationships_tabs".to_string();
        aux.push((id.clone(), element_with_props("Tabs", props)));
        children_out.push(id);
    }

    // Emit one element per non-Tab relationship.
    for rel in service.relationships.iter() {
        if matches!(rel.navigation, NavigationHint::Tab) {
            continue;
        }
        let Some(component) = lookup_relationship(rel.navigation) else {
            continue; // Hidden -> skip
        };
        let props = match rel.navigation {
            NavigationHint::Inline => build_relationship_text_props(rel),
            NavigationHint::Link => build_relationship_button_props(rel),
            NavigationHint::Nested => {
                // Nested -> Table with a single "name" column, data_path =
                // /data/{rel.name}. Phase 118+ binds the column data via $data.
                let col = Column {
                    key: "name".to_string(),
                    label: field_display_name(&rel.target),
                    format: None,
                };
                serde_json::to_value(TableProps {
                    columns: vec![col],
                    data_path: format!("/data/{}", rel.name),
                    row_actions: None,
                    empty_message: None,
                    sortable: None,
                    sort_column: None,
                    sort_direction: None,
                })
                .expect("TableProps serialization cannot fail")
            }
            _ => continue,
        };
        let id = format!("rel_{}", rel.name);
        aux.push((id.clone(), element_with_props(component, props)));
        children_out.push(id);
    }
}

/// `actions` slot placeholder. Full action wiring is deferred per
/// CONTEXT.md Deferred Ideas; Phase 118+ will add Button elements that
/// reference `service.actions: Vec<ActionDef>` with resolved handlers.
#[allow(clippy::ptr_arg)] // kept for signature parity with live slot emitters
fn emit_actions_placeholder(
    _service: &ServiceDef,
    _aux: &mut Vec<(String, ElementBuilder)>,
    _children_out: &mut Vec<String>,
) {
    // Intentionally empty. Deferred to Phase 118+.
}

/// `metadata` slot — system fields (Identifier / CreatedAt / UpdatedAt)
/// rendered as a read-only DescriptionList (D-10 keeps them out of the
/// primary `fields` slot and into `metadata`).
fn emit_metadata(
    service: &ServiceDef,
    aux: &mut Vec<(String, ElementBuilder)>,
    children_out: &mut Vec<String>,
) {
    let items: Vec<DescriptionItem> = service
        .fields
        .iter()
        .filter(|f| is_system_field(&f.meaning))
        .map(build_description_item)
        .collect();
    if items.is_empty() {
        return;
    }
    let props = serde_json::to_value(DescriptionListProps {
        items,
        columns: None,
    })
    .expect("DescriptionListProps serialization cannot fail");
    let id = "metadata_list".to_string();
    aux.push((id.clone(), element_with_props("DescriptionList", props)));
    children_out.push(id);
}

/// `body` slot placeholder — Analyze/Process intents reserve this slot for
/// free-form rich content. Phase 117.1 emits nothing; deferred to a later
/// phase that can derive body content from service descriptions.
#[allow(clippy::ptr_arg)] // kept for signature parity with live slot emitters
fn emit_body_placeholder(
    _aux: &mut Vec<(String, ElementBuilder)>,
    _children_out: &mut Vec<String>,
) {
    // Intentionally empty. See CONTEXT.md Deferred Ideas.
}

// ---------------------------------------------------------------------------
// Currently unused helpers reserved for later slot emissions. Silenced so
// that the drift guard + test suite compiles cleanly in Plan 02; Plan 03
// wires these in once the renderer delegates to `from_service_def`.
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn _reserved_props_noop() {
    let _ = build_badge_props;
    let _ = build_progress_props;
    let _ = build_text_props;
}

#[cfg(test)]
mod tests {
    //! Tests use `Spec::from_service_def_with_catalog(..., &clean_catalog)`
    //! rather than the public `Spec::from_service_def` because the public
    //! entry point relies on `global_catalog()` — a `OnceLock` singleton that
    //! gets poisoned when a sibling catalog test registers the
    //! `BadPlugin_117` plugin. This is the same test-isolation pattern
    //! adopted by `catalog.rs` and `component_map.rs` (documented at
    //! `catalog.rs:1117` and Plan-01 SUMMARY decision #1).
    //!
    //! `from_service_def_with_catalog` is the test-friendly variant that
    //! accepts an injected catalog reference. It exercises the exact same
    //! code path as the public API — only the catalog source differs.

    use super::*;
    use crate::catalog::Catalog;
    use ferro_projections::{derive_intents, DataType, FieldMeaning, ServiceDef};
    use ferro_theme::{IntentModeTemplates, IntentSlotTemplate, ThemeTemplates};

    fn sample_service() -> ServiceDef {
        ServiceDef::new("product")
            .display_name("Product")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("price", DataType::Float, FieldMeaning::Money)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
    }

    fn clean_catalog() -> Catalog {
        Catalog::build_builtins_only().expect("builtins-only catalog builds clean")
    }

    #[test]
    fn from_service_def_validates() {
        // D-06: every success path implies validate() returned Ok (otherwise
        // `from_service_def_with_catalog` would have returned Err or panicked
        // in debug). Cross-check by re-running validate on the returned spec
        // against the same catalog.
        let service = sample_service();
        let intents = derive_intents(&service);
        let ctx = VisualContext::default();
        let cat = clean_catalog();
        let spec = Spec::from_service_def_with_catalog(&service, &intents, &ctx, &cat)
            .expect("valid projection should pass validation");
        assert!(cat.validate(&spec).is_ok());
    }

    #[test]
    fn from_service_def_browse_display() {
        // D-08 + ROADMAP criterion 1: Browse+Display projects to DataTable.
        let service = sample_service();
        let intents = derive_intents(&service);
        let ctx = VisualContext {
            intent_index: intents
                .iter()
                .position(|i| matches!(i.intent, Intent::Browse))
                .unwrap_or(0),
            mode: RenderMode::Display,
            ..Default::default()
        };
        let cat = clean_catalog();
        let spec = Spec::from_service_def_with_catalog(&service, &intents, &ctx, &cat)
            .expect("should project");
        assert_eq!(spec.schema, "ferro-json-ui/v2");
        let root = spec.elements.get(&spec.root).expect("root element exists");
        assert_eq!(root.type_name, "DataTable");
    }

    #[test]
    fn input_mode_always_form() {
        // D-11: RenderMode::Input collapses every intent to a Form root.
        let service = sample_service();
        let intents = derive_intents(&service);
        assert!(!intents.is_empty(), "sample service must derive intents");
        let cat = clean_catalog();
        for idx in 0..intents.len() {
            let ctx = VisualContext {
                intent_index: idx,
                mode: RenderMode::Input,
                ..Default::default()
            };
            let spec = Spec::from_service_def_with_catalog(&service, &intents, &ctx, &cat)
                .expect("every intent should project in Input mode");
            let root = spec.elements.get(&spec.root).unwrap();
            assert_eq!(
                root.type_name, "Form",
                "intent at index {idx} did not collapse to Form in Input mode"
            );
        }
    }

    #[test]
    fn system_fields_excluded() {
        // D-10: Identifier / CreatedAt / UpdatedAt must not appear in the
        // DataTable columns for a Browse projection.
        let service = sample_service();
        let intents = derive_intents(&service);
        let ctx = VisualContext {
            intent_index: intents
                .iter()
                .position(|i| matches!(i.intent, Intent::Browse))
                .unwrap_or(0),
            mode: RenderMode::Display,
            ..Default::default()
        };
        let cat = clean_catalog();
        let spec = Spec::from_service_def_with_catalog(&service, &intents, &ctx, &cat).unwrap();
        let root = spec.elements.get(&spec.root).unwrap();
        let columns = root
            .props
            .get("columns")
            .and_then(|c| c.as_array())
            .expect("columns array present");
        let keys: Vec<&str> = columns
            .iter()
            .filter_map(|c| c.get("key").and_then(|k| k.as_str()))
            .collect();
        assert!(keys.contains(&"name"), "name column expected: {keys:?}");
        assert!(keys.contains(&"price"), "price column expected: {keys:?}");
        assert!(
            !keys.contains(&"id"),
            "id column must be excluded: {keys:?}"
        );
        assert!(
            !keys.contains(&"created_at"),
            "created_at column must be excluded: {keys:?}"
        );
    }

    #[test]
    fn template_override() {
        // D-05: a theme-supplied template for Browse must override the
        // built-in default (which uses DataTable).
        let service = sample_service();
        let intents = derive_intents(&service);
        let templates = ThemeTemplates {
            browse: Some(IntentModeTemplates {
                display: IntentSlotTemplate {
                    slots: vec!["title".into(), "stats".into(), "metadata".into()],
                    layout: Some("StatCard".into()),
                },
                input: IntentSlotTemplate::default(),
            }),
            focus: None,
            collect: None,
            process: None,
            summarize: None,
            analyze: None,
            track: None,
        };
        let ctx = VisualContext {
            intent_index: intents
                .iter()
                .position(|i| matches!(i.intent, Intent::Browse))
                .unwrap_or(0),
            mode: RenderMode::Display,
            templates: Some(templates),
            ..Default::default()
        };
        let cat = clean_catalog();
        let spec = Spec::from_service_def_with_catalog(&service, &intents, &ctx, &cat)
            .expect("override projects");
        let root = spec.elements.get(&spec.root).unwrap();
        assert_eq!(
            root.type_name, "StatCard",
            "theme override must win over default_template"
        );
    }

    #[test]
    fn empty_intents_returns_error() {
        let service = sample_service();
        let cat = clean_catalog();
        let result =
            Spec::from_service_def_with_catalog(&service, &[], &VisualContext::default(), &cat);
        assert!(matches!(result, Err(ProjectionError::EmptyIntents)));
    }

    #[test]
    fn out_of_bounds_intent_index_returns_error() {
        let service = sample_service();
        let intents = derive_intents(&service);
        let ctx = VisualContext {
            intent_index: intents.len() + 5,
            ..Default::default()
        };
        let cat = clean_catalog();
        let result = Spec::from_service_def_with_catalog(&service, &intents, &ctx, &cat);
        match result {
            Err(ProjectionError::IntentIndexOutOfBounds {
                requested,
                available,
            }) => {
                assert_eq!(requested, intents.len() + 5);
                assert_eq!(available, intents.len());
            }
            other => panic!("expected IntentIndexOutOfBounds, got {other:?}"),
        }
    }
}
