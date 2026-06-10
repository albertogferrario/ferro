use std::sync::Arc;

use ferro_projections::render::Renderer;
use ferro_projections::{Error as ProjError, IntentScore, ServiceDef};
use rmcp::model::{Tool, ToolAnnotations};

/// Context for MCP rendering. Carries no state in Phase 197;
/// Phase 200 will extend with tenant/policy context.
#[derive(Debug, Clone, Default)]
pub struct McpContext;

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
    use ferro_projections::{derive_intents, DataType, FieldMeaning, ServiceDef};

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
            .render(service, &intents, &McpContext)
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
        let tools = render_exposed_tools(&services, &McpContext).expect("render ok");

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
}
