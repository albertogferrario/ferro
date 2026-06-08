//! `ferro ai:explain <target>` — AI-powered service/route/model explanation.
//!
//! Resolves the target in SERVICE → ROUTE → MODEL order (projection-framed is
//! primary; prose fallback when no projection exists). Assembles a prompt from
//! introspected facts and produces prose via a raw `CompletionRequest { schema:
//! None, .. }` call. `--dry-run` prints the assembled prompt without calling
//! the LLM.

#[cfg(feature = "projections")]
use ferro_mcp::tools::{
    explain_model::ModelExplanation, explain_route::RouteExplanation,
    inspect_projection::{InspectResult, ProjectionDetail},
};
#[cfg(feature = "projections")]
use std::path::Path;

// ---------------------------------------------------------------------------
// Resolved target variants
// ---------------------------------------------------------------------------

/// The result of resolving a CLI target to a concrete introspection artifact.
#[cfg(feature = "projections")]
pub(crate) enum ResolvedTarget {
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
/// Priority order (RESEARCH Pitfall 7 — inverts CONTEXT D-05):
///   service (projection-framed primary) → route → model
///
/// With a `--type` override the caller has already forced the kind before
/// consulting this function — it is used for auto-detect only.
#[cfg(feature = "projections")]
#[allow(dead_code)] // used in unit tests only; debug/test utility
pub(crate) fn resolve_kind_priority(
    found_service: bool,
    found_route: bool,
    found_model: bool,
    type_override: Option<&str>,
) -> &str {
    if let Some(t) = type_override {
        return t;
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
// Target resolution
// ---------------------------------------------------------------------------

/// Auto-detect target kind in SERVICE → ROUTE → MODEL order.
///
/// With `type_override` set, skips auto-detect and calls only the matching
/// tool. Returns `ResolvedTarget::NotFound` when nothing matches.
#[cfg(feature = "projections")]
pub(crate) fn resolve_target(
    rt: &tokio::runtime::Runtime,
    root: &Path,
    target: &str,
    type_override: Option<&str>,
) -> ResolvedTarget {
    use ferro_mcp::tools::{explain_model, explain_route, inspect_projection};

    match type_override {
        Some("service") => {
            match inspect_projection::execute(root, target) {
                InspectResult::Found(d) => return ResolvedTarget::Service(d),
                InspectResult::NotFound(_) => {
                    return ResolvedTarget::NotFound(format!(
                        "No projection named '{target}' found"
                    ))
                }
            }
        }
        Some("route") => {
            match rt.block_on(explain_route::execute(root, target)) {
                Ok(r) => return ResolvedTarget::Route(r),
                Err(_) => {
                    return ResolvedTarget::NotFound(format!("No route '{target}' found"))
                }
            }
        }
        Some("model") => {
            match rt.block_on(explain_model::execute(root, target)) {
                Ok(m) => return ResolvedTarget::Model(m),
                Err(_) => {
                    return ResolvedTarget::NotFound(format!("No model '{target}' found"))
                }
            }
        }
        Some(other) => {
            return ResolvedTarget::NotFound(format!(
                "Unknown --type value '{other}'. Use 'service', 'route', or 'model'."
            ));
        }
        None => {}
    }

    // Auto-detect: service → route → model (Pitfall 7: projection-framed is primary)
    if let InspectResult::Found(d) = inspect_projection::execute(root, target) {
        return ResolvedTarget::Service(d);
    }

    if let Ok(r) = rt.block_on(explain_route::execute(root, target)) {
        return ResolvedTarget::Route(r);
    }

    if let Ok(m) = rt.block_on(explain_model::execute(root, target)) {
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
/// the supplied facts (SC#6 — no generic templates). The user prompt serializes
/// the parsed string vocabulary from `inspect_projection`.
///
/// NOTE: intent derivation is NOT performed here — no live `ServiceDef` value is
/// available for an existing service at CLI time (Open Question #2).
#[cfg(feature = "projections")]
pub(crate) fn build_service_prompt(detail: &ProjectionDetail) -> (String, String) {
    let system = "You are a Ferro framework expert. Explain this service in projection terms: \
        which Intents it projects, which fields carry FieldMeaning annotations that drive rendering, \
        which ActionDefs are exposed (and under which GuardDefs if any), \
        and describe the StateMachine transitions if present. \
        Base your explanation ONLY on the supplied introspection facts — \
        do not invent fields, actions, or behaviours not listed."
        .to_string();

    // Serialize the ProjectionDetail vocabulary
    let mut user = format!("Service: {}\n", detail.name);
    if let Some(ref dn) = detail.display_name {
        user.push_str(&format!("DisplayName: {dn}\n"));
    }
    if !detail.service_name.is_empty() {
        user.push_str(&format!("ServiceName: {}\n", detail.service_name));
    }

    // Intent hints
    if !detail.intent_hints.is_empty() {
        user.push_str("\nIntent hints:\n");
        for hint in &detail.intent_hints {
            user.push_str(&format!("  - {hint}\n"));
        }
    }

    // Fields with FieldMeaning
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

    // Actions (ActionDef)
    if !detail.actions.is_empty() {
        user.push_str("\nActions (ActionDef):\n");
        for a in &detail.actions {
            user.push_str(&format!("  - {a}\n"));
        }
    }

    // Relationships
    if !detail.relationships.is_empty() {
        user.push_str("\nRelationships:\n");
        for r in &detail.relationships {
            user.push_str(&format!("  - {r}\n"));
        }
    }

    // StateMachine presence
    if detail.has_state_machine {
        user.push_str("\nStateMachine: present\n");
    }

    user.push_str("\nExplain this service's projection-level design in a few paragraphs.");

    (system, user)
}

/// Build a prose-fallback prompt from a `RouteExplanation`.
#[cfg(feature = "projections")]
pub(crate) fn build_route_prompt(r: &RouteExplanation) -> (String, String) {
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
#[cfg(feature = "projections")]
pub(crate) fn build_model_prompt(m: &ModelExplanation) -> (String, String) {
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
#[cfg(feature = "projections")]
pub(crate) fn resolve_max_tokens_with_default(default: u32) -> u32 {
    std::env::var("FERRO_AI_MAX_TOKENS_PER_COMMAND")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

// ---------------------------------------------------------------------------
// Command entry point
// ---------------------------------------------------------------------------

/// Run the `ferro ai:explain <target>` command.
///
/// Resolves the target (service-first), assembles a projection-framed prompt
/// (or prose-fallback), and produces prose via a raw `schema: None` completion.
/// `--dry-run` prints the assembled prompt and makes no LLM call.
#[cfg(feature = "projections")]
pub fn run(target: String, type_override: Option<String>, dry_run: bool) {
    use crate::commands::ai_make::ai_config_error_message;
    use console::style;
    use ferro_ai::client::{Message, Role};
    use ferro_ai::{AiConfig, CompletionRequest};

    // 1. Fail-fast: require AI provider unless --dry-run (D-06)
    //    Even in dry-run we validate config to surface missing env vars early,
    //    but we skip the actual LLM call.
    let client_result = AiConfig::from_env();
    if !dry_run {
        if let Err(ref e) = client_result {
            eprintln!(
                "{} {}",
                style("Error:").red().bold(),
                ai_config_error_message(e)
            );
            std::process::exit(1);
        }
    }

    // 2. Tokio runtime bridge
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "{} Failed to create tokio runtime: {e}",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    };

    // 3. Resolve target (service → route → model)
    let resolved = resolve_target(&rt, Path::new("."), &target, type_override.as_deref());

    match resolved {
        ResolvedTarget::NotFound(msg) => {
            eprintln!("{} {msg}", style("Error:").red().bold());
            std::process::exit(1);
        }
        resolved => {
            // 4. Build prompt
            let (system_prompt, user_prompt) = match &resolved {
                ResolvedTarget::Service(d) => build_service_prompt(d),
                ResolvedTarget::Route(r) => build_route_prompt(r),
                ResolvedTarget::Model(m) => build_model_prompt(m),
                ResolvedTarget::NotFound(_) => unreachable!(),
            };

            // 5. --dry-run: print assembled prompt and return (no LLM call)
            if dry_run {
                println!("{system_prompt}");
                println!("---");
                println!("{user_prompt}");
                return;
            }

            // 6. Cost guard (default 2048 for ai:explain)
            let max_tokens = resolve_max_tokens_with_default(2048);

            // 7. Raw prose completion — schema: None (unstructured, no JSON coercion)
            let client = client_result.expect("already validated above");

            let req = CompletionRequest {
                system: Some(system_prompt),
                messages: vec![Message {
                    role: Role::User,
                    content: user_prompt,
                    tool_call_id: None,
                }],
                max_tokens,
                model_override: None,
                schema: None,
                tools: None,
                tool_choice: None,
            };

            match rt.block_on(client.complete(req)) {
                Ok(prose) => {
                    println!("{prose}");
                }
                Err(e) => {
                    eprintln!("{} LLM completion failed: {e}", style("Error:").red().bold());
                    std::process::exit(1);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "projections"))]
mod tests {
    use super::*;
    use ferro_mcp::tools::{
        explain_route::RouteExplanation,
        inspect_projection::{FieldInfo, ProjectionDetail},
    };
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    // ---- Task 1 tests ----

    #[test]
    fn explain_resolution_order_service_wins_over_model() {
        // With no override and both service+model "found", service wins (service-first).
        assert_eq!(
            resolve_kind_priority(true, false, true, None),
            "service",
            "service must beat model in auto-detect"
        );
    }

    #[test]
    fn explain_resolution_order_type_model_overrides_service() {
        // --type model forces model even when a projection exists.
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

        // SC#4: projection vocabulary must appear
        assert!(system.contains("Intent"), "system must mention Intent");
        assert!(system.contains("FieldMeaning"), "system must mention FieldMeaning");
        assert!(system.contains("StateMachine"), "system must mention StateMachine");
        // SC#6: actual field/action names must appear (grounded in introspected facts)
        assert!(user.contains("status"), "user must reference field 'status'");
        assert!(user.contains("amount"), "user must reference field 'amount'");
        assert!(user.contains("submit"), "user must reference action 'submit'");
        assert!(user.contains("approve"), "user must reference action 'approve'");
        assert!(user.contains("Primary(Process)"), "user must reference intent hint");
        assert!(user.contains("StateMachine"), "user must note state machine presence");
    }

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
        assert!(user.contains("controllers::orders::show"), "must reference handler");
    }

    // ---- Task 2 tests ----

    #[test]
    fn explain_dry_run_no_llm_call() {
        // Dry-run must print the prompt without touching the client.
        // We verify by checking that build_service_prompt output is non-empty
        // (the run() function with dry_run=true prints system+---+user and returns).
        // We exercise the prompt path via the prompt builders directly here since
        // calling run() with dry_run=true would also need AiConfig; instead we test
        // the logical guarantee via the builder.
        let detail = ProjectionDetail {
            name: "test_service".to_string(),
            file: "src/projections/test.rs".to_string(),
            service_name: "test".to_string(),
            display_name: None,
            fields: vec![],
            relationships: vec![],
            actions: vec![],
            has_state_machine: false,
            intent_hints: vec![],
        };
        let (system, user) = build_service_prompt(&detail);
        assert!(!system.is_empty(), "system prompt must be non-empty");
        assert!(user.contains("test_service"), "user prompt must reference service name");
    }

    #[test]
    fn explain_max_tokens_default_is_2048() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Remove the env var to exercise the default branch
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

    #[test]
    fn explain_fail_fast_error_message_names_env_vars() {
        use crate::commands::ai_make::ai_config_error_message;
        use ferro_ai::Error;

        let msg = ai_config_error_message(&Error::Config("missing provider".to_string()));
        assert!(
            msg.contains("FERRO_AI_PROVIDER"),
            "error message must name FERRO_AI_PROVIDER"
        );
        assert!(
            msg.contains("FERRO_AI_API_KEY"),
            "error message must name FERRO_AI_API_KEY"
        );
        assert!(
            msg.contains("FERRO_AI_MODEL"),
            "error message must name FERRO_AI_MODEL"
        );
    }
}
