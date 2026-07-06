/// Location of an API parameter in the HTTP request.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamLocation {
    Path,
    Query,
    Header,
}

/// A single parameter for an API operation.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiParam {
    pub name: String,
    pub location: ParamLocation,
    pub required: bool,
    pub schema: serde_json::Value,
    pub description: Option<String>,
}

/// A parsed API operation ready for MCP tool generation.
///
/// Each operation maps to one MCP tool. The `input_schema` field holds
/// the merged JSON Schema for the tool's input, built by the schema module
/// from parameters and request body.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiOperation {
    pub tool_name: String,
    pub method: String,
    pub path: String,
    pub description: String,
    pub parameters: Vec<ApiParam>,
    pub request_body_schema: Option<serde_json::Value>,
    pub input_schema: serde_json::Value,
    pub hint: Option<String>,
}

impl ApiOperation {
    /// Creates a new `ApiOperation` with an empty input schema.
    ///
    /// The `input_schema` defaults to `{"type": "object", "properties": {}}`
    /// and will be populated by the schema module during spec parsing.
    pub fn new(tool_name: String, method: String, path: String, description: String) -> Self {
        Self {
            tool_name,
            method,
            path,
            description,
            parameters: Vec::new(),
            request_body_schema: None,
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
            hint: None,
        }
    }
}
