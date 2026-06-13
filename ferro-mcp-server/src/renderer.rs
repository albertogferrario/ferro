use std::collections::HashMap;
use std::sync::Arc;

use ferro_projections::render::Renderer;
use ferro_projections::{Error as ProjError, IntentScore, ServiceDef};
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

/// Renders every MCP-exposed projection in `services` into an MCP tool.
///
/// Projections without `mcp_exposed = true` are skipped (AMCP-01 opt-in filter).
/// The returned tools carry `readOnlyHint = true` and have their `inputSchema`
/// derived from `ServiceDef` fields.
pub fn render_exposed_tools(
    services: &[ServiceDef],
    ctx: &McpContext,
) -> std::result::Result<Vec<Tool>, ProjError> {
    let renderer = McpRenderer;
    services
        .iter()
        .filter(|s| s.mcp_exposed)
        .map(|s| renderer.render(s, &ferro_projections::derive_intents(s), ctx))
        .collect()
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
    /// existing read tool (3 total for this fixture).
    #[test]
    fn test_one_write_tool_per_action() {
        let tools = render_exposed_tools(&[order_service_with_actions()], &McpContext::default())
            .expect("render ok");

        assert_eq!(
            tools.len(),
            3,
            "expected list_order + submit_order + update_notes"
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
}
