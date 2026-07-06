//! Core logic for the `ai_explain` MCP tool and the `ferro ai:explain` CLI wrapper.
//!
//! `explain_core` is the single definition site for projection-framed explanation.
//! Two-branch contract (D-03):
//! - When the target resolves to a `ServiceDef` (via `inspect_projection`), returns
//!   the `ProjectionDetail` serialized as `serde_json::Value` — **zero LLM tokens**.
//! - When the target resolves to a route or model, returns `{ "prose": … }` via a
//!   raw `CompletionRequest { schema: None, .. }` call to the configured LLM provider.
//!
//! **Threat model (T-172-PI-EXPLAIN):** The `target` argument is used as a lookup key
//! only — resolution returns `ResolvedTarget::NotFound` (an `Err`, no LLM call) for
//! unmatched input. The prose prompt is built from introspected artifact facts
//! (`RouteExplanation` / `ModelExplanation`), not from the raw `target` string.
//! The structured branch makes zero LLM calls, so there is no injection surface there.
//! Residual risk is low: the route/model fact content is project-owned, not
//! attacker-controlled.
//!
//! No process termination, no coloring, no runtime bridge in this module (D-04).
//! No feature gates anywhere — ferro-mcp depends on ferro-projections unconditionally.

use crate::tools::{
    explain_model::{self, ModelExplanation},
    explain_route::{self, RouteExplanation},
    inspect_projection::{self, InspectResult, ProjectionDetail},
};
use ferro_ai::client::{Message, Role};
use ferro_ai::{AiConfig, CompletionRequest};
use std::path::Path;

// ---------------------------------------------------------------------------
// Resolved target variants
// ---------------------------------------------------------------------------

/// The result of resolving a target to a concrete introspection artifact.
pub enum ResolvedTarget {
    /// Projection found (primary path): framed in projection vocabulary.
    Service(ProjectionDetail),
    /// Route found (prose fallback).
    Route(RouteExplanation),
    /// Model found (prose fallback).
    Model(ModelExplanation),
    /// Nothing found.
    NotFound(String),
}

// ---------------------------------------------------------------------------
// Pure priority helper (testable without real introspection)
// ---------------------------------------------------------------------------

/// Return the resolution kind that wins given which lookups succeeded.
///
/// Priority order: service (projection-framed primary) → route → model.
///
/// With a `type_override` the caller has already forced the kind before
/// consulting this function — it is used for auto-detect only.
pub fn resolve_kind_priority(
    found_service: bool,
    found_route: bool,
    found_model: bool,
    type_override: Option<&str>,
) -> &'static str {
    if let Some(t) = type_override {
        // Map the override string to a 'static str that this fn owns.
        // The CLI only passes "service", "route", "model", or "not_found".
        // We match on the known values; unknown values fall through to "not_found".
        match t {
            "service" => return "service",
            "route" => return "route",
            "model" => return "model",
            _ => return "not_found",
        }
    }
    if found_service {
        return "service";
    }
    if found_route {
        return "route";
    }
    if found_model {
        return "model";
    }
    "not_found"
}

// ---------------------------------------------------------------------------
// Target resolution (async — no rt param)
// ---------------------------------------------------------------------------

