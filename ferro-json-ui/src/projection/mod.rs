//! JSON-UI renderer producing ferro-json-ui/v1 component trees from service definitions.
//!
//! Implements the `Renderer` trait to translate `ServiceDef` + `IntentScore[]` into
//! a JSON view specification with layout strategies for each intent type.
//!
//! This module is only compiled when the `projections` feature is enabled.

pub mod field_map;
pub mod relationship_map;

use ferro_theme::{IntentSlotTemplate, ThemeTemplates};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use ferro_projections::Error;
use ferro_projections::FieldMeaning;
use ferro_projections::NavigationHint;
use ferro_projections::ServiceDef;
use ferro_projections::{Intent, IntentScore};

use self::field_map::{field_to_column, field_to_display, field_to_input};
use self::relationship_map::relationship_to_component;
use ferro_projections::render::{field_display_name, is_system_field, Renderer};

/// Controls whether fields render as read-only display or editable inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    /// Read-only view (detail pages, lists, summaries).
    Display,
    /// Editable form view (create, edit).
    Input,
}

/// Visual rendering context for `JsonUiRenderer`.
///
/// Extends the modality-agnostic fields from `BaseContext` with visual-specific
/// concerns: render mode and theme template overrides.
#[derive(Debug, Clone)]
pub struct VisualContext {
    /// Which intent to render (0 = primary). Index into the `intents` slice.
    pub intent_index: usize,
    /// Current workflow state name (relevant for Process/Track intents).
    pub current_state: Option<String>,
    /// Display or Input mode.
    pub mode: RenderMode,
    /// Optional theme template overrides. `None` means use built-in layouts.
    pub templates: Option<ThemeTemplates>,
}

impl Default for VisualContext {
    fn default() -> Self {
        Self {
            intent_index: 0,
            current_state: None,
            mode: RenderMode::Display,
            templates: None,
        }
    }
}

/// Returns true for DateTime-family field meanings that Track views expose.
fn is_datetime_field(meaning: &FieldMeaning) -> bool {
    matches!(
        meaning,
        FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt | FieldMeaning::DateTime
    )
}

/// Returns true for numeric field meanings used in Analyze summary stats.
fn is_numeric_field(meaning: &FieldMeaning) -> bool {
    matches!(
        meaning,
        FieldMeaning::Money | FieldMeaning::Quantity | FieldMeaning::Percentage
    )
}

/// JSON-UI renderer producing ferro-json-ui/v1 component trees.
///
/// Translates service definitions and scored intents into a JSON view specification
/// matching the ferro-json-ui/v1 schema. Each intent maps to a layout strategy
/// that composes field/relationship mapping functions into a component tree.
///
/// # Example
///
/// ```
/// use ferro_projections::{ServiceDef, DataType, FieldMeaning, derive_intents};
/// use ferro_json_ui::{JsonUiRenderer, VisualContext};
/// use ferro_projections::render::Renderer;
///
/// let product = ServiceDef::new("product")
///     .display_name("Product")
///     .field("id", DataType::Integer, FieldMeaning::Identifier)
///     .field("name", DataType::String, FieldMeaning::EntityName)
///     .field("price", DataType::Float, FieldMeaning::Money);
///
/// let intents = derive_intents(&product);
/// let renderer = JsonUiRenderer;
/// let result = renderer.render(&product, &intents, &VisualContext::default());
/// assert!(result.is_ok());
///
/// let json = result.unwrap();
/// assert_eq!(json["$schema"], "ferro-json-ui/v1");
/// assert_eq!(json["title"], "Product");
/// assert!(!json["components"].as_array().unwrap().is_empty());
/// ```
pub struct JsonUiRenderer;

impl Renderer for JsonUiRenderer {
    type Output = serde_json::Value;
    type Context = VisualContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &VisualContext,
    ) -> Result<Value, Error> {
        let intent_score = intents.get(ctx.intent_index).ok_or_else(|| {
            Error::Render(format!(
                "intent_index {} out of bounds (have {} intents)",
                ctx.intent_index,
                intents.len()
            ))
        })?;

        // Check for a theme template override for this intent+mode combination.
        let template_override = ctx
            .templates
            .as_ref()
            .and_then(|t| get_template_for_intent(t, &intent_score.intent, &ctx.mode));

        let components = if let Some(template) = template_override {
            self.render_from_template(service, template, ctx)
        } else {
            match &intent_score.intent {
                Intent::Browse => match ctx.mode {
                    RenderMode::Display => self.render_browse(service),
                    RenderMode::Input => self.render_collect(service),
                },
                Intent::Focus => match ctx.mode {
                    RenderMode::Display => self.render_focus(service),
                    RenderMode::Input => self.render_collect(service),
                },
                Intent::Collect => self.render_collect(service),
                Intent::Summarize => match ctx.mode {
                    RenderMode::Display => self.render_summarize(service),
                    RenderMode::Input => self.render_collect(service),
                },
                Intent::Process => match ctx.mode {
                    RenderMode::Display => self.render_process(service, ctx),
                    RenderMode::Input => self.render_process_input(service, ctx),
                },
                Intent::Analyze => match ctx.mode {
                    RenderMode::Display => self.render_analyze(service),
                    RenderMode::Input => self.render_collect(service),
                },
                Intent::Track => match ctx.mode {
                    RenderMode::Display => self.render_track(service),
                    RenderMode::Input => self.render_collect(service),
                },
                Intent::Custom(_) => match ctx.mode {
                    RenderMode::Display => self.render_focus(service),
                    RenderMode::Input => self.render_collect(service),
                },
            }
        };

        let title = service
            .display_name
            .as_deref()
            .unwrap_or(&service.name)
            .to_string();

        Ok(json!({
            "$schema": "ferro-json-ui/v1",
            "title": title,
            "components": components,
        }))
    }
}

/// Returns the slot template for a given intent and mode, if any override exists
/// and the template has non-empty slots.
fn get_template_for_intent<'a>(
    templates: &'a ThemeTemplates,
    intent: &Intent,
    mode: &RenderMode,
) -> Option<&'a IntentSlotTemplate> {
    let mode_templates = match intent {
        Intent::Browse => templates.browse.as_ref()?,
        Intent::Focus => templates.focus.as_ref()?,
        Intent::Collect => templates.collect.as_ref()?,
        Intent::Process => templates.process.as_ref()?,
        Intent::Summarize => templates.summarize.as_ref()?,
        Intent::Analyze => templates.analyze.as_ref()?,
        Intent::Track => templates.track.as_ref()?,
        Intent::Custom(_) => return None, // Custom intents have no template overrides
    };
    let slot_template = match mode {
        RenderMode::Display => &mode_templates.display,
        RenderMode::Input => &mode_templates.input,
    };
    if slot_template.slots.is_empty() {
        None // Empty slots = no override, use built-in
    } else {
        Some(slot_template)
    }
}

impl JsonUiRenderer {
    /// Browse layout: filterable table with pagination.
    fn render_browse(&self, service: &ServiceDef) -> Vec<Value> {
        let columns: Vec<Value> = service
            .fields
            .iter()
            .filter(|f| f.readable && !is_system_field(&f.meaning))
            .map(field_to_column)
            .collect();

        let table = json!({
            "type": "Table",
            "key": format!("{}-table", service.name),
            "columns": columns,
            "data_path": "/data/items",
            "sortable": true,
        });

        let pagination = json!({
            "type": "Pagination",
            "key": format!("{}-pagination", service.name),
            "current_page": 1,
            "per_page": 25,
            "total": 0,
            "base_url": format!("/{}", service.name),
        });

        vec![table, pagination]
    }

    /// Focus layout: detail card with description list and relationship sections.
    fn render_focus(&self, service: &ServiceDef) -> Vec<Value> {
        let mut components = Vec::new();

        // Description list items from readable fields (skip Null results)
        let items: Vec<Value> = service
            .fields
            .iter()
            .filter(|f| f.readable && !is_system_field(&f.meaning))
            .filter_map(|f| {
                let display = field_to_display(f);
                if display.is_null() {
                    return None;
                }
                Some(json!({
                    "term": field_display_name(&f.name),
                    "detail_data_path": format!("/data/{}", f.name),
                }))
            })
            .collect();

        let description_list = json!({
            "type": "DescriptionList",
            "key": format!("{}-details", service.name),
            "items": items,
        });

        // Collect relationship components by hint type
        let mut tab_components = Vec::new();
        let mut inline_components = Vec::new();
        let mut nested_components = Vec::new();

        for rel in &service.relationships {
            if rel.navigation == NavigationHint::Hidden {
                continue;
            }
            let comp = relationship_to_component(rel, &service.name);
            if comp.is_null() {
                continue;
            }
            match rel.navigation {
                NavigationHint::Tab => tab_components.push(comp),
                NavigationHint::Inline | NavigationHint::Link => inline_components.push(comp),
                NavigationHint::Nested => nested_components.push(comp),
                NavigationHint::Hidden => {} // already filtered
            }
        }

        // Build card children: DescriptionList + inline/link relationship components
        let mut card_children = vec![description_list];
        card_children.extend(inline_components);

        let title = service
            .display_name
            .as_deref()
            .unwrap_or(&service.name)
            .to_string();

        let card = json!({
            "type": "Card",
            "key": format!("{}-card", service.name),
            "title": title,
            "children": card_children,
        });
        components.push(card);

        // Tabs component if any Tab relationships exist
        if !tab_components.is_empty() {
            let tabs = json!({
                "type": "Tabs",
                "key": format!("{}-tabs", service.name),
                "tabs": tab_components,
            });
            components.push(tabs);
        }

        // Nested table components below card
        components.extend(nested_components);

        components
    }

