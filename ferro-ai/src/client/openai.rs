use crate::client::{
    CompletionRequest, CompletionResponse, LlmClient, Role, TokenStream, ToolChoice, ToolUseBlock,
};
use crate::error::Error;
use async_trait::async_trait;
use futures::{stream, StreamExt};
use reqwest_eventsource::{Event, RequestBuilderExt};

/// OpenAI Chat Completions API client.
///
/// Implements [`LlmClient`] against `{base_url}/v1/chat/completions`. Also
/// serves as the Groq client when `base_url` is set to
/// `https://api.groq.com/openai` — Groq exposes an OpenAI-compatible API.
///
/// # Authentication
///
/// Uses `Authorization: Bearer {api_key}` for all requests.
///
/// # Embeddings
///
/// `embed()` posts to `{base_url}/v1/embeddings` using model
/// `text-embedding-3-small` and extracts `data[0].embedding` as `Vec<f32>`.
pub struct OpenAiClient {
    client: reqwest::Client,
    api_key: String,
    model: Option<String>,
    base_url: String,
}

impl OpenAiClient {
    /// Create a new client.
    ///
    /// - `model`: optional model override; `None` resolves to `default_model()`.
    /// - `base_url`: optional base URL override; `None` defaults to
    ///   `https://api.openai.com`. Pass `Some("https://api.groq.com/openai")`
    ///   for Groq compatibility.
    ///
    /// The internal `reqwest::Client` uses a 60-second timeout (T-165-04).
    pub fn new(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build reqwest client");
        let base_url = base_url.unwrap_or_else(|| "https://api.openai.com".to_string());
        Self {
            client,
            api_key,
            model,
            base_url,
        }
    }

    /// The embedding model for `/v1/embeddings`.
    ///
    /// Reads `FERRO_AI_EMBED_MODEL`; falls back to `"text-embedding-3-small"`.
    /// Intentionally separate from `default_model()` (the chat model).
    pub(crate) fn embed_model() -> String {
        std::env::var("FERRO_AI_EMBED_MODEL")
            .unwrap_or_else(|_| "text-embedding-3-small".to_string())
    }

    /// Build the request body for the Chat Completions API.
    ///
    /// Includes `response_format.type = "json_schema"` only when
    /// `request.schema` is `Some`. Sets `"stream": stream`.
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
                    // OpenAI wire format: role "tool" with tool_call_id as a real field.
                    // tool_call_id must not be embedded in the content string.
                    let call_id = m.tool_call_id.as_deref().unwrap_or("");
                    serde_json::json!({
                        "role": "tool",
                        "tool_call_id": call_id,
                        "content": m.content,
                    })
                }
                Role::User => serde_json::json!({"role": "user", "content": m.content}),
                Role::Assistant => {
                    serde_json::json!({"role": "assistant", "content": m.content})
                }
            })
            .collect();

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
            "max_tokens": request.max_tokens,
            "stream": stream,
        });

        if let Some(schema) = &request.schema {
            body["response_format"] = serde_json::json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "output",
                    "schema": schema,
                    "strict": true,
                }
            });
        }

        if let Some(tools) = &request.tools {
            let tools_json: Vec<serde_json::Value> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters_schema,
                            "strict": true,
                        }
                    })
                })
                .collect();
            body["tools"] = serde_json::Value::Array(tools_json);
            // WR-01: honor request.tool_choice; default to "auto" when not specified.
            body["tool_choice"] = match request.tool_choice.as_ref() {
                Some(ToolChoice::None) => serde_json::json!("none"),
                Some(ToolChoice::Auto) | None => serde_json::json!("auto"),
            };
        }

        body
    }
}

/// Parse tool_calls from an OpenAI response into [`ToolUseBlock`]s.
pub(crate) fn parse_openai_tool_calls(json: &serde_json::Value) -> Vec<ToolUseBlock> {
    let Some(tool_calls) = json["choices"][0]["message"]["tool_calls"].as_array() else {
        return vec![];
    };
    tool_calls
        .iter()
        .filter_map(|c| {
            Some(ToolUseBlock {
                id: c["id"].as_str()?.to_string(),
                name: c["function"]["name"].as_str()?.to_string(),
                input: serde_json::from_str(c["function"]["arguments"].as_str()?).ok()?,
            })
        })
        .collect()
}

/// Result of parsing a single OpenAI SSE chunk data string.
#[derive(Debug, PartialEq)]
pub(crate) enum OpenAiDelta {
    /// The stream is terminated (`data: [DONE]`).
    Done,
    /// A non-empty text token.
    Token(String),
    /// Empty delta or role-only chunk — skip.
    Skip,
}

