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
    FieldDef, FieldMeaning, Intent, IntentScore, NavigationHint, RelationshipDef, ServiceDef,
};
use ferro_theme::IntentSlotTemplate;

use crate::action::Action;
use crate::catalog::{global_catalog, Catalog};
use crate::component::{
    ActionGroupProps, ActionItem, CardAppearance, CardProps, Column, DataTableProps,
    DescriptionItem, DescriptionListProps, DropdownMenuAction, FormProps, KanbanBoardProps,
    KanbanColumnProps, StatCardProps, Tab, TableProps, TabsProps,
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
        // Argument-shape errors short-circuit before `global_catalog()` is
        // touched — callers with malformed inputs should not pay the cost
        // of OnceLock catalog construction, and this keeps error paths
        // immune to OnceLock pollution in test processes.
        if intents.is_empty() {
            return Err(ProjectionError::EmptyIntents);
        }
        if ctx.base.intent_index >= intents.len() {
            return Err(ProjectionError::IntentIndexOutOfBounds {
                requested: ctx.base.intent_index,
                available: intents.len(),
            });
        }
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
                .get(ctx.base.intent_index)
                .ok_or(ProjectionError::IntentIndexOutOfBounds {
                    requested: ctx.base.intent_index,
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
            build_display_spec(service, &intent_score.intent, &template.display, ctx)?
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
        id: None,
        enctype: None,
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
        let props = input_props_for(type_name, field)?;
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
/// The `type_name` comes from `lookup_meaning(...).input`, which today is
/// one of `"Input" | "Select" | "Switch"` per the Plan 01 component_map.
///
/// Unknown `type_name` values surface as `ProjectionError::UnknownComponent`
/// rather than being silently coerced into `InputProps`. The existing
/// catalog-name drift guard (`meaning_table_components_exist_in_catalog`)
/// only checks that referenced names exist as catalog components — it does
/// not prove that every `input` value is one of the three dispatched names.
/// If a future meaning adds a new input component (e.g. `DatePicker`) this
/// branch is the choke point that forces the dispatch table to be updated
/// alongside the meaning table.
fn input_props_for(
    type_name: &str,
    field: &FieldDef,
) -> Result<serde_json::Value, ProjectionError> {
    match type_name {
        "Input" => Ok(build_input_props(field)),
        "Select" => Ok(build_select_props(field)),
        "Switch" => Ok(build_switch_props(field)),
        other => Err(ProjectionError::UnknownComponent {
            type_name: other.to_string(),
        }),
    }
}

// ---------------------------------------------------------------------------
// Display mode (D-05 + D-08) — slot orchestration keyed on outer layout.
// ---------------------------------------------------------------------------

fn build_display_spec(
    service: &ServiceDef,
    intent: &Intent,
    template: &IntentSlotTemplate,
    ctx: &VisualContext,
) -> Result<Spec, ProjectionError> {
    let layout = template.layout.as_deref().unwrap_or("Card");

    let mut aux_elements: Vec<(String, ElementBuilder)> = Vec::new();

    let root = match layout {
        "DataTable" => emit_datatable_root(service),
        "Card" => emit_card_root(service, &template.slots, &mut aux_elements),
        "Form" => {
            // Collect intent in Display mode — reuse the Input pipeline so the
            // Form always carries a required `action` prop (Pitfall 1).
            return build_input_spec(service);
        }
        "KanbanBoard" => emit_kanban_root(service, ctx),
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
///
/// Meanings with `column: None` in `lookup_meaning` (Identifier, ForeignKey,
/// ImageUrl, Sensitive) are excluded — this prevents sensitive data (e.g.
/// password hashes) from leaking into Browse/Track projections even when
/// the underlying `FieldDef` is marked `readable = true`.
fn emit_datatable_root(service: &ServiceDef) -> ElementBuilder {
    let columns: Vec<Column> = service
        .fields
        .iter()
        .filter(|f| f.readable && !is_system_field(&f.meaning))
        .filter(|f| lookup_meaning(&f.meaning).column.is_some())
        .map(build_column_for_field)
        .collect();
    let row_actions: Option<Vec<DropdownMenuAction>> = if service.actions.is_empty() {
        None
    } else {
        Some(
            service
                .actions
                .iter()
                .map(|a| DropdownMenuAction {
                    label: a.display_name.as_deref().unwrap_or(&a.name).to_string(),
                    action: Action::new(format!("/{}/{{row_key}}/{}", service.name, a.name)),
                    destructive: false,
                    visible_if: None,
                })
                .collect(),
        )
    };
    let row_key = if service.actions.is_empty() {
        None
    } else {
        Some("id".to_string())
    };
    let props = serde_json::to_value(DataTableProps {
        columns,
        data_path: format!("/data/{}", service.name),
        row_actions,
        empty_message: None,
        row_key,
        row_href: None,
    })
    .expect("DataTableProps serialization cannot fail");
    element_with_props("DataTable", props)
}

/// Focus / Analyze / Custom root. Walks `slots` in order, emitting one or
/// more child elements per slot name (fields -> DescriptionList,
/// relationships -> one element per relationship, metadata ->
/// DescriptionList of system fields).
///
/// Card-layout children are attached inline to the returned `ElementBuilder`;
/// the outer builder owns only `aux_elements`. Slot emitters write both into
/// `aux` (flat element table) and a local `children` vec, and the Card
/// element receives them via `el.child(..)` before being returned.
fn emit_card_root(
    service: &ServiceDef,
    slots: &[String],
    aux: &mut Vec<(String, ElementBuilder)>,
) -> ElementBuilder {
    let mut children: Vec<String> = Vec::new();
    for slot in slots {
        match slot.as_str() {
            "title" => { /* captured as CardProps.title below */ }
            "fields" => emit_fields_as_description_list(service, aux, &mut children),
            "relationships" => emit_relationships(service, aux, &mut children),
            "actions" => emit_actions_placeholder(service, aux, &mut children),
            "metadata" => emit_metadata(service, aux, &mut children),
            "body" => emit_body_placeholder(aux, &mut children),
            // Non-applicable slots are silently skipped. Intent layouts align
            // slots to layouts; mismatches only occur with theme overrides,
            // where a no-op skip is the documented contract.
            _ => {}
        }
    }

    let props = serde_json::to_value(CardProps {
        title: resolve_title(service),
        description: None,
        subtitle: None,
        badge: None,
        max_width: None,
        footer: Vec::new(),
        appearance: CardAppearance::Bordered,
    })
    .expect("CardProps serialization cannot fail");
    let mut el = element_with_props("Card", props);
    for id in children {
        el = el.child(id);
    }
    el
}

/// Process root. KanbanBoard emits a single root element with lane structure
/// in props — no child elements — so depth stays at 1 (Pitfall 3).
///
/// A kanban is fixed lanes plus items sorted into them by a status field. When
/// `service.state_machine` is present, one `KanbanColumnProps` (id + title) is
/// emitted per `StateDef` in declaration order — the lane structure — and the
/// content bindings are set so the renderer buckets handler entities into
/// lanes at render time:
/// - `items_path = /data/{service.name}` — the flat entity array (same path
///   `DataTable` reads), kept flat so handlers need no per-status bucketing.
/// - `group_by` — the `FieldMeaning::Status` field whose value selects the lane
///   (lane id == status value == state name).
/// - `card_title_key` / `card_description_key` — `EntityName` (or identifier)
///   and `Money` field bindings for the prescribed card shape.
/// - `row_actions` / `row_key` — derived from `service.actions`, matching
///   `emit_datatable_root`.
///
/// When `state_machine` is `None` a single placeholder lane (carrying the
/// service display name) is emitted with no content bindings.
///
/// `ctx.current_state` marks the active column as `mobile_default_column`
/// (Risk 3 option a — `KanbanColumnProps` has no `active` field; this is the
/// documented approximation for mobile default tab selection).
fn emit_kanban_root(service: &ServiceDef, ctx: &VisualContext) -> ElementBuilder {
    let columns: Vec<KanbanColumnProps> = service
        .state_machine
        .as_ref()
        .map(|sm| {
            sm.states
                .iter()
                .map(|s| KanbanColumnProps {
                    id: s.name.clone(),
                    title: s.display_name.as_deref().unwrap_or(&s.name).to_string(),
                    count: 0,
                    children: Vec::new(),
                })
                .collect()
        })
        .unwrap_or_else(|| {
            vec![KanbanColumnProps {
                id: "default".to_string(),
                title: resolve_title(service),
                count: 0,
                children: Vec::new(),
            }]
        });

    let field_name_by = |pred: fn(&FieldMeaning) -> bool| -> Option<String> {
        service
            .fields
            .iter()
            .find(|f| f.readable && pred(&f.meaning))
            .map(|f| f.name.clone())
    };

    // Content bindings are emitted only alongside the state-machine lanes — the
    // single placeholder lane has no status field to bucket by.
    let has_state_machine = service.state_machine.is_some();

    // items_path: the flat entity array (same path DataTable reads). Bucketing
    // by `group_by` happens in the renderer, so handlers stay flat.
    let items_path = has_state_machine.then(|| format!("/data/{}", service.name));

    // group_by: the field whose value selects the lane (== state name).
    let group_by = has_state_machine
        .then(|| field_name_by(|m| matches!(m, FieldMeaning::Status)))
        .flatten();

    // Card title prefers a human label (EntityName), falling back to the
    // identifier so a card is never blank.
    let card_title_key = has_state_machine
        .then(|| {
            field_name_by(|m| matches!(m, FieldMeaning::EntityName))
                .or_else(|| field_name_by(|m| matches!(m, FieldMeaning::Identifier)))
        })
        .flatten();

    let card_description_key = has_state_machine
        .then(|| field_name_by(|m| matches!(m, FieldMeaning::Money)))
        .flatten();

    // row_actions / row_key mirror emit_datatable_root — per-card dropdown of
    // the service's actions, with `{row_key}` interpolated from `id`.
    let row_actions: Option<Vec<DropdownMenuAction>> =
        if !has_state_machine || service.actions.is_empty() {
            None
        } else {
            Some(
                service
                    .actions
                    .iter()
                    .map(|a| DropdownMenuAction {
                        label: a.display_name.as_deref().unwrap_or(&a.name).to_string(),
                        action: Action::new(format!("/{}/{{row_key}}/{}", service.name, a.name)),
                        destructive: false,
                        visible_if: None,
                    })
                    .collect(),
            )
        };
    let row_key = row_actions.as_ref().map(|_| "id".to_string());

    // current_state marks the active column on mobile (Risk 3 option a — no
    // KanbanColumnProps.active field exists; mobile_default_column is the
    // documented approximation).
    let mobile_default_column = ctx.base.current_state.clone();

    let props = serde_json::to_value(KanbanBoardProps {
        columns,
        items_path,
        group_by,
        card_title_key,
        card_description_key,
        row_actions,
        row_key,
        mobile_default_column,
        empty_label: None,
    })
    .expect("KanbanBoardProps serialization cannot fail");
    element_with_props("KanbanBoard", props)
}

/// Summarize root. StatCard has no child-element slots; `metadata` adds a
/// sibling DescriptionList in `spec.elements` but it is intentionally **not
/// reachable from the root via `children`** — StatCard's catalog shape
/// forbids child elements.
///
/// The orphan is accepted by `Catalog::validate` (which does not enforce
/// reachability) and is pinned by the
/// `statcard_metadata_is_orphan_element` regression test. Consumers that
/// expect every element to be reachable from the root must treat the
/// Summarize/StatCard + metadata combination as a known exception, or
/// switch to the Card layout (Analyze intent already does this).
///
/// A future phase that introduces a StatCard-with-metadata wrapper (e.g.
/// Card(StatCard, DescriptionList)) should remove this branch and delete
/// the orphan-pinning test.
fn emit_statcard_root(
    service: &ServiceDef,
    slots: &[String],
    aux: &mut Vec<(String, ElementBuilder)>,
) -> ElementBuilder {
    // `dropped` is intentionally thrown away — the id from emit_metadata is
    // not wired into StatCard. See the doc comment above for the contract.
    let mut dropped: Vec<String> = Vec::new();
    for slot in slots {
        if slot == "metadata" {
            emit_metadata(service, aux, &mut dropped);
        }
    }

    // Single primary StatCard bound to the first Money/Quantity readable field
    // (Risk 1: multi-stat Grid is deferred). value_path binds runtime data via
    // render_stat_card → resolve_path_string. T-213-06: only readable fields
    // are eligible; Sensitive/ForeignKey meanings are not Money/Quantity so
    // they are structurally excluded.
    let primary_field = service
        .fields
        .iter()
        .find(|f| f.readable && matches!(f.meaning, FieldMeaning::Money | FieldMeaning::Quantity));
    let (label, value_path) = primary_field
        .map(|f| {
            (
                field_display_name(&f.name),
                Some(format!("/data/{}/{}", service.name, f.name)),
            )
        })
        .unwrap_or_else(|| (resolve_title(service), None));

    let props = serde_json::to_value(StatCardProps {
        label,
        value: String::new(),
        tone: crate::component::Tone::Neutral,
        value_path,
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
///
/// Meanings with `display: None` in `lookup_meaning` (ForeignKey, Sensitive)
/// are excluded — this prevents sensitive data (e.g. password hashes) from
/// leaking into Focus projections even when the underlying `FieldDef` is
/// marked `readable = true`.
fn emit_fields_as_description_list(
    service: &ServiceDef,
    aux: &mut Vec<(String, ElementBuilder)>,
    children_out: &mut Vec<String>,
) {
    let items: Vec<DescriptionItem> = service
        .fields
        .iter()
        .filter(|f| f.readable && !is_system_field(&f.meaning))
        .filter(|f| lookup_meaning(&f.meaning).display.is_some())
        .map(build_description_item)
        .collect();
    if items.is_empty() {
        return;
    }
    let props = serde_json::to_value(DescriptionListProps {
        items,
        columns: None,
        data_path: None,
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
                    align: None,
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

/// `actions` slot. Emits a single `ActionGroup` element carrying one item
/// per `ServiceDef.action`. Action URLs follow the convention
/// `POST /{service.name}/{action.name}` — `ActionDef` has no route field, so
/// the consumer's route table must match this convention for the buttons to
/// resolve (documented as the projection action-route contract, Risk 4).
fn emit_actions_placeholder(
    service: &ServiceDef,
    aux: &mut Vec<(String, ElementBuilder)>,
    children_out: &mut Vec<String>,
) {
    if service.actions.is_empty() {
        return;
    }
    let items: Vec<ActionItem> = service
        .actions
        .iter()
        .map(|a| ActionItem {
            label: a.display_name.as_deref().unwrap_or(&a.name).to_string(),
            action: Action::new(format!("/{}/{}", service.name, a.name)),
            destructive: false,
            variant: None,
            icon: None,
            visible_if: None,
        })
        .collect();
    let props = serde_json::to_value(ActionGroupProps {
        items,
        menu_id: format!("actions_{}", service.name),
        max_inline: None,
        overflow_label: None,
        row_key: None,
    })
    .expect("ActionGroupProps serialization cannot fail");
    let id = "actions_menu".to_string();
    aux.push((id.clone(), element_with_props("ActionGroup", props)));
    children_out.push(id);
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
        data_path: None,
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
    use ferro_projections::render::BaseContext;
    use ferro_projections::{
        derive_intents, ActionDef, DataType, FieldMeaning, ServiceDef, StateDef, StateMachine,
        Transition,
    };
    use ferro_theme::{IntentModeTemplates, IntentSlotTemplate, ThemeTemplates};

    fn sample_service() -> ServiceDef {
        ServiceDef::new("product")
            .display_name("Product")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("price", DataType::Float, FieldMeaning::Money)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
    }

    fn service_with_actions() -> ServiceDef {
        ServiceDef::new("staff")
            .display_name("Staff")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .action(ActionDef::new("view").display_name("View"))
            .action(ActionDef::new("edit").display_name("Edit"))
            .action(ActionDef::new("delete").display_name("Delete"))
    }

    fn service_with_state_machine() -> ServiceDef {
        ServiceDef::new("order")
            .display_name("Order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .state_machine(
                StateMachine::new("lifecycle")
                    .initial("draft")
                    .state(StateDef::new("draft").display_name("Draft"))
                    .state(StateDef::new("submitted").display_name("Submitted"))
                    .state(StateDef::new("done").display_name("Done").final_state())
                    .transition(Transition::new("draft", "submit", "submitted"))
                    .transition(Transition::new("submitted", "complete", "done")),
            )
    }

    // Reserved for Gap C (statcard value binding) tests in plan 03.
    #[allow(dead_code)]
    fn service_with_money_field() -> ServiceDef {
        // sample_service() already carries a Money field (`price`); this fixture
        // names it explicitly for the Gap C statcard tests.
        ServiceDef::new("statistics")
            .display_name("Statistics")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("total_revenue", DataType::Float, FieldMeaning::Money)
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
            base: BaseContext {
                intent_index: intents
                    .iter()
                    .position(|i| matches!(i.intent, Intent::Browse))
                    .unwrap_or(0),
                ..Default::default()
            },
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
                base: BaseContext {
                    intent_index: idx,
                    ..Default::default()
                },
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
            base: BaseContext {
                intent_index: intents
                    .iter()
                    .position(|i| matches!(i.intent, Intent::Browse))
                    .unwrap_or(0),
                ..Default::default()
            },
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
            base: BaseContext {
                intent_index: intents
                    .iter()
                    .position(|i| matches!(i.intent, Intent::Browse))
                    .unwrap_or(0),
                ..Default::default()
            },
            mode: RenderMode::Display,
            templates: Some(templates),
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
    fn sensitive_field_never_appears_in_display_or_column() {
        // WR-01 regression: a `readable=true` field with `Sensitive` meaning
        // (e.g. password hash) must not appear in a DataTable column or a
        // DescriptionList item — even though `is_system_field` returns false
        // for it. The gate is `lookup_meaning(&meaning).column.is_some()` /
        // `.display.is_some()` in the respective slot emitters.
        let service = ServiceDef::new("user")
            .display_name("User")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("password_hash", DataType::String, FieldMeaning::Sensitive);
        let intents = derive_intents(&service);
        let cat = clean_catalog();

        // Browse (DataTable) projection must not list `password_hash` as a column.
        let browse_idx = intents
            .iter()
            .position(|i| matches!(i.intent, Intent::Browse))
            .unwrap_or(0);
        let browse_ctx = VisualContext {
            base: BaseContext {
                intent_index: browse_idx,
                ..Default::default()
            },
            mode: RenderMode::Display,
            ..Default::default()
        };
        let browse_spec =
            Spec::from_service_def_with_catalog(&service, &intents, &browse_ctx, &cat).unwrap();
        let browse_root = browse_spec.elements.get(&browse_spec.root).unwrap();
        let columns = browse_root
            .props
            .get("columns")
            .and_then(|c| c.as_array())
            .expect("columns array present");
        let column_keys: Vec<&str> = columns
            .iter()
            .filter_map(|c| c.get("key").and_then(|k| k.as_str()))
            .collect();
        assert!(
            !column_keys.contains(&"password_hash"),
            "Sensitive field leaked into DataTable columns: {column_keys:?}"
        );

        // Focus (Card with DescriptionList) projection must not list
        // `password_hash` as a DescriptionItem.
        let focus_idx = intents
            .iter()
            .position(|i| matches!(i.intent, Intent::Focus))
            .unwrap_or(0);
        let focus_ctx = VisualContext {
            base: BaseContext {
                intent_index: focus_idx,
                ..Default::default()
            },
            mode: RenderMode::Display,
            ..Default::default()
        };
        let focus_spec =
            Spec::from_service_def_with_catalog(&service, &intents, &focus_ctx, &cat).unwrap();
        // Look for a DescriptionList element; confirm no item labels match
        // the Sensitive field's display name.
        let leaked = focus_spec.elements.values().any(|el| {
            el.type_name == "DescriptionList"
                && el
                    .props
                    .get("items")
                    .and_then(|i| i.as_array())
                    .map(|arr| {
                        arr.iter().any(|item| {
                            item.get("label")
                                .and_then(|l| l.as_str())
                                .map(|s| s.to_lowercase().contains("password"))
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
        });
        assert!(
            !leaked,
            "Sensitive field leaked into a DescriptionList item"
        );
    }

    #[test]
    fn input_props_for_unknown_type_returns_unknown_component_error() {
        // WR-04 regression: input_props_for must refuse unknown type names
        // rather than silently coercing them to InputProps. This is the
        // choke point that forces the dispatch table to be updated when a
        // new input component is added to `component_map.rs`.
        let field = ferro_projections::FieldDef {
            name: "x".into(),
            data_type: DataType::String,
            meaning: FieldMeaning::Email,
            required: false,
            is_list: false,
            readable: true,
            writable: true,
            render_hint: None,
        };
        let result = super::input_props_for("DatePicker", &field);
        match result {
            Err(ProjectionError::UnknownComponent { type_name }) => {
                assert_eq!(type_name, "DatePicker");
            }
            other => panic!("expected UnknownComponent, got {other:?}"),
        }
    }

    #[test]
    fn statcard_metadata_is_orphan_element() {
        // WR-03 contract pin: Summarize → StatCard with a `metadata` slot
        // emits a DescriptionList into `spec.elements` that is deliberately
        // NOT reachable from the root via `children`. StatCard's catalog
        // shape forbids children, so the metadata lives as a sibling that
        // validates but is not rendered as a child of the root. This test
        // fails loudly if a future refactor either (a) stops emitting the
        // element or (b) starts wiring it into the root — both require
        // updating the documented contract in `emit_statcard_root`.
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
            base: BaseContext {
                intent_index: intents
                    .iter()
                    .position(|i| matches!(i.intent, Intent::Browse))
                    .unwrap_or(0),
                ..Default::default()
            },
            mode: RenderMode::Display,
            templates: Some(templates),
        };
        let cat = clean_catalog();
        let spec = Spec::from_service_def_with_catalog(&service, &intents, &ctx, &cat)
            .expect("StatCard+metadata projects");
        // `metadata_list` is present as a sibling of the root.
        assert!(
            spec.elements.contains_key("metadata_list"),
            "metadata DescriptionList must be emitted as a sibling element"
        );
        // …but it is NOT referenced from the root's children.
        let root = spec.elements.get(&spec.root).unwrap();
        assert_eq!(root.type_name, "StatCard");
        assert!(
            !root.children.contains(&"metadata_list".to_string()),
            "StatCard root must not claim metadata_list as a child: {:?}",
            root.children
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
            base: BaseContext {
                intent_index: intents.len() + 5,
                ..Default::default()
            },
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

    // -- Gap A render tests (TDD RED added in Plan 02 Task 1; GREEN wired below) --

    #[test]
    fn kanban_root_derives_columns_from_state_machine() {
        use crate::component::KanbanBoardProps;
        let service = service_with_state_machine();
        let ctx = VisualContext::default();
        let el = emit_kanban_root(&service, &ctx);
        let built = el.build();
        let props: KanbanBoardProps =
            serde_json::from_value(built.props).expect("props decode as KanbanBoardProps");
        // Lane structure: one column per state, in declaration order.
        assert_eq!(props.columns.len(), 3);
        assert_eq!(props.columns[0].id, "draft");
        assert_eq!(props.columns[0].title, "Draft");
        assert_eq!(props.columns[1].id, "submitted");
        assert_eq!(props.columns[1].title, "Submitted");
        assert_eq!(props.columns[2].id, "done");
        assert_eq!(props.columns[2].title, "Done");
        // Content bindings: flat array path + status grouping field. The
        // fixture has no EntityName, so the card title falls back to the
        // identifier; no Money field, so no description binding.
        assert_eq!(props.items_path.as_deref(), Some("/data/order"));
        assert_eq!(props.group_by.as_deref(), Some("status"));
        assert_eq!(props.card_title_key.as_deref(), Some("id"));
        assert!(props.card_description_key.is_none());
    }

    #[test]
    fn kanban_root_fallback_when_no_state_machine() {
        use crate::component::KanbanBoardProps;
        let service = sample_service(); // no state machine
        let ctx = VisualContext::default();
        let el = emit_kanban_root(&service, &ctx);
        let built = el.build();
        let props: KanbanBoardProps =
            serde_json::from_value(built.props).expect("props decode as KanbanBoardProps");
        assert_eq!(props.columns.len(), 1);
        assert!(props.items_path.is_none());
        assert!(props.group_by.is_none());
    }

    // -- Gap B render tests (TDD RED added in Task 1; GREEN wired in Task 2) --

    #[test]
    fn actions_slot_emits_action_group_from_service_actions() {
        use crate::component::ActionGroupProps;
        let service = service_with_actions();
        let mut aux: Vec<(String, ElementBuilder)> = Vec::new();
        let mut children: Vec<String> = Vec::new();
        emit_actions_placeholder(&service, &mut aux, &mut children);
        assert_eq!(children, vec!["actions_menu".to_string()]);
        let pos = aux
            .iter()
            .position(|(id, _)| id == "actions_menu")
            .expect("ActionGroup must be emitted");
        let (_, el) = aux.remove(pos);
        let built = el.build();
        let props: ActionGroupProps =
            serde_json::from_value(built.props).expect("props decode as ActionGroupProps");
        assert_eq!(props.items.len(), service.actions.len());
        assert_eq!(props.items[0].label, "View");
    }

    #[test]
    fn datatable_root_has_row_actions_from_service_actions() {
        use crate::component::DataTableProps;
        let service = service_with_actions();
        let el = emit_datatable_root(&service);
        let built = el.build();
        let props: DataTableProps =
            serde_json::from_value(built.props).expect("props decode as DataTableProps");
        let ra = props.row_actions.expect("row_actions must be populated");
        assert_eq!(ra.len(), service.actions.len());
    }

    // -- Gap C render tests (TDD RED added here; GREEN wired in Task 2) --

    #[test]
    fn statcard_root_binds_primary_stat_field() {
        // Gap C: emit_statcard_root must bind value_path to the primary
        // Money/Quantity readable field path.
        use crate::component::StatCardProps;
        let service = service_with_money_field();
        let mut aux: Vec<(String, ElementBuilder)> = Vec::new();
        let el = emit_statcard_root(&service, &[], &mut aux);
        let built = el.build();
        let props: StatCardProps =
            serde_json::from_value(built.props).expect("props decode as StatCardProps");
        assert_eq!(
            props.value_path.as_deref(),
            Some("/data/statistics/total_revenue"),
            "value_path must bind to the primary Money field path"
        );
    }

    #[test]
    fn statcard_root_empty_when_no_stat_field() {
        // Gap C: a service with no Money/Quantity readable field must emit
        // a StatCard with value_path None (no data binding).
        use crate::component::StatCardProps;
        let service = ServiceDef::new("note")
            .display_name("Note")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("body", DataType::String, FieldMeaning::FreeText);
        let mut aux: Vec<(String, ElementBuilder)> = Vec::new();
        let el = emit_statcard_root(&service, &[], &mut aux);
        let built = el.build();
        let props: StatCardProps =
            serde_json::from_value(built.props).expect("props decode as StatCardProps");
        assert!(
            props.value_path.is_none(),
            "value_path must be None when no Money/Quantity field exists"
        );
    }

    // -- Gap D render tests (TDD RED added in Plan 04 Task 1; GREEN wired below) --

    #[test]
    fn datatable_root_includes_image_url_column() {
        // Gap D: an ImageUrl field must appear as a DataTable column (was excluded).
        use crate::component::DataTableProps;
        let service = ServiceDef::new("staff")
            .display_name("Staff")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("avatar_url", DataType::String, FieldMeaning::ImageUrl);
        let el = emit_datatable_root(&service);
        let built = el.build();
        let props: DataTableProps =
            serde_json::from_value(built.props).expect("props decode as DataTableProps");
        assert!(
            props.columns.iter().any(|c| c.key == "avatar_url"),
            "avatar_url column must appear in DataTable columns; got: {:?}",
            props.columns.iter().map(|c| &c.key).collect::<Vec<_>>()
        );
    }

    #[test]
    fn image_column_has_image_format() {
        // Gap D: the ImageUrl column must carry ColumnFormat::Image.
        use crate::component::{ColumnFormat, DataTableProps};
        let service = ServiceDef::new("staff")
            .display_name("Staff")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("avatar_url", DataType::String, FieldMeaning::ImageUrl);
        let el = emit_datatable_root(&service);
        let built = el.build();
        let props: DataTableProps =
            serde_json::from_value(built.props).expect("props decode as DataTableProps");
        let col = props
            .columns
            .iter()
            .find(|c| c.key == "avatar_url")
            .expect("avatar_url column must exist");
        assert_eq!(
            col.format,
            Some(ColumnFormat::Image),
            "ImageUrl column format must be Image"
        );
    }
}
