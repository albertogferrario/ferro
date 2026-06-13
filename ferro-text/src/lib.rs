//! Conversational-text renderer for Ferro service projections.
//!
//! `TextRenderer` projects a [`ServiceDef`] to deterministic plain text for the
//! cleanly-mapping intents (Browse, Collect, Process, Summarize, Track), guard-filtered
//! and verbosity-aware, with a defined fallback for Focus and Analyze.
//!
//! # Crate boundary
//!
//! This crate is the sole home of the text `Renderer` implementation.
//! `ferro-projections` owns the trait and schema types; this crate owns the rendering logic.

use std::collections::HashMap;
use std::fmt::Write;

use ferro_projections::render::{
    field_display_name, is_system_field, BaseContext, Renderer, Verbosity,
};
use ferro_projections::{
    ActionDef, Error, FieldDef, FieldMeaning, IntentScore, RenderHint, ServiceDef,
};

/// Renders a service projection to conversational plain text.
///
/// Dispatches on the primary intent (by index in `ctx.intent_index`), applies
/// guard filtering via `ctx.evaluated_guards`, and respects `ctx.verbosity`.
///
/// Returns `Err(Error::NoIntents)` when the intent slice is empty or the index
/// is out of bounds.
pub struct TextRenderer;

impl Renderer for TextRenderer {
    type Output = String;
    type Context = BaseContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &BaseContext,
    ) -> Result<String, Error> {
        let score = intents.get(ctx.intent_index).ok_or(Error::NoIntents)?;
        let out = match score.intent.label() {
            "browse" => render_browse(service, ctx),
            "collect" => render_collect(service, ctx),
            "process" => render_process(service, ctx),
            "summarize" => render_summarize(service, ctx),
            "track" => render_track(service, ctx),
            "focus" => render_focus(service, ctx),
            "analyze" => render_analyze(service, ctx),
            _ => render_browse(service, ctx), // Custom intents: best-effort browse
        };
        Ok(out)
    }
}

// --- Guard filtering (D-09) ---

/// Returns `true` if the action should be rendered.
///
/// An action is hidden only if **any** of its `preconditions` maps to an explicit
/// `false` in `evaluated_guards`. An absent key means the guard is unconstrained
/// and the action renders.
fn action_passes_guards(action: &ActionDef, evaluated_guards: &HashMap<String, bool>) -> bool {
    action
        .preconditions
        .iter()
        .all(|g| evaluated_guards.get(g.as_str()).copied().unwrap_or(true))
}

// --- render_hint helper (D-12) ---

/// Returns the display string for a field, applying its `render_hint` if present.
///
/// - `Skip` → `None` (caller omits the field)
/// - `AltText(s)` → `Some(s)` (use the alt text verbatim)
/// - `None` on `ImageUrl`/`Url` → `Some("<label> (image)")` / `Some("<label> (link)")`
/// - `None` on other fields → `Some("<label>")`
fn render_field_value(f: &FieldDef) -> Option<String> {
    match &f.render_hint {
        Some(RenderHint::Skip) => None,
        Some(RenderHint::AltText(s)) => Some(s.clone()),
        None => {
            let label = field_display_name(&f.name);
            match &f.meaning {
                FieldMeaning::ImageUrl => Some(format!("{label} (image)")),
                FieldMeaning::Url => Some(format!("{label} (link)")),
                _ => Some(label),
            }
        }
    }
}

// --- Per-intent strategy functions ---

fn render_browse(service: &ServiceDef, ctx: &BaseContext) -> String {
    let entity = service.display_name.as_deref().unwrap_or(&service.name);
    let mut out = String::new();

    let domain_fields: Vec<&FieldDef> = service
        .fields
        .iter()
        .filter(|f| !is_system_field(&f.meaning))
        .collect();

    match ctx.verbosity {
        Verbosity::Brief => {
            // Headline: entity + primary EntityName field only
            let primary = domain_fields
                .iter()
                .find(|f| f.meaning == FieldMeaning::EntityName)
                .map(|f| field_display_name(&f.name));
            if let Some(primary_label) = primary {
                let _ = writeln!(out, "{entity} — {primary_label}");
            } else {
                let _ = writeln!(out, "{entity}");
            }
        }
        Verbosity::Full => {
            let _ = writeln!(out, "{entity}");
            if !domain_fields.is_empty() {
                let _ = writeln!(out, "Fields:");
                for f in &domain_fields {
                    let _ = writeln!(out, "  - {}", field_display_name(&f.name));
                }
            }
        }
    }

    out
}

