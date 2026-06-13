use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use ferro_projections::render::Renderer;
use ferro_projections::{ActionDef, Error as ProjError, IntentScore, ServiceDef};
use rmcp::model::{Tool, ToolAnnotations};

/// Per-request MCP context — tenant identity and evaluated permission guards.
///
/// `tenant_id`: resolved from the auth credential (JWT or API key); `None` =
/// unauthenticated or single-tenant (dispatch fails closed if the projection
/// requires a tenant).
/// `evaluated_guards`: populated in Phase 218/219; absent key = allow,
/// explicit `false` = deny (same semantics as `BaseContext`). Empty in 217.
/// `scope`: credential scope. `None` = OAuth JWT path (full access);
/// `"read"` = read-only key; `"read_write"` = full key.
#[derive(Debug, Clone, Default)]
pub struct McpContext {
    pub tenant_id: Option<i64>,
    pub evaluated_guards: HashMap<String, bool>,
    pub scope: Option<String>,
}

/// Renders a `ServiceDef` projection into an MCP tool definition.
///
/// The tool name is `list_<service.name>`. The `inputSchema` is derived
/// entirely from the projection's fields via [`crate::schema::build_input_schema`]
/// — there is no separately declared schema (AMCP-02).
pub struct McpRenderer;

impl Renderer for McpRenderer {
    type Output = Tool;
    type Context = McpContext;

    fn render(
        &self,
        service: &ServiceDef,
        _intents: &[IntentScore],
        _ctx: &McpContext,
    ) -> std::result::Result<Tool, ProjError> {
        let name = format!("list_{}", service.name);
        let description = service.description.clone().unwrap_or_else(|| {
            format!(
                "List {} records",
                service.display_name.as_deref().unwrap_or(&service.name)
            )
        });

        let schema_value = crate::schema::build_input_schema(service)
            .map_err(|e| ProjError::Render(e.to_string()))?;

        let schema_map = match schema_value {
            serde_json::Value::Object(m) => m,
            _ => return Err(ProjError::Render("inputSchema must be an object".into())),
        };

        let annotations = ToolAnnotations::new().read_only(true);

        Ok(Tool::new(name, description, Arc::new(schema_map)).annotate(annotations))
    }
}

/// Renders every MCP-exposed projection in `services` into MCP tools.
///
/// Projections without `mcp_exposed = true` are skipped (AMCP-01 opt-in filter).
/// For each exposed service, emits the `list_<service>` read tool first, then one
/// write tool per `ActionDef` in declaration order (guard-filtered). See
/// `render_action_tool` for write-tool semantics.
pub fn render_exposed_tools(
    services: &[ServiceDef],
    ctx: &McpContext,
) -> std::result::Result<Vec<Tool>, ProjError> {
    let renderer = McpRenderer;
    // Collect (service_name, Tool) pairs so the collision pass can rename by service.
    let mut tagged: Vec<(String, Tool)> = Vec::new();

    for service in services.iter().filter(|s| s.mcp_exposed) {
        // Read tool first (existing behavior, always named list_<service>).
        let read_tool =
            renderer.render(service, &ferro_projections::derive_intents(service), ctx)?;
        tagged.push((service.name.clone(), read_tool));

        // Then one write tool per ActionDef, in declaration order, guard-filtered.
        for action in &service.actions {
            if let Some(tool) = render_action_tool(service, action, ctx)? {
                tagged.push((service.name.clone(), tool));
            }
        }
    }

    // D-01 collision pass: write tools whose bare action.name collides across services
    // are renamed to <action.name>_on_<service.name>. Read tools (list_*) are never renamed.
    disambiguate_write_tool_collisions(&mut tagged);

    // Phase 220: synthesize request_confirm_<name> + confirm_<name> for each
    // destructive action AFTER the disambiguation pass, using the post-disambiguation
    // name as base (Pitfall 3 fix — routing via strip_prefix must return a valid name).
    #[cfg(feature = "confirmation")]
    {
        // Collect (service_name, base_name, action) for destructive tools.
        // We snapshot after disambiguation so the confirm tools use the final names.
        let destructive: Vec<(String, String, ferro_projections::ActionDef)> = tagged
            .iter()
            .filter_map(|(svc_name, tool)| {
                // Find the original action via the post-disambiguation tool name.
                // The disambiguated name is either bare or <action>_on_<service>;
                // find_action_by_tool_name locates the ActionDef for routing.
                // We need the ActionDef to build the inputSchema and description.
                services
                    .iter()
                    .filter(|s| s.mcp_exposed && s.name == *svc_name)
                    .flat_map(|s| s.actions.iter())
                    .find(|a| {
                        // Match by original name or disambiguated name.
                        let disambiguated = format!("{}_on_{}", a.name, svc_name);
                        tool.name.as_ref() == a.name || tool.name.as_ref() == disambiguated
                    })
                    .filter(|a| a.transition_trigger.is_some())
                    .map(|a| (svc_name.clone(), tool.name.to_string(), a.clone()))
            })
            .collect();

        for (svc_name, base_name, action) in destructive {
            if let Some(req_tool) =
                render_request_confirm_tool(&base_name, &action, services, &svc_name, ctx)?
            {
                tagged.push((svc_name.clone(), req_tool));
            }
            if let Some(cfm_tool) =
                render_confirm_tool(&base_name, &action, services, &svc_name, ctx)?
            {
                tagged.push((svc_name.clone(), cfm_tool));
            }
        }
    }

    Ok(tagged.into_iter().map(|(_, t)| t).collect())
}

