use crate::client::{
    CompletionRequest, CompletionResponse, LlmClient, Role, TokenStream, ToolChoice, ToolUseBlock,
};
use crate::error::Error;
use async_trait::async_trait;
use futures::{stream, StreamExt};
use reqwest_eventsource::{Event, RequestBuilderExt};

/// Anthropic Messages API client.
///
/// Implements [`LlmClient`] against `https://api.anthropic.com/v1/messages`.
/// Uses `output_config.format.type = "json_schema"` for structured output
/// when [`CompletionRequest::schema`] is `Some`.
///
/// # Embeddings
///
/// Anthropic has no embeddings endpoint. [`AnthropicClient::embed`] returns
/// `Err(Error::Unsupported)` (D-13).
///
/// # Authentication
///
/// Provide the API key via `new(api_key, ...)` or read it from
/// `FERRO_AI_API_KEY` (with `ANTHROPIC_API_KEY` as fallback) via
/// [`crate::config::AiConfig::from_env`].
pub struct AnthropicClient {
    client: reqwest::Client,
    api_key: String,
    model: Option<String>,
}

impl AnthropicClient {
    /// Create a new client with an explicit API key and optional model override.
    ///
    /// The internal `reqwest::Client` uses a 60-second timeout (T-165-04).
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build reqwest client");
        Self {
            client,
            api_key,
            model,
        }
    }

    /// Build the request body for the Anthropic Messages API.
    ///
    /// Includes `output_config.format.type = "json_schema"` only when
    /// `request.schema` is `Some` (D-11). System prompt uses ephemeral
    /// prompt caching. Sets `"stream": stream`.
    pub(crate) fn build_body(
        &self,
        request: &CompletionRequest,
        stream: bool,
    ) -> serde_json::Value {
        let model = request
            .model_override
            .as_deref()
            .unwrap_or_else(|| self.default_model());

        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| match m.role {
                Role::Tool => {
                    // Anthropic wire format: role "user" with a tool_result content block.
                    // tool_use_id must be a real field, not embedded in the content string.
                    let tool_use_id = m.tool_call_id.as_deref().unwrap_or("");
                    serde_json::json!({
                        "role": "user",
                        "content": [{
                            "type": "tool_result",
                            "tool_use_id": tool_use_id,
                            "content": m.content,
                        }]
                    })
                }
                Role::User => serde_json::json!({"role": "user", "content": m.content}),
                Role::Assistant => {
                    // In a tool-use loop, `assistant_content` (from
                    // `CompletionResponse::ToolUse`) is the raw Anthropic content
                    // blocks array, JSON-stringified — it carries the `tool_use`
                    // blocks the next `tool_result` must reference. Spread it back
                    // as structured content when it parses as an array; otherwise
                    // send it as a plain text string (a normal assistant turn).
                    // Without this, the array is sent as a single text block, the
                    // `tool_use` block is lost, and Anthropic rejects the following
                    // `tool_result` with "no corresponding tool_use block".
                    match serde_json::from_str::<serde_json::Value>(&m.content) {
                        Ok(content @ serde_json::Value::Array(_)) => {
                            serde_json::json!({"role": "assistant", "content": content})
                        }
                        _ => serde_json::json!({"role": "assistant", "content": m.content}),
                    }
                }
            })
            .collect();

        let mut body = serde_json::json!({
            "model": model,
            "max_tokens": request.max_tokens,
            "messages": messages,
            "stream": stream,
        });

        if let Some(system) = &request.system {
            body["system"] = serde_json::json!([{
                "type": "text",
                "text": system,
                "cache_control": {"type": "ephemeral"},
            }]);
        }

        if let Some(schema) = &request.schema {
            body["output_config"] = serde_json::json!({
                "format": {
                    "type": "json_schema",
                    "schema": schema,
                }
            });
        }

        if let Some(tools) = &request.tools {
            let tools_json: Vec<serde_json::Value> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "input_schema": t.parameters_schema,
                    })
                })
                .collect();
            body["tools"] = serde_json::Value::Array(tools_json);
            if let Some(choice) = &request.tool_choice {
                body["tool_choice"] = match choice {
                    ToolChoice::Auto => serde_json::json!({"type": "auto"}),
                    ToolChoice::None => serde_json::json!({"type": "none"}),
                };
            }
        }

        body
    }
}