/// Parse one OpenAI SSE chunk data string.
///
/// Handles the `[DONE]` sentinel (Pitfall 4) before JSON-parsing.
/// Returns `Done` on termination, `Token(text)` for content, `Skip` otherwise.
pub(crate) fn parse_openai_delta(data: &str) -> OpenAiDelta {
    if data == "[DONE]" {
        return OpenAiDelta::Done;
    }
    let Ok(v) = serde_json::from_str::<serde_json::Value>(data) else {
        return OpenAiDelta::Skip;
    };
    // Terminate on finish_reason being non-null
    if !v["choices"][0]["finish_reason"].is_null() {
        if let Some(reason) = v["choices"][0]["finish_reason"].as_str() {
            if !reason.is_empty() {
                return OpenAiDelta::Done;
            }
        }
    }
    match v["choices"][0]["delta"]["content"].as_str() {
        Some(text) if !text.is_empty() => OpenAiDelta::Token(text.to_string()),
        _ => OpenAiDelta::Skip,
    }
}

/// Parse the embeddings response, extracting `data[0].embedding` as `Vec<f32>`.
pub(crate) fn parse_embedding(json: &serde_json::Value) -> Result<Vec<f32>, Error> {
    json["data"][0]["embedding"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect()
        })
        .ok_or_else(|| Error::Deserialization("no embedding in response".into()))
}

#[async_trait]
impl LlmClient for OpenAiClient {
    fn default_model(&self) -> &str {
        self.model.as_deref().unwrap_or("gpt-4o")
    }