fn render_collect(service: &ServiceDef, ctx: &BaseContext) -> String {
    let entity = service.display_name.as_deref().unwrap_or(&service.name);
    let writable: Vec<&FieldDef> = service
        .fields
        .iter()
        .filter(|f| f.writable && !is_system_field(&f.meaning))
        .collect();

    let mut out = String::new();

    match ctx.verbosity {
        Verbosity::Brief => {
            let _ = writeln!(out, "{entity} — {} fields to fill in", writable.len());
        }
        Verbosity::Full => {
            let _ = writeln!(out, "{entity}");
            if writable.is_empty() {
                let _ = writeln!(out, "No fields to fill in.");
            } else {
                let _ = writeln!(out, "Fields to fill in:");
                for f in &writable {
                    let required_marker = if f.required { " (required)" } else { "" };
                    let _ = writeln!(
                        out,
                        "  - {}{}",
                        field_display_name(&f.name),
                        required_marker
                    );
                }
            }
        }
    }

    out
}

fn render_process(service: &ServiceDef, ctx: &BaseContext) -> String {
    let entity = service.display_name.as_deref().unwrap_or(&service.name);

    // Resolve current state: ctx.current_state → sm.initial_state → "(unknown)"
    let current_state = ctx
        .current_state
        .as_deref()
        .or_else(|| {
            service
                .state_machine
                .as_ref()
                .map(|sm| sm.initial_state.as_str())
        })
        .unwrap_or("(unknown)");

    // Guard-passing actions only
    let passing_actions: Vec<&ActionDef> = service
        .actions
        .iter()
        .filter(|a| action_passes_guards(a, &ctx.evaluated_guards))
        .collect();

    let mut out = String::new();

    match ctx.verbosity {
        Verbosity::Brief => {
            let _ = write!(out, "{entity} — Currently: {current_state}.");
            if !passing_actions.is_empty() {
                let verbs: Vec<&str> = passing_actions
                    .iter()
                    .map(|a| a.display_name.as_deref().unwrap_or(&a.name))
                    .collect();
                let _ = write!(out, " You can: {}.", verbs.join(", "));
            }
            let _ = writeln!(out);
        }
        Verbosity::Full => {
            let _ = writeln!(out, "{entity} — process");
            let _ = writeln!(out);
            let _ = writeln!(out, "Currently: {current_state}");

            // Domain fields
            let domain_fields: Vec<&FieldDef> = service
                .fields
                .iter()
                .filter(|f| !is_system_field(&f.meaning))
                .collect();
            if !domain_fields.is_empty() {
                let labels: Vec<String> = domain_fields
                    .iter()
                    .map(|f| field_display_name(&f.name))
                    .collect();
                let _ = writeln!(out, "Fields: {}", labels.join(", "));
            }

            if passing_actions.is_empty() {
                let _ = writeln!(out, "Available actions: (none)");
            } else {
                let verbs: Vec<&str> = passing_actions
                    .iter()
                    .map(|a| a.display_name.as_deref().unwrap_or(&a.name))
                    .collect();
                let _ = writeln!(out, "Available actions: {}", verbs.join(", "));
            }
        }
    }

    out
}