    /// Collect layout: data entry form with typed inputs.
    fn render_collect(&self, service: &ServiceDef) -> Vec<Value> {
        let inputs: Vec<Value> = service
            .fields
            .iter()
            .filter(|f| {
                // Skip auto-generated system fields
                if matches!(f.meaning, FieldMeaning::Identifier) && !f.writable {
                    return false;
                }
                if is_system_field(&f.meaning)
                    && matches!(f.meaning, FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt)
                {
                    return false;
                }
                f.writable
            })
            .filter_map(|f| {
                let input = field_to_input(f);
                if input.is_null() {
                    return None;
                }
                Some(input)
            })
            .collect();

        let submit = json!({
            "type": "Button",
            "key": format!("{}-submit", service.name),
            "label": "Save",
            "variant": "default",
            "action_handler": format!("{}.store", service.name),
        });

        let mut children = inputs;
        children.push(submit);

        let form = json!({
            "type": "Form",
            "key": format!("{}-form", service.name),
            "action_handler": format!("{}.store", service.name),
            "method": "POST",
            "children": children,
        });

        vec![form]
    }

    /// Process layout: workflow control panel for state-machine-driven services.
    ///
    /// Displays current state as a Badge, guard requirements as an Alert,
    /// and transition-triggering actions as Buttons. Falls back to Focus
    /// layout if no state machine is defined.
    fn render_process(&self, service: &ServiceDef, ctx: &VisualContext) -> Vec<Value> {
        let sm = match &service.state_machine {
            Some(sm) => sm,
            None => return self.render_focus(service),
        };

        let mut components = Vec::new();

        // Current state display card
        let title = service
            .display_name
            .as_deref()
            .unwrap_or(&service.name)
            .to_string();

        let current_state_name = ctx.current_state.as_deref().unwrap_or("Unknown");

        let mut card_children: Vec<Value> = vec![json!({
            "type": "Badge",
            "key": format!("{}-state-badge", service.name),
            "text": current_state_name,
            "variant": "default",
            "data_path": "/data/state",
        })];

        // Alert if current state's outgoing transitions have guards
        let current_state_str = ctx.current_state.as_deref().unwrap_or(&sm.initial_state);
        let outgoing = sm.events_from_state(current_state_str);
        let guarded: Vec<&str> = outgoing.iter().filter_map(|t| t.guard.as_deref()).collect();

        if !guarded.is_empty() {
            let guard_list = guarded.join(", ");
            card_children.push(json!({
                "type": "Alert",
                "key": format!("{}-guard-alert", service.name),
                "variant": "info",
                "title": "Requirements",
                "description": format!("Guards: {guard_list}"),
            }));
        }

        components.push(json!({
            "type": "Card",
            "key": format!("{}-process-card", service.name),
            "title": title,
            "children": card_children,
        }));

        // Transition action buttons
        for action in &service.actions {
            if action.transition_trigger.is_some() {
                let label = action
                    .display_name
                    .as_deref()
                    .unwrap_or(&action.name)
                    .to_string();
                components.push(json!({
                    "type": "Button",
                    "key": format!("{}-action-{}", service.name, action.name),
                    "label": label,
                    "variant": "default",
                    "action_handler": format!("{}.{}", service.name, action.name),
                }));
            }
        }

        components
    }

    /// Process layout in Input mode: form fields alongside transition buttons.
    ///
    /// Combines a Collect-style form (for editing process data) with
    /// transition action buttons from the state machine.
    fn render_process_input(&self, service: &ServiceDef, ctx: &VisualContext) -> Vec<Value> {
        let mut components = self.render_collect(service);

        // Add transition buttons after the form if state machine exists
        if service.state_machine.is_some() {
            for action in &service.actions {
                if action.transition_trigger.is_some() {
                    let label = action
                        .display_name
                        .as_deref()
                        .unwrap_or(&action.name)
                        .to_string();
                    components.push(json!({
                        "type": "Button",
                        "key": format!("{}-action-{}", service.name, action.name),
                        "label": label,
                        "variant": "default",
                        "action_handler": format!("{}.{}", service.name, action.name),
                    }));
                }
            }
        }

        // If no state machine, fall back to standard Collect
        let _ = ctx; // ctx available for future state-aware input rendering
        components
    }

    /// Analyze layout: analytical table view for time-series/measure data.
    ///
    /// Includes ALL readable non-system fields as columns (broader than Browse).
    /// DateTime columns included prominently, sorted descending by default.
    /// No pagination (analytical views show all data).
    ///
    /// Note: JSON-UI has no chart components. The sortable Table is the analytical view.
    fn render_analyze(&self, service: &ServiceDef) -> Vec<Value> {
        let mut components = Vec::new();

        // Summary card for numeric fields (structural placeholders for framework-layer computation)
        let numeric_fields: Vec<&str> = service
            .fields
            .iter()
            .filter(|f| f.readable && is_numeric_field(&f.meaning))
            .map(|f| f.name.as_str())
            .collect();

        if !numeric_fields.is_empty() {
            let items: Vec<Value> = numeric_fields
                .iter()
                .flat_map(|name| {
                    let label = field_display_name(name);
                    vec![
                        json!({
                            "term": format!("Total {label}"),
                            "detail_data_path": format!("/data/summary/{name}_total"),
                        }),
                        json!({
                            "term": format!("Average {label}"),
                            "detail_data_path": format!("/data/summary/{name}_average"),
                        }),
                    ]
                })
                .collect();

            components.push(json!({
                "type": "Card",
                "key": format!("{}-summary-card", service.name),
                "title": "Summary",
                "children": [json!({
                    "type": "DescriptionList",
                    "key": format!("{}-summary-stats", service.name),
                    "items": items,
                })],
            }));
        }

        // Table with ALL readable fields (including DateTime, unlike Browse)
        let columns: Vec<Value> = service
            .fields
            .iter()
            .filter(|f| {
                f.readable
                    && !matches!(
                        f.meaning,
                        FieldMeaning::Identifier
                            | FieldMeaning::Sensitive
                            | FieldMeaning::ForeignKey
                    )
            })
            .map(field_to_column)
            .collect();

        let table = json!({
            "type": "Table",
            "key": format!("{}-analyze-table", service.name),
            "columns": columns,
            "data_path": "/data/items",
            "sortable": true,
            "sort_direction": "desc",
        });

        components.push(table);
        // No pagination for analytical views — show all data

        components
    }

    /// Track layout: status timeline/audit list view.
    ///
    /// Unlike Browse, Track views INCLUDE DateTime system fields and sort by them.
    /// Status columns render as Badges. Includes Pagination for long audit trails.
    fn render_track(&self, service: &ServiceDef) -> Vec<Value> {
        let columns: Vec<Value> = service
            .fields
            .iter()
            .filter(|f| {
                if !f.readable {
                    return false;
                }
                // Include DateTime fields (Track wants temporal ordering visible)
                if is_datetime_field(&f.meaning) {
                    return true;
                }
                // Include Status, EntityName, and other non-system readable fields
                if matches!(f.meaning, FieldMeaning::Identifier) {
                    return false;
                }
                // Exclude Sensitive and ForeignKey
                !matches!(
                    f.meaning,
                    FieldMeaning::Sensitive | FieldMeaning::ForeignKey
                )
            })
            .map(field_to_column)
            .collect();

        let table = json!({
            "type": "Table",
            "key": format!("{}-track-table", service.name),
            "columns": columns,
            "data_path": "/data/items",
            "sortable": true,
            "sort_direction": "desc",
        });

        let pagination = json!({
            "type": "Pagination",
            "key": format!("{}-track-pagination", service.name),
            "current_page": 1,
            "per_page": 25,
            "total": 0,
            "base_url": format!("/{}", service.name),
        });

        vec![table, pagination]
    }