/// Auto-detect target kind in SERVICE → ROUTE → MODEL order.
///
/// With `type_override` set, skips auto-detect and calls only the matching tool.
/// Returns `ResolvedTarget::NotFound` when nothing matches.
///
/// The runtime bridge parameter from the CLI version is removed;
/// every async call uses `.await` directly (Pitfall 1 in Phase 172 RESEARCH).
pub async fn resolve_target(
    root: &Path,
    target: &str,
    type_override: Option<&str>,
) -> ResolvedTarget {
    match type_override {
        Some("service") => match inspect_projection::execute(root, target) {
            InspectResult::Found(d) => return ResolvedTarget::Service(d),
            InspectResult::NotFound(_) => {
                return ResolvedTarget::NotFound(format!("No projection named '{target}' found"))
            }
        },
        Some("route") => match explain_route::execute(root, target).await {
            Ok(r) => return ResolvedTarget::Route(r),
            Err(_) => return ResolvedTarget::NotFound(format!("No route '{target}' found")),
        },
        Some("model") => match explain_model::execute(root, target).await {
            Ok(m) => return ResolvedTarget::Model(m),
            Err(_) => return ResolvedTarget::NotFound(format!("No model '{target}' found")),
        },
        Some(other) => {
            return ResolvedTarget::NotFound(format!(
                "Unknown --type value '{other}'. Use 'service', 'route', or 'model'."
            ));
        }
        None => {}
    }

    // Auto-detect: service → route → model (projection-framed is primary).
    if let InspectResult::Found(d) = inspect_projection::execute(root, target) {
        return ResolvedTarget::Service(d);
    }

    if let Ok(r) = explain_route::execute(root, target).await {
        return ResolvedTarget::Route(r);
    }

    if let Ok(m) = explain_model::execute(root, target).await {
        return ResolvedTarget::Model(m);
    }

    ResolvedTarget::NotFound(format!(
        "Target '{target}' not found as a projection, route, or model."
    ))
}

// ---------------------------------------------------------------------------
// Prompt builders
// ---------------------------------------------------------------------------

/// Build a projection-framed prompt from a `ProjectionDetail`.
///
/// The system prompt instructs the LLM to explain the service in projection
/// terms (Intent, FieldMeaning, ActionDef, GuardDef, StateMachine) using ONLY
/// the supplied facts. The user prompt serializes the parsed string vocabulary
/// from `inspect_projection`.
///
/// NOTE: `build_service_prompt` is kept `pub` for the CLI dry-run path — the CLI
/// may still print the assembled service prompt under `--dry-run` even though
/// `explain_core`'s structured branch does not call this function.
pub fn build_service_prompt(detail: &ProjectionDetail) -> (String, String) {
    let system = "You are a Ferro framework expert. Explain this service in projection terms: \
        which Intents it projects, which fields carry FieldMeaning annotations that drive rendering, \
        which ActionDefs are exposed (and under which GuardDefs if any), \
        and describe the StateMachine transitions if present. \
        Base your explanation ONLY on the supplied introspection facts — \
        do not invent fields, actions, or behaviours not listed."
        .to_string();

    let mut user = format!("Service: {}\n", detail.name);
    if let Some(ref dn) = detail.display_name {
        user.push_str(&format!("DisplayName: {dn}\n"));
    }
    if !detail.service_name.is_empty() {
        user.push_str(&format!("ServiceName: {}\n", detail.service_name));
    }

    if !detail.intent_hints.is_empty() {
        user.push_str("\nIntent hints:\n");
        for hint in &detail.intent_hints {
            user.push_str(&format!("  - {hint}\n"));
        }
    }

    if !detail.fields.is_empty() {
        user.push_str("\nFields (FieldMeaning annotations):\n");
        for f in &detail.fields {
            let access = match (f.readable, f.writable) {
                (true, true) => "read+write",
                (true, false) => "read-only",
                (false, true) => "write-only",
                (false, false) => "inaccessible",
            };
            user.push_str(&format!(
                "  - {}: DataType={}, FieldMeaning={}, access={}\n",
                f.name, f.data_type, f.meaning, access
            ));
        }
    }

    if !detail.actions.is_empty() {
        user.push_str("\nActions (ActionDef):\n");
        for a in &detail.actions {
            user.push_str(&format!("  - {a}\n"));
        }
    }

    if !detail.relationships.is_empty() {
        user.push_str("\nRelationships:\n");
        for r in &detail.relationships {
            user.push_str(&format!("  - {r}\n"));
        }
    }

    if detail.has_state_machine {
        user.push_str("\nStateMachine: present\n");
    }

    user.push_str("\nExplain this service's projection-level design in a few paragraphs.");

    (system, user)
}