    async fn complete(&self, request: CompletionRequest) -> Result<String, Error> {
        let body = self.build_body(&request, false);

        let resp = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
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

        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Deserialization("no content in response".into()))
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<TokenStream, Error> {
        let body = self.build_body(&request, true);

        let builder = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
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
                    Some(Ok(Event::Message(msg))) => match parse_openai_delta(&msg.data) {
                        OpenAiDelta::Done => {
                            es.close();
                            return None;
                        }
                        OpenAiDelta::Token(text) => return Some((Ok(text), es)),
                        OpenAiDelta::Skip => continue,
                    },
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

    async fn embed(&self, text: &str) -> Result<Vec<f32>, Error> {
        let body = serde_json::json!({
            "model": Self::embed_model(),
            "input": text,
        });

        let resp = self
            .client
            .post(format!("{}/v1/embeddings", self.base_url))
            .bearer_auth(&self.api_key)
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

        parse_embedding(&json)
    }

    async fn complete_with_tools(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, Error> {
        let body = self.build_body(&request, false);

        let resp = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
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

        let finish_reason = json["choices"][0]["finish_reason"].as_str().unwrap_or("");
        if finish_reason == "tool_calls" {
            let blocks = parse_openai_tool_calls(&json);
            let assistant_content = json["choices"][0]["message"]["tool_calls"].to_string();
            return Ok(CompletionResponse::ToolUse {
                blocks,
                assistant_content,
            });
        }

        // stop or any other finish_reason → extract text content
        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Deserialization("no content in response".into()))?;

        Ok(CompletionResponse::Text(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Message;

    #[test]
    fn test_openai_default_model() {
        let client = OpenAiClient::new("k".into(), None, None);
        assert_eq!(client.default_model(), "gpt-4o");
    }

    #[test]
    fn test_openai_default_base_url() {
        let client = OpenAiClient::new("k".into(), None, None);
        assert_eq!(client.base_url, "https://api.openai.com");
    }

    #[test]
    fn test_openai_groq_base_url() {
        let client =
            OpenAiClient::new("k".into(), None, Some("https://api.groq.com/openai".into()));
        assert_eq!(client.base_url, "https://api.groq.com/openai");
    }

    #[test]
    fn test_build_body_response_format_with_schema() {
        let client = OpenAiClient::new("k".into(), None, None);
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

        assert_eq!(body["response_format"]["type"], "json_schema");
        assert_eq!(body["response_format"]["json_schema"]["name"], "output");
        assert_eq!(body["response_format"]["json_schema"]["schema"], schema);
        assert_eq!(body["response_format"]["json_schema"]["strict"], true);
    }

    #[test]
    fn test_build_body_no_response_format_without_schema() {
        let client = OpenAiClient::new("k".into(), None, None);
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
        assert!(body.get("response_format").is_none());
    }

    #[test]
    fn test_parse_openai_delta_done() {
        assert_eq!(parse_openai_delta("[DONE]"), OpenAiDelta::Done);
    }

    #[test]
    fn test_parse_openai_delta_token() {
        // Fixture from RESEARCH line 635
        let data = r#"{"id":"chatcmpl-1","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        assert_eq!(
            parse_openai_delta(data),
            OpenAiDelta::Token("Hello".to_string())
        );
    }

    #[test]
    fn test_parse_openai_delta_skip_empty_content() {
        let data = r#"{"choices":[{"index":0,"delta":{"role":"assistant","content":null},"finish_reason":null}]}"#;
        assert_eq!(parse_openai_delta(data), OpenAiDelta::Skip);
    }

    #[test]
    fn test_parse_openai_delta_finish_reason() {
        let data = r#"{"choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#;
        assert_eq!(parse_openai_delta(data), OpenAiDelta::Done);
    }

    #[test]
    fn test_parse_embedding() {
        let json = serde_json::json!({
            "data": [{"embedding": [0.1, -0.2, 0.3], "index": 0}],
            "usage": {}
        });
        let result = parse_embedding(&json).unwrap();
        assert_eq!(result.len(), 3);
        assert!((result[0] - 0.1f32).abs() < 1e-6);
        assert!((result[1] - (-0.2f32)).abs() < 1e-6);
        assert!((result[2] - 0.3f32).abs() < 1e-6);
    }

    #[test]
    fn test_parse_embedding_missing() {
        let json = serde_json::json!({"data": []});
        assert!(matches!(
            parse_embedding(&json),
            Err(Error::Deserialization(_))
        ));
    }

    #[test]
    fn test_openai_is_object_safe() {
        let _: Box<dyn LlmClient> = Box::new(OpenAiClient::new("k".into(), None, None));
    }

    /// CR-03 regression: OpenAI tool result messages must include `tool_call_id` as a
    /// real top-level field. The id must not be embedded inside the content string.
    #[test]
    fn test_build_body_tool_result_wire_format() {
        let client = OpenAiClient::new("k".into(), None, None);
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
                    tool_call_id: Some("call_abc123".into()),
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

        let tool_msg = &msgs[1];
        assert_eq!(tool_msg["role"], "tool");
        assert_eq!(
            tool_msg["tool_call_id"], "call_abc123",
            "tool_call_id must be a real top-level field"
        );
        assert_eq!(tool_msg["content"], "4");
        // The id must not also appear embedded in the content string.
        assert!(
            !tool_msg["content"]
                .as_str()
                .unwrap_or("")
                .contains("call_abc123"),
            "tool_call_id must not be embedded in content"
        );
    }

    /// WR-01 regression: OpenAI build_body must honor request.tool_choice.
    #[test]
    fn test_build_body_tool_choice_none() {
        use crate::client::{ToolChoice, ToolRequest};

        let client = OpenAiClient::new("k".into(), None, None);
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
            tools: Some(vec![ToolRequest {
                name: "my_tool".into(),
                description: "does stuff".into(),
                parameters_schema: serde_json::json!({"type": "object"}),
            }]),
            tool_choice: Some(ToolChoice::None),
        };
        let body = client.build_body(&request, false);
        assert_eq!(
            body["tool_choice"], "none",
            "ToolChoice::None must emit tool_choice: 'none'"
        );
    }

    /// WR-01: Auto tool_choice (explicit) and default (None) both emit "auto".
    #[test]
    fn test_build_body_tool_choice_auto() {
        use crate::client::{ToolChoice, ToolRequest};

        let client = OpenAiClient::new("k".into(), None, None);
        let tools = Some(vec![ToolRequest {
            name: "my_tool".into(),
            description: "does stuff".into(),
            parameters_schema: serde_json::json!({"type": "object"}),
        }]);

        // Explicit Auto.
        let req_auto = CompletionRequest {
            system: None,
            messages: vec![Message {
                role: Role::User,
                content: "hi".into(),
                tool_call_id: None,
            }],
            max_tokens: 100,
            model_override: None,
            schema: None,
            tools: tools.clone(),
            tool_choice: Some(ToolChoice::Auto),
        };
        let body = client.build_body(&req_auto, false);
        assert_eq!(body["tool_choice"], "auto");

        // Default None → also "auto".
        let req_default = CompletionRequest {
            tool_choice: None,
            ..req_auto
        };
        let body2 = client.build_body(&req_default, false);
        assert_eq!(body2["tool_choice"], "auto");
    }

    #[test]
    fn embed_model_default_is_text_embedding_3_small() {
        let _g = crate::ENV_LOCK.lock().unwrap();
        std::env::remove_var("FERRO_AI_EMBED_MODEL");
        assert_eq!(OpenAiClient::embed_model(), "text-embedding-3-small");
    }

    #[test]
    fn embed_model_from_env() {
        let _g = crate::ENV_LOCK.lock().unwrap();
        std::env::set_var("FERRO_AI_EMBED_MODEL", "text-embedding-ada-002");
        assert_eq!(OpenAiClient::embed_model(), "text-embedding-ada-002");
        std::env::remove_var("FERRO_AI_EMBED_MODEL");
    }
}