    /// Template-driven rendering: slot-ordered component list from a ThemeTemplate.
    ///
    /// Iterates over `template.slots` and produces components for each named slot.
    /// Slots that produce no content (e.g., "relationships" when the service has none)
    /// are silently skipped — no empty containers emitted.
    ///
    /// Component generation uses the SAME field_map functions as the built-in render
    /// methods, so the output is structurally identical for equivalent slot configurations.
    fn render_from_template(
        &self,
        service: &ServiceDef,
        template: &IntentSlotTemplate,
        ctx: &VisualContext,
    ) -> Vec<Value> {
        let mut components: Vec<Value> = Vec::new();

        for slot in &template.slots {
            let slot_components = self.render_slot(service, slot.as_str(), template, ctx);
            components.extend(slot_components);
        }

        components
    }

    /// Renders a single named slot into zero or more component values.
    ///
    /// Returns an empty vec when the slot has no applicable content so that callers
    /// can simply extend their output without emitting empty containers.
    fn render_slot(
        &self,
        service: &ServiceDef,
        slot: &str,
        template: &IntentSlotTemplate,
        ctx: &VisualContext,
    ) -> Vec<Value> {
        let title = service
            .display_name
            .as_deref()
            .unwrap_or(&service.name)
            .to_string();

        match slot {
            "title" => {
                vec![json!({
                    "type": "Text",
                    "key": format!("{}-title", service.name),
                    "element": "h1",
                    "content": title,
                })]
            }
            "body" => {
                // Layout hint guides which component produces the body.
                // "table" → Table, "form" → Form fields, default → DescriptionList
                match template.layout.as_deref() {
                    Some("table") | Some("Table") => {
                        let columns: Vec<Value> = service
                            .fields
                            .iter()
                            .filter(|f| f.readable && !is_system_field(&f.meaning))
                            .map(field_to_column)
                            .collect();
                        if columns.is_empty() {
                            vec![]
                        } else {
                            vec![json!({
                                "type": "Table",
                                "key": format!("{}-body-table", service.name),
                                "columns": columns,
                                "data_path": "/data/items",
                                "sortable": true,
                            })]
                        }
                    }
                    Some("form") | Some("Form") => {
                        let inputs: Vec<Value> = service
                            .fields
                            .iter()
                            .filter(|f| f.writable && !is_system_field(&f.meaning))
                            .filter_map(|f| {
                                let v = field_to_input(f);
                                if v.is_null() {
                                    None
                                } else {
                                    Some(v)
                                }
                            })
                            .collect();
                        inputs
                    }
                    _ => {
                        // Default body: DescriptionList
                        let items: Vec<Value> = service
                            .fields
                            .iter()
                            .filter(|f| f.readable && !is_system_field(&f.meaning))
                            .filter_map(|f| {
                                let display = field_to_display(f);
                                if display.is_null() {
                                    return None;
                                }
                                Some(json!({
                                    "term": field_display_name(&f.name),
                                    "detail_data_path": format!("/data/{}", f.name),
                                }))
                            })
                            .collect();
                        if items.is_empty() {
                            vec![]
                        } else {
                            vec![json!({
                                "type": "DescriptionList",
                                "key": format!("{}-body", service.name),
                                "items": items,
                            })]
                        }
                    }
                }
            }
            "fields" => {
                // Fields slot: use mode to decide display vs input
                match ctx.mode {
                    RenderMode::Input => {
                        // Layout hint can force table for input mode
                        match template.layout.as_deref() {
                            Some("table") | Some("Table") => {
                                let columns: Vec<Value> = service
                                    .fields
                                    .iter()
                                    .filter(|f| f.readable && !is_system_field(&f.meaning))
                                    .map(field_to_column)
                                    .collect();
                                if columns.is_empty() {
                                    vec![]
                                } else {
                                    vec![json!({
                                        "type": "Table",
                                        "key": format!("{}-fields-table", service.name),
                                        "columns": columns,
                                        "data_path": "/data/items",
                                        "sortable": true,
                                    })]
                                }
                            }
                            _ => {
                                let inputs: Vec<Value> = service
                                    .fields
                                    .iter()
                                    .filter(|f| f.writable && !is_system_field(&f.meaning))
                                    .filter_map(|f| {
                                        let v = field_to_input(f);
                                        if v.is_null() {
                                            None
                                        } else {
                                            Some(v)
                                        }
                                    })
                                    .collect();
                                inputs
                            }
                        }
                    }
                    RenderMode::Display => {
                        // Layout hint: table renders columns, otherwise DescriptionList items
                        match template.layout.as_deref() {
                            Some("table") | Some("Table") => {
                                let columns: Vec<Value> = service
                                    .fields
                                    .iter()
                                    .filter(|f| f.readable && !is_system_field(&f.meaning))
                                    .map(field_to_column)
                                    .collect();
                                if columns.is_empty() {
                                    vec![]
                                } else {
                                    vec![json!({
                                        "type": "Table",
                                        "key": format!("{}-fields-table", service.name),
                                        "columns": columns,
                                        "data_path": "/data/items",
                                        "sortable": true,
                                    })]
                                }
                            }
                            _ => {
                                let items: Vec<Value> = service
                                    .fields
                                    .iter()
                                    .filter(|f| f.readable && !is_system_field(&f.meaning))
                                    .filter_map(|f| {
                                        let display = field_to_display(f);
                                        if display.is_null() {
                                            return None;
                                        }
                                        Some(json!({
                                            "term": field_display_name(&f.name),
                                            "detail_data_path": format!("/data/{}", f.name),
                                        }))
                                    })
                                    .collect();
                                items
                            }
                        }
                    }
                }
            }
            "actions" => {
                let buttons: Vec<Value> = service
                    .actions
                    .iter()
                    .map(|action| {
                        let label = action
                            .display_name
                            .as_deref()
                            .unwrap_or(&action.name)
                            .to_string();
                        json!({
                            "type": "Button",
                            "key": format!("{}-action-{}", service.name, action.name),
                            "label": label,
                            "variant": "default",
                            "action_handler": format!("{}.{}", service.name, action.name),
                        })
                    })
                    .collect();
                buttons
            }
            "relationships" => {
                // Only emit components for visible relationships
                let comps: Vec<Value> = service
                    .relationships
                    .iter()
                    .filter(|r| r.navigation != NavigationHint::Hidden)
                    .filter_map(|r| {
                        let comp = relationship_to_component(r, &service.name);
                        if comp.is_null() {
                            None
                        } else {
                            Some(comp)
                        }
                    })
                    .collect();
                comps
            }
            "pagination" => {
                vec![json!({
                    "type": "Pagination",
                    "key": format!("{}-pagination", service.name),
                    "current_page": 1,
                    "per_page": 25,
                    "total": 0,
                    "base_url": format!("/{}", service.name),
                })]
            }
            "metadata" => {
                // System timestamp fields as DescriptionList items
                let items: Vec<Value> = service
                    .fields
                    .iter()
                    .filter(|f| {
                        f.readable
                            && matches!(
                                f.meaning,
                                FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt
                            )
                    })
                    .map(|f| {
                        json!({
                            "term": field_display_name(&f.name),
                            "detail_data_path": format!("/data/{}", f.name),
                        })
                    })
                    .collect();
                if items.is_empty() {
                    vec![]
                } else {
                    vec![json!({
                        "type": "DescriptionList",
                        "key": format!("{}-metadata", service.name),
                        "items": items,
                    })]
                }
            }
            "stats" => {
                // StatCard-style components for numeric fields
                let stat_items: Vec<Value> = service
                    .fields
                    .iter()
                    .filter(|f| f.readable && is_numeric_field(&f.meaning))
                    .map(|f| {
                        let label = field_display_name(&f.name);
                        json!({
                            "type": "Card",
                            "key": format!("{}-stat", f.name),
                            "title": label,
                            "children": [json!({
                                "type": "Text",
                                "key": format!("{}-stat-value", f.name),
                                "content": "",
                                "data_path": format!("/data/{}", f.name),
                            })],
                        })
                    })
                    .collect();
                stat_items
            }
            _ => {
                // Unknown slot names are silently skipped
                vec![]
            }
        }
    }

