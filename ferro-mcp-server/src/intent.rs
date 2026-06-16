//! NL intent loop types and helpers.
//!
//! Provides `ToolSelection` (the classifier output type),
//! `render_tool_descriptions` (a text formatter for the classifier system prompt),
//! and `process_nl_turn` (the conversational-turn core that classifies an NL
//! message and routes it through the existing read/write/confirm/clarify machinery).
//!
//! Gated by the `ai` Cargo feature.

use crate::renderer::{render_exposed_tools, McpContext};
use ferro_projections::ServiceDef;
use serde::{Deserialize, Serialize};
use serde_json::Map;

#[cfg(feature = "ai")]
use crate::jsonrpc::handle_tools_call;
#[cfg(feature = "ai")]
use crate::write_dispatch::handle_write_call;
#[cfg(feature = "ai")]
use crate::write_dispatch::write_tool_error_result;
#[cfg(feature = "ai")]
use crate::WriteDispatcher;
#[cfg(feature = "ai")]
use sea_orm::DatabaseConnection;
#[cfg(feature = "ai")]
use serde_json::json;
#[cfg(feature = "ai")]
use std::sync::Arc;

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

/// Classify an NL message and route it through the existing read/write/confirm/clarify machinery.
///
/// This is the conversational-turn core for AMCP-06. It composes existing entry
/// points and adds zero new dispatch, guard, confirmation, or envelope logic:
///
/// - `list_*` tool names route to the read path (`handle_tools_call`), but only
///   after the `authorize_read` policy gate passes (see below).
/// - All other tool names route to `handle_write_call`, which owns scope check,
///   guard re-evaluation, idempotency, the D-08 confirmation seam, execute, and audit.
/// - `Error::LowConfidence` maps to a `needs_clarification` structured response
///   without invoking any dispatch path (SC#5).
///
/// The classifier output (`ToolSelection`) is UNTRUSTED model output (prompt-injection
/// surface). It enters the identical `tools/call` pipeline as any direct call — no
/// trust shortcut (SC#1). `tenant_id` is derived from the authenticated principal
/// param, never from the classified arguments (T-221-07).
///
/// `authorize_read` is the app-level ability gate for READ tools, mirroring the
/// direct `/mcp` path's `Gate::authorize_for` + `mcp_ability` fail-closed check
/// (AMCP-11). After classification resolves a `list_*` tool to its `ServiceDef`,
/// the resolved `service.mcp_ability` (an `Option<&str>`) is passed to the closure;
/// a `false` return denies the turn before any dispatch. The closure is expected to
/// be fail-closed (a `None` ability denies). Writes are NOT gated here — the scope
/// gate (Phase 217) + live-DB guard re-eval inside `handle_write_call` cover the
/// write authorization surface, exactly as the direct path documents.
#[cfg(feature = "ai")]
#[allow(clippy::too_many_arguments)]
pub async fn process_nl_turn(
    nl_message: &str,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &McpContext,
    authorize_read: &(dyn Fn(Option<&str>) -> bool + Sync),
    provider: Arc<dyn ferro_ai::ClassificationProvider>,
    classifier_config: ferro_ai::ClassifierConfig,
    dispatcher: &WriteDispatcher,
    #[cfg(feature = "confirmation")] store: &dyn ferro_ai::ConfirmationStore,
    #[cfg(feature = "confirmation")] config: &crate::McpServerConfig,
) -> serde_json::Value {
    // Step 1: build the system prompt from exposed tool descriptions.
    let system = match render_tool_descriptions(services, ctx) {
        Ok(s) => s,
        Err(e) => {
            return json!({ "result": write_tool_error_result(json!({
                "error_kind": "render_error",
                "message": e.to_string()
            })) });
        }
    };

    // Step 2: build the ToolSelection JSON schema (snake_case keys matching the serde repr).
    let schema = json!({
        "type": "object",
        "properties": {
            "tool_name": { "type": "string", "description": "The tool to invoke" },
            "arguments": { "type": "object", "description": "Arguments for the tool" },
            "confidence": { "type": "number", "description": "Classifier confidence in [0.0, 1.0]" }
        },
        "required": ["tool_name", "arguments", "confidence"]
    });

    // Step 3: classify.
    let classifier = ferro_ai::Classifier::<ToolSelection>::new(provider, classifier_config);

    match classifier.classify(&system, nl_message, &schema).await {
        // Step 4a: low-confidence → needs_clarification, no dispatch (SC#5, T-221-06).
        Err(ferro_ai::Error::LowConfidence {
            best_guess,
            confidence,
        }) => {
            let question = format!(
                "I'm not sure what you mean (confidence {:.0}%). Did you mean to {}? \
                 Or could you be more specific?",
                confidence * 100.0,
                best_guess
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("do something")
            );
            json!({
                "result": {
                    "content": [{ "type": "text", "text": question }],
                    "isError": false,
                    "structuredContent": {
                        "status": "needs_clarification",
                        "question": question,
                        "best_guess": best_guess
                    }
                }
            })
        }

        // Step 4b: other errors → error envelope.
        Err(other) => {
            json!({ "result": write_tool_error_result(json!({
                "error_kind": "classification_error",
                "message": other.to_string()
            })) })
        }

        // Step 4c: successful classification → route.
        Ok(result) => {
            let sel = result.value;
            // Build call_params in the same shape as a normal tools/call request.
            let call_params = json!({
                "name": sel.tool_name,
                "arguments": sel.arguments
            });

            if let Some(service_name) = sel.tool_name.strip_prefix("list_") {
                // Read path (SC#1 read). Apply the SAME app-ability authorization the
                // direct /mcp path enforces (WR-01 / AMCP-11): resolve the target service,
                // then let the app-provided `authorize_read` closure decide. The classifier's
                // tool_name is UNTRUSTED, so this gate runs BEFORE any dispatch. Fail-closed:
                // a service with no declared mcp_ability is denied by the closure.
                match services
                    .iter()
                    .find(|s| s.name == service_name && s.mcp_exposed)
                {
                    None => {
                        // Unknown read tool — method not found (mirrors direct path -32601).
                        return json!({ "result": write_tool_error_result(json!({
                            "error_kind": "method_not_found",
                            "message": "Method not found"
                        })) });
                    }
                    Some(service) => {
                        if !authorize_read(service.mcp_ability.as_deref()) {
                            // D-09: deny envelope discloses no rows, columns, or filters.
                            return json!({
                                "result": {
                                    "content": [{
                                        "type": "text",
                                        "text": "Access denied. You do not have permission to view this resource."
                                    }],
                                    "isError": true,
                                    "structuredContent": { "status": "access_denied" }
                                }
                            });
                        }
                    }
                }
                // Authorized — handle_tools_call owns service lookup + dispatch.
                handle_tools_call(
                    call_params,
                    services,
                    db,
                    tenant_id,
                    ctx,
                    dispatcher,
                    #[cfg(feature = "confirmation")]
                    store,
                    #[cfg(feature = "confirmation")]
                    config,
                )
                .await
            } else {
                // Write path (SC#1 write, SC#2): handle_write_call owns scope check,
                // guard re-eval (live DB), idempotency, D-08 confirmation seam, execute,
                // and audit. No parallel pipeline here.
                handle_write_call(
                    call_params,
                    services,
                    db,
                    tenant_id,
                    ctx,
                    dispatcher,
                    #[cfg(feature = "confirmation")]
                    store,
                    #[cfg(feature = "confirmation")]
                    config,
                )
                .await
            }
        }
    }
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
        let sel: ToolSelection = serde_json::from_value(json.clone()).expect("must deserialize");
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
        assert!(
            result.is_ok(),
            "snake_case keys must deserialize: {:?}",
            result.err()
        );
        // camelCase keys must NOT deserialize (schema mismatch guard).
        let camel = serde_json::json!({
            "toolName": "approve",
            "arguments": {},
            "confidence": 0.9
        });
        let bad = serde_json::from_value::<ToolSelection>(camel);
        assert!(bad.is_err(), "camelCase tool_name key must not deserialize");
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
        let text = render_tool_descriptions(&[svc], &ctx).expect("must render");

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
            .action(ActionDef::new("approve").input(InputDef::new(
                "id",
                DataType::Integer,
                FieldMeaning::Identifier,
            )));

        let ctx = McpContext::default();
        let text = render_tool_descriptions(&[svc], &ctx).expect("must render");

        assert!(
            text.contains("[args:"),
            "must contain [args: ...] listing; got: {text}"
        );
    }
}