/// Renames write tools whose name collides across services to `<name>_on_<service>`.
///
/// Read tools (names starting with `list_`) are excluded from collision detection
/// and renaming — they are already unique per service (D-01 / ARCHITECTURE Decision (b)).
fn disambiguate_write_tool_collisions(tagged: &mut [(String, Tool)]) {
    // Count how many distinct services each write tool name appears in.
    let mut name_to_services: HashMap<String, HashSet<String>> = HashMap::new();
    for (service_name, tool) in tagged.iter() {
        if !tool.name.starts_with("list_") {
            name_to_services
                .entry(tool.name.to_string())
                .or_default()
                .insert(service_name.clone());
        }
    }

    // Rename colliding write tools: <action.name>_on_<service.name>.
    // Only rename when the name appears in MORE THAN ONE distinct service —
    // intra-service duplicate action names are an authoring error, not a
    // cross-service collision, and cannot be disambiguated by this pass.
    for (service_name, tool) in tagged.iter_mut() {
        if !tool.name.starts_with("list_")
            && name_to_services
                .get(tool.name.as_ref())
                .map_or(0, |s| s.len())
                > 1
        {
            let new_name = format!("{}_on_{}", tool.name, service_name);
            tool.name = new_name.into();
        }
    }
}

/// Renders one write tool from an `ActionDef`, or `None` if any precondition guard
/// evaluates to explicit `false` for the calling tenant (D-03).
///
/// This guard check is a VISIBILITY filter, NOT an authorization gate — a hidden tool is
/// simply not listed, not "uncallable". Server-side guard enforcement is Phase 219; the
/// 217 scope gate is the read/write boundary. Do not treat this as the security boundary.
fn render_action_tool(
    service: &ServiceDef,
    action: &ActionDef,
    ctx: &McpContext,
) -> std::result::Result<Option<Tool>, ProjError> {
    for precondition in &action.preconditions {
        if ctx.evaluated_guards.get(precondition) == Some(&false) {
            return Ok(None);
        }
    }

    let name = action.name.clone(); // D-01: verbatim, never starts with "list_"
    let description = action
        .description
        .clone()
        .or_else(|| action.display_name.clone())
        .unwrap_or_else(|| format!("{} {}", action.name, service.name));

    let schema_value = crate::schema::build_action_input_schema(action, service)
        .map_err(|e| ProjError::Render(e.to_string()))?;
    let schema_map = match schema_value {
        serde_json::Value::Object(m) => m,
        _ => {
            return Err(ProjError::Render(
                "action inputSchema must be an object".into(),
            ))
        }
    };

    // NOTE: destructive_hint defaults to true when absent in rmcp — always set it explicitly (D-04).
    // NOTE: write-tool calls currently return -32601 (no executor until Phase 219) — correct for 218.
    let annotations = ToolAnnotations::new()
        .read_only(false)
        .destructive(action.transition_trigger.is_some()); // D-04

    Ok(Some(
        Tool::new(name, description, Arc::new(schema_map)).annotate(annotations),
    ))
}