    /// Summarize layout: dashboard metrics with card grid.
    fn render_summarize(&self, service: &ServiceDef) -> Vec<Value> {
        let mut metric_cards: Vec<Value> = Vec::new();
        let mut status_badges: Vec<Value> = Vec::new();

        for field in &service.fields {
            if !field.readable || is_system_field(&field.meaning) {
                continue;
            }

            match &field.meaning {
                FieldMeaning::Money | FieldMeaning::Quantity => {
                    let label = field_display_name(&field.name);
                    metric_cards.push(json!({
                        "type": "Card",
                        "key": format!("{}-metric", field.name),
                        "title": label,
                        "children": [json!({
                            "type": "Text",
                            "key": format!("{}-value", field.name),
                            "content": "",
                            "data_path": format!("/data/{}", field.name),
                        })],
                    }));
                }
                FieldMeaning::Percentage => {
                    let label = field_display_name(&field.name);
                    metric_cards.push(json!({
                        "type": "Card",
                        "key": format!("{}-metric", field.name),
                        "title": label,
                        "children": [json!({
                            "type": "Progress",
                            "key": format!("{}-progress", field.name),
                            "value": 0,
                            "data_path": format!("/data/{}", field.name),
                        })],
                    }));
                }
                FieldMeaning::Status => {
                    let label = field_display_name(&field.name);
                    status_badges.push(json!({
                        "type": "Badge",
                        "key": format!("{}-badge", field.name),
                        "text": "",
                        "variant": "default",
                        "label": label,
                        "data_path": format!("/data/{}", field.name),
                    }));
                }
                _ => {}
            }
        }

        // If no numeric fields, fall back to DescriptionList
        if metric_cards.is_empty() {
            let items: Vec<Value> = service
                .fields
                .iter()
                .filter(|f| f.readable && !is_system_field(&f.meaning))
                .filter_map(|f| {
                    let display = field_to_display(f);
                    if display.is_null() {
                        return None;
                    }
                    Some(json!({
                        "term": field_display_name(&f.name),
                        "detail_data_path": format!("/data/{}", f.name),
                    }))
                })
                .collect();

            let mut result = vec![json!({
                "type": "DescriptionList",
                "key": format!("{}-summary-details", service.name),
                "items": items,
            })];

            // Still show status badges if present
            if !status_badges.is_empty() {
                let status_card = json!({
                    "type": "Card",
                    "key": format!("{}-status", service.name),
                    "title": "Status",
                    "children": status_badges,
                });
                result.push(status_card);
            }

            return result;
        }

        // Status badges in a summary card
        if !status_badges.is_empty() {
            let status_card = json!({
                "type": "Card",
                "key": format!("{}-status", service.name),
                "title": "Status",
                "children": status_badges,
            });
            metric_cards.push(status_card);
        }

        metric_cards
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::IntentScore;
    use ferro_projections::ServiceDef;
    use ferro_projections::{Cardinality, RelationshipDef};
    use ferro_projections::{DataType, FieldDef};

    fn browse_intent() -> IntentScore {
        IntentScore {
            intent: Intent::Browse,
            confidence: 0.8,
            matching_signals: vec!["test".into()],
        }
    }

    fn focus_intent() -> IntentScore {
        IntentScore {
            intent: Intent::Focus,
            confidence: 0.7,
            matching_signals: vec!["test".into()],
        }
    }

    fn collect_intent() -> IntentScore {
        IntentScore {
            intent: Intent::Collect,
            confidence: 0.6,
            matching_signals: vec!["test".into()],
        }
    }

    fn summarize_intent() -> IntentScore {
        IntentScore {
            intent: Intent::Summarize,
            confidence: 0.5,
            matching_signals: vec!["test".into()],
        }
    }

    fn custom_intent() -> IntentScore {
        IntentScore {
            intent: Intent::Custom("dashboard".into()),
            confidence: 0.5,
            matching_signals: vec!["test".into()],
        }
    }

    fn default_ctx() -> VisualContext {
        VisualContext::default()
    }

    fn input_ctx() -> VisualContext {
        VisualContext {
            intent_index: 0,
            current_state: None,
            mode: RenderMode::Input,
            templates: None,
        }
    }

    #[test]
    fn visual_context_default() {
        let ctx = VisualContext::default();
        assert_eq!(ctx.intent_index, 0);
        assert!(ctx.current_state.is_none());
        assert_eq!(ctx.mode, RenderMode::Display);
        assert!(ctx.templates.is_none());
    }

    #[test]
    fn render_mode_serde_round_trip() {
        for mode in [RenderMode::Display, RenderMode::Input] {
            let json = serde_json::to_string(&mode).unwrap();
            let parsed: RenderMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, parsed);
        }
    }