fn render_summarize(service: &ServiceDef, ctx: &BaseContext) -> String {
    let entity = service.display_name.as_deref().unwrap_or(&service.name);

    let metric_fields: Vec<&FieldDef> = service
        .fields
        .iter()
        .filter(|f| {
            matches!(
                f.meaning,
                FieldMeaning::Money
                    | FieldMeaning::Percentage
                    | FieldMeaning::Quantity
                    | FieldMeaning::Status
            )
        })
        .collect();

    let mut out = String::new();

    match ctx.verbosity {
        Verbosity::Brief => {
            let first_metric = metric_fields.first().map(|f| field_display_name(&f.name));
            if let Some(label) = first_metric {
                let _ = writeln!(out, "{entity} — {label}");
            } else {
                let _ = writeln!(out, "{entity}");
            }
        }
        Verbosity::Full => {
            let _ = writeln!(out, "{entity}");
            if metric_fields.is_empty() {
                let _ = writeln!(out, "No metric or status fields.");
            } else {
                let labels: Vec<String> = metric_fields
                    .iter()
                    .map(|f| field_display_name(&f.name))
                    .collect();
                let _ = writeln!(out, "Key metrics: {}", labels.join(", "));
            }
        }
    }

    out
}

fn render_track(service: &ServiceDef, ctx: &BaseContext) -> String {
    let entity = service.display_name.as_deref().unwrap_or(&service.name);

    let sm = service.state_machine.as_ref();

    // Resolve current state
    let current_state = ctx
        .current_state
        .as_deref()
        .or_else(|| sm.map(|s| s.initial_state.as_str()))
        .unwrap_or("(unknown)");

    let mut out = String::new();

    match ctx.verbosity {
        Verbosity::Brief => {
            let _ = writeln!(out, "{entity} — Currently: {current_state}.");
        }
        Verbosity::Full => {
            let _ = writeln!(out, "{entity}");
            let _ = writeln!(out, "Currently: {current_state}");

            // Check if current state is terminal
            if let Some(sm) = sm {
                let is_terminal = sm
                    .states
                    .iter()
                    .find(|s| s.name == current_state)
                    .map(|s| s.is_final)
                    .unwrap_or(false);

                if is_terminal {
                    let _ = writeln!(out, "Status: completed (terminal state)");
                } else {
                    // List available transitions
                    let transitions = sm.events_from_state(current_state);
                    if !transitions.is_empty() {
                        let next_states: Vec<&str> =
                            transitions.iter().map(|t| t.to.as_str()).collect();
                        let unique: Vec<&str> = {
                            let mut seen = std::collections::HashSet::new();
                            next_states
                                .into_iter()
                                .filter(|s| seen.insert(*s))
                                .collect()
                        };
                        let _ = writeln!(out, "Next possible states: {}", unique.join(", "));
                    }
                }
            }
        }
    }

    out
}

fn render_focus(service: &ServiceDef, _ctx: &BaseContext) -> String {
    let entity = service.display_name.as_deref().unwrap_or(&service.name);
    let mut out = String::new();

    let _ = writeln!(out, "{entity}");
    for f in service
        .fields
        .iter()
        .filter(|f| !is_system_field(&f.meaning))
    {
        if let Some(value) = render_field_value(f) {
            let _ = writeln!(out, "  - {value}");
        }
    }
    let _ = writeln!(
        out,
        "Note: This is a media/navigational view; full text representation is limited."
    );

    out
}