/// Renders the `request_confirm_<name>` tool for a destructive action.
///
/// Uses the post-disambiguation `base_name` (not `action.name` directly) so the
/// `strip_prefix("request_confirm_")` routing in `handle_write_call` returns a
/// valid action name (Pitfall 3). Schema is identical to the bare action tool.
/// `destructiveHint=false` — this step only issues a token.
#[cfg(feature = "confirmation")]
fn render_request_confirm_tool(
    base_name: &str,
    action: &ferro_projections::ActionDef,
    services: &[ferro_projections::ServiceDef],
    service_name: &str,
    ctx: &McpContext,
) -> std::result::Result<Option<Tool>, ProjError> {
    // Apply the same guard-visibility filter as the bare action tool.
    for precondition in &action.preconditions {
        if ctx.evaluated_guards.get(precondition) == Some(&false) {
            return Ok(None);
        }
    }

    // Find the owning service to build the full action schema (includes identifier field).
    let service = services
        .iter()
        .find(|s| s.name == service_name && s.mcp_exposed)
        .ok_or_else(|| ProjError::Render(format!("service '{service_name}' not found")))?;

    let name = format!("request_confirm_{base_name}");
    let description = format!(
        "Request confirmation to: {}",
        action
            .description
            .as_deref()
            .or(action.display_name.as_deref())
            .unwrap_or(&action.name)
    );

    let schema_value = crate::schema::build_action_input_schema(action, service)
        .map_err(|e| ProjError::Render(e.to_string()))?;
    let schema_map = match schema_value {
        serde_json::Value::Object(m) => m,
        _ => {
            return Err(ProjError::Render(
                "action inputSchema must be an object".into(),
            ))
        }
    };

    let annotations = ToolAnnotations::new().read_only(false).destructive(false); // request step issues token only, not destructive

    Ok(Some(
        Tool::new(name, description, Arc::new(schema_map)).annotate(annotations),
    ))
}