/// Build a prose-fallback prompt from a `RouteExplanation`.
pub fn build_route_prompt(r: &RouteExplanation) -> (String, String) {
    let system = "You are a Ferro framework expert. \
        Explain the purpose and business context of this route in plain prose. \
        Base your explanation only on the supplied route facts."
        .to_string();

    let user = format!(
        "Route: {} {}\nHandler: {}\nPurpose: {}\nBusiness context: {}\nGuards: {}\nRelated routes: {}\n\n\
         Explain this route in a few paragraphs.",
        r.method,
        r.route,
        r.handler,
        r.purpose,
        r.business_context,
        r.guards.join(", "),
        r.related_routes.join(", ")
    );

    (system, user)
}

/// Build a prose-fallback prompt from a `ModelExplanation`.
pub fn build_model_prompt(m: &ModelExplanation) -> (String, String) {
    let system = "You are a Ferro framework expert. \
        Explain the domain meaning and relationships of this model in plain prose. \
        Base your explanation only on the supplied model facts."
        .to_string();

    let field_lines: Vec<String> = m
        .fields
        .iter()
        .map(|f| format!("  - {} ({}): {}", f.name, f.field_type, f.meaning))
        .collect();

    let user = format!(
        "Model: {}\nDomain meaning: {}\nTable: {}\nFields:\n{}\nRelationships: {}\nRelated routes: {}\n\n\
         Explain this model in a few paragraphs.",
        m.model,
        m.domain_meaning,
        m.table.as_deref().unwrap_or("(default)"),
        field_lines.join("\n"),
        m.relationships.join(", "),
        m.related_routes.join(", ")
    );

    (system, user)
}

// ---------------------------------------------------------------------------
// Cost guard helper
// ---------------------------------------------------------------------------

