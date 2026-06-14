//! NL intent loop types and helpers.
//!
//! Provides `ToolSelection` (the classifier output type) and
//! `render_tool_descriptions` (a text formatter for the classifier system prompt).
//!
//! Gated by the `ai` Cargo feature. The `process_nl_turn` function that ties
//! classification to dispatch is added in Plan 02.

use crate::renderer::{render_exposed_tools, McpContext};
use ferro_projections::ServiceDef;
use serde::{Deserialize, Serialize};
use serde_json::Map;

/// Classifier output for a single conversational turn.
///
/// Defined here (not in ferro-ai) because it is projection-specific (D-01).
/// The JSON schema field names match the serde representation (snake_case).
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ToolSelection {
    /// The name of the MCP tool to invoke.
    pub tool_name: String,
    /// Arguments to pass to the tool (matches the tool's input schema).
    pub arguments: Map<String, serde_json::Value>,
    /// Classifier confidence score in [0.0, 1.0].
    pub confidence: f64,
}

/// Format the guard-filtered tool list as a classifier system-prompt block.
///
/// Calls `render_exposed_tools` (not a second projection renderer) and formats
/// the resulting `Vec<Tool>` as a concise text block: one line per tool with
/// name, description, and input property names (no full type annotations).
pub fn render_tool_descriptions(
    services: &[ServiceDef],
    ctx: &McpContext,
) -> Result<String, ferro_projections::Error> {
    let tools = render_exposed_tools(services, ctx)?;
    let lines: Vec<String> = tools
        .iter()
        .map(|t| {
            let props: Vec<&str> = t
                .input_schema
                .get("properties")
                .and_then(|v| v.as_object())
                .map(|m| m.keys().map(|k| k.as_str()).collect())
                .unwrap_or_default();
            format!(
                "- {}: {} [args: {}]",
                t.name,
                t.description.as_deref().unwrap_or(""),
                props.join(", ")
            )
        })
        .collect();
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::{ActionDef, DataType, FieldMeaning, InputDef, ServiceDef};

    /// ToolSelection round-trips through its snake_case JSON schema.
    #[test]
    fn tool_selection_roundtrip_snake_case() {
        let json = serde_json::json!({
            "tool_name": "list_order",
            "arguments": { "limit": 10 },
            "confidence": 0.95
        });
        let sel: ToolSelection =
            serde_json::from_value(json.clone()).expect("must deserialize");
        assert_eq!(sel.tool_name, "list_order");
        assert_eq!(sel.confidence, 0.95);
        let re_serialized = serde_json::to_value(&sel).expect("must serialize");
        assert_eq!(
            re_serialized.get("tool_name").and_then(|v| v.as_str()),
            Some("list_order"),
            "tool_name key must be snake_case"
        );
        assert_eq!(
            re_serialized.get("confidence").and_then(|v| v.as_f64()),
            Some(0.95)
        );
    }

    /// The JSON schema for ToolSelection uses snake_case property names.
    #[test]
    fn tool_selection_schema_uses_snake_case() {
        // Build a sample object and verify round-trip using snake_case keys.
        let json = serde_json::json!({
            "tool_name": "approve",
            "arguments": {},
            "confidence": 0.9
        });
        // Deserialize succeeds (snake_case matches serde representation).
        let result = serde_json::from_value::<ToolSelection>(json);
        assert!(result.is_ok(), "snake_case keys must deserialize: {:?}", result.err());
        // camelCase keys must NOT deserialize (schema mismatch guard).
        let camel = serde_json::json!({
            "toolName": "approve",
            "arguments": {},
            "confidence": 0.9
        });
        let bad = serde_json::from_value::<ToolSelection>(camel);
        assert!(
            bad.is_err(),
            "camelCase tool_name key must not deserialize"
        );
    }

    /// render_tool_descriptions returns a non-empty string containing each
    /// exposed tool's name for a guard-passing context.
    #[test]
    fn render_tool_descriptions_includes_tool_names() {
        let svc = ServiceDef::new("order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status);

        let ctx = McpContext::default();
        let text =
            render_tool_descriptions(&[svc], &ctx).expect("must render");

        assert!(!text.is_empty(), "output must not be empty");
        assert!(
            text.contains("list_order"),
            "must contain the read tool name; got: {text}"
        );
    }

    /// render_tool_descriptions includes the [args: ...] property listing for a
    /// tool with input properties.
    #[test]
    fn render_tool_descriptions_includes_args_listing() {
        let svc = ServiceDef::new("order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .action(
                ActionDef::new("approve")
                    .input(InputDef::new("id", DataType::Integer, FieldMeaning::Identifier)),
            );

        let ctx = McpContext::default();
        let text =
            render_tool_descriptions(&[svc], &ctx).expect("must render");

        assert!(
            text.contains("[args:"),
            "must contain [args: ...] listing; got: {text}"
        );
    }
}