fn render_analyze(service: &ServiceDef, _ctx: &BaseContext) -> String {
    let entity = service.display_name.as_deref().unwrap_or(&service.name);
    let mut out = String::new();

    let _ = writeln!(out, "{entity}");
    let domain_fields: Vec<&FieldDef> = service
        .fields
        .iter()
        .filter(|f| !is_system_field(&f.meaning))
        .collect();
    if !domain_fields.is_empty() {
        let labels: Vec<String> = domain_fields
            .iter()
            .map(|f| field_display_name(&f.name))
            .collect();
        let _ = writeln!(out, "Fields: {}", labels.join(", "));
    }
    let _ = writeln!(
        out,
        "Note: Time-series and trend data has no full text representation in this channel."
    );

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::render::BaseContext;
    use ferro_projections::{
        derive_intents, ActionDef, DataType, Error, FieldMeaning, GuardDef, Intent, IntentScore,
        RenderHint, ServiceDef, StateDef, StateMachine, Transition, Verbosity,
    };

    /// Construct a single-entry intent slice forcing a specific intent.
    fn force_intent(intent: Intent) -> Vec<IntentScore> {
        vec![IntentScore {
            intent,
            confidence: 1.0,
            matching_signals: vec![],
        }]
    }

    // Copied verbatim from ferro-projections/src/render/sketch/cli.rs test module (D-15)
    fn approval_workflow_fixture() -> ServiceDef {
        ServiceDef::new("approval_workflow")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("title", DataType::String, FieldMeaning::EntityName)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("amount", DataType::Float, FieldMeaning::Money)
            .guard(GuardDef::new("has_required_fields"))
            .guard(GuardDef::new("is_approver"))
            .guard(GuardDef::new("is_cancellable"))
            .state_machine(
                StateMachine::new("approval_lifecycle")
                    .initial("draft")
                    .state(StateDef::new("draft"))
                    .state(StateDef::new("submitted"))
                    .state(StateDef::new("approved").final_state())
                    .state(StateDef::new("rejected").final_state())
                    .state(StateDef::new("cancelled").final_state())
                    .transition(
                        Transition::new("draft", "submit", "submitted")
                            .guard("has_required_fields"),
                    )
                    .transition(
                        Transition::new("submitted", "approve", "approved").guard("is_approver"),
                    )
                    .transition(
                        Transition::new("submitted", "reject", "rejected").guard("is_approver"),
                    )
                    .transition(
                        Transition::new("draft", "cancel", "cancelled").guard("is_cancellable"),
                    )
                    .transition(
                        Transition::new("submitted", "cancel", "cancelled").guard("is_cancellable"),
                    ),
            )
            .action(
                ActionDef::new("submit")
                    .precondition("has_required_fields")
                    .transition_trigger("submit"),
            )
            .action(
                ActionDef::new("approve")
                    .precondition("is_approver")
                    .transition_trigger("approve"),
            )
            .action(
                ActionDef::new("reject")
                    .precondition("is_approver")
                    .transition_trigger("reject"),
            )
            .action(
                ActionDef::new("cancel")
                    .precondition("is_cancellable")
                    .transition_trigger("cancel"),
            )
    }

    // 1. Process anchor — empty guards: all four actions render
    #[test]
    fn process_unfiltered_renders_all_four_actions() {
        let svc = approval_workflow_fixture();
        let intents = derive_intents(&svc);
        let ctx = BaseContext {
            evaluated_guards: HashMap::new(),
            ..Default::default()
        };
        let result = TextRenderer.render(&svc, &intents, &ctx).unwrap();
        insta::assert_snapshot!("process_unfiltered", result);
        assert!(result.contains("submit"), "should contain submit");
        assert!(result.contains("approve"), "should contain approve");
        assert!(result.contains("reject"), "should contain reject");
        assert!(result.contains("cancel"), "should contain cancel");
    }

    // 2. Process anchor — is_approver: false → approve/reject hidden, submit/cancel visible
    #[test]
    fn process_filtered_hides_approve_reject() {
        let svc = approval_workflow_fixture();
        let intents = derive_intents(&svc);
        let ctx = BaseContext {
            evaluated_guards: [("is_approver".to_string(), false)].into(),
            ..Default::default()
        };
        let result = TextRenderer.render(&svc, &intents, &ctx).unwrap();
        insta::assert_snapshot!("process_filtered", result);
        assert!(
            !result.contains("approve"),
            "approve should be hidden when is_approver=false"
        );
        assert!(
            !result.contains("reject"),
            "reject should be hidden when is_approver=false"
        );
        assert!(result.contains("submit"), "submit should remain visible");
        assert!(result.contains("cancel"), "cancel should remain visible");
    }

    // 3. Brief differs from Full
    #[test]
    fn brief_differs_from_full() {
        let svc = approval_workflow_fixture();
        let intents = derive_intents(&svc);
        let full_ctx = BaseContext {
            verbosity: Verbosity::Full,
            ..Default::default()
        };
        let brief_ctx = BaseContext {
            verbosity: Verbosity::Brief,
            ..Default::default()
        };
        let full_out = TextRenderer.render(&svc, &intents, &full_ctx).unwrap();
        let brief_out = TextRenderer.render(&svc, &intents, &brief_ctx).unwrap();
        insta::assert_snapshot!("process_full", full_out);
        insta::assert_snapshot!("process_brief", brief_out);
        assert!(
            brief_out.len() < full_out.len(),
            "Brief output should be shorter than Full"
        );
        assert_ne!(full_out, brief_out, "Full and Brief must differ");
    }

    // 4a. Browse intent
    #[test]
    fn browse_intent_snapshot() {
        let svc = ServiceDef::new("product")
            .display_name("Product")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("price", DataType::Float, FieldMeaning::Money)
            .field("sku", DataType::String, FieldMeaning::Custom("sku".into()));
        let intents = derive_intents(&svc);
        // Force browse by finding it
        let browse_idx = intents
            .iter()
            .position(|s| s.intent.label() == "browse")
            .unwrap_or(0);
        let ctx = BaseContext {
            intent_index: browse_idx,
            ..Default::default()
        };
        let result = TextRenderer.render(&svc, &intents, &ctx).unwrap();
        insta::assert_snapshot!("browse_full", result);
        assert!(result.contains("Product"), "should mention entity");
    }

    // 4b. Collect intent
    #[test]
    fn collect_intent_snapshot() {
        let svc = ServiceDef::new("registration")
            .display_name("Registration")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("email", DataType::String, FieldMeaning::Email)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .optional_field("phone", DataType::String, FieldMeaning::Phone);
        let intents = derive_intents(&svc);
        let collect_idx = intents
            .iter()
            .position(|s| s.intent.label() == "collect")
            .unwrap_or(0);
        let ctx = BaseContext {
            intent_index: collect_idx,
            ..Default::default()
        };
        let result = TextRenderer.render(&svc, &intents, &ctx).unwrap();
        insta::assert_snapshot!("collect_full", result);
        assert!(result.contains("Registration"), "should mention entity");
    }

    // 4c. Summarize intent
    #[test]
    fn summarize_intent_snapshot() {
        let svc = ServiceDef::new("invoice")
            .display_name("Invoice")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("items_count", DataType::Integer, FieldMeaning::Quantity);
        let intents = derive_intents(&svc);
        let summ_idx = intents
            .iter()
            .position(|s| s.intent.label() == "summarize")
            .unwrap_or(0);
        let ctx = BaseContext {
            intent_index: summ_idx,
            ..Default::default()
        };
        let result = TextRenderer.render(&svc, &intents, &ctx).unwrap();
        insta::assert_snapshot!("summarize_full", result);
        assert!(result.contains("Invoice"), "should mention entity");
    }

    // 4d. Track intent
    #[test]
    fn track_intent_snapshot() {
        let svc = ServiceDef::new("shipment")
            .display_name("Shipment")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field(
                "tracking_number",
                DataType::String,
                FieldMeaning::EntityName,
            )
            .state_machine(
                StateMachine::new("shipping_lifecycle")
                    .initial("pending")
                    .state(StateDef::new("pending"))
                    .state(StateDef::new("shipped"))
                    .state(StateDef::new("delivered").final_state())
                    .transition(Transition::new("pending", "ship", "shipped"))
                    .transition(Transition::new("shipped", "deliver", "delivered")),
            );
        let intents = derive_intents(&svc);
        let track_idx = intents
            .iter()
            .position(|s| s.intent.label() == "track")
            .unwrap_or(0);
        let ctx = BaseContext {
            intent_index: track_idx,
            ..Default::default()
        };
        let result = TextRenderer.render(&svc, &intents, &ctx).unwrap();
        insta::assert_snapshot!("track_full", result);
        assert!(result.contains("Shipment"), "should mention entity");
    }

    // 5. ImageUrl with None hint renders with (image) label, not raw URL
    #[test]
    fn image_url_none_hint_labels_not_raw() {
        // Test render_field_value directly: ImageUrl with no hint → "(image)" suffix
        let f = ferro_projections::FieldDef {
            name: "cover_photo".to_string(),
            data_type: DataType::String,
            meaning: FieldMeaning::ImageUrl,
            required: false,
            is_list: false,
            readable: true,
            writable: true,
            render_hint: None,
        };
        let result = render_field_value(&f).unwrap();
        assert!(
            result.contains("(image)"),
            "ImageUrl field without hint should use (image) label; got: {result}"
        );
        assert!(
            !result.contains("http"),
            "should not contain raw URL string; got: {result}"
        );
    }

    // 6. Url with AltText hint renders the alt text
    #[test]
    fn url_alt_text_renders_alt() {
        // Test render_field_value directly since .field() builder does not set render_hint
        let f = ferro_projections::FieldDef {
            name: "photo_url".to_string(),
            data_type: DataType::String,
            meaning: FieldMeaning::Url,
            required: true,
            is_list: false,
            readable: true,
            writable: true,
            render_hint: Some(RenderHint::AltText("Photo".to_string())),
        };
        let result = render_field_value(&f).unwrap();
        assert_eq!(
            result, "Photo",
            "AltText should render the alt text verbatim"
        );
        assert!(
            !result.contains("http"),
            "should not contain raw URL; got: {result}"
        );
    }

    // 7. Url with Skip hint omits the field
    #[test]
    fn url_skip_omits_field() {
        let f = ferro_projections::FieldDef {
            name: "thumbnail".to_string(),
            data_type: DataType::String,
            meaning: FieldMeaning::Url,
            required: false,
            is_list: false,
            readable: true,
            writable: true,
            render_hint: Some(RenderHint::Skip),
        };
        let result = render_field_value(&f);
        assert!(
            result.is_none(),
            "Skip hint should return None; got: {result:?}"
        );
    }

    // 8. Focus fallback — not empty, contains the limited-modality note
    #[test]
    fn focus_fallback_present() {
        let svc = ServiceDef::new("profile")
            .display_name("Profile")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("avatar", DataType::String, FieldMeaning::ImageUrl)
            .field("website", DataType::String, FieldMeaning::Url);
        // Force focus intent — derive_intents may not score focus as primary for this fixture
        let intents = force_intent(Intent::Focus);
        let ctx = BaseContext::default();
        let result = TextRenderer.render(&svc, &intents, &ctx).unwrap();
        insta::assert_snapshot!("focus_fallback", result);
        assert!(!result.is_empty(), "Focus fallback must not be empty");
        assert!(
            result.contains("limited"),
            "Focus fallback must contain the limited-modality note; got: {result}"
        );
    }

    // 9. Analyze fallback — not empty, no fabricated statistics, contains the channel note
    #[test]
    fn analyze_fallback_present() {
        let svc = ServiceDef::new("sales_report")
            .display_name("Sales Report")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("revenue", DataType::Float, FieldMeaning::Money)
            .field("units", DataType::Integer, FieldMeaning::Quantity);
        // Force analyze intent — structural signals (Money + Quantity) score as Summarize by default
        let intents = force_intent(Intent::Analyze);
        let ctx = BaseContext::default();
        let result = TextRenderer.render(&svc, &intents, &ctx).unwrap();
        insta::assert_snapshot!("analyze_fallback", result);
        assert!(!result.is_empty(), "Analyze fallback must not be empty");
        assert!(
            result.contains("Time-series"),
            "Analyze fallback must contain the channel note; got: {result}"
        );
        // Verify no fabricated numbers
        assert!(
            !result.contains("%"),
            "Analyze must not fabricate statistics; got: {result}"
        );
    }

    // 10. Empty intents returns Error::NoIntents
    #[test]
    fn empty_intents_returns_no_intents() {
        let svc = approval_workflow_fixture();
        let result = TextRenderer.render(&svc, &[], &BaseContext::default());
        assert!(
            matches!(result, Err(Error::NoIntents)),
            "empty intents must return Error::NoIntents; got: {result:?}"
        );
    }
}