/// Read `FERRO_AI_MAX_TOKENS_PER_COMMAND` from env; fall back to `default`.
pub fn resolve_max_tokens_with_default(default: u32) -> u32 {
    std::env::var("FERRO_AI_MAX_TOKENS_PER_COMMAND")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

// ---------------------------------------------------------------------------
// Prose completion helper
// ---------------------------------------------------------------------------

/// Call the configured LLM provider with a prose (unstructured) request.
///
/// Uses `schema: None` — this is a prose path, not a structured JSON completion.
/// Returns the prose string on success or an error string on failure.
async fn call_llm_prose(system: String, user: String) -> Result<String, String> {
    let client = AiConfig::from_env().map_err(|e| e.to_string())?;
    let max_tokens = resolve_max_tokens_with_default(2048);
    let req = CompletionRequest {
        system: Some(system),
        messages: vec![Message {
            role: Role::User,
            content: user,
            tool_call_id: None,
        }],
        max_tokens,
        model_override: None,
        schema: None,
        tools: None,
        tool_choice: None,
    };
    client.complete(req).await.map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Core function
// ---------------------------------------------------------------------------

/// Return structured projection JSON (zero LLM tokens) or `{ "prose": … }` via LLM.
///
/// Two-branch contract (D-03):
/// - `ResolvedTarget::Service(detail)` → `serde_json::to_value(&detail)` — no LLM call.
/// - `ResolvedTarget::Route` / `ResolvedTarget::Model` → prose via `call_llm_prose`.
/// - `ResolvedTarget::NotFound(msg)` → `Err(msg)`.
///
/// No process termination, no coloring, no runtime bridge here (D-04).
pub async fn explain_core(
    target: &str,
    type_override: Option<&str>,
    project_root: &Path,
) -> Result<serde_json::Value, String> {
    match resolve_target(project_root, target, type_override).await {
        // Zero-token branch: ProjectionDetail is Serialize — no LLM call.
        ResolvedTarget::Service(detail) => serde_json::to_value(&detail).map_err(|e| e.to_string()),
        // Prose branches: build prompt from introspected facts, then call LLM.
        ResolvedTarget::Route(r) => {
            let (sys, user) = build_route_prompt(&r);
            let prose = call_llm_prose(sys, user).await?;
            Ok(serde_json::json!({ "prose": prose }))
        }
        ResolvedTarget::Model(m) => {
            let (sys, user) = build_model_prompt(&m);
            let prose = call_llm_prose(sys, user).await?;
            Ok(serde_json::json!({ "prose": prose }))
        }
        ResolvedTarget::NotFound(msg) => Err(msg),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::inspect_projection::FieldInfo;
    use crate::ENV_LOCK;

    // ---- resolve_kind_priority tests (relocated from ferro-cli) ----

    #[test]
    fn explain_resolution_order_service_wins_over_model() {
        assert_eq!(
            resolve_kind_priority(true, false, true, None),
            "service",
            "service must beat model in auto-detect"
        );
    }

    #[test]
    fn explain_resolution_order_type_model_overrides_service() {
        assert_eq!(
            resolve_kind_priority(true, false, true, Some("model")),
            "model",
            "--type model override must win"
        );
    }

    #[test]
    fn explain_resolution_order_not_found() {
        assert_eq!(
            resolve_kind_priority(false, false, false, None),
            "not_found"
        );
    }

    #[test]
    fn explain_resolution_order_route_beats_model() {
        assert_eq!(
            resolve_kind_priority(false, true, true, None),
            "route",
            "route must beat model"
        );
    }

    #[test]
    fn explain_resolution_order_service_wins_over_route() {
        assert_eq!(
            resolve_kind_priority(true, true, false, None),
            "service",
            "service must beat route in auto-detect"
        );
    }

    #[test]
    fn explain_resolution_order_type_service_override() {
        assert_eq!(
            resolve_kind_priority(false, true, false, Some("service")),
            "service",
            "--type service override must win"
        );
    }

    // ---- build_service_prompt tests (relocated from ferro-cli) ----

    #[test]
    fn explain_service_prompt_contains_projection_vocabulary() {
        let detail = ProjectionDetail {
            name: "order_service".to_string(),
            file: "src/projections/order.rs".to_string(),
            service_name: "order".to_string(),
            display_name: Some("Order".to_string()),
            fields: vec![
                FieldInfo {
                    name: "status".to_string(),
                    data_type: "String".to_string(),
                    meaning: "Status".to_string(),
                    readable: true,
                    writable: false,
                },
                FieldInfo {
                    name: "amount".to_string(),
                    data_type: "Float".to_string(),
                    meaning: "Money".to_string(),
                    readable: true,
                    writable: true,
                },
            ],
            relationships: vec!["items".to_string()],
            actions: vec!["submit".to_string(), "approve".to_string()],
            has_state_machine: true,
            intent_hints: vec!["Primary(Process)".to_string()],
        };

        let (system, user) = build_service_prompt(&detail);

        assert!(system.contains("Intent"), "system must mention Intent");
        assert!(
            system.contains("FieldMeaning"),
            "system must mention FieldMeaning"
        );
        assert!(
            system.contains("StateMachine"),
            "system must mention StateMachine"
        );
        assert!(
            user.contains("status"),
            "user must reference field 'status'"
        );
        assert!(
            user.contains("amount"),
            "user must reference field 'amount'"
        );
        assert!(
            user.contains("submit"),
            "user must reference action 'submit'"
        );
        assert!(
            user.contains("approve"),
            "user must reference action 'approve'"
        );
        assert!(
            user.contains("Primary(Process)"),
            "user must reference intent hint"
        );
        assert!(
            user.contains("StateMachine"),
            "user must note state machine presence"
        );
    }

    // ---- build_route_prompt tests (relocated from ferro-cli) ----

    #[test]
    fn explain_route_prompt_references_route_facts() {
        let route = RouteExplanation {
            route: "/orders/{id}".to_string(),
            method: "GET".to_string(),
            purpose: "Display a single order by ID".to_string(),
            business_context: "Order management".to_string(),
            guards: vec!["auth".to_string()],
            related_routes: vec!["GET /orders".to_string()],
            usage_examples: vec!["GET /orders/123".to_string()],
            name: Some("orders.show".to_string()),
            handler: "controllers::orders::show".to_string(),
        };

        let (_system, user) = build_route_prompt(&route);

        assert!(user.contains("/orders/{id}"), "must reference route path");
        assert!(
            user.contains("controllers::orders::show"),
            "must reference handler"
        );
    }

    // ---- build_model_prompt tests (relocated from ferro-cli) ----

    #[test]
    fn explain_model_prompt_references_model_facts() {
        use crate::tools::explain_model::FieldExplanation;

        let model = ModelExplanation {
            model: "Order".to_string(),
            domain_meaning: "A customer purchase order".to_string(),
            table: Some("orders".to_string()),
            fields: vec![FieldExplanation {
                name: "total".to_string(),
                field_type: "Decimal".to_string(),
                meaning: "Order total in euros".to_string(),
                is_primary_key: false,
                is_optional: false,
            }],
            relationships: vec!["Customer".to_string()],
            related_routes: vec!["GET /orders".to_string()],
            common_queries: vec![],
            path: "src/models/order.rs".to_string(),
        };

        let (_system, user) = build_model_prompt(&model);

        assert!(user.contains("Order"), "must reference model name");
        assert!(user.contains("total"), "must reference field name");
    }

    // ---- resolve_max_tokens_with_default tests (relocated from ferro-cli) ----

    #[test]
    fn explain_max_tokens_default_is_2048() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("FERRO_AI_MAX_TOKENS_PER_COMMAND");
        }
        assert_eq!(
            resolve_max_tokens_with_default(2048),
            2048,
            "default must be 2048 when FERRO_AI_MAX_TOKENS_PER_COMMAND is unset"
        );
    }

    #[test]
    fn explain_max_tokens_env_applied() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("FERRO_AI_MAX_TOKENS_PER_COMMAND", "512");
        }
        let result = resolve_max_tokens_with_default(2048);
        unsafe {
            std::env::remove_var("FERRO_AI_MAX_TOKENS_PER_COMMAND");
        }
        assert_eq!(result, 512, "env value must override default");
    }

    // ---- structured branch serialization test (zero LLM tokens) ----

    #[test]
    fn projection_detail_serializes_expected_keys() {
        // Directly test the structured-branch logic: build a ProjectionDetail literal,
        // serialize it, and assert the JSON keys are present.
        // This proves the zero-token path works without requiring FERRO_AI_* env.
        let detail = ProjectionDetail {
            name: "invoice_service".to_string(),
            file: "src/projections/invoice.rs".to_string(),
            service_name: "invoice".to_string(),
            display_name: Some("Invoice".to_string()),
            fields: vec![FieldInfo {
                name: "amount".to_string(),
                data_type: "Decimal".to_string(),
                meaning: "Money".to_string(),
                readable: true,
                writable: false,
            }],
            relationships: vec!["customer".to_string()],
            actions: vec!["send".to_string()],
            has_state_machine: false,
            intent_hints: vec!["Primary(Browse)".to_string()],
        };

        let value = serde_json::to_value(&detail).expect("ProjectionDetail must serialize");

        assert_eq!(value["name"], "invoice_service");
        assert!(value["fields"].is_array(), "fields must be an array");
        assert_eq!(value["has_state_machine"], false);
        assert!(
            value["intent_hints"].is_array(),
            "intent_hints must be an array"
        );
        assert_eq!(value["intent_hints"][0], "Primary(Browse)");
    }
}