/// Renders the `confirm_<name>` tool for a destructive action.
///
/// Schema: `{ "confirmation_token": string, "id": integer }` only — the agent
/// supplies the token returned by `request_confirm_<name>` and the record id
/// for the binding mismatch check. `destructiveHint=true` — this step executes.
#[cfg(feature = "confirmation")]
fn render_confirm_tool(
    base_name: &str,
    action: &ferro_projections::ActionDef,
    _services: &[ferro_projections::ServiceDef],
    _service_name: &str,
    ctx: &McpContext,
) -> std::result::Result<Option<Tool>, ProjError> {
    // Apply the same guard-visibility filter.
    for precondition in &action.preconditions {
        if ctx.evaluated_guards.get(precondition) == Some(&false) {
            return Ok(None);
        }
    }

    let name = format!("confirm_{base_name}");
    let description = format!(
        "Confirm and execute: {}. Supply the confirmation_token from request_confirm_{base_name}.",
        action
            .description
            .as_deref()
            .or(action.display_name.as_deref())
            .unwrap_or(&action.name)
    );

    // Minimal schema: token + record id for binding check.
    let mut schema = serde_json::Map::new();
    schema.insert("type".to_string(), serde_json::json!("object"));
    let mut props = serde_json::Map::new();
    props.insert(
        "confirmation_token".to_string(),
        serde_json::json!({ "type": "string", "description": "Token returned by request_confirm" }),
    );
    props.insert(
        "id".to_string(),
        serde_json::json!({ "type": "integer", "description": "Record id (must match the one used in request_confirm)" }),
    );
    schema.insert("properties".to_string(), serde_json::Value::Object(props));
    schema.insert(
        "required".to_string(),
        serde_json::json!(["confirmation_token", "id"]),
    );

    let annotations = ToolAnnotations::new().read_only(false).destructive(true); // confirm step executes the destructive action

    Ok(Some(
        Tool::new(name, description, Arc::new(schema)).annotate(annotations),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::{
        derive_intents, ActionDef, DataType, FieldMeaning, InputDef, ServiceDef,
    };

    fn order_service() -> ServiceDef {
        ServiceDef::new("order")
            .display_name("Order")
            .description("Manages customer orders")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
    }

    fn render_service(service: &ServiceDef) -> Tool {
        let renderer = McpRenderer;
        let intents = derive_intents(service);
        renderer
            .render(service, &intents, &McpContext::default())
            .expect("render ok")
    }

    #[test]
    fn test_render_tool_name() {
        let tool = render_service(&order_service());
        assert_eq!(tool.name.as_ref(), "list_order");
    }

    #[test]
    fn test_render_read_only() {
        let tool = render_service(&order_service());
        let annotations = tool.annotations.expect("annotations present");
        assert_eq!(
            annotations.read_only_hint,
            Some(true),
            "readOnlyHint must be true"
        );
    }

    #[test]
    fn test_render_schema_embedded() {
        let tool = render_service(&order_service());
        // input_schema is Arc<Map<String, Value>>; access via deref
        let props = tool
            .input_schema
            .get("properties")
            .and_then(|v| v.as_object())
            .expect("properties object");
        assert!(props.contains_key("limit"), "limit missing from schema");
        assert!(
            props.contains_key("status"),
            "filter field 'status' missing"
        );
    }

    #[test]
    fn test_mcp_exposed_filter() {
        let exposed = ServiceDef::new("product").mcp_exposed(true).field(
            "id",
            DataType::Integer,
            FieldMeaning::Identifier,
        );
        let hidden = ServiceDef::new("internal_log").field(
            "id",
            DataType::Integer,
            FieldMeaning::Identifier,
        );

        let services = vec![exposed, hidden];
        let tools = render_exposed_tools(&services, &McpContext::default()).expect("render ok");

        assert_eq!(
            tools.len(),
            1,
            "exactly one tool for the exposed projection"
        );
        assert_eq!(tools[0].name.as_ref(), "list_product");
    }

    #[test]
    fn adding_field_changes_schema() {
        let base = ServiceDef::new("item")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status);

        let extended =
            base.clone()
                .field("category_id", DataType::Integer, FieldMeaning::ForeignKey);

        let tool_base = render_service(&base);
        let tool_ext = render_service(&extended);

        let count_base = tool_base
            .input_schema
            .get("properties")
            .and_then(|v| v.as_object())
            .map(|m| m.len())
            .expect("properties");

        let count_ext = tool_ext
            .input_schema
            .get("properties")
            .and_then(|v| v.as_object())
            .map(|m| m.len())
            .expect("properties");

        assert!(
            count_ext > count_base,
            "adding a filter field must increase inputSchema property count: base={count_base} ext={count_ext}"
        );
    }

    // -------------------------------------------------------------------------
    // Phase 218 RED tests for write-tool emission (SC#1, SC#3, SC#4, T-218-02).
    //
    // These tests assert behaviour that the un-extended `render_exposed_tools`
    // does NOT yet produce — currently it emits only the `list_order` read tool,
    // so write-tool lookups return None and every assertion below fails.
    // The renderer extension is implemented in Plan 02; these tests turn GREEN
    // there.
    // -------------------------------------------------------------------------

    /// Fixture: one mcp_exposed service with two actions.
    /// - submit_order: guarded by "has_items", has a transition_trigger (→ destructive).
    /// - update_notes: no guard, no transition_trigger (→ non-destructive).
    fn order_service_with_actions() -> ServiceDef {
        ServiceDef::new("order")
            .display_name("Order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .action(
                ActionDef::new("submit_order")
                    .description("Submit an order for processing")
                    .input(
                        InputDef::new("notes", DataType::String, FieldMeaning::FreeText)
                            .required(false),
                    )
                    .precondition("has_items")
                    .transition_trigger("submit"),
            )
            .action(ActionDef::new("update_notes").input(InputDef::new(
                "notes",
                DataType::String,
                FieldMeaning::FreeText,
            )))
    }

    /// SC#1: render_exposed_tools must emit one write tool per ActionDef plus the
    /// existing read tool. With the confirmation feature on, two extra confirm tools
    /// are synthesized for the destructive `submit_order` action (5 total).
    /// Without the confirmation feature, 3 tools.
    #[test]
    fn test_one_write_tool_per_action() {
        let tools = render_exposed_tools(&[order_service_with_actions()], &McpContext::default())
            .expect("render ok");

        // Feature off: 1 read + 2 write = 3. Feature on: + request_confirm_ + confirm_ = 5.
        #[cfg(not(feature = "confirmation"))]
        assert_eq!(
            tools.len(),
            3,
            "expected list_order + submit_order + update_notes; got {}",
            tools.len()
        );
        #[cfg(feature = "confirmation")]
        assert_eq!(
            tools.len(),
            5,
            "expected list_order + submit_order + update_notes + request_confirm_submit_order + confirm_submit_order; got {}",
            tools.len()
        );

        assert!(
            tools.iter().any(|t| t.name.as_ref() == "list_order"),
            "read tool 'list_order' must be present"
        );
        assert!(
            tools.iter().any(|t| t.name.as_ref() == "submit_order"),
            "write tool 'submit_order' must be present"
        );
        assert!(
            tools.iter().any(|t| t.name.as_ref() == "update_notes"),
            "write tool 'update_notes' must be present"
        );
        #[cfg(feature = "confirmation")]
        {
            assert!(
                tools
                    .iter()
                    .any(|t| t.name.as_ref() == "request_confirm_submit_order"),
                "request_confirm_submit_order must be present"
            );
            assert!(
                tools
                    .iter()
                    .any(|t| t.name.as_ref() == "confirm_submit_order"),
                "confirm_submit_order must be present"
            );
        }
    }

    /// SC#4: A write tool for an action with transition_trigger must carry
    /// readOnlyHint=false and destructiveHint=true.
    #[test]
    fn test_write_tool_annotations_transition() {
        let tools = render_exposed_tools(&[order_service_with_actions()], &McpContext::default())
            .expect("render ok");

        let submit = tools
            .iter()
            .find(|t| t.name.as_ref() == "submit_order")
            .expect("submit_order tool must be present");

        let ann = submit
            .annotations
            .as_ref()
            .expect("annotations must be present on write tool");

        assert_eq!(
            ann.read_only_hint,
            Some(false),
            "write tool readOnlyHint must be false"
        );
        assert_eq!(
            ann.destructive_hint,
            Some(true),
            "transition action destructiveHint must be true"
        );
    }

    /// SC#4: A write tool for an action WITHOUT transition_trigger must carry
    /// readOnlyHint=false and destructiveHint=false.
    #[test]
    fn test_write_tool_annotations_non_transition() {
        let tools = render_exposed_tools(&[order_service_with_actions()], &McpContext::default())
            .expect("render ok");

        let update = tools
            .iter()
            .find(|t| t.name.as_ref() == "update_notes")
            .expect("update_notes tool must be present");

        let ann = update
            .annotations
            .as_ref()
            .expect("annotations must be present on write tool");

        assert_eq!(
            ann.read_only_hint,
            Some(false),
            "write tool readOnlyHint must be false"
        );
        assert_eq!(
            ann.destructive_hint,
            Some(false),
            "non-transition action destructiveHint must be false"
        );
    }

    /// SC#3 / T-218-02: A tool whose precondition guard evaluates to explicit
    /// false must be OMITTED from tools/list. (Visibility filter, not auth gate.)
    #[test]
    fn test_guard_false_omits_tool() {
        let mut ctx = McpContext::default();
        ctx.evaluated_guards.insert("has_items".to_string(), false);

        let tools = render_exposed_tools(&[order_service_with_actions()], &ctx).expect("render ok");

        assert!(
            !tools.iter().any(|t| t.name.as_ref() == "submit_order"),
            "submit_order must be omitted when has_items guard = false"
        );
        assert!(
            tools.iter().any(|t| t.name.as_ref() == "update_notes"),
            "update_notes (no guard) must still be present"
        );
        assert!(
            tools.iter().any(|t| t.name.as_ref() == "list_order"),
            "read tool list_order must still be present"
        );
    }

    /// SC#3: A tool whose precondition guard evaluates to explicit true must be
    /// present in tools/list.
    #[test]
    fn test_guard_true_includes_tool() {
        let mut ctx = McpContext::default();
        ctx.evaluated_guards.insert("has_items".to_string(), true);

        let tools = render_exposed_tools(&[order_service_with_actions()], &ctx).expect("render ok");

        assert!(
            tools.iter().any(|t| t.name.as_ref() == "submit_order"),
            "submit_order must be present when has_items guard = true"
        );
    }

    /// SC#3: When the guard key is absent (McpContext::default()), the tool must
    /// be present (absent key = allow, same semantics as BaseContext).
    #[test]
    fn test_guard_absent_includes_tool() {
        let tools = render_exposed_tools(&[order_service_with_actions()], &McpContext::default())
            .expect("render ok");

        assert!(
            tools.iter().any(|t| t.name.as_ref() == "submit_order"),
            "submit_order must be present when guard key is absent"
        );
    }

    /// WR-01 regression: cross-service collision renames both tools;
    /// non-colliding tools from the same services are left untouched.
    #[test]
    fn test_collision_rename_across_services() {
        let svc_invoice = ServiceDef::new("invoice")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .action(ActionDef::new("approve"));

        let svc_refund = ServiceDef::new("refund")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .action(ActionDef::new("approve"))
            .action(ActionDef::new("cancel")); // non-colliding, must not be renamed

        let tools = render_exposed_tools(&[svc_invoice, svc_refund], &McpContext::default())
            .expect("render ok");

        let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(
            names.contains(&"approve_on_invoice"),
            "invoice approve must be renamed; got: {names:?}"
        );
        assert!(
            names.contains(&"approve_on_refund"),
            "refund approve must be renamed; got: {names:?}"
        );
        assert!(
            !names.contains(&"approve"),
            "bare 'approve' must not appear after rename; got: {names:?}"
        );
        assert!(
            names.contains(&"cancel"),
            "non-colliding 'cancel' must not be renamed; got: {names:?}"
        );
    }

    /// WR-01 regression: intra-service duplicate action names (authoring error)
    /// are NOT renamed by the cross-service pass — one distinct service means
    /// no cross-service collision by definition.
    #[test]
    fn test_intra_service_duplicate_not_renamed() {
        // Two actions with the same name in ONE service — malformed projection,
        // but the collision pass must not touch them (can't disambiguate).
        let svc = ServiceDef::new("order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .action(ActionDef::new("submit"))
            .action(ActionDef::new("submit")); // duplicate within same service

        let tools = render_exposed_tools(&[svc], &McpContext::default()).expect("render ok");

        let submit_count = tools.iter().filter(|t| t.name.as_ref() == "submit").count();
        assert_eq!(
            submit_count,
            2,
            "intra-service duplicates must remain unrenamed (both keep bare 'submit'); got: {:?}",
            tools.iter().map(|t| t.name.as_ref()).collect::<Vec<_>>()
        );
    }
}