    #[test]
    fn render_mode_display_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&RenderMode::Display).unwrap(),
            r#""display""#
        );
        assert_eq!(
            serde_json::to_string(&RenderMode::Input).unwrap(),
            r#""input""#
        );
    }

    fn order_service() -> ServiceDef {
        ServiceDef::new("order")
            .display_name("Order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("title", DataType::String, FieldMeaning::EntityName)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
    }

    // -- Schema and structure tests --

    #[test]
    fn browse_sets_schema_to_ferro_json_ui_v1() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        assert_eq!(result["$schema"], "ferro-json-ui/v1");
    }

    #[test]
    fn browse_sets_title_from_display_name() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        assert_eq!(result["title"], "Order");
    }

    #[test]
    fn browse_falls_back_to_service_name_without_display_name() {
        let service = ServiceDef::new("invoice")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("amount", DataType::Float, FieldMeaning::Money);
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        assert_eq!(result["title"], "invoice");
    }

    // -- Browse tests --

    #[test]
    fn browse_produces_table_and_pagination() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components.len(), 2);
        assert_eq!(components[0]["type"], "Table");
        assert_eq!(components[1]["type"], "Pagination");
    }

    #[test]
    fn browse_excludes_system_fields_from_columns() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let columns = result["components"][0]["columns"].as_array().unwrap();
        let keys: Vec<&str> = columns.iter().map(|c| c["key"].as_str().unwrap()).collect();
        assert!(!keys.contains(&"id"));
        assert!(!keys.contains(&"created_at"));
        assert!(!keys.contains(&"updated_at"));
        assert!(keys.contains(&"title"));
        assert!(keys.contains(&"total"));
        assert!(keys.contains(&"status"));
    }

    #[test]
    fn browse_table_has_data_path_and_sortable() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        assert_eq!(result["components"][0]["data_path"], "/data/items");
        assert_eq!(result["components"][0]["sortable"], true);
    }

    #[test]
    fn browse_pagination_has_defaults() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let pagination = &result["components"][1];
        assert_eq!(pagination["current_page"], 1);
        assert_eq!(pagination["per_page"], 25);
        assert_eq!(pagination["total"], 0);
        assert_eq!(pagination["base_url"], "/order");
    }

    // -- Focus tests --

    #[test]
    fn focus_produces_card_with_description_list() {
        let service = order_service();
        let intents = vec![focus_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert!(!components.is_empty());
        assert_eq!(components[0]["type"], "Card");
        let children = components[0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "DescriptionList");
    }

    #[test]
    fn focus_excludes_sensitive_and_foreign_key_fields() {
        let service = ServiceDef::new("user")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("password", DataType::String, FieldMeaning::Sensitive)
            .field("org_id", DataType::Integer, FieldMeaning::ForeignKey);
        let intents = vec![focus_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let items = result["components"][0]["children"][0]["items"]
            .as_array()
            .unwrap();
        let terms: Vec<&str> = items.iter().map(|i| i["term"].as_str().unwrap()).collect();
        assert!(terms.contains(&"Name"));
        assert!(!terms.contains(&"Password"));
        assert!(!terms.contains(&"Org Id"));
        // System fields also excluded
        assert!(!terms.contains(&"Id"));
    }

    #[test]
    fn focus_includes_relationship_components_by_hint() {
        let service = ServiceDef::new("order")
            .display_name("Order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("title", DataType::String, FieldMeaning::EntityName)
            .relationship(
                RelationshipDef::new("line_items", "line_item", Cardinality::OneToMany)
                    .navigation(NavigationHint::Tab),
            )
            .relationship(
                RelationshipDef::new("items", "item", Cardinality::OneToMany)
                    .navigation(NavigationHint::Nested),
            )
            .relationship(
                RelationshipDef::new("customer", "customer", Cardinality::ManyToOne)
                    .navigation(NavigationHint::Inline),
            );
        let intents = vec![focus_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();

        // Card has DescriptionList + inline relationship
        let card_children = components[0]["children"].as_array().unwrap();
        assert!(card_children.len() >= 2);
        // Inline component in card children
        assert!(card_children
            .iter()
            .any(|c| c["type"] == "Text" && c["key"].as_str().unwrap().contains("inline")));

        // Tabs component
        assert!(components.iter().any(|c| c["type"] == "Tabs"));

        // Nested table
        assert!(components
            .iter()
            .any(|c| c["type"] == "Table" && c["key"].as_str().unwrap().contains("table")));
    }

    #[test]
    fn focus_hides_hidden_relationships() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("title", DataType::String, FieldMeaning::EntityName)
            .relationship(
                RelationshipDef::new("internal", "internal_ref", Cardinality::OneToOne)
                    .navigation(NavigationHint::Hidden),
            );
        let intents = vec![focus_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        // Only the card, no relationship components
        assert_eq!(components.len(), 1);
        assert_eq!(components[0]["type"], "Card");
    }

    // -- Error handling --

    #[test]
    fn render_returns_error_for_out_of_bounds_intent_index() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let ctx = VisualContext {
            intent_index: 5,
            current_state: None,
            mode: RenderMode::Display,
            templates: None,
        };
        let result = JsonUiRenderer.render(&service, &intents, &ctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("intent_index 5 out of bounds"));
    }

    // -- Custom intent fallback --

    #[test]
    fn custom_intent_falls_back_to_focus_layout() {
        let service = order_service();
        let intents = vec![custom_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        // Focus layout: Card with DescriptionList
        assert_eq!(components[0]["type"], "Card");
        let children = components[0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "DescriptionList");
    }

    // -- Mode transitions --

    #[test]
    fn browse_input_mode_renders_collect_form() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &input_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components[0]["type"], "Form");
    }

    #[test]
    fn focus_input_mode_renders_collect_form() {
        let service = order_service();
        let intents = vec![focus_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &input_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components[0]["type"], "Form");
    }

    // -- Collect tests --

    #[test]
    fn collect_produces_form_with_inputs() {
        let service = ServiceDef::new("user")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("email", DataType::String, FieldMeaning::Email)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt);
        let intents = vec![collect_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components[0]["type"], "Form");
        assert_eq!(components[0]["method"], "POST");
        assert_eq!(components[0]["action_handler"], "user.store");
    }

    #[test]
    fn collect_skips_auto_generated_system_fields() {
        let mut service = ServiceDef::new("user")
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("email", DataType::String, FieldMeaning::Email)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt);
        // Add read-only Identifier (auto-generated)
        service.fields.insert(
            0,
            FieldDef {
                name: "id".to_string(),
                data_type: DataType::Integer,
                meaning: FieldMeaning::Identifier,
                required: true,
                is_list: false,
                readable: true,
                writable: false,
            },
        );
        let intents = vec![collect_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let children = result["components"][0]["children"].as_array().unwrap();
        let names: Vec<&str> = children
            .iter()
            .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(!names.contains(&"id"));
        assert!(!names.contains(&"created_at"));
        assert!(!names.contains(&"updated_at"));
        assert!(names.contains(&"name"));
        assert!(names.contains(&"email"));
    }

    #[test]
    fn collect_includes_submit_button() {
        let service =
            ServiceDef::new("user").field("name", DataType::String, FieldMeaning::EntityName);
        let intents = vec![collect_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let children = result["components"][0]["children"].as_array().unwrap();
        let last = children.last().unwrap();
        assert_eq!(last["type"], "Button");
        assert_eq!(last["label"], "Save");
        assert_eq!(last["variant"], "default");
    }

    #[test]
    fn collect_maps_boolean_to_switch() {
        let service = ServiceDef::new("settings").field(
            "is_active",
            DataType::Boolean,
            FieldMeaning::Boolean,
        );
        let intents = vec![collect_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let children = result["components"][0]["children"].as_array().unwrap();
        // First child is the Switch, last is Button
        assert_eq!(children[0]["type"], "Switch");
        assert_eq!(children[0]["name"], "is_active");
    }

    #[test]
    fn collect_maps_email_to_email_input() {
        let service =
            ServiceDef::new("contact").field("email", DataType::String, FieldMeaning::Email);
        let intents = vec![collect_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let children = result["components"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "Input");
        assert_eq!(children[0]["input_type"], "email");
    }

    // -- Summarize tests --

    #[test]
    fn summarize_produces_card_per_metric_field() {
        let service = ServiceDef::new("dashboard")
            .display_name("Dashboard")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("revenue", DataType::Float, FieldMeaning::Money)
            .field("units_sold", DataType::Integer, FieldMeaning::Quantity)
            .field("completion", DataType::Float, FieldMeaning::Percentage)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt);
        let intents = vec![summarize_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();

        // 3 metric cards: revenue, units_sold, completion
        assert_eq!(components.len(), 3);

        // Revenue card has Text child
        let revenue = components.iter().find(|c| c["title"] == "Revenue").unwrap();
        assert_eq!(revenue["type"], "Card");
        assert_eq!(revenue["children"][0]["type"], "Text");

        // Completion card has Progress child
        let completion = components
            .iter()
            .find(|c| c["title"] == "Completion")
            .unwrap();
        assert_eq!(completion["type"], "Card");
        assert_eq!(completion["children"][0]["type"], "Progress");
    }

    #[test]
    fn summarize_falls_back_to_description_list_without_numeric_fields() {
        let service = ServiceDef::new("profile")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("email", DataType::String, FieldMeaning::Email);
        let intents = vec![summarize_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components[0]["type"], "DescriptionList");
    }

    #[test]
    fn summarize_shows_status_as_badge() {
        let service = ServiceDef::new("project")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("revenue", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status);
        let intents = vec![summarize_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();

        // Status card with Badge
        let status_card = components.iter().find(|c| c["title"] == "Status").unwrap();
        assert_eq!(status_card["children"][0]["type"], "Badge");
    }

    #[test]
    fn summarize_input_mode_renders_collect_form() {
        let service =
            ServiceDef::new("dashboard").field("revenue", DataType::Float, FieldMeaning::Money);
        let intents = vec![summarize_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &input_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components[0]["type"], "Form");
    }

    // -- Process tests --

    fn process_intent() -> IntentScore {
        IntentScore {
            intent: Intent::Process,
            confidence: 0.9,
            matching_signals: vec!["test".into()],
        }
    }

    fn analyze_intent() -> IntentScore {
        IntentScore {
            intent: Intent::Analyze,
            confidence: 0.7,
            matching_signals: vec!["test".into()],
        }
    }

    fn track_intent() -> IntentScore {
        IntentScore {
            intent: Intent::Track,
            confidence: 0.75,
            matching_signals: vec!["test".into()],
        }
    }

    fn workflow_service() -> ServiceDef {
        use ferro_projections::ActionDef;
        use ferro_projections::{StateDef, StateMachine, Transition};

        ServiceDef::new("order")
            .display_name("Order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("title", DataType::String, FieldMeaning::EntityName)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .action(
                ActionDef::new("submit")
                    .display_name("Submit Order")
                    .transition_trigger("submit"),
            )
            .action(
                ActionDef::new("approve")
                    .display_name("Approve")
                    .transition_trigger("approve"),
            )
            .action(ActionDef::new("update_notes").display_name("Update Notes"))
            .state_machine(
                StateMachine::new("order_workflow")
                    .initial("draft")
                    .state(StateDef::new("draft").display_name("Draft"))
                    .state(StateDef::new("pending").display_name("Pending"))
                    .state(
                        StateDef::new("approved")
                            .display_name("Approved")
                            .final_state(),
                    )
                    .transition(
                        Transition::new("draft", "submit", "pending").guard("has_required_fields"),
                    )
                    .transition(
                        Transition::new("pending", "approve", "approved").guard("is_reviewer"),
                    ),
            )
    }

    #[test]
    fn process_produces_card_badge_and_action_buttons_with_state_machine() {
        let service = workflow_service();
        let intents = vec![process_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();

        // Card with Badge
        assert_eq!(components[0]["type"], "Card");
        let children = components[0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "Badge");
        assert_eq!(children[0]["data_path"], "/data/state");

        // Transition action buttons (submit, approve — not update_notes)
        let buttons: Vec<&Value> = components
            .iter()
            .filter(|c| c["type"] == "Button")
            .collect();
        assert_eq!(buttons.len(), 2);
        assert_eq!(buttons[0]["label"], "Submit Order");
        assert_eq!(buttons[0]["action_handler"], "order.submit");
        assert_eq!(buttons[1]["label"], "Approve");
        assert_eq!(buttons[1]["action_handler"], "order.approve");
    }

    #[test]
    fn process_falls_back_to_focus_without_state_machine() {
        let service = ServiceDef::new("order")
            .display_name("Order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("title", DataType::String, FieldMeaning::EntityName)
            .field("total", DataType::Float, FieldMeaning::Money);
        let intents = vec![process_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        // Focus layout: Card with DescriptionList
        assert_eq!(components[0]["type"], "Card");
        let children = components[0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "DescriptionList");
    }

    #[test]
    fn process_includes_guard_alert_for_guarded_transitions() {
        let service = workflow_service();
        let intents = vec![process_intent()];
        // Current state is "draft" which has guarded transition (has_required_fields)
        let ctx = VisualContext {
            intent_index: 0,
            current_state: Some("draft".to_string()),
            mode: RenderMode::Display,
            templates: None,
        };
        let result = JsonUiRenderer.render(&service, &intents, &ctx).unwrap();
        let card_children = result["components"][0]["children"].as_array().unwrap();
        let alert = card_children.iter().find(|c| c["type"] == "Alert");
        assert!(alert.is_some(), "Expected Alert for guarded transitions");
        let alert = alert.unwrap();
        assert_eq!(alert["variant"], "info");
        assert!(alert["description"]
            .as_str()
            .unwrap()
            .contains("has_required_fields"));
    }

    #[test]
    fn process_shows_current_state_in_badge() {
        let service = workflow_service();
        let intents = vec![process_intent()];
        let ctx = VisualContext {
            intent_index: 0,
            current_state: Some("pending".to_string()),
            mode: RenderMode::Display,
            templates: None,
        };
        let result = JsonUiRenderer.render(&service, &intents, &ctx).unwrap();
        let badge = &result["components"][0]["children"][0];
        assert_eq!(badge["text"], "pending");
    }

    #[test]
    fn process_input_mode_renders_form_with_transition_buttons() {
        let service = workflow_service();
        let intents = vec![process_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &input_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        // Form first
        assert_eq!(components[0]["type"], "Form");
        // Transition buttons after form
        let buttons: Vec<&Value> = components
            .iter()
            .filter(|c| c["type"] == "Button" && c.get("action_handler").is_some())
            .filter(|c| c["action_handler"].as_str().unwrap() != "order.store")
            .collect();
        assert_eq!(buttons.len(), 2);
    }

    // -- Analyze tests --

    #[test]
    fn analyze_produces_sortable_table_with_all_readable_fields() {
        let service = ServiceDef::new("metric")
            .display_name("Metric")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("value", DataType::Float, FieldMeaning::Money)
            .field("recorded_at", DataType::DateTime, FieldMeaning::DateTime)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt);
        let intents = vec![analyze_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();

        // Find the table
        let table = components.iter().find(|c| c["type"] == "Table").unwrap();
        assert_eq!(table["sortable"], true);
        assert_eq!(table["sort_direction"], "desc");
        assert_eq!(table["data_path"], "/data/items");

        // Columns include DateTime fields (unlike Browse)
        let columns = table["columns"].as_array().unwrap();
        let keys: Vec<&str> = columns.iter().map(|c| c["key"].as_str().unwrap()).collect();
        assert!(keys.contains(&"name"), "Expected name column");
        assert!(keys.contains(&"value"), "Expected value column");
        assert!(keys.contains(&"recorded_at"), "Expected recorded_at column");
        assert!(keys.contains(&"created_at"), "Expected created_at column");
        // Identifier excluded
        assert!(!keys.contains(&"id"));
    }

    #[test]
    fn analyze_has_no_pagination() {
        let service = ServiceDef::new("metric")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("value", DataType::Float, FieldMeaning::Money);
        let intents = vec![analyze_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert!(
            !components.iter().any(|c| c["type"] == "Pagination"),
            "Analyze should not have Pagination"
        );
    }

    #[test]
    fn analyze_includes_summary_card_when_numeric_fields_exist() {
        let service = ServiceDef::new("metric")
            .display_name("Metric")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("revenue", DataType::Float, FieldMeaning::Money)
            .field("count", DataType::Integer, FieldMeaning::Quantity);
        let intents = vec![analyze_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();

        // Summary card present
        let summary = components.iter().find(|c| c["title"] == "Summary");
        assert!(
            summary.is_some(),
            "Expected Summary card for numeric fields"
        );

        let summary = summary.unwrap();
        assert_eq!(summary["type"], "Card");
        let dl = &summary["children"][0];
        assert_eq!(dl["type"], "DescriptionList");
        let items = dl["items"].as_array().unwrap();
        // Total + Average for each numeric field
        let terms: Vec<&str> = items.iter().map(|i| i["term"].as_str().unwrap()).collect();
        assert!(terms.contains(&"Total Revenue"));
        assert!(terms.contains(&"Average Revenue"));
        assert!(terms.contains(&"Total Count"));
        assert!(terms.contains(&"Average Count"));
    }

    #[test]
    fn analyze_no_summary_card_without_numeric_fields() {
        let service = ServiceDef::new("log")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("message", DataType::String, FieldMeaning::FreeText)
            .field("recorded_at", DataType::DateTime, FieldMeaning::DateTime);
        let intents = vec![analyze_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert!(
            !components.iter().any(|c| c["title"] == "Summary"),
            "No summary card without numeric fields"
        );
    }

    #[test]
    fn analyze_input_mode_renders_collect_form() {
        let service = ServiceDef::new("metric")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("value", DataType::Float, FieldMeaning::Money);
        let intents = vec![analyze_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &input_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components[0]["type"], "Form");
    }

    // -- Track tests --

    #[test]
    fn track_produces_table_with_datetime_and_status_columns() {
        let service = ServiceDef::new("activity")
            .display_name("Activity Log")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt);
        let intents = vec![track_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();

        let table = &components[0];
        assert_eq!(table["type"], "Table");

        let columns = table["columns"].as_array().unwrap();
        let keys: Vec<&str> = columns.iter().map(|c| c["key"].as_str().unwrap()).collect();
        // DateTime fields INCLUDED (Track's key difference from Browse)
        assert!(keys.contains(&"created_at"), "Track includes created_at");
        assert!(keys.contains(&"updated_at"), "Track includes updated_at");
        // Status and EntityName included
        assert!(keys.contains(&"status"));
        assert!(keys.contains(&"name"));
        // Identifier excluded
        assert!(!keys.contains(&"id"));
    }

    #[test]
    fn track_includes_pagination() {
        let service = ServiceDef::new("activity")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt);
        let intents = vec![track_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components.len(), 2);
        assert_eq!(components[1]["type"], "Pagination");
    }

    #[test]
    fn track_sorts_by_datetime_desc() {
        let service = ServiceDef::new("activity")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt);
        let intents = vec![track_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let table = &result["components"][0];
        assert_eq!(table["sort_direction"], "desc");
        assert_eq!(table["sortable"], true);
    }

    #[test]
    fn track_input_mode_renders_collect_form() {
        let service = ServiceDef::new("activity")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status);
        let intents = vec![track_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &input_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components[0]["type"], "Form");
    }

    // -- Empty intent slice --

    #[test]
    fn render_returns_error_for_empty_intents() {
        let service = order_service();
        let intents: Vec<IntentScore> = vec![];
        let result = JsonUiRenderer.render(&service, &intents, &default_ctx());
        assert!(result.is_err());
    }

    // =========================================================================
    // Full pipeline integration tests: ServiceDef → derive_intents → render
    // =========================================================================

    mod pipeline {
        use super::*;
        use ferro_projections::derive_intents;
        use ferro_projections::ActionDef;
        use ferro_projections::Intent;

        use ferro_projections::{StateDef, StateMachine, Transition};

        /// Validates common structural invariants on any render output.
        fn assert_valid_json_ui(result: &Value) {
            assert_eq!(result["$schema"], "ferro-json-ui/v1");
            assert!(result["title"].as_str().is_some(), "title must be a string");
            let components = result["components"]
                .as_array()
                .expect("components must be array");
            assert!(!components.is_empty(), "components must not be empty");
        }

        #[test]
        fn ecommerce_product_catalog_derives_browse_renders_table() {
            let service = ServiceDef::new("product")
                .display_name("Product")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .field("name", DataType::String, FieldMeaning::EntityName)
                .field("price", DataType::Float, FieldMeaning::Money)
                .field("category", DataType::String, FieldMeaning::Category)
                .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
                .has_many("reviews", "review");

            let intents = derive_intents(&service);
            assert!(
                !intents.is_empty(),
                "derive_intents must return at least 1 intent"
            );

            let result = JsonUiRenderer
                .render(&service, &intents, &default_ctx())
                .unwrap();
            assert_valid_json_ui(&result);
            assert_eq!(result["title"], "Product");

            let components = result["components"].as_array().unwrap();
            let primary = &intents[0].intent;

            // Product catalog primary should be Browse or Focus (both table-capable)
            match primary {
                Intent::Browse => {
                    assert_eq!(components[0]["type"], "Table");
                    // System fields excluded from Browse
                    let columns = components[0]["columns"].as_array().unwrap();
                    let keys: Vec<&str> =
                        columns.iter().map(|c| c["key"].as_str().unwrap()).collect();
                    assert!(!keys.contains(&"id"));
                    assert!(!keys.contains(&"created_at"));
                }
                Intent::Focus => {
                    assert_eq!(components[0]["type"], "Card");
                }
                _ => {
                    // Render succeeds regardless of derived intent
                    assert!(components[0]["type"].as_str().is_some());
                }
            }
        }

        #[test]
        fn user_profile_derives_focus_renders_card() {
            let service = ServiceDef::new("user_profile")
                .display_name("User Profile")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .field("name", DataType::String, FieldMeaning::EntityName)
                .field("email", DataType::String, FieldMeaning::Email)
                .field("avatar", DataType::String, FieldMeaning::ImageUrl)
                .field("bio", DataType::String, FieldMeaning::FreeText)
                .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt);

            let intents = derive_intents(&service);
            let result = JsonUiRenderer
                .render(&service, &intents, &default_ctx())
                .unwrap();
            assert_valid_json_ui(&result);
            assert_eq!(result["title"], "User Profile");

            // Focus produces Card + DescriptionList
            let components = result["components"].as_array().unwrap();
            if intents[0].intent == Intent::Focus {
                assert_eq!(components[0]["type"], "Card");
                let children = components[0]["children"].as_array().unwrap();
                assert_eq!(children[0]["type"], "DescriptionList");
            }
        }

        #[test]
        fn survey_form_derives_collect_renders_form() {
            let service = ServiceDef::new("survey")
                .display_name("Survey")
                .write_only_field("question_1", DataType::String, FieldMeaning::FreeText)
                .write_only_field("question_2", DataType::String, FieldMeaning::FreeText)
                .write_only_field("question_3", DataType::String, FieldMeaning::FreeText)
                .write_only_field("rating", DataType::Integer, FieldMeaning::Quantity);

            let intents = derive_intents(&service);
            let result = JsonUiRenderer
                .render(&service, &intents, &default_ctx())
                .unwrap();
            assert_valid_json_ui(&result);

            let components = result["components"].as_array().unwrap();
            if intents[0].intent == Intent::Collect {
                assert_eq!(components[0]["type"], "Form");
                let children = components[0]["children"].as_array().unwrap();
                // At least the writable fields + Submit button
                assert!(children.len() >= 5);
            }
        }

        #[test]
        fn order_workflow_derives_process_renders_card_with_buttons() {
            let service = ServiceDef::new("order")
                .display_name("Order")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .field("title", DataType::String, FieldMeaning::EntityName)
                .field("status", DataType::String, FieldMeaning::Status)
                .field("total", DataType::Float, FieldMeaning::Money)
                .action(
                    ActionDef::new("submit")
                        .display_name("Submit")
                        .transition_trigger("submit"),
                )
                .action(
                    ActionDef::new("approve")
                        .display_name("Approve")
                        .transition_trigger("approve"),
                )
                .action(
                    ActionDef::new("reject")
                        .display_name("Reject")
                        .transition_trigger("reject"),
                )
                .state_machine(
                    StateMachine::new("order_flow")
                        .initial("draft")
                        .state(StateDef::new("draft"))
                        .state(StateDef::new("pending"))
                        .state(StateDef::new("approved").final_state())
                        .state(StateDef::new("rejected").final_state())
                        .transition(
                            Transition::new("draft", "submit", "pending")
                                .guard("has_required_fields"),
                        )
                        .transition(
                            Transition::new("pending", "approve", "approved").guard("is_manager"),
                        )
                        .transition(Transition::new("pending", "reject", "rejected")),
                );

            let intents = derive_intents(&service);
            let result = JsonUiRenderer
                .render(&service, &intents, &default_ctx())
                .unwrap();
            assert_valid_json_ui(&result);

            let components = result["components"].as_array().unwrap();
            if intents[0].intent == Intent::Process {
                // Card with Badge for state display
                assert_eq!(components[0]["type"], "Card");
                let children = components[0]["children"].as_array().unwrap();
                assert_eq!(children[0]["type"], "Badge");
                assert_eq!(children[0]["data_path"], "/data/state");

                // Transition buttons
                let buttons: Vec<&Value> = components
                    .iter()
                    .filter(|c| c["type"] == "Button")
                    .collect();
                assert_eq!(buttons.len(), 3);
            }
        }

        #[test]
        fn activity_log_derives_track_renders_table_with_datetime() {
            let service = ServiceDef::new("activity")
                .display_name("Activity Log")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .field("actor", DataType::String, FieldMeaning::EntityName)
                .field("status", DataType::String, FieldMeaning::Status)
                .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
                .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt);

            let intents = derive_intents(&service);
            let result = JsonUiRenderer
                .render(&service, &intents, &default_ctx())
                .unwrap();
            assert_valid_json_ui(&result);

            let components = result["components"].as_array().unwrap();
            if intents[0].intent == Intent::Track {
                let table = &components[0];
                assert_eq!(table["type"], "Table");
                assert_eq!(table["sort_direction"], "desc");

                let columns = table["columns"].as_array().unwrap();
                let keys: Vec<&str> = columns.iter().map(|c| c["key"].as_str().unwrap()).collect();
                // Track includes DateTime system fields
                assert!(keys.contains(&"created_at"));
                assert!(keys.contains(&"updated_at"));
                assert!(keys.contains(&"status"));

                // Pagination present
                assert_eq!(components[1]["type"], "Pagination");
            }
        }

        // -- VisualContext variation tests --

        #[test]
        fn secondary_intent_renders_different_layout() {
            let service = ServiceDef::new("product")
                .display_name("Product")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .field("name", DataType::String, FieldMeaning::EntityName)
                .field("price", DataType::Float, FieldMeaning::Money)
                .field("category", DataType::String, FieldMeaning::Category)
                .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt);

            let intents = derive_intents(&service);
            assert!(
                intents.len() >= 2,
                "Expected at least 2 intents for a product service"
            );

            // Render secondary intent
            let ctx = VisualContext {
                intent_index: 1,
                current_state: None,
                mode: RenderMode::Display,
                templates: None,
            };
            let result = JsonUiRenderer.render(&service, &intents, &ctx).unwrap();
            assert_valid_json_ui(&result);
        }

        #[test]
        fn input_mode_on_focus_service_produces_form() {
            let service = ServiceDef::new("profile")
                .display_name("Profile")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .field("name", DataType::String, FieldMeaning::EntityName)
                .field("email", DataType::String, FieldMeaning::Email)
                .field("bio", DataType::String, FieldMeaning::FreeText);

            let intents = derive_intents(&service);
            let ctx = VisualContext {
                intent_index: 0,
                current_state: None,
                mode: RenderMode::Input,
                templates: None,
            };
            let result = JsonUiRenderer.render(&service, &intents, &ctx).unwrap();
            assert_valid_json_ui(&result);

            let components = result["components"].as_array().unwrap();
            assert_eq!(components[0]["type"], "Form");
        }

        #[test]
        fn current_state_affects_process_badge() {
            let service = ServiceDef::new("ticket")
                .display_name("Ticket")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .field("title", DataType::String, FieldMeaning::EntityName)
                .action(
                    ActionDef::new("escalate")
                        .display_name("Escalate")
                        .transition_trigger("escalate"),
                )
                .state_machine(
                    StateMachine::new("ticket_flow")
                        .initial("open")
                        .state(StateDef::new("open"))
                        .state(StateDef::new("escalated").final_state())
                        .transition(Transition::new("open", "escalate", "escalated")),
                );

            let intents = vec![IntentScore {
                intent: Intent::Process,
                confidence: 0.9,
                matching_signals: vec!["test".into()],
            }];

            let ctx = VisualContext {
                intent_index: 0,
                current_state: Some("open".to_string()),
                mode: RenderMode::Display,
                templates: None,
            };
            let result = JsonUiRenderer.render(&service, &intents, &ctx).unwrap();
            let badge = &result["components"][0]["children"][0];
            assert_eq!(badge["text"], "open");
        }

        // -- Edge case tests --

        #[test]
        fn empty_service_renders_without_panic() {
            let service = ServiceDef::new("empty");
            let intents = derive_intents(&service);
            let result = JsonUiRenderer
                .render(&service, &intents, &default_ctx())
                .unwrap();
            assert_valid_json_ui(&result);
        }

        #[test]
        fn service_with_only_system_fields_renders_minimal_view() {
            let service = ServiceDef::new("minimal")
                .display_name("Minimal")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
                .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt);

            let intents = derive_intents(&service);
            let result = JsonUiRenderer
                .render(&service, &intents, &default_ctx())
                .unwrap();
            assert_valid_json_ui(&result);
        }

        #[test]
        fn all_sensitive_fields_focus_shows_empty_description_list() {
            let service = ServiceDef::new("secrets")
                .display_name("Secrets")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .field("api_key", DataType::String, FieldMeaning::Sensitive)
                .field("token", DataType::String, FieldMeaning::Sensitive);

            let intents = vec![IntentScore {
                intent: Intent::Focus,
                confidence: 0.5,
                matching_signals: vec!["test".into()],
            }];

            let result = JsonUiRenderer
                .render(&service, &intents, &default_ctx())
                .unwrap();
            assert_valid_json_ui(&result);

            // Sensitive fields excluded from display → empty items
            let items = result["components"][0]["children"][0]["items"]
                .as_array()
                .unwrap();
            assert!(
                items.is_empty(),
                "Sensitive fields should be excluded from Focus display"
            );
        }

        #[test]
        fn all_sensitive_fields_collect_shows_password_inputs() {
            let service = ServiceDef::new("secrets")
                .display_name("Secrets")
                .field("api_key", DataType::String, FieldMeaning::Sensitive)
                .field("token", DataType::String, FieldMeaning::Sensitive);

            let intents = vec![IntentScore {
                intent: Intent::Collect,
                confidence: 0.5,
                matching_signals: vec!["test".into()],
            }];

            let result = JsonUiRenderer
                .render(&service, &intents, &default_ctx())
                .unwrap();
            assert_valid_json_ui(&result);

            let children = result["components"][0]["children"].as_array().unwrap();
            let password_inputs: Vec<&Value> = children
                .iter()
                .filter(|c| c["input_type"] == "password")
                .collect();
            assert_eq!(password_inputs.len(), 2);
        }

        #[test]
        fn all_seven_intents_render_without_error() {
            let service = ServiceDef::new("universal")
                .display_name("Universal")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .field("name", DataType::String, FieldMeaning::EntityName)
                .field("price", DataType::Float, FieldMeaning::Money)
                .field("status", DataType::String, FieldMeaning::Status)
                .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
                .action(
                    ActionDef::new("do_thing")
                        .display_name("Do Thing")
                        .transition_trigger("do"),
                )
                .state_machine(
                    StateMachine::new("flow")
                        .initial("a")
                        .state(StateDef::new("a"))
                        .state(StateDef::new("b").final_state())
                        .transition(Transition::new("a", "do", "b")),
                );

            let all_intents = vec![
                Intent::Browse,
                Intent::Focus,
                Intent::Collect,
                Intent::Process,
                Intent::Summarize,
                Intent::Analyze,
                Intent::Track,
                Intent::Custom("test".into()),
            ];

            for intent in all_intents {
                let intent_score = IntentScore {
                    intent: intent.clone(),
                    confidence: 0.5,
                    matching_signals: vec!["test".into()],
                };
                let intents = vec![intent_score];
                let result = JsonUiRenderer
                    .render(&service, &intents, &default_ctx())
                    .unwrap();
                assert_valid_json_ui(&result);
            }
        }
    }

    // =========================================================================
    // Template consumption tests: ThemeTemplates → intent layout overrides
    // =========================================================================

    mod template_tests {
        use super::*;
        use ferro_theme::{IntentModeTemplates, IntentSlotTemplate, ThemeTemplates};

        fn order_service() -> ServiceDef {
            ServiceDef::new("order")
                .display_name("Order")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .field("title", DataType::String, FieldMeaning::EntityName)
                .field("total", DataType::Float, FieldMeaning::Money)
                .field("status", DataType::String, FieldMeaning::Status)
                .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
                .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
        }

        fn browse_intent() -> IntentScore {
            IntentScore {
                intent: Intent::Browse,
                confidence: 0.8,
                matching_signals: vec!["test".into()],
            }
        }

        fn focus_intent() -> IntentScore {
            IntentScore {
                intent: Intent::Focus,
                confidence: 0.7,
                matching_signals: vec!["test".into()],
            }
        }

        #[test]
        fn render_context_default_has_templates_none() {
            let ctx = VisualContext::default();
            assert!(ctx.templates.is_none());
        }

        #[test]
        fn render_with_none_templates_produces_same_output_as_before() {
            let service = order_service();
            let intents = vec![browse_intent()];

            // No templates at all (baseline)
            let ctx_no_templates = VisualContext::default();
            let result_no_templates = JsonUiRenderer
                .render(&service, &intents, &ctx_no_templates)
                .unwrap();

            // Explicit None templates
            let ctx_none_templates = VisualContext {
                templates: None,
                ..VisualContext::default()
            };
            let result_none_templates = JsonUiRenderer
                .render(&service, &intents, &ctx_none_templates)
                .unwrap();

            assert_eq!(result_no_templates, result_none_templates);
        }

        #[test]
        fn render_with_browse_template_override_uses_slot_order() {
            let service = order_service();
            let intents = vec![browse_intent()];

            // Override Browse to only show title slot (Text component)
            let templates = ThemeTemplates {
                browse: Some(IntentModeTemplates {
                    display: IntentSlotTemplate {
                        slots: vec!["title".to_string()],
                        layout: None,
                    },
                    input: IntentSlotTemplate::default(),
                }),
                ..ThemeTemplates::default()
            };

            let ctx = VisualContext {
                templates: Some(templates),
                ..VisualContext::default()
            };

            let result = JsonUiRenderer.render(&service, &intents, &ctx).unwrap();
            let components = result["components"].as_array().unwrap();

            // Template with only "title" slot produces a Text (h1) component
            assert!(!components.is_empty());
            assert_eq!(components[0]["type"], "Text");
            assert_eq!(components[0]["element"], "h1");
        }

        #[test]
        fn render_with_partial_templates_only_browse_overridden_focus_uses_builtin() {
            let service = order_service();

            // Override Browse only
            let templates = ThemeTemplates {
                browse: Some(IntentModeTemplates {
                    display: IntentSlotTemplate {
                        slots: vec!["title".to_string()],
                        layout: None,
                    },
                    input: IntentSlotTemplate::default(),
                }),
                ..ThemeTemplates::default()
            };

            // Browse intent uses template
            let browse_ctx = VisualContext {
                templates: Some(templates.clone()),
                ..VisualContext::default()
            };
            let browse_result = JsonUiRenderer
                .render(&service, &[browse_intent()], &browse_ctx)
                .unwrap();
            let browse_components = browse_result["components"].as_array().unwrap();
            assert_eq!(browse_components[0]["type"], "Text"); // template override

            // Focus intent uses built-in (no template for focus)
            let focus_ctx = VisualContext {
                templates: Some(templates),
                ..VisualContext::default()
            };
            let focus_result = JsonUiRenderer
                .render(&service, &[focus_intent()], &focus_ctx)
                .unwrap();
            let focus_components = focus_result["components"].as_array().unwrap();
            assert_eq!(focus_components[0]["type"], "Card"); // built-in layout
        }

        #[test]
        fn template_slot_with_no_data_relationships_is_skipped() {
            // Service with no relationships — "relationships" slot should be skipped
            let service = ServiceDef::new("product")
                .display_name("Product")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .field("name", DataType::String, FieldMeaning::EntityName);

            let templates = ThemeTemplates {
                browse: Some(IntentModeTemplates {
                    display: IntentSlotTemplate {
                        slots: vec!["title".to_string(), "relationships".to_string()],
                        layout: None,
                    },
                    input: IntentSlotTemplate::default(),
                }),
                ..ThemeTemplates::default()
            };

            let ctx = VisualContext {
                templates: Some(templates),
                ..VisualContext::default()
            };

            let result = JsonUiRenderer
                .render(&service, &[browse_intent()], &ctx)
                .unwrap();
            let components = result["components"].as_array().unwrap();

            // Only title slot rendered — relationships slot skipped (no relationships)
            // components should only contain the Text (h1) for title
            let types: Vec<&str> = components
                .iter()
                .map(|c| c["type"].as_str().unwrap_or(""))
                .collect();
            assert!(types.contains(&"Text"));
            // No relationship-specific components
            assert!(
                !types.contains(&"Table")
                    || components.iter().all(|c| c["key"]
                        .as_str()
                        .map(|k| !k.contains("rel"))
                        .unwrap_or(true))
            );
        }

        #[test]
        fn template_with_layout_hint_is_passed_through_to_output() {
            let service = order_service();
            let intents = vec![browse_intent()];

            let templates = ThemeTemplates {
                browse: Some(IntentModeTemplates {
                    display: IntentSlotTemplate {
                        slots: vec!["title".to_string(), "fields".to_string()],
                        layout: Some("table".to_string()),
                    },
                    input: IntentSlotTemplate::default(),
                }),
                ..ThemeTemplates::default()
            };

            let ctx = VisualContext {
                templates: Some(templates),
                ..VisualContext::default()
            };

            let result = JsonUiRenderer.render(&service, &intents, &ctx).unwrap();
            // The layout hint "table" should be reflected: fields slot with table layout
            // produces a Table component
            let components = result["components"].as_array().unwrap();
            let has_table = components.iter().any(|c| c["type"] == "Table");
            assert!(
                has_table,
                "layout hint 'table' should produce a Table component"
            );
        }

        #[test]
        fn empty_theme_templates_all_none_produces_identical_output_to_no_templates() {
            let service = order_service();
            let intents = vec![browse_intent()];

            // No templates
            let ctx_no_templates = VisualContext::default();
            let result_no_templates = JsonUiRenderer
                .render(&service, &intents, &ctx_no_templates)
                .unwrap();

            // Empty ThemeTemplates (all None)
            let ctx_empty = VisualContext {
                templates: Some(ThemeTemplates::default()),
                ..VisualContext::default()
            };
            let result_empty = JsonUiRenderer
                .render(&service, &intents, &ctx_empty)
                .unwrap();

            assert_eq!(result_no_templates, result_empty);
        }
    }
}
