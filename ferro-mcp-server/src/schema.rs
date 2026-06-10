use ferro_projections::ServiceDef;

/// Builds the MCP tool `inputSchema` as a JSON Schema object derived from
/// the projection's fields. Implemented in plan 02.
pub fn build_input_schema(_service: &ServiceDef) -> crate::Result<serde_json::Value> {
    Ok(serde_json::json!({ "type": "object", "properties": {} }))
}