/// Parse tool_use content blocks from an Anthropic response `content` array.
pub(crate) fn parse_anthropic_tool_use_blocks(content: &serde_json::Value) -> Vec<ToolUseBlock> {
    let Some(arr) = content.as_array() else {
        return vec![];
    };
    arr.iter()
        .filter(|item| item["type"].as_str() == Some("tool_use"))
        .filter_map(|item| {
            Some(ToolUseBlock {
                id: item["id"].as_str()?.to_string(),
                name: item["name"].as_str()?.to_string(),
                input: item["input"].clone(),
            })
        })
        .collect()
}

/// Parse a `content_block_delta` SSE event data string, returning the delta
/// text token when present and non-empty.
///
/// Unit-testable without a live server; verified against the Anthropic SSE
/// fixture from the research notes.
pub(crate) fn parse_anthropic_delta(data: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(data).ok()?;
    let text = v["delta"]["text"].as_str()?;
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    fn default_model(&self) -> &str {
        self.model.as_deref().unwrap_or("claude-sonnet-4-6")
    }

    async fn complete(&self, request: CompletionRequest) -> Result<String, Error> {
        let body = self.build_body(&request, false);

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    Error::Timeout
                } else {
                    Error::Provider {
                        status: None,
                        message: e.to_string(),
                    }
                }
            })?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Provider {
                status: Some(status),
                message: text,
            });
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Deserialization(e.to_string()))?;

        let text = json["content"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|i| i["text"].as_str())
            .ok_or_else(|| Error::Deserialization(format!("unexpected response: {json}")))?;

        Ok(text.to_string())
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<TokenStream, Error> {
        let body = self.build_body(&request, true);

        let builder = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body);

        let es = builder.eventsource().map_err(|_| Error::Provider {
            status: None,
            message: "request not cloneable".into(),
        })?;

        let token_stream = stream::unfold(es, |mut es| async move {
            loop {
                match es.next().await {
                    None => return None,
                    Some(Ok(Event::Open)) => continue,
                    Some(Ok(Event::Message(msg))) => {
                        if msg.event == "message_stop" {
                            es.close();
                            return None;
                        }
                        if msg.event == "content_block_delta" {
                            if let Some(token) = parse_anthropic_delta(&msg.data) {
                                return Some((Ok(token), es));
                            }
                        }
                        continue;
                    }
                    Some(Err(e)) => {
                        es.close();
                        return Some((
                            Err(Error::Provider {
                                status: None,
                                message: e.to_string(),
                            }),
                            es,
                        ));
                    }
                }
            }
        });

        Ok(Box::pin(token_stream))
    }

    /// Returns `Err(Error::Unsupported)` — Anthropic has no embeddings endpoint (D-13).
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, Error> {
        Err(Error::Unsupported)
    }

    async fn complete_with_tools(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, Error> {
        let body = self.build_body(&request, false);

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    Error::Timeout
                } else {
                    Error::Provider {
                        status: None,
                        message: e.to_string(),
                    }
                }
            })?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Provider {
                status: Some(status),
                message: text,
            });
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Deserialization(e.to_string()))?;

        let stop_reason = json["stop_reason"].as_str().unwrap_or("");
        if stop_reason == "tool_use" {
            let blocks = parse_anthropic_tool_use_blocks(&json["content"]);
            let assistant_content = json["content"].to_string();
            return Ok(CompletionResponse::ToolUse {
                blocks,
                assistant_content,
            });
        }

        // end_turn or any other stop reason → extract text
        let text = json["content"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|i| i["text"].as_str())
            .ok_or_else(|| Error::Deserialization(format!("unexpected response: {json}")))?;

        Ok(CompletionResponse::Text(text.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Message;

    #[test]
    fn test_anthropic_default_model() {
        let client = AnthropicClient::new("k".into(), None);
        assert_eq!(client.default_model(), "claude-sonnet-4-6");
    }

    #[test]
    fn test_anthropic_default_model_override() {
        let client = AnthropicClient::new("k".into(), Some("claude-opus-4-6".into()));
        assert_eq!(client.default_model(), "claude-opus-4-6");
    }

    #[test]
    fn test_build_body_with_schema() {
        let client = AnthropicClient::new("k".into(), None);
        let schema = serde_json::json!({"type": "object", "properties": {"x": {"type": "string"}}});
        let request = CompletionRequest {
            system: None,
            messages: vec![Message {
                role: Role::User,
                content: "hi".into(),
                tool_call_id: None,
            }],
            max_tokens: 100,
            model_override: None,
            schema: Some(schema.clone()),
            tools: None,
            tool_choice: None,
        };
        let body = client.build_body(&request, false);

        assert_eq!(body["output_config"]["format"]["type"], "json_schema");
        assert_eq!(body["output_config"]["format"]["schema"], schema);
    }

    #[test]
    fn test_build_body_without_schema_omits_output_config() {
        let client = AnthropicClient::new("k".into(), None);
        let request = CompletionRequest {
            system: None,
            messages: vec![Message {
                role: Role::User,
                content: "hi".into(),
                tool_call_id: None,
            }],
            max_tokens: 100,
            model_override: None,
            schema: None,
            tools: None,
            tool_choice: None,
        };
        let body = client.build_body(&request, false);

        assert!(body.get("output_config").is_none());
    }

    #[test]
    fn test_build_body_stream_flag() {
        let client = AnthropicClient::new("k".into(), None);
        let request = CompletionRequest {
            system: None,
            messages: vec![Message {
                role: Role::User,
                content: "hi".into(),
                tool_call_id: None,
            }],
            max_tokens: 100,
            model_override: None,
            schema: None,
            tools: None,
            tool_choice: None,
        };
        let streaming_body = client.build_body(&request, true);
        let non_streaming_body = client.build_body(&request, false);

        assert_eq!(streaming_body["stream"], true);
        assert_eq!(non_streaming_body["stream"], false);
    }

    #[tokio::test]
    async fn test_anthropic_embed_unsupported() {
        let client = AnthropicClient::new("k".into(), None);
        assert!(matches!(client.embed("x").await, Err(Error::Unsupported)));
    }

    #[test]
    fn test_parse_anthropic_delta() {
        // Fixture from RESEARCH lines 582-583
        let data =
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hi"}}"#;
        assert_eq!(parse_anthropic_delta(data), Some("Hi".to_string()));
    }

    #[test]
    fn test_parse_anthropic_delta_empty_text() {
        let data =
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":""}}"#;
        assert_eq!(parse_anthropic_delta(data), None);
    }

    #[test]
    fn test_parse_anthropic_delta_non_delta_event() {
        let data = r#"{"type":"message_start","message":{"id":"msg_123"}}"#;
        assert_eq!(parse_anthropic_delta(data), None);
    }

    #[test]
    fn test_anthropic_is_object_safe() {
        let _: Box<dyn LlmClient> = Box::new(AnthropicClient::new("k".into(), None));
    }

    /// CR-01 regression: tool result messages must emit a tool_result content block,
    /// not a plain string. The tool_use_id must appear as a real field, not embedded
    /// inside the content string.
    #[test]
    fn test_build_body_tool_result_wire_format() {
        let client = AnthropicClient::new("k".into(), None);
        let request = CompletionRequest {
            system: None,
            messages: vec![
                Message {
                    role: Role::User,
                    content: "what is 2+2?".into(),
                    tool_call_id: None,
                },
                Message {
                    role: Role::Tool,
                    content: "4".into(),
                    tool_call_id: Some("toolu_abc123".into()),
                },
            ],
            max_tokens: 100,
            model_override: None,
            schema: None,
            tools: None,
            tool_choice: None,
        };
        let body = client.build_body(&request, false);
        let msgs = body["messages"].as_array().expect("messages must be array");
        assert_eq!(msgs.len(), 2);

        // The tool result message must use role "user" with a content array block.
        let tool_msg = &msgs[1];
        assert_eq!(
            tool_msg["role"], "user",
            "Anthropic tool result must use role 'user'"
        );

        let content_arr = tool_msg["content"]
            .as_array()
            .expect("tool result content must be an array of blocks");
        assert_eq!(content_arr.len(), 1);

        let block = &content_arr[0];
        assert_eq!(block["type"], "tool_result");
        assert_eq!(
            block["tool_use_id"], "toolu_abc123",
            "tool_use_id must be a real field, not embedded in content"
        );
        assert_eq!(block["content"], "4");
    }
}
